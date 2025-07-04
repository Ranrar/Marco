use gtk4::prelude::*;
use gtk4::{ScrolledWindow, TextView, Widget};
use pulldown_cmark::{Parser, Options, html, Event, Tag, CodeBlockKind};
use crate::syntect_highlight::CodeLanguageManager;

#[derive(Clone)]
pub struct MarkdownCodeView {
    widget: ScrolledWindow,
    text_view: TextView,
    language_manager: CodeLanguageManager,
}

impl MarkdownCodeView {
    pub fn new() -> Self {
        let text_view = TextView::new();
        text_view.set_editable(false);
        text_view.set_cursor_visible(false);

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_child(Some(&text_view));
        scrolled_window.set_vexpand(true);
        scrolled_window.set_size_request(200, -1); // Minimum width of 200px

        Self {
            widget: scrolled_window,
            text_view,
            language_manager: CodeLanguageManager::new(),
        }
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
        
        let preview_buffer = self.text_view.buffer();
        preview_buffer.set_text(&html_content);
    }

    /// Process pulldown-cmark events to add syntax highlighting to code blocks
    /// This mirrors the implementation in view_html.rs
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
                        // Generate highlighted HTML for the code block
                        let highlighted_html = self.generate_highlighted_code_block(&code_block_content, &code_block_lang);
                        // Push the highlighted HTML as raw HTML event
                        events.push(Event::Html(highlighted_html.into()));
                        code_block_content.clear();
                        code_block_lang.clear();
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

    /// Generate highlighted HTML for a code block
    /// This mirrors the implementation in view_html.rs
    fn generate_highlighted_code_block(&self, code: &str, language: &str) -> String {
        if language.is_empty() {
            // Plain code block without language specification
            format!(r#"<div class="code-block code-block-plain"><pre><code>{}</code></pre></div>"#, 
                    self.html_escape(code))
        } else {
            // Use syntax highlighting
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

    #[allow(dead_code)]
    pub fn get_text_view(&self) -> &TextView {
        &self.text_view
    }
}