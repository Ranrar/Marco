//! Windows-only native PDF export via WebView2's `ICoreWebView2_7::PrintToPdf`.
//!
//! This module replaces the previous headless-Chromium subprocess path used
//! for PDF export on Windows.  It runs entirely against the WebView2 COM
//! interface that is already embedded in our preview (`wry`'s underlying
//! `ICoreWebView2`), so:
//!
//! * No external Chromium / Edge install is required at runtime.
//! * No subprocess is spawned; `wait_with_pump` keeps the GTK / Win32 message
//!   loop responsive while the COM async operation completes, so the
//!   "Exporting…" progress dialog (`ui::dialogs::exporting`) animates
//!   smoothly.
//!
//! ## Concurrency
//!
//! All calls in this module must run on the GTK main thread (the same thread
//! that owns the `wry::WebView`).  COM async completion handlers fire via
//! `PostMessage` to the WebView2 thread, which is the same thread, so
//! `wait_with_pump` simply pumps Win32 messages until the result is delivered.
//!
//! ## Paper-size mapping
//!
//! WebView2 wants page width/height in **inches**; our public API takes the
//! human-friendly paper-name strings (`"A4"`, `"Letter"`, …) used by the
//! export dialog.  See [`paper_inches`].

use std::path::Path;
use std::sync::mpsc;

use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Environment6, ICoreWebView2_2, ICoreWebView2_7,
    COREWEBVIEW2_PRINT_ORIENTATION_LANDSCAPE, COREWEBVIEW2_PRINT_ORIENTATION_PORTRAIT,
};
use webview2_com::PrintToPdfCompletedHandler;
use windows::core::Interface;

use wry::WebViewExtWindows;

/// Convert a paper-name string (case-insensitive) to (width_in, height_in)
/// for *portrait* orientation.  Unknown names fall back to A4.
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
    // `wrap_html_document_paged`).  Asking WebView2's PrintToPdf to *also*
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

    // Convert path to a wide-string PCWSTR.  WebView2 requires an absolute
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
            .SetShouldPrintBackgrounds(true.into())
            .map_err(|e| format!("SetShouldPrintBackgrounds failed: {}", e))?;
        // We render our own page numbers via paged.js, so disable the
        // browser-injected header/footer.
        settings
            .SetShouldPrintHeaderAndFooter(false.into())
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
    // before the callback runs (it just kicks off the async op).  We still
    // need to drain the channel.  In webview2-com 0.38, `wait_for_async_operation`
    // internally pumps until the callback returns, so by this point `rx`
    // should have a value waiting.
    rx.recv()
        .map_err(|_| "PrintToPdf completion channel closed unexpectedly".to_string())?
        .map_err(|e| e)?;

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
