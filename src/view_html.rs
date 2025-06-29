use gtk4::prelude::*;
use gtk4::{Widget, ScrolledWindow};
use webkit6::prelude::*;
use webkit6::WebView;
use pulldown_cmark::{Parser, Options, html};
use std::cell::RefCell;
use std::rc::Rc;
use crate::theme::ThemeManager;

#[derive(Clone)]
pub struct MarkdownHtmlView {
    widget: ScrolledWindow,
    webview: WebView,
    current_content: Rc<RefCell<String>>,
    current_markdown: Rc<RefCell<String>>, // Store original markdown for theme refresh
    is_first_load: Rc<RefCell<bool>>,
    saved_scroll_position: Rc<RefCell<f64>>,
    theme_manager: Rc<RefCell<Option<ThemeManager>>>,
    custom_css: Rc<RefCell<String>>,
}

impl MarkdownHtmlView {
    pub fn new() -> Self {
        // Create the WebView for rendering HTML
        let webview = WebView::new();
        
        // Configure WebView settings - enable JavaScript for scroll preservation
        if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
            settings.set_enable_javascript(true); // Need JS for scroll position preservation
        }
        
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_child(Some(&webview));
        scrolled_window.set_vexpand(true);
        scrolled_window.set_size_request(200, -1);

        let view = Self {
            widget: scrolled_window,
            webview,
            current_content: Rc::new(RefCell::new(String::new())),
            current_markdown: Rc::new(RefCell::new(String::new())),
            is_first_load: Rc::new(RefCell::new(true)),
            saved_scroll_position: Rc::new(RefCell::new(0.0)),
            theme_manager: Rc::new(RefCell::new(None)),
            custom_css: Rc::new(RefCell::new(String::new())),
        };

        // Initialize with a default empty document to show proper background
        view.initialize_empty_document();
        
        view
    }

    pub fn widget(&self) -> &Widget {
        self.widget.upcast_ref()
    }

    pub fn set_theme_manager(&self, theme_manager: ThemeManager) {
        *self.theme_manager.borrow_mut() = Some(theme_manager);
        // Immediately refresh with new theme or initialize with empty themed document
        self.refresh_with_current_content();
    }

    /// Force refresh the HTML view with current content (useful for theme changes)
    pub fn refresh_with_current_content(&self) {
        let current_markdown = self.current_markdown.borrow().clone();
        if !current_markdown.is_empty() {
            // Clear current content to force regeneration with new theme
            *self.current_content.borrow_mut() = String::new();
            // Reprocess the markdown with the new theme
            self.update_content(&current_markdown);
        } else {
            // If no content, initialize with empty themed document
            self.initialize_empty_document();
        }
    }

    /// Initialize an empty document with the correct theme styling
    fn initialize_empty_document(&self) {
        let complete_html = self.create_html_document_with_embedded_scroll("", 0.0);
        self.webview.load_html(&complete_html, None);
    }

    pub fn update_content(&self, markdown_text: &str) {
        // Store the original markdown for theme refresh
        *self.current_markdown.borrow_mut() = markdown_text.to_string();
        
        // Preprocess markdown to handle custom task lists and ensure compact rendering
        let processed_markdown = self.preprocess_for_compact_html(markdown_text);
        
        // Convert markdown to HTML using pulldown-cmark for better feature support
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS); // Keep for GitHub-style task lists
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        
        let parser = Parser::new_ext(&processed_markdown, options);
        let mut html_content = String::new();
        html::push_html(&mut html_content, parser);
        
        // Check if content has actually changed to avoid unnecessary reloads
        let current_content = self.current_content.borrow();
        if *current_content == html_content {
            return; // No change, don't reload
        }
        drop(current_content);
        
        // Store the content for future reference
        *self.current_content.borrow_mut() = html_content.clone();
        
        // Handle first load vs updates differently
        let is_first = *self.is_first_load.borrow();
        if is_first {
            // First load - load with scroll preservation capability
            let complete_html = self.create_html_document_with_embedded_scroll(&html_content, 0.0);
            self.webview.load_html(&complete_html, None);
            *self.is_first_load.borrow_mut() = false;
        } else {
            // Subsequent updates - preserve scroll position
            self.update_content_preserving_scroll(&html_content);
        }
    }
    
    fn update_content_preserving_scroll(&self, html_content: &str) {
        // Simple approach: embed the current scroll position directly in the HTML
        let current_scroll = *self.saved_scroll_position.borrow();
        let complete_html = self.create_html_document_with_embedded_scroll(html_content, current_scroll);
        
        // Save scroll position before loading new content (we'll update this when we can get the real position)
        *self.saved_scroll_position.borrow_mut() = current_scroll;
        
        self.webview.load_html(&complete_html, None);
    }
    
    fn create_html_document_with_embedded_scroll(&self, html_content: &str, scroll_y: f64) -> String {
        // Determine the theme class to apply to the body
        let theme_class = if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            match theme_manager.get_effective_theme() {
                crate::theme::Theme::Light => "theme-light",
                crate::theme::Theme::Dark => "theme-dark",
                crate::theme::Theme::System => "", // Let CSS media query handle it
            }
        } else {
            "" // No theme manager, let CSS media query handle it
        };

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Markdown Preview</title>
    <style>
{}
    </style>
    <script>
        // Simple and reliable scroll restoration
        window.addEventListener('DOMContentLoaded', function() {{
            // Restore to the specified position
            window.scrollTo(0, {});
            
            // Set up scroll position tracking for future updates
            var scrollTimeout;
            window.addEventListener('scroll', function() {{
                clearTimeout(scrollTimeout);
                scrollTimeout = setTimeout(function() {{
                    // Store current position (this will be read by Rust when needed)
                    window.currentScrollY = window.pageYOffset || document.documentElement.scrollTop || 0;
                }}, 100);
            }}, {{ passive: true }});
        }});
        
        // Also try on load as backup
        window.addEventListener('load', function() {{
            window.scrollTo(0, {});
        }});
    </script>
</head>
<body class="{}">
{}
</body>
</html>"#,
            self.load_css_content(),
            scroll_y,
            scroll_y,
            theme_class,
            html_content
        )
    }
    
    fn load_css_content(&self) -> String {
        // Check if custom CSS is set
        let custom_css = self.custom_css.borrow();
        let mut css_content = if !custom_css.is_empty() {
            custom_css.clone()
        } else {
            // Use the unified CSS file that supports both light and dark themes
            let css_file = "css/standard.css";

            // Try to load the unified CSS file, fallback to basic styles if not found
            std::fs::read_to_string(css_file).unwrap_or_else(|_| {
                // Fallback CSS if file is not found - use dark theme if we can detect it
                let fallback_bg = if let Some(ref theme_manager) = *self.theme_manager.borrow() {
                    match theme_manager.get_effective_theme() {
                        crate::theme::Theme::Dark => "#1a1a1a",
                        _ => "#fff",
                    }
                } else {
                    "#fff"
                };
                let fallback_color = if fallback_bg == "#1a1a1a" { "#e1e1e1" } else { "#24292e" };
                
                format!(r#"
body {{
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.4;
    color: {};
    background-color: {};
    margin: 1rem;
    padding: 0;
}}
h1, h2, h3, h4, h5, h6 {{
    font-weight: 600;
    margin-top: 0.8em;
    margin-bottom: 0.4em;
}}
h1 {{ font-size: 1.8em; }}
h2 {{ font-size: 1.4em; }}
h3 {{ font-size: 1.2em; }}
p {{ 
    margin: 0.5em 0; 
    line-height: 1.4;
}}
ul, ol {{ 
    margin: 0.5em 0;
    padding-left: 1.5em;
}}
li {{
    margin: 0.2em 0;
}}
table {{
    border-collapse: collapse;
    margin: 0.5em 0;
    width: 100%;
}}
th, td {{
    border: 1px solid {};
    padding: 0.3em 0.6em;
    text-align: left;
}}
th {{
    background-color: {};
    font-weight: 600;
}}
blockquote {{
    border-left: 0.25em solid {};
    color: {};
    margin: 0.5em 0;
    padding: 0 0.8em;
}}
code {{
    background-color: {};
    border-radius: 3px;
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    font-size: 85%;
    padding: 0.1em 0.3em;
}}
pre {{
    background-color: {};
    border-radius: 6px;
    font-size: 85%;
    line-height: 1.45;
    overflow: auto;
    padding: 0.8em;
}}
pre code {{
    background-color: transparent;
    border: 0;
    font-size: 100%;
    margin: 0;
    padding: 0;
    white-space: pre;
    word-break: normal;
}}
"#, 
                fallback_color, fallback_bg,
                if fallback_bg == "#1a1a1a" { "#555" } else { "#ddd" }, // table border
                if fallback_bg == "#1a1a1a" { "#333" } else { "#f6f8fa" }, // table header bg
                if fallback_bg == "#1a1a1a" { "#555" } else { "#dfe2e5" }, // blockquote border
                if fallback_bg == "#1a1a1a" { "#8b949e" } else { "#6a737d" }, // blockquote color
                if fallback_bg == "#1a1a1a" { "#333" } else { "rgba(175,184,193,0.2)" }, // code bg
                if fallback_bg == "#1a1a1a" { "#2d3748" } else { "#f6f8fa" } // pre bg
                )
            })
        };

        // Add theme override CSS if we have a theme manager
        if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            let theme_override = theme_manager.get_theme_override_css();
            if !theme_override.is_empty() {
                css_content.push_str("\n\n");
                css_content.push_str(&theme_override);
            }
        }

        css_content
    }

    /// Preprocess markdown for compact HTML rendering
    /// Handles both dash and no-dash task list formats and ensures proper paragraph wrapping:
    /// - `[ ] Task` → paragraph with checkbox HTML without dash
    /// - `- [ ] Task` → GitHub-style (handled by pulldown_cmark)
    /// - `[x] Task` → paragraph with checked checkbox HTML without dash
    /// - `- [x] Task` → GitHub-style (handled by pulldown_cmark)
    /// - Ensures standalone text gets proper paragraph treatment
    fn preprocess_for_compact_html(&self, markdown: &str) -> String {
        use regex::Regex;
        
        // Create regex patterns for task lists without dashes (custom format)
        let open_task_pattern = Regex::new(r"^(\s*)(\[ \])(.*)$").unwrap();
        let closed_task_pattern = Regex::new(r"^(\s*)(\[x\])(.*)$").unwrap();
        
        let mut result = String::new();
        
        for line in markdown.lines() {
            let processed_line = if let Some(captures) = open_task_pattern.captures(line) {
                // Convert standalone "[ ] Task" to HTML paragraph with checkbox
                let task_text = &captures[3];
                format!("<p><input type=\"checkbox\" disabled> {}</p>", task_text.trim())
            } else if let Some(captures) = closed_task_pattern.captures(line) {
                // Convert standalone "[x] Task" to HTML paragraph with checked checkbox
                let task_text = &captures[3];
                format!("<p><input type=\"checkbox\" checked disabled> {}</p>", task_text.trim())
            } else {
                line.to_string()
            };
            
            result.push_str(&processed_line);
            result.push('\n');
        }
        
        result
    }
    
    /// Set custom CSS for the preview
    pub fn set_custom_css(&self, css_content: &str) {
        *self.custom_css.borrow_mut() = css_content.to_string();
    }
    
    /// Get current custom CSS
    pub fn get_custom_css(&self) -> String {
        self.custom_css.borrow().clone()
    }

    /// Refresh the HTML view (useful when theme changes)
    pub fn refresh(&self) {
        let current_content = self.current_content.borrow().clone();
        if !current_content.is_empty() {
            self.update_content(&current_content);
        }
    }
}
