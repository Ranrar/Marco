//! Footer module for Marco markdown editor
//! 
//! This module provides footer functionality for displaying various editor states including:
//! - Cursor position (row and column)
//! - Line count 
//! - Word and character counts
//! - Current encoding
//! - Insert/overwrite mode
//! - Markdown syntax trace for the current line
//!
//! ## Threading Safety
//! All footer update functions are designed to be thread-safe. They use `set_label_text` 
//! helper which automatically detects whether it's running on the main GTK thread and 
//! schedules updates using `glib::idle_add_local` if necessary.
//!
//! ## Usage
//! Footer updates can be triggered individually using specific update functions, or in 
//! batch using `apply_footer_update` with a `FooterUpdate::Snapshot`.

use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation};
use crate::logic::parser::{parse_document_blocks, MarkdownSyntaxMap};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static UPDATE_VIS_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Gate footer debug output behind an env var so normal runs are quiet.
#[macro_export]
macro_rules! footer_dbg {
    ($($arg:tt)*) => {{
        if std::env::var("MARCO_DEBUG_FOOTER").is_ok() {
            eprintln!($($arg)*);
        }
    }};
}

/// Message type used to update the footer from any thread via a MainContext channel
#[derive(Debug)]
pub enum FooterUpdate {
    Snapshot {
        row: usize,
        col: usize,
    // lines removed
        words: usize,
        chars: usize,
        syntax_display: String,
        encoding: String,
        is_insert: bool,
    },
}

#[derive(Clone)]
pub struct FooterLabels {
    pub cursor_row: Label,
    pub cursor_col: Label,
    pub encoding: Label,
    pub insert_mode: Label,
    pub formatting: Label,
    pub word_count: Label,
    pub char_count: Label,
}

/// Updates the formatting label with the Markdown syntax trace for the active line
/// Pure helper: produce the display string for a line given a syntax map
pub fn format_syntax_trace(line: &str, syntax_map: &MarkdownSyntaxMap) -> String {
    // Run a small block-level pre-pass so Setext, frontmatter and link-defs
    // are detected even when only a single line is passed. We ignore the
    // aggregated link definitions for the footer display, but calling the
    // function ensures `collect_link_definitions` and friends are exercised
    // and avoids dead-code warnings.
    let (chain, _link_defs) = parse_document_blocks(line, syntax_map);

    // Debug-only: if MARCO_DEBUG_FOOTER_TRACE is set, print detailed token info to stderr.
    if std::env::var("MARCO_DEBUG_FOOTER_TRACE").is_ok() {
        eprintln!("[footer trace] input line: {:?}", line);
        for (i, t) in chain.iter().enumerate() {
            // Print core fields we care about. Use Debug formatting for Options.
            eprintln!(
                "[footer trace] token {}: node_type='{}', depth={:?}, ordered={:?}'",
                i, t.node_type, t.depth, t.ordered
            );
        }
    }

    if chain.is_empty() {
        "Format: Plain text".to_string()
    } else {
        // Build a more informative node trace using display hints from the schema
        let hints_map = syntax_map.build_display_hints();
        let parts: Vec<String> = chain.iter().map(|t| {
            // Helper to fetch a capture value if present
            let cap = |name: &str| -> Option<String> {
                t.captures.as_ref().and_then(|c| c.get(name)).cloned()
            };

            if let Some(hints) = hints_map.get(&t.node_type) {
                for hint in hints {
                    if let Some(val) = cap(hint) {
                        // For link hints, include target if present
                        if hint == "h" {
                            let tval = cap("t").unwrap_or_default();
                            return format!("{} → {}", val, tval);
                        }
                        return val;
                    }
                }
            }

            // Fallbacks for structured tokens
            if let Some(d) = t.depth {
                format!("{}({})", t.node_type, d)
            } else if let Some(ord) = t.ordered {
                format!("{}({})", t.node_type, if ord { "ordered" } else { "unordered" })
            } else {
                t.node_type.clone()
            }
        }).collect();

        // If frontmatter was captured, extract top-level key:value pairs (first 3)
        let mut extras: Vec<String> = Vec::new();
        for t in &chain {
            if t.node_type == "frontmatter" {
                if let Some(caps) = &t.captures {
                    if let Some(value) = caps.get("value") {
                        // Collect key:value pairs like `title: Value` from the frontmatter
                        let kv_re = regex::Regex::new(r"(?m)^\s*(?P<key>[A-Za-z0-9_-]+)\s*:\s*(?P<val>.+)\s*$").unwrap();
                        let mut pairs: Vec<String> = Vec::new();
                        for kc in kv_re.captures_iter(value).take(3) {
                            if let (Some(k), Some(v)) = (kc.name("key"), kc.name("val")) {
                                // Truncate long values for footer readability
                                let mut val = v.as_str().trim().to_string();
                                if val.len() > 30 {
                                    val.truncate(27);
                                    val.push_str("...");
                                }
                                pairs.push(format!("{}: {}", k.as_str(), val));
                            }
                        }
                        if !pairs.is_empty() {
                            extras.push(format!("Frontmatter: {}", pairs.join(", ")));
                        } else {
                            extras.push("Frontmatter".to_string());
                        }
                    }
                }
            }
        }

        // Append up to 3 link id->url examples from the aggregated defs
        if !_link_defs.is_empty() {
            let mut examples: Vec<String> = Vec::new();
            for (id, (url, _title)) in _link_defs.iter().take(3) {
                let mut short = url.clone();
                if short.len() > 40 {
                    short.truncate(37);
                    short.push_str("...");
                }
                examples.push(format!("{} → {}", id, short));
            }
            if !examples.is_empty() {
                extras.push(format!("Links: {}", examples.join(", ")));
            }
        }

        let base = format!("Format: {}", parts.join(" → "));
        if extras.is_empty() {
            base
        } else {
            format!("{} — {}", base, extras.join("; "))
        }
    }
}

/// Update the row label independently
pub fn update_cursor_row(labels: &FooterLabels, row: usize) {
    let text = format!("Row: {}", row);
    footer_dbg!("[footer] update_cursor_row called: {}", text);
    set_label_text(&labels.cursor_row, text);
}

/// Update the column label independently
pub fn update_cursor_col(labels: &FooterLabels, col: usize) {
    let text = format!("Column: {}", col);
    footer_dbg!("[footer] update_cursor_col called: {}", text);
    set_label_text(&labels.cursor_col, text);
}

// line count removed: no-op omitted

/// Updates the encoding label
pub fn update_encoding(labels: &FooterLabels, encoding: &str) {
    let enc = encoding.to_string();
    footer_dbg!("[footer] update_encoding called: {}", enc);
    set_label_text(&labels.encoding, enc);
}

/// Updates the insert/overwrite mode label
pub fn update_insert_mode(labels: &FooterLabels, is_insert: bool) {
    let text = if is_insert { "INS" } else { "OVR" };
    footer_dbg!("[footer] update_insert_mode called: {}", text);
    set_label_text(&labels.insert_mode, text.to_string());
}

/// Updates the formatting label with the Markdown syntax trace for the active line
pub fn update_syntax_trace(labels: &FooterLabels, line: &str, syntax_map: &MarkdownSyntaxMap) {
    let display = format_syntax_trace(line, syntax_map);
    footer_dbg!("[footer] update_syntax_trace called: {}", display);
    set_label_text(&labels.formatting, display);
}
/// Updates the word count label
pub fn update_word_count(labels: &FooterLabels, words: usize) {
    let text = format!("Words: {}", words);
    footer_dbg!("[footer] update_word_count called: {}", text);
    set_label_text(&labels.word_count, text);
}

/// Updates the character count label
pub fn update_char_count(labels: &FooterLabels, chars: usize) {
    let text = format!("Characters: {}", chars);
    footer_dbg!("[footer] update_char_count called: {}", text);
    set_label_text(&labels.char_count, text);
}

/// Apply a FooterUpdate snapshot to the labels. Must be called on main context.
pub fn apply_footer_update(labels: &FooterLabels, update: FooterUpdate) {
    match update {
    FooterUpdate::Snapshot { row, col, /*lines,*/ words, chars, syntax_display, encoding, is_insert } => {
            update_cursor_row(labels, row);
            update_cursor_col(labels, col);
            update_word_count(labels, words);
            update_char_count(labels, chars);
            // Use consistent pattern: call the proper update function instead of set_label_text directly
            footer_dbg!("[footer] apply_footer_update called for syntax_display: {}", syntax_display);
            set_label_text(&labels.formatting, syntax_display);
            update_encoding(labels, &encoding);
            update_insert_mode(labels, is_insert);
        }
    }
}

/// Helper: set a Label's text on the main GTK context, scheduling if needed.
/// This function ensures thread safety and provides consistent label updating.
fn set_label_text(label: &Label, text: String) {
    let mut final_text = text.clone();
    
    // If debug env var set, append a small counter so updates are visually detectable
    if std::env::var("MARCO_DEBUG_FOOTER_VIS").is_ok() {
        let n = UPDATE_VIS_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
        final_text = format!("{} [{}]", text, n);
    }

    let use_markup = std::env::var("MARCO_DEBUG_FOOTER_VIS").is_ok();
    
    // Check if we're on the main thread
    if glib::MainContext::default().is_owner() {
        // We're on the main thread, update immediately
        update_label_immediate(label, &final_text, use_markup);
    } else {
        // We're not on the main thread, schedule the update
        let lbl = label.clone();
        glib::idle_add_local(move || {
            update_label_immediate(&lbl, &final_text, use_markup);
            glib::ControlFlow::Break
        });
    }
}

/// Immediately update a label on the main thread
fn update_label_immediate(label: &Label, text: &str, use_markup: bool) {
    if use_markup {
        // Escape and set markup for a bold visual (debug mode)
        let escaped_text = glib::markup_escape_text(text);
        label.set_markup(&format!("<b>{}</b>", escaped_text));
    } else {
        label.set_text(text);
    }
    
    footer_dbg!("[footer] set_label_text immediate -> {}", label.text());
    footer_dbg!("[footer] label visible: {}, parent visible: {}", 
        label.is_visible(), 
        label.parent().map(|p| p.is_visible()).unwrap_or(false));
    
    // Ensure widget is visible and request a redraw for better reliability
    label.set_visible(true);
    // Avoid calling queue_draw() directly here; GTK may issue warnings when widgets
    // are not yet allocated. Rely on set_visible and normal GTK redraw scheduling.
    
    // Also ensure parent is visible
    if let Some(parent) = label.parent() {
        parent.set_visible(true);
    }
}

pub fn create_footer() -> (Box, Rc<FooterLabels>) {
    let footer_box = Box::new(Orientation::Horizontal, 10);
    footer_box.set_margin_top(5);
    footer_box.set_margin_bottom(5);
    footer_box.set_margin_start(10);
    footer_box.set_margin_end(10);
    
    // Ensure footer is visible and properly allocated
    footer_box.set_visible(true);
    footer_box.set_can_focus(false);
    footer_box.set_vexpand(false);
    footer_box.set_hexpand(true);
    footer_box.set_height_request(0); // Minimum height
    
    // Add CSS class for potential styling
    footer_box.add_css_class("footer");

    // Formatting label (left side)
    let formatting_label = Label::new(Some("Format:"));
    formatting_label.set_halign(gtk4::Align::Start);
    formatting_label.set_xalign(0.0);
    formatting_label.set_visible(true);
    footer_box.append(&formatting_label);

    // Spacer to push items to the sides
    let spacer = Label::new(None);
    spacer.set_hexpand(true);
    spacer.set_visible(true);
    footer_box.append(&spacer);

    // Info labels (right side)
    let word_count_label = Label::new(Some("Words: 0"));
    word_count_label.set_visible(true);
    footer_box.append(&word_count_label);

    let char_count_label = Label::new(Some("Characters: 0"));
    char_count_label.set_visible(true);
    footer_box.append(&char_count_label);

    let cursor_row_label = Label::new(Some("Row 1"));
    cursor_row_label.set_visible(true);
    footer_box.append(&cursor_row_label);

    let cursor_col_label = Label::new(Some("Column 1"));
    cursor_col_label.set_visible(true);
    footer_box.append(&cursor_col_label);


    let encoding_label = Label::new(Some("UTF-8"));
    encoding_label.set_visible(true);
    footer_box.append(&encoding_label);

    let insert_mode_label = Label::new(Some("INS"));
    insert_mode_label.set_visible(true);
    footer_box.append(&insert_mode_label);

    let labels = FooterLabels {
        cursor_row: cursor_row_label,
        cursor_col: cursor_col_label,
        encoding: encoding_label,
        insert_mode: insert_mode_label,
        formatting: formatting_label,
        word_count: word_count_label,
        char_count: char_count_label,
    };

    (footer_box, Rc::new(labels))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_test_map() -> MarkdownSyntaxMap {
        let mut rules = std::collections::HashMap::new();
    rules.insert("**".to_string(), crate::logic::parser::SyntaxRule { node_type: "bold".to_string(), depth: None, ordered: None, markdown_syntax: "**".to_string(), is_regex: false, regex: None });
    rules.insert("#".to_string(), crate::logic::parser::SyntaxRule { node_type: "heading".to_string(), depth: Some(1), ordered: None, markdown_syntax: "#".to_string(), is_regex: false, regex: None });
    rules.insert("##".to_string(), crate::logic::parser::SyntaxRule { node_type: "heading".to_string(), depth: Some(2), ordered: None, markdown_syntax: "##".to_string(), is_regex: false, regex: None });
    rules.insert("*".to_string(), crate::logic::parser::SyntaxRule { node_type: "italic".to_string(), depth: None, ordered: None, markdown_syntax: "*".to_string(), is_regex: false, regex: None });
    rules.insert("-".to_string(), crate::logic::parser::SyntaxRule { node_type: "list".to_string(), depth: None, ordered: Some(false), markdown_syntax: "-".to_string(), is_regex: false, regex: None });
    rules.insert("1.".to_string(), crate::logic::parser::SyntaxRule { node_type: "list".to_string(), depth: None, ordered: Some(true), markdown_syntax: "1.".to_string(), is_regex: false, regex: None });
    MarkdownSyntaxMap { rules, display_hints: None }
    }

    #[test]
    fn test_format_syntax_trace_plain() {
        let map = make_test_map();
        let out = format_syntax_trace("plain text", &map);
        assert_eq!(out, "Format: Plain text");
    }

    #[test]
    fn test_format_syntax_trace_complex() {
        let map = make_test_map();
        
        // Test heading with bold
        let out = format_syntax_trace("# **Bold heading**", &map);
        assert!(out.starts_with("Format: "));
        assert!(out.contains("heading(1)") || out.contains("bold"));
        
        // Test list with italic
        let out2 = format_syntax_trace("- *italic item*", &map);
        assert!(out2.starts_with("Format: "));
        assert!(out2.contains("list") || out2.contains("italic"));
        
        // Test heading depth
        let out3 = format_syntax_trace("## Level 2 heading", &map);
        assert!(out3.contains("heading(2)"));
        
        // Test ordered list
        let out4 = format_syntax_trace("1. ordered item", &map);
        assert!(out4.contains("list(ordered)"));
        
        // Test unordered list  
        let out5 = format_syntax_trace("- unordered item", &map);
        assert!(out5.contains("list(unordered)"));
    }

    #[test]
    fn test_format_syntax_trace_empty() {
        let map = make_test_map();
        let out = format_syntax_trace("", &map);
        assert_eq!(out, "Format: Plain text");
    }

    #[test]
    fn test_footer_update_functions_update_labels() {
        // Initialize GTK for tests that create widgets. If GTK is already initialized,
        // this is a no-op. If GTK cannot be initialized (e.g., no display), skip the test
        if gtk4::is_initialized() == false {
            if let Err(_) = gtk4::init() {
                footer_dbg!("Skipping GTK test - no display available");
                return;
            }
        }

        // Create Label widgets and a FooterLabels instance
    let formatting_label = gtk4::Label::new(Some(""));
    let word_count_label = gtk4::Label::new(Some(""));
    let char_count_label = gtk4::Label::new(Some(""));
    let cursor_row_label = gtk4::Label::new(Some(""));
    let cursor_col_label = gtk4::Label::new(Some(""));
    // line_count removed
        let encoding_label = gtk4::Label::new(Some(""));
        let insert_mode_label = gtk4::Label::new(Some(""));

        let labels = FooterLabels {
            cursor_row: cursor_row_label.clone(),
            cursor_col: cursor_col_label.clone(),
            encoding: encoding_label.clone(),
            insert_mode: insert_mode_label.clone(),
            formatting: formatting_label.clone(),
            word_count: word_count_label.clone(),
            char_count: char_count_label.clone(),
        };

        // Call update helpers
    update_cursor_row(&labels, 3);
    update_cursor_col(&labels, 7);
    // update_line_count removed
        update_encoding(&labels, "UTF-16");
        update_insert_mode(&labels, false);
        update_word_count(&labels, 123);
        update_char_count(&labels, 456);

        // Formatting update uses parse helper; test for plain text behavior
        let map = make_test_map();
        update_syntax_trace(&labels, "plain text", &map);

        // Verify Label texts
    assert!(cursor_row_label.text().contains("Row: 3"));
    assert!(cursor_col_label.text().contains("Column: 7"));
    // line count assertions removed
        assert_eq!(encoding_label.text().as_str(), "UTF-16");
        assert_eq!(insert_mode_label.text().as_str(), "OVR");
        assert_eq!(word_count_label.text().as_str(), "Words: 123");
        assert_eq!(char_count_label.text().as_str(), "Characters: 456");
        assert!(formatting_label.text().starts_with("Format:"));
    }

    #[test]
    fn test_apply_footer_update_snapshot() {
        if gtk4::is_initialized() == false {
            if let Err(_) = gtk4::init() {
                footer_dbg!("Skipping GTK test - no display available");
                return;
            }
        }

        let formatting_label = gtk4::Label::new(Some(""));
        let word_count_label = gtk4::Label::new(Some(""));
        let char_count_label = gtk4::Label::new(Some(""));
        let cursor_row_label = gtk4::Label::new(Some(""));
        let cursor_col_label = gtk4::Label::new(Some(""));
    // line_count removed
        let encoding_label = gtk4::Label::new(Some(""));
        let insert_mode_label = gtk4::Label::new(Some(""));

        let labels = FooterLabels {
            cursor_row: cursor_row_label.clone(),
            cursor_col: cursor_col_label.clone(),
            encoding: encoding_label.clone(),
            insert_mode: insert_mode_label.clone(),
            formatting: formatting_label.clone(),
            word_count: word_count_label.clone(),
            char_count: char_count_label.clone(),
        };

        let update = FooterUpdate::Snapshot {
            row: 5,
            col: 10,
            // lines removed
            words: 200,
            chars: 1000,
            syntax_display: "Format: Test syntax".to_string(),
            encoding: "UTF-8".to_string(),
            is_insert: true,
        };

        apply_footer_update(&labels, update);

    // Verify all labels were updated via the snapshot
    assert!(cursor_row_label.text().contains("Row: 5"));
    assert!(cursor_col_label.text().contains("Column: 10"));
        assert!(word_count_label.text().contains("Words: 200"));
        assert!(char_count_label.text().contains("Characters: 1000"));
        assert!(formatting_label.text().contains("Format: Test syntax"));
        assert_eq!(encoding_label.text().as_str(), "UTF-8");
        assert_eq!(insert_mode_label.text().as_str(), "INS");
    }
}