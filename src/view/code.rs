use gtk4::prelude::*;
use gtk4::{ScrolledWindow, TextView, Widget, TextBuffer, TextTagTable};
use pulldown_cmark::{Parser, Options, html, Event, Tag, CodeBlockKind};
use crate::markdown::syntect::CodeLanguageManager;
use crate::theme::ThemeManager;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct MarkdownCodeView {
    widget: ScrolledWindow,
    text_view: TextView,
    language_manager: CodeLanguageManager,
    theme_manager: Rc<RefCell<Option<ThemeManager>>>,
}

impl MarkdownCodeView {
    pub fn new() -> Self {
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
            language_manager: CodeLanguageManager::new(),
            theme_manager: Rc::new(RefCell::new(None)),
        }
    }
    
    /// Sets up the preview context menu for the Code view
    pub fn setup_context_menu(&self, editor: &crate::editor::MarkdownEditor) {
        let preview_menu = crate::view::context_menu::PreviewContextMenu::new();
        preview_menu.setup_gesture_for_widget(&self.text_view, editor);
    }

    pub fn widget(&self) -> &Widget {
        self.widget.upcast_ref()
    }

    pub fn update_content(&self, markdown_text: &str) {
        // Use the same syntax highlighting pipeline as the HTML view
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        
        let parser = Parser::new_ext(markdown_text, options);
        let mut html_content = String::new();
        
        // Process events with syntax highlighting to match the HTML view exactly
        let events = self.process_events_with_code_highlighting(parser);
        html::push_html(&mut html_content, events.into_iter());
        
        // Format HTML as a full W3C-standard page with proper indentation
        // NOTE: The HTML content from html::push_html is already escaped properly
        let formatted_html = self.format_as_complete_html_document(&html_content);
        
        // Apply syntax highlighting directly to the HTML content using the HTML language
        // No need to escape again as the HTML is already properly escaped
        let highlighted_html = self.language_manager.highlight_code(&formatted_html, "html");
        
        // Set the HTML content in the text view
        let preview_buffer = self.text_view.buffer();
        
        // For a purely monospace display of the highlighted HTML:
        // Convert HTML to plain text (removing HTML tags from the highlighted HTML)
        let plain_text = self.html_to_plain_text(&highlighted_html);
        preview_buffer.set_text(&plain_text);
        
        // Update text view font and styling based on theme
        self.update_theme_styling();
    }

    /// Process pulldown-cmark events to add syntax highlighting to code blocks
    /// This mirrors the implementation in view_html.rs but with specific HTML handling
    fn process_events_with_code_highlighting<'a>(&self, parser: Parser<'a, 'a>) -> Vec<Event<'a>> {
        let mut events = Vec::new();
        let mut in_code_block = false;
        let mut code_block_lang = String::new();
        let mut code_block_content = String::new();
        
        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                    in_code_block = true;
                    code_block_lang = lang.to_string();
                    code_block_content.clear();
                    // Don't push the original start tag, we'll create our own
                }
                Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                    if in_code_block {
                        in_code_block = false;
                        
                        // Special handling for HTML code blocks
                        if code_block_lang.to_lowercase() == "html" {
                            // For HTML code blocks, just preserve them with proper escaping
                            // They will be processed later in update_content
                            let highlighted_html = self.generate_highlighted_code_block(&code_block_content, "html");
                            events.push(Event::Html(highlighted_html.into()));
                        } else {
                            // Regular code blocks get normal syntax highlighting
                            let highlighted_html = self.generate_highlighted_code_block(&code_block_content, &code_block_lang);
                            events.push(Event::Html(highlighted_html.into()));
                        }
                        
                        code_block_content.clear();
                        code_block_lang.clear();
                    }
                }
                Event::End(Tag::CodeBlock(_)) => {
                    // Handle indented code blocks
                    if in_code_block {
                        in_code_block = false;
                        let highlighted_html = self.generate_highlighted_code_block(&code_block_content, &code_block_lang);
                        events.push(Event::Html(highlighted_html.into()));
                        code_block_content.clear();
                        code_block_lang.clear();
                    } else {
                        events.push(Event::End(Tag::CodeBlock(CodeBlockKind::Indented)));
                    }
                }
                Event::Text(ref text) => {
                    if in_code_block {
                        // Accumulate code block content
                        code_block_content.push_str(text);
                    } else {
                        // Regular text, pass through
                        events.push(event);
                    }
                }
                _ => {
                    // All other events pass through unchanged
                    events.push(event);
                }
            }
        }
        
        events
    }

    /// Generate highlighted HTML for a code block with improved HTML handling
    /// This extends the implementation from view_html.rs
    fn generate_highlighted_code_block(&self, code: &str, language: &str) -> String {
        if language.is_empty() {
            // Plain code block without language specification
            format!(r#"<div class="code-block code-block-plain"><pre><code>{}</code></pre></div>"#, 
                    self.html_escape(code))
        } else if language.to_lowercase() == "html" {
            // Special handling for HTML code
            // First escape the HTML to prevent rendering, then highlight it
            let escaped_code = self.html_escape(code);
            
            // Use syntax highlighting on the escaped HTML
            let highlighted = self.language_manager.highlight_code(&escaped_code, "html");
            
            // Wrap in a code editor style container
            format!(r#"<div class="html-code-editor code-block-html">{}</div>"#, highlighted)
        } else {
            // Standard syntax highlighting for other languages
            self.language_manager.highlight_code(code, language)
        }
    }

    /// HTML escape function to prevent XSS
    fn html_escape(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
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
        format!(
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
            content.lines()
                .map(|line| format!("      {}", line))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
    
    /// Convert HTML with syntax highlighting to plain text for TextView display
    fn html_to_plain_text(&self, html: &str) -> String {
        // The highlighted HTML from syntect contains spans with syntax classes
        // We need to extract just the text content while properly handling HTML entities
        
        // Process text to:
        // 1. Remove all HTML tags but preserve whitespace
        // 2. Convert HTML entities back to their characters for display
        // This works because we're displaying in a monospace TextView
        let mut result = String::new();
        let mut in_tag = false;
        let mut in_entity = false;
        let mut entity = String::new();
        
        let mut chars = html.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                '&' if !in_tag && !in_entity => {
                    in_entity = true;
                    entity.clear();
                },
                ';' if in_entity => {
                    in_entity = false;
                    // Convert the entity back to its character representation
                    match entity.as_str() {
                        "lt" => result.push('<'),
                        "gt" => result.push('>'),
                        "amp" => result.push('&'),
                        "quot" => result.push('"'),
                        "#x27" => result.push('\''),
                        _ => {
                            // Unknown entity, preserve as is
                            result.push('&');
                            result.push_str(&entity);
                            result.push(';');
                        }
                    }
                    entity.clear();
                },
                _ if in_entity => {
                    entity.push(c);
                },
                _ if !in_tag && !in_entity => result.push(c),
                _ => {}
            }
        }
        
        result
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
                },
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
                },
                crate::theme::Theme::System => {
                    // System theme (detect and apply appropriate theme)
                    let system_theme = crate::theme::ThemeManager::detect_system_theme();
                    match system_theme {
                        crate::theme::Theme::Dark => {
                            text_view.set_css_classes(&["theme-dark"]);
                            text_view.add_css_class("theme-dark");
                            text_view.remove_css_class("theme-light");
                        },
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