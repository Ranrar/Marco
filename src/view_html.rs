use gtk4::prelude::*;
use gtk4::{Widget, ScrolledWindow};
use webkit6::prelude::*;
use webkit6::{WebView, UserContentManager, UserStyleSheet, UserStyleLevel};
use pulldown_cmark::{Parser, Options, html, Event, Tag, CodeBlockKind};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;
use regex::Regex;
use crate::theme::ThemeManager;
use crate::syntect_highlight::CodeLanguageManager;

/// Cached compiled regexes for performance
struct CachedRegexes {
    open_task_pattern: Regex,
    closed_task_pattern: Regex,
    highlight_pattern: Regex,
    subscript_pattern: Regex,
    superscript_pattern: Regex,
    header_regex: Regex,
}

impl CachedRegexes {
    fn new() -> Self {
        Self {
            open_task_pattern: Regex::new(r"^(\s*)(\[ \])(.*)$").unwrap(),
            closed_task_pattern: Regex::new(r"^(\s*)(\[x\])(.*)$").unwrap(),
            highlight_pattern: Regex::new(r"==([^=]+)==").unwrap(),
            subscript_pattern: Regex::new(r"~([^~]+)~").unwrap(),
            superscript_pattern: Regex::new(r"(^|\s)\^([^^]+)\^").unwrap(),
            header_regex: Regex::new(r"<(h[1-6])>([^<]+)</h[1-6]>").unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct MarkdownHtmlView {
    widget: ScrolledWindow,
    webview: WebView,
    current_content: Rc<RefCell<String>>,
    current_markdown: Rc<RefCell<String>>,
    is_first_load: Rc<RefCell<bool>>,
    saved_scroll_position: Rc<RefCell<f64>>,
    theme_manager: Rc<RefCell<Option<ThemeManager>>>,
    custom_css: Rc<RefCell<String>>,
    base_path: Rc<RefCell<Option<std::path::PathBuf>>>,
    code_language_manager: Rc<RefCell<CodeLanguageManager>>,
    last_update: Rc<RefCell<Option<Instant>>>,
    cached_html: Rc<RefCell<HashMap<String, String>>>,
    user_content_manager: UserContentManager,
    cached_regexes: Rc<RefCell<CachedRegexes>>,
    cached_css: Rc<RefCell<Option<String>>>,
}

impl MarkdownHtmlView {
    pub fn new() -> Self {
        // Create UserContentManager for efficient CSS injection
        let user_content_manager = UserContentManager::new();
        
        // Create the WebView with the UserContentManager
        let webview = WebView::builder()
            .user_content_manager(&user_content_manager)
            .build();
        
        // Configure WebView settings
        if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
            settings.set_enable_javascript(true);
            settings.set_auto_load_images(true);
            settings.set_allow_file_access_from_file_urls(true);
            settings.set_allow_universal_access_from_file_urls(true);
            settings.set_enable_media(true);
            settings.set_enable_webgl(false);
            settings.set_hardware_acceleration_policy(webkit6::HardwareAccelerationPolicy::Never);
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
            base_path: Rc::new(RefCell::new(None)),
            code_language_manager: Rc::new(RefCell::new(CodeLanguageManager::new())),
            last_update: Rc::new(RefCell::new(None)),
            cached_html: Rc::new(RefCell::new(HashMap::new())),
            user_content_manager,
            cached_regexes: Rc::new(RefCell::new(CachedRegexes::new())),
            cached_css: Rc::new(RefCell::new(None)),
        };

        // Inject syntax highlighting CSS once at initialization
        view.inject_syntax_highlighting_css();
        
        // Initialize with a default empty document
        view.initialize_empty_document();
        
        view
    }

    pub fn widget(&self) -> &Widget {
        self.widget.upcast_ref()
    }

    pub fn set_theme_manager(&self, theme_manager: ThemeManager) {
        *self.theme_manager.borrow_mut() = Some(theme_manager);
        self.refresh_with_current_content();
    }

    pub fn refresh_with_current_content(&self) {
        // Clear caches to force regeneration with new theme
        *self.cached_css.borrow_mut() = None;
        self.cached_html.borrow_mut().clear();
        
        let current_markdown = self.current_markdown.borrow().clone();
        if !current_markdown.is_empty() {
            *self.current_content.borrow_mut() = String::new();
            self.update_content(&current_markdown);
        } else {
            self.initialize_empty_document();
        }
    }

    fn initialize_empty_document(&self) {
        let complete_html = self.create_html_document_with_embedded_scroll("", 0.0);
        let base_uri = self.get_base_uri();
        self.webview.load_html(&complete_html, base_uri.as_deref());
    }

    pub fn set_base_path(&self, path: Option<std::path::PathBuf>) {
        *self.base_path.borrow_mut() = path;
    }

    fn get_base_uri(&self) -> Option<String> {
        if let Some(ref path) = *self.base_path.borrow() {
            if let Some(parent) = path.parent() {
                let parent_str = parent.to_string_lossy();
                let base_uri = format!("file://{}/", parent_str);
                return Some(base_uri);
            }
        }
        
        if let Ok(current_dir) = std::env::current_dir() {
            let current_dir_str = current_dir.to_string_lossy();
            let base_uri = format!("file://{}/", current_dir_str);
            Some(base_uri)
        } else {
            None
        }
    }

    pub fn update_content(&self, markdown_text: &str) {
        // Enhanced debouncing: don't update too frequently
        let now = Instant::now();
        if let Some(last_update) = *self.last_update.borrow() {
            if now.duration_since(last_update) < Duration::from_millis(200) {
                *self.current_markdown.borrow_mut() = markdown_text.to_string();
                return;
            }
        }
        *self.last_update.borrow_mut() = Some(now);
        
        // Create a proper cache key based on markdown content and theme
        let theme_key = if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            format!("{:?}", theme_manager.get_effective_theme())
        } else {
            "default".to_string()
        };
        
        let cache_key = format!("{}-{}", 
            {
                let mut hasher = DefaultHasher::new();
                hasher.write(markdown_text.as_bytes());
                hasher.finish()
            },
            {
                let mut hasher = DefaultHasher::new();
                hasher.write(theme_key.as_bytes());
                hasher.finish()
            }
        );
        
        // Check cache first
        {
            let cache = self.cached_html.borrow();
            if let Some(cached_html) = cache.get(&cache_key) {
                let current_content = self.current_content.borrow();
                if *current_content != *cached_html {
                    drop(current_content);
                    *self.current_content.borrow_mut() = cached_html.clone();
                    self.load_html_content(cached_html);
                }
                return;
            }
        }
        
        // Store the original markdown for theme refresh
        *self.current_markdown.borrow_mut() = markdown_text.to_string();
        
        // Clear cache if needed to prevent memory leaks
        self.clear_cache_if_needed();
        
        // Process markdown (only if not cached)
        let html_content = self.process_markdown_to_html(markdown_text);
        
        // Check if content has actually changed to avoid unnecessary reloads
        let current_content = self.current_content.borrow();
        if *current_content == html_content {
            return;
        }
        drop(current_content);
        
        // Cache the processed HTML
        self.cached_html.borrow_mut().insert(cache_key, html_content.clone());
        
        // Store the content for future reference
        *self.current_content.borrow_mut() = html_content.clone();
        
        // Load the HTML content
        self.load_html_content(&html_content);
    }
    
    fn process_markdown_to_html(&self, markdown_text: &str) -> String {
        let processed_markdown = self.preprocess_for_compact_html(markdown_text);
        
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        
        let parser = Parser::new_ext(&processed_markdown, options);
        let mut html_content = String::new();
        
        let events = self.process_events_with_code_highlighting(parser);
        html::push_html(&mut html_content, events.into_iter());
        
        self.add_header_ids_to_html(&html_content)
    }
    
    fn load_html_content(&self, html_content: &str) {
        let is_first = *self.is_first_load.borrow();
        if is_first {
            let complete_html = self.create_minimal_html_document(html_content, 0.0);
            let base_uri = self.get_base_uri();
            self.webview.load_html(&complete_html, base_uri.as_deref());
            *self.is_first_load.borrow_mut() = false;
        } else {
            self.update_content_preserving_scroll(html_content);
        }
    }
    
    fn update_content_preserving_scroll(&self, html_content: &str) {
        let current_scroll = *self.saved_scroll_position.borrow();
        let complete_html = self.create_html_document_with_embedded_scroll(html_content, current_scroll);
        
        let base_uri = self.get_base_uri();
        self.webview.load_html(&complete_html, base_uri.as_deref());
        
        *self.saved_scroll_position.borrow_mut() = current_scroll;
    }
    
    fn create_html_document_with_embedded_scroll(&self, html_content: &str, scroll_y: f64) -> String {
        let theme_class = if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            match theme_manager.get_effective_theme() {
                crate::theme::Theme::Light => "theme-light",
                crate::theme::Theme::Dark => "theme-dark",
                crate::theme::Theme::System => "",
            }
        } else {
            ""
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
        window.addEventListener('DOMContentLoaded', function() {{
            window.scrollTo(0, {});
            
            var scrollTimeout;
            window.addEventListener('scroll', function() {{
                clearTimeout(scrollTimeout);
                scrollTimeout = setTimeout(function() {{
                    window.currentScrollY = window.pageYOffset || document.documentElement.scrollTop || 0;
                }}, 100);
            }}, {{ passive: true }});
        }});
        
        window.addEventListener('load', function() {{
            window.scrollTo(0, {});
        }});
    </script>
</head>
<body class="{}">
{}
</body>
</html>"#,
            self.load_css_content_cached(),
            scroll_y,
            scroll_y,
            theme_class,
            html_content
        )
    }
    
    fn create_minimal_html_document(&self, html_content: &str, _scroll_y: f64) -> String {
        let theme_class = if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            match theme_manager.get_effective_theme() {
                crate::theme::Theme::Light => "theme-light",
                crate::theme::Theme::Dark => "theme-dark",
                crate::theme::Theme::System => "",
            }
        } else {
            ""
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
</head>
<body class="{}">
{}
</body>
</html>"#,
            self.load_css_content_cached(),
            theme_class,
            html_content
        )
    }
    
    fn load_css_content_cached(&self) -> String {
        // Check if we have a cached version for current theme
        if let Some(cached_css) = self.cached_css.borrow().as_ref() {
            return cached_css.clone();
        }
        
        let css_content = if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            // Use theme manager to load CSS content
            let css_theme = theme_manager.get_current_css_theme();
            match theme_manager.set_css_theme(&css_theme) {
                Ok(css) => {
                    let mut complete_css = css;
                    
                    // Add theme override CSS if needed
                    let theme_override = theme_manager.get_theme_override_css();
                    if !theme_override.is_empty() {
                        complete_css.push_str("\n\n");
                        complete_css.push_str(&theme_override);
                    }
                    
                    complete_css
                }
                Err(_) => self.get_fallback_css()
            }
        } else {
            self.get_fallback_css()
        };
        
        // Cache the result
        *self.cached_css.borrow_mut() = Some(css_content.clone());
        css_content
    }
    
    fn get_fallback_css(&self) -> String {
        let fallback_bg = "#fff";
        let fallback_color = "#24292e";
        
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
li {{ margin: 0.2em 0; }}
blockquote {{
    margin: 1em 0;
    padding: 0.5em 1em;
    border-left: 4px solid #ddd;
    background-color: #f9f9f9;
}}
pre {{
    background-color: #f6f8fa;
    padding: 1em;
    border-radius: 4px;
    overflow-x: auto;
}}
code {{
    background-color: #f6f8fa;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
}}
"#, fallback_color, fallback_bg)
    }
    
    fn clear_cache_if_needed(&self) {
        let mut cache = self.cached_html.borrow_mut();
        if cache.len() > 50 {
            cache.clear();
        }
    }
    
    fn inject_syntax_highlighting_css(&self) {
        let css_content = self.get_syntax_highlighting_css();
        
        let user_style_sheet = UserStyleSheet::new(
            &css_content,
            webkit6::UserContentInjectedFrames::TopFrame,
            UserStyleLevel::User,
            &[],
            &[]
        );
        
        self.user_content_manager.add_style_sheet(&user_style_sheet);
    }
    
    fn get_syntax_highlighting_css(&self) -> String {
        if let Some(ref cached_css) = *self.cached_css.borrow() {
            return cached_css.clone();
        }
        
        let css_content = match std::fs::read_to_string("src/code.css") {
            Ok(css) => css,
            Err(_) => {
                self.generate_syntax_highlighting_css()
            }
        };
        
        *self.cached_css.borrow_mut() = Some(css_content.clone());
        css_content
    }
    
    fn preprocess_for_compact_html(&self, markdown: &str) -> String {
        let regexes = self.cached_regexes.borrow();
        let mut result = String::new();
        
        for line in markdown.lines() {
            let mut processed_line = if let Some(captures) = regexes.open_task_pattern.captures(line) {
                let task_text = &captures[3];
                format!("<p><input type=\"checkbox\" disabled> {}</p>", task_text.trim())
            } else if let Some(captures) = regexes.closed_task_pattern.captures(line) {
                let task_text = &captures[3];
                format!("<p><input type=\"checkbox\" checked disabled> {}</p>", task_text.trim())
            } else {
                line.to_string()
            };
            
            processed_line = regexes.highlight_pattern.replace_all(&processed_line, "<mark>$1</mark>").to_string();
            processed_line = regexes.subscript_pattern.replace_all(&processed_line, "<sub>$1</sub>").to_string();
            processed_line = regexes.superscript_pattern.replace_all(&processed_line, "$1<sup>$2</sup>").to_string();
            
            result.push_str(&processed_line);
            result.push('\n');
        }
        
        result
    }
    
    fn add_header_ids_to_html(&self, html_content: &str) -> String {
        let regexes = self.cached_regexes.borrow();
        
        regexes.header_regex.replace_all(html_content, |caps: &regex::Captures| {
            let tag = &caps[1];
            let content = &caps[2];
            let anchor_id = self.generate_anchor_link(content);
            format!("<{} id=\"{}\">{}</{}>", tag, anchor_id, content, tag)
        }).to_string()
    }
    
    fn generate_anchor_link(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' {
                    '-'
                } else {
                    '\0'
                }
            })
            .filter(|&c| c != '\0')
            .collect::<String>()
            .trim_matches('-')
            .replace("--", "-")
    }
    
    pub fn set_custom_css(&self, css_content: &str) {
        *self.custom_css.borrow_mut() = css_content.to_string();
        *self.cached_css.borrow_mut() = None;
    }
    
    pub fn get_custom_css(&self) -> String {
        self.custom_css.borrow().clone()
    }

    pub fn refresh(&self) {
        let current_content = self.current_content.borrow().clone();
        if !current_content.is_empty() {
            self.update_content(&current_content);
        }
    }

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
                }
                Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                    if in_code_block {
                        in_code_block = false;
                        let highlighted_html = self.generate_highlighted_code_block(&code_block_content, &code_block_lang);
                        events.push(Event::Html(highlighted_html.into()));
                        code_block_content.clear();
                        code_block_lang.clear();
                    }
                }
                Event::End(Tag::CodeBlock(_)) => {
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
                Event::Text(text) => {
                    if in_code_block {
                        code_block_content.push_str(&text);
                    } else {
                        events.push(Event::Text(text));
                    }
                }
                _ => {
                    events.push(event);
                }
            }
        }
        
        events
    }
    
    fn generate_highlighted_code_block(&self, code: &str, language: &str) -> String {
        let code_lang_manager = self.code_language_manager.borrow();
        
        if !language.is_empty() && code_lang_manager.get_language(language).is_some() {
            code_lang_manager.highlight_code(code, language)
        } else {
            let escaped_code = CodeLanguageManager::html_escape(code);
            format!(
                r#"<div class="code-block code-block-plain"><pre><code>{}</code></pre></div>"#,
                escaped_code
            )
        }
    }

    fn generate_syntax_highlighting_css(&self) -> String {
        r#"
/* Fallback syntax highlighting CSS */
.code-block {
    margin: 1rem 0 !important;
    border-radius: 6px !important;
    border: 1px solid #e1e4e8 !important;
    overflow-x: auto !important;
    background-color: #f8f9fa !important;
}

.code-block pre {
    margin: 0 !important;
    padding: 1rem !important;
    background-color: transparent !important;
    color: #24292e !important;
    font-family: 'SFMono-Regular', 'Consolas', 'Liberation Mono', 'Menlo', monospace !important;
    font-size: 14px !important;
    line-height: 1.5 !important;
    white-space: pre !important;
    overflow-x: auto !important;
}

.code-block code {
    background: none !important;
    padding: 0 !important;
    border: none !important;
    font-size: inherit !important;
    color: inherit !important;
}

.code-block .keyword {
    color: #d73a49 !important;
    font-weight: 600 !important;
}

.code-block .comment {
    color: #6a737d !important;
    font-style: italic !important;
}

.code-block .string {
    color: #032f62 !important;
}

.code-block .number {
    color: #005cc5 !important;
}

.code-block .function {
    color: #6f42c1 !important;
    font-weight: 600 !important;
}

.code-block .class {
    color: #e36209 !important;
    font-weight: 600 !important;
}

@media (prefers-color-scheme: dark) {
    .code-block {
        background-color: #0d1117 !important;
        border-color: #30363d !important;
    }
    
    .code-block pre {
        color: #e6edf3 !important;
    }
    
    .code-block .keyword { color: #ff7b72 !important; }
    .code-block .comment { color: #8b949e !important; }
    .code-block .string { color: #a5d6ff !important; }
    .code-block .number { color: #79c0ff !important; }
    .code-block .function { color: #d2a8ff !important; }
    .code-block .class { color: #ffa657 !important; }
}
        "#.to_string()
    }
}
