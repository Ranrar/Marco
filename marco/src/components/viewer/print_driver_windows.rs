//! Windows print driver (wry / WebView2).
//!
//! This module isolates the **native print dialog** path on Windows. PDF
//! export now flows through the unified `export_pipeline` (see
//! `WindowsExportBackend`) rather than living here, so the file is
//! intentionally small.

use crate::components::viewer::preview_types::PlatformWebView;

/// Open the native browser print UI for the current WebView content.
///
/// Before triggering the print UI, this injects the shared print-export CSS
/// (`marco_shared::logic::print_css::make_print_export_css`) into the live
/// WebView so paged.js layout maps cleanly to printer pages, matching the
/// Linux `print_driver::trigger_print_dialog` behavior.
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
}
