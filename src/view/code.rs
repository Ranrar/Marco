use crate::theme::ThemeManager;
use gtk4::prelude::*;
use gtk4::{ScrolledWindow, TextBuffer, TextTagTable, TextView, Widget};
// use pulldown_cmark::{Parser, Options, html};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct MarkdownCodeView {
    widget: ScrolledWindow,
    text_view: TextView,
    theme_manager: Rc<RefCell<Option<ThemeManager>>>,
}

impl MarkdownCodeView {
    /// Focus the TextView for keyboard input
    pub fn grab_focus(&self) {
        self.text_view.grab_focus();
    }
    pub fn new() -> Self {
        eprintln!("[DEBUG] MarkdownCodeView::new() called");
        let text_view = TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);
        text_view.set_monospace(true); // Use monospace font for code display

        // Set up text view styling
        text_view.set_left_margin(15);
        text_view.set_right_margin(15);
        text_view.set_top_margin(15);
        text_view.set_bottom_margin(15);

        // Create tag table and buffer with syntax highlighting support
        let tag_table = TextTagTable::new();
        let buffer = TextBuffer::new(Some(&tag_table));
        text_view.set_buffer(Some(&buffer));

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_child(Some(&text_view));
        scrolled_window.set_vexpand(true);
        scrolled_window.set_hexpand(true);
        scrolled_window.set_size_request(200, -1); // Minimum width of 200px


        Self {
            widget: scrolled_window,
            text_view,
            theme_manager: Rc::new(RefCell::new(None)),
        }
    }

    /// Sets up the preview context menu for the Code view
    pub fn setup_context_menu(&self, editor: &crate::editor::MarkdownEditor) {
        let preview_menu = crate::view::context_menu::PreviewContextMenu::new();
        preview_menu.setup_gesture_for_widget(&self.text_view, editor);
    }

    pub fn widget(&self) -> &Widget {
        eprintln!("[DEBUG] MarkdownCodeView::widget() called");
        self.widget.upcast_ref()
    }

    pub fn update_content(&self, markdown_text: &str) {
        eprintln!("[DEBUG] update_content called with input: {} chars", markdown_text.len());
        // Convert markdown to HTML
        use pulldown_cmark::{Parser, Options, html};
        let mut html_output = String::new();
        let parser = Parser::new_ext(markdown_text, Options::all());
        html::push_html(&mut html_output, parser);

        // Wrap the HTML in a full HTML5 document with title, meta, etc.
        let full_html = self.format_as_complete_html_document(&html_output);

        // The indent_html method was removed; use full_html directly
        let indented_html = &full_html;

        // Syntax highlight the HTML code using syntect
        use syntect::easy::HighlightLines;
        use syntect::highlighting::ThemeSet;
        use syntect::parsing::SyntaxSet;
        use gtk4::TextTag;
        use gtk4::gdk::RGBA;

        let buffer = self.text_view.buffer();
        let tag_table = buffer.tag_table();
        buffer.set_text("");

        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        // Use a dark theme if available, fallback to any
        let theme = ts.themes.get("base16-ocean.dark").or_else(|| ts.themes.values().next()).unwrap();
        let syntax = ps.find_syntax_by_extension("html").unwrap_or_else(|| ps.find_syntax_plain_text());
        let mut h = HighlightLines::new(syntax, theme);

        for line in indented_html.lines() {
            match h.highlight_line(line, &ps) {
                Ok(ranges) => {
                    for (style, text) in ranges {
                        if text.is_empty() {
                            continue;
                        }
                        let tag_name = format!("fg#{:02x}{:02x}{:02x}", style.foreground.r, style.foreground.g, style.foreground.b);
                        let tag = if let Some(tag) = tag_table.lookup(&tag_name) {
                            tag
                        } else {
                            let rgba = RGBA::new(
                                style.foreground.r as f32 / 255.0,
                                style.foreground.g as f32 / 255.0,
                                style.foreground.b as f32 / 255.0,
                                1.0,
                            );
                            let tag = TextTag::builder().name(&tag_name)
                                .foreground_rgba(&rgba)
                                .build();
                            tag_table.add(&tag);
                            tag
                        };
                        let mut insert_iter = buffer.end_iter();
                        buffer.insert_with_tags(&mut insert_iter, text, &[&tag]);
                    }
                    // Add newline (not highlighted)
                    let mut insert_iter = buffer.end_iter();
                    buffer.insert(&mut insert_iter, "\n");
                }
                Err(_) => {
                    // Fallback: insert plain text
                    let mut insert_iter = buffer.end_iter();
                    buffer.insert(&mut insert_iter, line);
                    buffer.insert(&mut insert_iter, "\n");
                }
            }
        }
    }


    /// Format HTML content as a complete W3C-standard HTML document
    fn format_as_complete_html_document(&self, content: &str) -> String {
        // Extract title if present, otherwise use a default
        let title = if let Some(title_start) = content.find("<title>") {
            if let Some(title_end) = content[title_start + 7..].find("</title>") {
                &content[title_start + 7..title_start + 7 + title_end]
            } else {
                "Markdown Document"
            }
        } else {
            "Markdown Document"
        };

        // Create a full HTML document with proper indentation and structure
        format!{
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Markdown document generated by Marco">
    <meta name="generator" content="Marco - markdown Composer">
    <meta name="github" content="https://github.com/Ranrar/Marco">
    <title>{}</title>
  </head>
  <body>
    <main>
{}
    </main>
  </body>
</html>"#,
            title,
            // Indent content appropriately
            content
                .lines()
                .map(|line| format!("      {}", line))
                .collect::<Vec<String>>()
                .join("\n")
        }
    }


    #[allow(dead_code)]
    pub fn get_text_view(&self) -> &TextView {
        &self.text_view
    }

    pub fn set_theme_manager(&self, theme_manager: ThemeManager) {
        *self.theme_manager.borrow_mut() = Some(theme_manager);

        // Update the view styling based on theme
        self.update_theme_styling();
    }

    /// Update the text view styling based on the current theme
    fn update_theme_styling(&self) {
        if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            // Apply theme-specific styling
            let text_view = &self.text_view;

            match theme_manager.get_effective_theme() {
                crate::theme::Theme::Light => {
                    // Light theme
                    text_view.set_css_classes(&["theme-light"]);
                    text_view.add_css_class("theme-light");
                    text_view.remove_css_class("theme-dark");

                    // Set monospace font and styling
                    text_view.set_monospace(true);
                    text_view.set_top_margin(15);
                    text_view.set_bottom_margin(15);
                    text_view.set_left_margin(15);
                    text_view.set_right_margin(15);
                }
                crate::theme::Theme::Dark => {
                    // Dark theme
                    text_view.set_css_classes(&["theme-dark"]);
                    text_view.add_css_class("theme-dark");
                    text_view.remove_css_class("theme-light");

                    // Set monospace font and styling
                    text_view.set_monospace(true);
                    text_view.set_top_margin(15);
                    text_view.set_bottom_margin(15);
                    text_view.set_left_margin(15);
                    text_view.set_right_margin(15);
                }
                crate::theme::Theme::System => {
                    // System theme (detect and apply appropriate theme)
                    let system_theme = crate::theme::ThemeManager::detect_system_theme();
                    match system_theme {
                        crate::theme::Theme::Dark => {
                            text_view.set_css_classes(&["theme-dark"]);
                            text_view.add_css_class("theme-dark");
                            text_view.remove_css_class("theme-light");
                        }
                        _ => {
                            text_view.set_css_classes(&["theme-light"]);
                            text_view.add_css_class("theme-light");
                            text_view.remove_css_class("theme-dark");
                        }
                    }

                    // Set monospace font and styling
                    text_view.set_monospace(true);
                    text_view.set_top_margin(15);
                    text_view.set_bottom_margin(15);
                    text_view.set_left_margin(15);
                    text_view.set_right_margin(15);
                }    
            }        
        }
    }
}