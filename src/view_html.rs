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
        
        // Convert markdown to HTML using pulldown-cmark for better feature support
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        
        let parser = Parser::new_ext(markdown_text, options);
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
    line-height: 1.6;
    color: {};
    background-color: {};
    margin: 2rem;
    padding: 0;
}}
h1, h2, h3, h4, h5, h6 {{
    font-weight: 600;
    margin-top: 24px;
    margin-bottom: 16px;
}}
h1 {{ font-size: 2em; }}
h2 {{ font-size: 1.5em; }}
h3 {{ font-size: 1.25em; }}
p {{ margin: 16px 0; }}
code {{
    background-color: {};
    border-radius: 3px;
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    font-size: 85%;
    padding: 0.2em 0.4em;
}}
pre {{
    background-color: {};
    border-radius: 6px;
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
    overflow: auto;
    padding: 16px;
}}
blockquote {{
    border-left: 0.25em solid {};
    color: {};
    margin: 0;
    padding: 0 1em;
}}
ul, ol {{ padding-left: 2em; }}
"#, 
                fallback_color, 
                fallback_bg,
                if fallback_bg == "#1a1a1a" { "#2a2a2a" } else { "rgba(27, 31, 35, 0.05)" },
                if fallback_bg == "#1a1a1a" { "#2a2a2a" } else { "#f6f8fa" },
                if fallback_bg == "#1a1a1a" { "#444" } else { "#dfe2e5" },
                if fallback_bg == "#1a1a1a" { "#999" } else { "#6a737d" }
            )
        })
    }
}
