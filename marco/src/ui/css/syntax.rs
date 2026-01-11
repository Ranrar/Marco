//! Centralized syntax helpers for Marco UI.
//!
//! This module is the **single source of truth** for editor Markdown highlight
//! colors.
//!
//! The `core` crate produces highlight spans tagged with a small enum
//! (`core::lsp::HighlightTag`). In the editor we apply those spans as GTK
//! `TextTag`s on the SourceView buffer.
//!
//! The color palette for those `TextTag`s is defined here (light/dark maps).
//! This intentionally keeps editor syntax coloring **independent** from
//! SourceView style schemes and GTK CSS.

use gtk4::prelude::*;
use std::collections::HashMap;

/// All syntax tag names used by the editor highlight pipeline.
///
/// Keep this list in sync with:
/// - `marco/src/components/editor/lsp_integration.rs` (tag naming)
/// - `core::lsp::HighlightTag` (tag variants)
pub const LSP_TAG_NAMES: &[&str] = &[
    "heading1",
    "heading2",
    "heading3",
    "heading4",
    "heading5",
    "heading6",
    "emphasis",
    "strong",
    "strikethrough",
    "mark",
    "superscript",
    "subscript",
    "link",
    "image",
    "code-span",
    "code-block",
    "inline-html",
    "hard-break",
    "soft-break",
    "thematic-break",
    "blockquote",
    "html-block",
    "list",
    "list-item",
];

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
    m.insert("strikethrough", "#6E7781");
    m.insert("mark", "#9A6700");
    m.insert("superscript", "#8250DF");
    m.insert("subscript", "#8250DF");
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
    // Lists can be visually subtle; give them a slight tint so they read as a structure.
    m.insert("list", "#6E7781");
    m.insert("list-item", "#6E7781");
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
    m.insert("strikethrough", "#808080");
    m.insert("mark", "#D7BA7D");
    m.insert("superscript", "#C586C0");
    m.insert("subscript", "#C586C0");
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
    // Lists can be visually subtle; give them a slight tint so they read as a structure.
    m.insert("list", "#9CDCFE");
    m.insert("list-item", "#9CDCFE");
    m
}

// Note: We intentionally use hex color strings directly when setting
// TextTag foregrounds because `gtk4::TextTag::set_foreground` expects an
// Option<&str> containing the color (e.g. "#RRGGBB"). Keeping the hex
// strings avoids extra conversions and preserves fidelity with the XML
// theme files.

/// Apply LSP style tags (colors only) to the provided `sourceview5::Buffer`.
///
/// This will create `TextTag`s named like `heading1`, `emphasis`, `code-span`, etc.
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

    // Apply our hardcoded palette. (No scheme lookup: code-driven colors only.)
    for name in LSP_TAG_NAMES {
        let Some(hex) = map.get(name) else {
            log::warn!("Missing syntax color for tag '{name}' in theme_mode='{theme_mode}'");
            continue;
        };

        if let Some(existing) = tag_table.lookup(name) {
            existing.set_foreground(Some(hex));
        } else {
            let tag = gtk4::TextTag::new(Some(name));
            tag.set_foreground(Some(hex));
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
    let tag_table = buffer.tag_table();
    for name in LSP_TAG_NAMES {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_all_tag_names_have_light_colors() {
        let map = light_color_map();
        for name in LSP_TAG_NAMES {
            assert!(map.contains_key(name), "missing light color for '{name}'");
        }
    }

    #[test]
    fn smoke_test_all_tag_names_have_dark_colors() {
        let map = dark_color_map();
        for name in LSP_TAG_NAMES {
            assert!(map.contains_key(name), "missing dark color for '{name}'");
        }
    }
}
