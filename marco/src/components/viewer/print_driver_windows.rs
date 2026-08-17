//! Windows print / PDF export driver (wry / WebView2).
//!
//! Mirrors the Linux `print_driver_linux` module's shape: one file for both the
//! native print dialog and PDF export, since both are thin wrappers over the
//! same live WebView2 instance.
//!
//! * [`trigger_print_dialog`] — opens the native browser print UI.
//! * [`print_to_pdf`] — headless export via `ICoreWebView2_7::PrintToPdf`
//!   (no Chromium subprocess; replaces the old headless-Chromium path).
//!
//! ## Concurrency (PDF export)
//!
//! [`print_to_pdf`] must run on the GTK main thread (the same thread that
//! owns the `wry::WebView`). COM async completion handlers fire via
//! `PostMessage` to the WebView2 thread, which is the same thread, so
//! `wait_for_async_operation` simply pumps Win32 messages until the result is
//! delivered — this keeps the "Exporting…" progress dialog
//! (`ui::dialogs::exporting`) animating smoothly.
//!
//! ## Paper-size mapping
//!
//! WebView2 wants page width/height in **inches**; our public API takes the
//! human-friendly paper-name strings (`"A4"`, `"Letter"`, …) used by the
//! export dialog. See [`paper_inches`].

use std::path::Path;
use std::sync::mpsc;

use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Environment6, ICoreWebView2_2, ICoreWebView2_7,
    COREWEBVIEW2_PRINT_ORIENTATION_LANDSCAPE, COREWEBVIEW2_PRINT_ORIENTATION_PORTRAIT,
};
use webview2_com::PrintToPdfCompletedHandler;
use windows::core::Interface;

use wry::WebViewExtWindows;

use crate::components::viewer::preview_types::PlatformWebView;

// ---------------------------------------------------------------------------
// Print dialog
// ---------------------------------------------------------------------------

/// Open the native browser print UI for the current WebView content.
///
/// Before triggering the print UI, this injects the shared print-export CSS
/// (`marco_shared::logic::print_css::make_print_export_css`) into the live
/// WebView so paged.js layout maps cleanly to printer pages, matching the
/// Linux `print_driver_linux::trigger_print_dialog` behavior.
///
/// The injected `<style>` element carries a known id so it can be located by
/// JS. We schedule its removal after a generous timeout so the live preview
/// returns to its normal appearance after the user dismisses the print UI.
pub fn trigger_print_dialog(
    webview: &PlatformWebView,
    paper: &str,
    orientation: &str,
    dark_mode: bool,
) {
    inject_pre_print_css(webview, paper, orientation, dark_mode);
    webview.trigger_print_dialog();
}

const PRE_PRINT_STYLE_ID: &str = "marco-pre-print-export-css";
/// Time after which the injected pre-print CSS is removed from the live WebView.
/// Native print UIs (WebView2 / Edge) are non-blocking, so we cannot detect
/// dialog dismissal; this fallback restores normal preview styling.
const PRE_PRINT_CSS_TTL_MS: u32 = 60_000;

/// Inject the shared print-export CSS into the live WebView via JS, with a
/// pending self-removal timer so the live preview eventually returns to normal.
fn inject_pre_print_css(
    webview: &PlatformWebView,
    paper: &str,
    orientation: &str,
    dark_mode: bool,
) {
    let css = marco_shared::logic::print_css::make_print_export_css(paper, orientation, dark_mode);
    let css_json = json_string_literal(&css);
    let script = format!(
        r#"(function() {{
    try {{
        var existing = document.getElementById('{id}');
        if (existing) {{ existing.parentNode.removeChild(existing); }}
        var style = document.createElement('style');
        style.id = '{id}';
        style.appendChild(document.createTextNode({css}));
        (document.head || document.documentElement).appendChild(style);
        setTimeout(function() {{
            var s = document.getElementById('{id}');
            if (s && s.parentNode) {{ s.parentNode.removeChild(s); }}
        }}, {ttl});
    }} catch (e) {{ console.error('marco pre-print css injection failed', e); }}
}})();"#,
        id = PRE_PRINT_STYLE_ID,
        css = css_json,
        ttl = PRE_PRINT_CSS_TTL_MS,
    );
    webview.evaluate_script(&script);
}

/// Encode an arbitrary string as a JavaScript string literal (double-quoted).
/// Escapes characters that would otherwise terminate or corrupt the literal.
fn json_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            // Escape closing </script> / </style> to keep the literal safe
            // even when the embedded CSS contains '<' characters.
            '<' => out.push_str("\\u003c"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ---------------------------------------------------------------------------
// PDF export
// ---------------------------------------------------------------------------

/// Convert a paper-name string (case-insensitive) to (width_in, height_in)
/// for *portrait* orientation. Unknown names fall back to A4.
///
/// All values are exact mm/25.4 conversions rounded to 2 decimal places.
fn paper_inches(paper: &str) -> (f64, f64) {
    match paper.to_ascii_lowercase().as_str() {
        "a3" => (11.69, 16.54),
        "a4" => (8.27, 11.69),
        "a5" => (5.83, 8.27),
        "letter" => (8.5, 11.0),
        "legal" => (8.5, 14.0),
        "b5" => (6.93, 9.84),
        _ => (8.27, 11.69), // A4 fallback
    }
}

/// Print the current page contents of the supplied wry `WebView` to the given
/// PDF file, using the supplied page settings.
///
/// This blocks the calling thread (pumping Win32 messages via
/// `webview2_com::wait_with_pump`) until the COM async operation completes.
///
/// # Errors
///
/// Returns `Err(String)` with a human-readable message if any COM call fails
/// or the WebView2 reports an unsuccessful print.
pub fn print_to_pdf(
    webview: &wry::WebView,
    output_path: &Path,
    paper: &str,
    orientation: &str,
    margin_mm: u8,
) -> Result<(), String> {
    // `margin_mm` is intentionally unused here: paged.js already bakes the
    // requested page margin into each `.pagedjs_page` element as content
    // padding (via the `@page { margin: Nmm }` rule emitted by
    // `wrap_html_document_paged`). Asking WebView2's PrintToPdf to *also*
    // reserve a printer margin on top of that would force it to scale the
    // already-paper-sized paged.js pages down into a smaller printable area,
    // producing a visibly squashed PDF with double margins.
    //
    // The Linux backend behaves identically: the shared print CSS sets
    // `@page { margin: 0 !important; }` so the browser's printer margin is
    // zero, and the visible margin is owned by paged.js.
    let _ = margin_mm;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Convert path to a wide-string PCWSTR. WebView2 requires an absolute
    // path; we canonicalize so relative paths from CLI / test contexts work.
    let abs_path = match std::fs::canonicalize(output_path) {
        Ok(p) => p,
        Err(_) => output_path.to_path_buf(), // file doesn't exist yet — pass through
    };
    let path_str = abs_path
        .to_str()
        .ok_or_else(|| "Output path is not valid UTF-8".to_string())?;

    // Wide encoding for PCWSTR.
    let path_wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

    // Acquire ICoreWebView2 from wry.
    let core: webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2 = webview.webview();

    // ICoreWebView2_2 → Environment access.
    let core2: ICoreWebView2_2 = core
        .cast()
        .map_err(|e| format!("WebView2 missing ICoreWebView2_2: {}", e))?;

    // ICoreWebView2_7 → PrintToPdf method.
    let core7: ICoreWebView2_7 = core
        .cast()
        .map_err(|e| format!("WebView2 missing ICoreWebView2_7 (PrintToPdf): {}", e))?;

    // ── Build print settings ──────────────────────────────────────────────
    let settings = unsafe {
        let env = core2
            .Environment()
            .map_err(|e| format!("WebView2 Environment() failed: {}", e))?;
        let env6: ICoreWebView2Environment6 = env
            .cast()
            .map_err(|e| format!("WebView2 missing ICoreWebView2Environment6: {}", e))?;
        env6.CreatePrintSettings()
            .map_err(|e| format!("CreatePrintSettings failed: {}", e))?
    };

    let (width_in_portrait, height_in_portrait) = paper_inches(paper);
    let landscape = orientation.eq_ignore_ascii_case("landscape");
    let (page_width_in, page_height_in) = (width_in_portrait, height_in_portrait);

    unsafe {
        settings
            .SetPageWidth(page_width_in)
            .map_err(|e| format!("SetPageWidth failed: {}", e))?;
        settings
            .SetPageHeight(page_height_in)
            .map_err(|e| format!("SetPageHeight failed: {}", e))?;
        settings
            .SetOrientation(if landscape {
                COREWEBVIEW2_PRINT_ORIENTATION_LANDSCAPE
            } else {
                COREWEBVIEW2_PRINT_ORIENTATION_PORTRAIT
            })
            .map_err(|e| format!("SetOrientation failed: {}", e))?;
        // Printer margins are zero — paged.js owns the visible content
        // margin (see comment at top of `print_to_pdf`).
        settings
            .SetMarginTop(0.0)
            .map_err(|e| format!("SetMarginTop failed: {}", e))?;
        settings
            .SetMarginBottom(0.0)
            .map_err(|e| format!("SetMarginBottom failed: {}", e))?;
        settings
            .SetMarginLeft(0.0)
            .map_err(|e| format!("SetMarginLeft failed: {}", e))?;
        settings
            .SetMarginRight(0.0)
            .map_err(|e| format!("SetMarginRight failed: {}", e))?;
        // Background colors / images are part of the visual fidelity users
        // expect from "Export to PDF" — paged.js page backgrounds rely on it.
        settings
            .SetShouldPrintBackgrounds(true)
            .map_err(|e| format!("SetShouldPrintBackgrounds failed: {}", e))?;
        // We render our own page numbers via paged.js, so disable the
        // browser-injected header/footer.
        settings
            .SetShouldPrintHeaderAndFooter(false)
            .map_err(|e| format!("SetShouldPrintHeaderAndFooter failed: {}", e))?;
    }

    // ── Issue PrintToPdf and wait for the async completion ────────────────
    let (tx, rx) = mpsc::channel::<Result<(), String>>();

    PrintToPdfCompletedHandler::wait_for_async_operation(
        Box::new(move |handler| unsafe {
            let path_pcwstr = windows::core::PCWSTR(path_wide.as_ptr());
            core7
                .PrintToPdf(path_pcwstr, &settings, &handler)
                .map_err(webview2_com::Error::WindowsError)
        }),
        Box::new(move |error_code, is_successful| {
            // `error_code` is windows::core::Result<()>; `is_successful` is
            // already a Rust `bool` in this webview2-com binding.
            let outcome = match error_code {
                Ok(()) => {
                    if is_successful {
                        Ok(())
                    } else {
                        Err("WebView2 reported PrintToPdf was not successful".to_string())
                    }
                }
                Err(e) => Err(format!("WebView2 PrintToPdf error: {}", e)),
            };
            // Send result back to the waiting receiver; if the receiver was
            // dropped (caller aborted) we silently discard.
            let _ = tx.send(outcome);
            Ok(())
        }),
    )
    .map_err(|e| format!("PrintToPdf wait_for_async_operation failed: {:?}", e))?;

    // The callback above ALWAYS sends — but `wait_for_async_operation` returns
    // before the callback runs (it just kicks off the async op). We still
    // need to drain the channel. In webview2-com 0.38, `wait_for_async_operation`
    // internally pumps until the callback returns, so by this point `rx`
    // should have a value waiting.
    rx.recv()
        .map_err(|_| "PrintToPdf completion channel closed unexpectedly".to_string())??;

    // Sanity-check the file exists and is non-empty.
    match std::fs::metadata(&abs_path) {
        Ok(m) if m.len() > 0 => Ok(()),
        Ok(_) => Err("PrintToPdf produced an empty file".to_string()),
        Err(e) => Err(format!(
            "PrintToPdf reported success but output file is missing: {}",
            e
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_json_string_literal_escapes_quotes_backslashes_and_newlines() {
        let s = "alert(\"hi\");\nvar x = 1;\\path";
        let lit = json_string_literal(s);
        assert!(lit.starts_with('"') && lit.ends_with('"'));
        assert!(lit.contains("\\\""));
        assert!(lit.contains("\\n"));
        assert!(lit.contains("\\\\path"));
    }

    #[test]
    fn smoke_json_string_literal_escapes_lt_to_unicode_to_avoid_script_break() {
        // Embedding "</style>" inside a JS string literal that is later
        // injected into a <script> tag would otherwise terminate the script.
        let lit = json_string_literal("</style>");
        assert!(!lit.contains("</"));
        assert!(lit.contains("\\u003c"));
    }

    #[test]
    fn smoke_json_string_literal_escapes_control_characters() {
        let lit = json_string_literal("a\u{0001}b\u{001F}c");
        assert!(lit.contains("\\u0001"));
        assert!(lit.contains("\\u001f"));
    }

    #[test]
    fn paper_inches_known_papers() {
        // Smoke-test a couple of representative entries.
        let (w, h) = paper_inches("A4");
        assert!((w - 8.27).abs() < 0.01);
        assert!((h - 11.69).abs() < 0.01);

        let (w, h) = paper_inches("LETTER");
        assert_eq!(w, 8.5);
        assert_eq!(h, 11.0);
    }

    #[test]
    fn paper_inches_unknown_falls_back_to_a4() {
        let (w, h) = paper_inches("nonsense");
        assert!((w - 8.27).abs() < 0.01);
        assert!((h - 11.69).abs() < 0.01);
    }
}
