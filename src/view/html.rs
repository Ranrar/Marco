use crate::view::color_syntax::highlight_code_html;
use crate::theme::ThemeManager;
use gtk4::prelude::*;
use gtk4::{ScrolledWindow, Widget};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};
use crate::markdown::basic::MarkdownParser;
use crate::utils::cache::{AST_CACHE, hash_source};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::rc::Rc;
use std::time::{Duration, Instant};
use crate::utils::cache::get_regex;
use html_escape;
use webkit6::prelude::*;
use webkit6::{UserContentManager, UserStyleLevel, UserStyleSheet, WebView};


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
    last_update: Rc<RefCell<Option<Instant>>>,
    cached_html: Rc<RefCell<HashMap<String, String>>>,
    user_content_manager: UserContentManager,
    // Regexes are now cached globally using utils::regex_cache
    cached_css: Rc<RefCell<Option<String>>>,

    /// If true, use MarkdownParser from basic.rs instead of pulldown_cmark
    use_custom_parser: Rc<RefCell<bool>>,
    custom_parser: Rc<MarkdownParser>,
}

impl MarkdownHtmlView {
    /// Focus the WebView for keyboard input
    pub fn grab_focus(&self) {
        self.webview.grab_focus();
    }
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
            last_update: Rc::new(RefCell::new(None)),
            cached_html: Rc::new(RefCell::new(HashMap::new())),
            user_content_manager,
            // Regexes are now cached globally using utils::regex_cache
            cached_css: Rc::new(RefCell::new(None)),

            use_custom_parser: Rc::new(RefCell::new(false)),
            custom_parser: Rc::new(MarkdownParser::new()),
        };

        // Inject syntax highlighting CSS once at initialization
        view.inject_syntax_highlighting_css();

        // Connect external link handler
        view.connect_external_link_handler();

        // Initialize with a default empty document
        view.initialize_empty_document();

        view
    }

    /// Enable or disable the custom MarkdownParser for HTML conversion
        // No-op: highlighter now handled by color_syntax.rs
    pub fn set_use_custom_parser(&self, enabled: bool) {
        *self.use_custom_parser.borrow_mut() = enabled;
        self.refresh_with_current_content();
    }

    /// Sets up the preview context menu for the HTML view
    pub fn setup_context_menu(&self, editor: &crate::editor::MarkdownEditor) {
        let preview_menu = crate::view::context_menu::PreviewContextMenu::new();
        preview_menu.setup_gesture_for_widget(&self.webview, editor);
    }

    pub fn widget(&self) -> &Widget {
        self.widget.upcast_ref()
    }

    pub fn set_theme_manager(&self, theme_manager: ThemeManager) {
        // Update the syntax highlighter with the appropriate theme
        let syntax_theme_name = theme_manager.get_syntax_theme_name();
        println!("DEBUG: HTML view setting syntax theme to: {}", syntax_theme_name);

        // Invalidate all theme-dependent caches
        crate::utils::cache::on_theme_change();

        {
            // No-op: highlighter now handled by color_syntax.rs
        }

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

        let cache_key = format!(
            "{}-{}",
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

        // Invalidate AST cache for this content if changed
        let hash = hash_source(markdown_text);
        AST_CACHE.invalidate(hash);

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
        self.cached_html
            .borrow_mut()
            .insert(cache_key, html_content.clone());

        // Store the content for future reference
        *self.current_content.borrow_mut() = html_content.clone();

        // Load the HTML content
        self.load_html_content(&html_content);
    }

    fn process_markdown_to_html(&self, markdown_text: &str) -> String {
        // 1. Preprocess for compact HTML (tasks, etc.)
        let preprocessed = self.preprocess_for_compact_html(markdown_text);

        // 2. Use pulldown_cmark to parse markdown and process events with code highlighting
        let parser = Parser::new(&preprocessed);
        let events = self.process_events_with_code_highlighting(parser);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, events.into_iter());

        // 3. Add header IDs for anchor links
        let html_with_ids = self.add_header_ids_to_html(&html_output);
        html_with_ids
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
        let complete_html =
            self.create_html_document_with_embedded_scroll(html_content, current_scroll);

        let base_uri = self.get_base_uri();
        self.webview.load_html(&complete_html, base_uri.as_deref());

        *self.saved_scroll_position.borrow_mut() = current_scroll;
    }

    fn create_html_document_with_embedded_scroll(
        &self,
        html_content: &str,
        scroll_y: f64,
    ) -> String {
        let theme_class = if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            match theme_manager.get_effective_theme() {
                crate::theme::Theme::Light => "theme-light",
                crate::theme::Theme::Dark => "theme-dark",
                crate::theme::Theme::System => {
                    // Detect actual system theme and apply the appropriate class
                    match crate::theme::ThemeManager::detect_system_theme() {
                        crate::theme::Theme::Dark => "theme-dark",
                        _ => "theme-light",
                    }
                }
            }
        } else {
            "theme-light" // Default to light theme if no theme manager
        };

        // JavaScript to intercept external link clicks, but allow anchor (#) links
        let link_intercept_js = r#"
            document.addEventListener('click', function(e) {
                let target = e.target;
                while (target && target.tagName !== 'A') {
                    target = target.parentElement;
                }
                if (target && target.tagName === 'A') {
                    let href = target.getAttribute('href');
                    if (href && (href.startsWith('http://') || href.startsWith('https://'))) {
                        e.preventDefault();
                        window.open(href, '_blank');
                    }
                }
            }, true);
        "#;

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
        {}
    </script>
</head>
<body class="{}">
{}
</body>
</html>"#,
            self.load_css_content_cached(),
            scroll_y,
            scroll_y,
            link_intercept_js,
            theme_class,
            html_content
        )
    }
    /// Connect navigation policy to open external links in the OS browser
    pub fn connect_external_link_handler(&self) {
        use gtk4::glib;
        use webkit6::prelude::*;
        let webview = self.webview.clone();
        webview.connect_decide_policy(move |_webview, decision, _decision_type| {
            if let Some(nav) = decision.dynamic_cast_ref::<webkit6::NavigationPolicyDecision>() {
                if let Some(mut na) = nav.navigation_action() {
                    if let Some(request) = na.request() {
                        if let Some(url) = request.uri() {
                            if url.starts_with("http://") || url.starts_with("https://") {
                                // Open in OS browser
                                let ctx = glib::MainContext::default();
                                let url = url.to_string();
                                ctx.spawn_local(async move {
                                    let _ = open::that(url);
                                });
                                decision.ignore();
                                return true;
                            }
                        }
                    }
                }
            }
            false
        });
    }

    fn create_minimal_html_document(&self, html_content: &str, _scroll_y: f64) -> String {
        let theme_class = if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            match theme_manager.get_effective_theme() {
                crate::theme::Theme::Light => "theme-light",
                crate::theme::Theme::Dark => "theme-dark",
                crate::theme::Theme::System => {
                    // Detect actual system theme and apply the appropriate class
                    match crate::theme::ThemeManager::detect_system_theme() {
                        crate::theme::Theme::Dark => "theme-dark",
                        _ => "theme-light",
                    }
                }
            }
        } else {
            "theme-light" // Default to light theme if no theme manager
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

        let css_content = {
            let css_theme = crate::ui::css_theme::CssTheme::get_current_css_theme();
            match crate::ui::css_theme::CssTheme::set_css_theme(&css_theme) {
                Ok(css) => css,
                Err(e) => {
                    eprintln!("ERROR: Failed to load CSS theme '{}': {}", css_theme, e);
                    eprintln!("Using empty CSS content as no fallback is allowed");
                    String::new() // Return empty CSS instead of fallback
                }
            }
        };

        // Cache the result
        *self.cached_css.borrow_mut() = Some(css_content.clone());
        css_content
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
            &[],
        );

        self.user_content_manager.add_style_sheet(&user_style_sheet);
    }

    fn get_syntax_highlighting_css(&self) -> String {
        if let Some(ref cached_css) = *self.cached_css.borrow() {
            return cached_css.clone();
        }

        // Use ThemeManager to get the correct tmTheme for the current app theme
        let theme_name = if let Some(ref theme_manager) = *self.theme_manager.borrow() {
            let file_name = theme_manager.get_syntax_theme_name();
            if file_name.ends_with(".tmTheme") {
                file_name
            } else {
                format!("{}.tmTheme", file_name)
            }
        } else {
            "MarcoDark.tmTheme".to_string() // fallback to a default
        };

        let css_content = crate::view::color_syntax::generate_syntect_css(&theme_name)
            .unwrap_or_else(|| {
                eprintln!("ERROR: Failed to generate Syntect CSS for theme: {}", theme_name);
                String::new()
            });

        *self.cached_css.borrow_mut() = Some(css_content.clone());
        css_content
    }

    fn preprocess_for_compact_html(&self, markdown: &str) -> String {
        // Delegate all inline formatting to MarkdownParser for consistency
        let parser = &*self.custom_parser;
        let mut result = String::new();
        let open_task_pattern = get_regex(r"^(\s*)(\[ \])(.*)$");
        let closed_task_pattern = get_regex(r"^(\s*)(\[x\])(.*)$");

        for line in markdown.lines() {
            let processed_line =
                if let Some(captures) = open_task_pattern.captures(line) {
                    let task_text = &captures[3];
                    format!(
                        "<p><input type=\"checkbox\" disabled> {}</p>",
                        parser.process_inline_formatting(task_text.trim())
                    )
                } else if let Some(captures) = closed_task_pattern.captures(line) {
                    let task_text = &captures[3];
                    format!(
                        "<p><input type=\"checkbox\" checked disabled> {}</p>",
                        parser.process_inline_formatting(task_text.trim())
                    )
                } else {
                    parser.process_inline_formatting(line)
                };
            result.push_str(&processed_line);
            result.push('\n');
        }
        result
    }

    fn add_header_ids_to_html(&self, html_content: &str) -> String {
        let header_regex = get_regex(r"<(h[1-6])>([^<]+)</h[1-6]>");

        header_regex
            .replace_all(html_content, |caps: &regex::Captures| {
                let tag = &caps[1];
                let content = &caps[2];
                let anchor_id = self.generate_anchor_link(content);
                format!("<{} id=\"{}\">{}</{}>", tag, anchor_id, content, tag)
            })
            .to_string()
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
                        let highlighted_html = self
                            .generate_highlighted_code_block(&code_block_content, &code_block_lang);
                        events.push(Event::Html(highlighted_html.into()));
                        code_block_content.clear();
                        code_block_lang.clear();
                    }
                }
                Event::End(Tag::CodeBlock(_)) => {
                    if in_code_block {
                        in_code_block = false;
                        let highlighted_html = self
                            .generate_highlighted_code_block(&code_block_content, &code_block_lang);
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
        // Normalize language for CSS class (accent color)
        let lang_class = if !language.is_empty() {
            format!("code-block code-block-{}", language.to_lowercase().replace("+", "plus").replace("#", "sharp").replace(".", ""))
        } else {
            "code-block code-block-plain".to_string()
        };

        if let Some(html_inner) = highlight_code_html(code, language) {
            // If the highlighter returns HTML with <span>, wrap in accent div
            if html_inner.contains("<span") {
                format!(r#"<div class=\"{}\">{}</div>"#, lang_class, html_inner)
            } else {
                // Fallback: HTML-escape and wrap in <pre><code>
                let escaped_code = html_escape::encode_safe(&html_inner);
                format!(r#"<div class=\"{}\"><pre><code>{}</code></pre></div>"#, lang_class, escaped_code)
            }
        } else {
            // Fallback: HTML-escape and wrap in <pre><code>
            let escaped_code = html_escape::encode_safe(code);
            format!(r#"<div class=\"{}\"><pre><code>{}</code></pre></div>"#, lang_class, escaped_code)
        }
    }
}    