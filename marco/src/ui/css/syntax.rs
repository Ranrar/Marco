//! Centralized syntax helpers for Marco UI
//!
//! This module centralizes syntax (LSP) text color handling for Marco's editor.
//!
//! Instead of relying on separate XML files at runtime, we expose helpers that
//! can apply the colors directly to a `sourceview5::Buffer`'s tag table. This
//! keeps LSP text coloring independent from CSS and allows the UI to control
//! the exact colors used for syntax tags.
//!
//! The color maps were previously stored in
//! `assets/themes/syntax/syntaxlight.xml` and `syntaxdark.xml` — we keep the
//! same colors here to preserve behaviour while allowing programmatic control.

use gtk4::prelude::*;
use sourceview5::prelude::*; // bring BufferExt into scope for style_scheme()
use std::collections::HashMap;

/// Return a map of style name -> hex color string for the light theme.
fn light_color_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("heading1", "#0969DA");
    m.insert("heading2", "#1A7DC6");
    m.insert("heading3", "#2B8AB2");
    m.insert("heading4", "#3C979E");
    m.insert("heading5", "#4DA48A");
    m.insert("heading6", "#5EB176");
    m.insert("emphasis", "#A65E2B");
    m.insert("strong", "#CF222E");
    m.insert("link", "#0969DA");
    m.insert("image", "#0969DA");
    m.insert("code-span", "#0A3069");
    m.insert("code-block", "#6E7781");
    m.insert("inline-html", "#CF222E");
    m.insert("hard-break", "#6E7781");
    m.insert("soft-break", "#24292E");
    m.insert("thematic-break", "#8B7DAE");
    m.insert("blockquote", "#1A7F37");
    m.insert("html-block", "#CF222E");
    m.insert("list", "#24292E");
    m.insert("list-item", "#24292E");
    m
}

/// Return a map of style name -> hex color string for the dark theme.
fn dark_color_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("heading1", "#569CD6");
    m.insert("heading2", "#608CD6");
    m.insert("heading3", "#6A9CD6");
    m.insert("heading4", "#7AACD6");
    m.insert("heading5", "#8ABCD6");
    m.insert("heading6", "#9CDCFE");
    m.insert("emphasis", "#DCDCAA");
    m.insert("strong", "#CE9178");
    m.insert("link", "#4EC9B0");
    m.insert("image", "#4EC9B0");
    m.insert("code-span", "#B5CEA8");
    m.insert("code-block", "#CE9178");
    m.insert("inline-html", "#D16969");
    m.insert("hard-break", "#808080");
    m.insert("soft-break", "#D4D4D4");
    m.insert("thematic-break", "#C586C0");
    m.insert("blockquote", "#608B4E");
    m.insert("html-block", "#D16969");
    m.insert("list", "#D4D4D4");
    m.insert("list-item", "#D4D4D4");
    m
}

// Note: We intentionally use hex color strings directly when setting
// TextTag foregrounds because `gtk4::TextTag::set_foreground` expects an
// Option<&str> containing the color (e.g. "#RRGGBB"). Keeping the hex
// strings avoids extra conversions and preserves fidelity with the XML
// theme files.

/// Apply LSP style tags (colors only) to the provided `sourceview5::Buffer`.
///
/// This will create TextTags named like `lsp-heading1`, `lsp-emphasis`, etc.
/// If tags already exist they will be updated with the new foreground color.
pub fn apply_to_buffer(buffer: &sourceview5::Buffer, theme_mode: &str) {
    // Accept theme strings like "dark", "theme-dark", "theme-dark-foo".
    // Many callers pass values like "theme-dark" so we check for containment
    // instead of requiring an exact match.
    let map = if theme_mode.contains("dark") {
        dark_color_map()
    } else {
        light_color_map()
    };

    let tag_table = buffer.tag_table();

    // Try to prefer colors from the buffer's StyleScheme if available. Many
    // editor theme XMLs define styles named `lsp-{name}`; if a matching style
    // exists and has a foreground color, use that. Otherwise fall back to our
    // hardcoded hex map.
    let scheme_opt = buffer.style_scheme();

    for (name, hex) in map.iter() {
        // Prefer scheme style `lsp-{name}`
        let foreground_color = scheme_opt
            .as_ref()
            .and_then(|scheme| scheme.style(&format!("lsp-{}", name)))
            .and_then(|style| style.foreground())
            .map(|gstr| gstr.to_string())
            .unwrap_or_else(|| hex.to_string());

        if let Some(existing) = tag_table.lookup(name) {
            existing.set_foreground(Some(&foreground_color));
        } else {
            let tag = gtk4::TextTag::new(Some(name));
            tag.set_foreground(Some(&foreground_color));
            tag_table.add(&tag);
        }
    }
}

/// Remove or reset LSP style tag foregrounds from the provided `sourceview5::Buffer`.
///
/// This will clear the `foreground` property for any of the tags we create so the
/// editor falls back to default text color. We iterate the union of known tag names
/// to make sure both light and dark variants are covered.
pub fn remove_from_buffer(buffer: &sourceview5::Buffer) {
    let mut names = Vec::new();
    names.extend(light_color_map().keys().cloned());
    names.extend(dark_color_map().keys().cloned());

    let tag_table = buffer.tag_table();
    for name in names {
        if let Some(tag) = tag_table.lookup(name) {
            // Clearing the foreground will make the TextTag not override the default
            // text color used by the SourceView.
            tag.set_foreground(None::<&str>);
        }
    }
}

/// Generate a small CSS snippet for the UI CSS generator.
/// This keeps the module consistent with other `marco/src/ui/css/*` modules.
pub fn generate_css() -> String {
    // Syntax module doesn't contribute large GTK CSS; return a small marker
    String::from("/* syntax module: provides LSP tag colors for SourceView */\n")
}
