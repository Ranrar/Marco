//! Cross-platform HTML / JS builders for the "code view" preview surface.
//!
//! The code view is the read-only HTML-source companion to the rendered
//! Markdown preview. On Linux it is hosted in a `webkit6::WebView`; on
//! Windows the parity plan (§14.5 of `webkit6_wry_parity_audit.md`) calls
//! for the same syntect-highlighted HTML document to be hosted in a
//! `wry::WebView` via [`crate::components::viewer::wry_platform_webview::PlatformWebView`].
//!
//! Both platforms must produce **bit-identical** HTML and JS payloads so the
//! user experience is the same regardless of the underlying WebView engine.
//! This module is the single source of truth for those payloads.
//!
//! Two helpers are exposed:
//!
//! - [`build_full_page`] — returns the complete HTML document used at first
//!   load (passed to `load_html_with_base`).
//! - [`build_smooth_update_js`] — returns a JavaScript snippet that updates
//!   the existing page's syntect CSS, scrollbar CSS, body colors and
//!   highlighted code in place (passed to `evaluate_script`).
//!
//! Both helpers normalize `theme_mode` the same way (`"dark"` if it contains
//! the substring `"dark"`, otherwise `"light"`), generate syntect CSS via
//! [`crate::logic::syntax_highlighter::generate_css_with_global`], highlight
//! `html_source` as an `html` snippet via the global syntax highlighter, and
//! escape JS string content with [`escape_for_js_string`].

// On Windows these helpers are not yet wired into `wry.rs` (Step 5b of the
// parity audit). The Linux `webkit6` consumer keeps them live.
#![cfg_attr(target_os = "windows", allow(dead_code))]

use crate::components::viewer::css_utils;
use crate::logic::syntax_highlighter::{
    generate_css_with_global, global_syntax_highlighter, SYNTAX_HIGHLIGHTER,
};

/// Default placeholder used when the caller passes an empty `html_source`.
const EMPTY_PLACEHOLDER: &str = "<!-- No content yet -->";

/// Result of normalizing the user-facing theme name to syntect's two known modes.
fn normalize_theme(theme_mode: &str) -> &'static str {
    if theme_mode.contains("dark") {
        "dark"
    } else {
        "light"
    }
}

/// Resolve the body background / foreground colors for the code view.
///
/// If the caller provides both `editor_bg` and `editor_fg`, those win — this
/// keeps the code view visually synced with the editor's current theme.
/// Otherwise fall back to hard-coded Solarized-style defaults that match the
/// pre-refactor behaviour on both platforms.
fn resolve_colors<'a>(
    normalized_theme: &str,
    editor_bg: Option<&'a str>,
    editor_fg: Option<&'a str>,
) -> (&'a str, &'a str) {
    if let (Some(bg), Some(fg)) = (editor_bg, editor_fg) {
        (bg, fg)
    } else if normalized_theme == "dark" {
        ("#2b303b", "#c0c5ce")
    } else {
        // Solarized Light defaults.
        ("#fdf6e3", "#657b83")
    }
}

/// Build the syntect-highlighted HTML for the given source.
///
/// Initializes the global syntax highlighter on first use and forwards
/// errors as `String` so both webkit6 and wry callers see the same error
/// surface.
fn highlight_html_source(html_source: &str, normalized_theme: &str) -> Result<String, String> {
    global_syntax_highlighter()
        .map_err(|e| format!("Failed to initialize syntax highlighter: {}", e))?;

    let display_html = if html_source.is_empty() {
        EMPTY_PLACEHOLDER
    } else {
        html_source
    };

    SYNTAX_HIGHLIGHTER.with(|highlighter| {
        let h = highlighter.borrow();
        let syntax_highlighter = h
            .as_ref()
            .ok_or_else(|| "Syntax highlighter not initialized".to_string())?;

        syntax_highlighter
            .highlight_to_html(display_html, "html", normalized_theme)
            .map_err(|e| format!("Highlighting failed: {}", e))
    })
}

/// Escape a string for embedding as a single-quoted JavaScript string literal.
///
/// Mirrors the inline escape that lived in `webkit6.rs` so the produced JS
/// is identical across platforms.
fn escape_for_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Build the complete HTML document for the initial code-view load.
///
/// The document contains:
/// - Syntect-highlighted `html_source` inside `<pre><code>...</code></pre>`.
/// - A `<style>` block with the syntect theme CSS plus optional
///   webkit-scrollbar CSS so the scrollbar matches the editor.
/// - Body `background` / `color` resolved via [`resolve_colors`].
///
/// Callers should pass the returned string straight to the platform
/// `load_html_with_base` (Linux: webkit6, Windows: wry).
pub fn build_full_page(
    html_source: &str,
    theme_mode: &str,
    editor_bg: Option<&str>,
    editor_fg: Option<&str>,
    scrollbar_thumb: Option<&str>,
    scrollbar_track: Option<&str>,
) -> Result<String, String> {
    let normalized_theme = normalize_theme(theme_mode);

    let syntect_css = generate_css_with_global(normalized_theme)
        .map_err(|e| format!("Failed to generate CSS: {}", e))?;

    let highlighted_html = highlight_html_source(html_source, normalized_theme)?;

    let (bg_color, fg_color) = resolve_colors(normalized_theme, editor_bg, editor_fg);

    let scrollbar_css = match (scrollbar_thumb, scrollbar_track) {
        (Some(thumb), Some(track)) => css_utils::webkit_scrollbar_css(thumb, track),
        _ => String::new(),
    };

    Ok(format!(
        r#"<!DOCTYPE html>
<html style="height: 100%; margin: 0; padding: 0; overflow: hidden;">
  <head>
    <meta charset="UTF-8">
    <style>
      html, body {{
        height: 100%;
        margin: 0;
        padding: 0;
        overflow: hidden;
      }}
      body {{
        background: {bg};
        color: {fg};
        font-family: 'Fira Code', 'Monaco', 'Courier New', monospace;
        font-size: 12px;
        line-height: 1.5;
        display: flex;
        flex-direction: column;
      }}
      #code-container {{
        flex: 1;
        overflow: auto;
        padding: 16px;
        box-sizing: border-box;
      }}
      pre {{
        margin: 0;
        white-space: pre;
        word-wrap: normal;
      }}
      code {{
        font-family: inherit;
        white-space: pre;
      }}
      /* Syntect CSS */
      {syntect}
      /* Scrollbar styling */
      {scrollbar}
    </style>
  </head>
  <body>
    <div id="code-container">
      <pre><code>{code}</code></pre>
    </div>
  </body>
</html>"#,
        bg = bg_color,
        fg = fg_color,
        syntect = syntect_css,
        scrollbar = scrollbar_css,
        code = highlighted_html,
    ))
}

/// Build the JavaScript snippet that patches the live code-view document in
/// place — no full page reload, no white flash, scroll position preserved.
///
/// The script:
/// 1. Re-applies the body background / foreground colors.
/// 2. Updates (or creates) `<style id="marco-syntect-style">` with the latest
///    syntect CSS for the requested theme.
/// 3. Updates (or creates) `<style id="marco-scrollbar-style">` with the
///    latest webkit-scrollbar CSS (empty string when no scrollbar colors are
///    supplied — the previous styles, if any, are then cleared).
/// 4. Replaces the inner HTML of `pre code` with the freshly highlighted
///    source, saving and restoring `window.scrollY` around the swap.
pub fn build_smooth_update_js(
    html_source: &str,
    theme_mode: &str,
    editor_bg: Option<&str>,
    editor_fg: Option<&str>,
    scrollbar_thumb: Option<&str>,
    scrollbar_track: Option<&str>,
) -> Result<String, String> {
    let normalized_theme = normalize_theme(theme_mode);

    let syntect_css = generate_css_with_global(normalized_theme)
        .map_err(|e| format!("Failed to generate CSS: {}", e))?;

    let highlighted_html = highlight_html_source(html_source, normalized_theme)?;

    let (bg_color, fg_color) = resolve_colors(normalized_theme, editor_bg, editor_fg);

    let scrollbar_css = match (scrollbar_thumb, scrollbar_track) {
        (Some(thumb), Some(track)) => css_utils::webkit_scrollbar_css(thumb, track),
        _ => String::new(),
    };

    let escaped_html = escape_for_js_string(&highlighted_html);
    let escaped_css = escape_for_js_string(&syntect_css);
    let escaped_scrollbar_css = escape_for_js_string(&scrollbar_css);

    Ok(format!(
        r#"
        (function() {{
            try {{
                // Update body colors
                document.body.style.background = '{bg}';
                document.body.style.color = '{fg}';

                // Update syntect CSS
                var styleEl = document.getElementById('marco-syntect-style');
                if (!styleEl) {{
                    styleEl = document.createElement('style');
                    styleEl.id = 'marco-syntect-style';
                    document.head.appendChild(styleEl);
                }}
                styleEl.textContent = '{syntect}';

                // Update scrollbar CSS
                var scrollbarStyleEl = document.getElementById('marco-scrollbar-style');
                if (!scrollbarStyleEl) {{
                    scrollbarStyleEl = document.createElement('style');
                    scrollbarStyleEl.id = 'marco-scrollbar-style';
                    document.head.appendChild(scrollbarStyleEl);
                }}
                scrollbarStyleEl.textContent = '{scrollbar}';

                // Update code content
                var codeEl = document.querySelector('pre code');
                if (codeEl) {{
                    var scrollTop = window.scrollY;
                    codeEl.innerHTML = '{code}';
                    window.scrollTo(0, scrollTop);
                }} else {{
                    console.error('Code element not found');
                }}
            }} catch(e) {{
                console.error('Update failed:', e);
            }}
        }})();
        "#,
        bg = bg_color,
        fg = fg_color,
        syntect = escaped_css,
        scrollbar = escaped_scrollbar_css,
        code = escaped_html,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_normalize_theme_picks_dark_on_any_dark_substring() {
        assert_eq!(normalize_theme("dark"), "dark");
        assert_eq!(normalize_theme("solarized-dark"), "dark");
        assert_eq!(normalize_theme("light"), "light");
        assert_eq!(normalize_theme(""), "light");
    }

    #[test]
    fn smoke_test_resolve_colors_prefers_editor_overrides() {
        let (bg, fg) = resolve_colors("dark", Some("#000000"), Some("#ffffff"));
        assert_eq!(bg, "#000000");
        assert_eq!(fg, "#ffffff");
    }

    #[test]
    fn smoke_test_resolve_colors_falls_back_per_theme() {
        let (bg_dark, fg_dark) = resolve_colors("dark", None, None);
        assert_eq!(bg_dark, "#2b303b");
        assert_eq!(fg_dark, "#c0c5ce");
        let (bg_light, fg_light) = resolve_colors("light", None, None);
        assert_eq!(bg_light, "#fdf6e3");
        assert_eq!(fg_light, "#657b83");
    }

    #[test]
    fn smoke_test_escape_for_js_string_handles_quotes_backslashes_newlines() {
        assert_eq!(escape_for_js_string("a'b"), "a\\'b");
        assert_eq!(escape_for_js_string("a\\b"), "a\\\\b");
        assert_eq!(escape_for_js_string("a\nb"), "a\\nb");
        assert_eq!(escape_for_js_string("a\rb"), "a\\rb");
    }
}
