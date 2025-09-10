use crate::components::marco_engine::ast::{Node, Visitor};
use std::fmt::Write;
use log::debug;

#[derive(Debug, Clone)]
pub struct HtmlOptions {
    pub class_prefix: String,
    pub syntax_highlighting: bool,
    pub sanitize_html: bool,
    pub youtube_embed: bool,
    pub auto_links: bool,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            class_prefix: "marco-".to_string(),
            syntax_highlighting: true,
            sanitize_html: true,
            youtube_embed: true,
            auto_links: true,
        }
    }
}

pub struct HtmlRenderer {
    output: String,
    options: HtmlOptions,
}

impl HtmlRenderer {
    pub fn new(options: HtmlOptions) -> Self {
        Self {
            output: String::with_capacity(1024), // Pre-allocate for performance
            options,
        }
    }
    
    pub fn render(mut self, ast: &Node) -> String {
        debug!("HtmlRenderer::render - Starting render");
        debug!("HtmlRenderer::render - AST root: {:?}", std::mem::discriminant(ast));
        self.visit(ast);
        debug!("HtmlRenderer::render - Generated HTML: '{}'", self.output.chars().take(100).collect::<String>());
        self.output
    }
    
    fn write_class(&mut self, class: &str) {
        if !self.options.class_prefix.is_empty() {
            write!(self.output, " class=\"{}{}\"", self.options.class_prefix, class).unwrap();
        }
    }
    
    fn escape_html(&self, text: &str) -> String {
        if self.options.sanitize_html {
            text.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
                .replace('\'', "&#39;")
        } else {
            text.to_string()
        }
    }
    
    fn is_youtube_url(&self, url: &str) -> bool {
        url.contains("youtube.com/watch") || url.contains("youtu.be/")
    }
    
    fn extract_youtube_id<'a>(&self, url: &'a str) -> Option<&'a str> {
        if url.contains("youtu.be/") {
            url.split("youtu.be/").nth(1)?.split('?').next()
        } else if url.contains("youtube.com/watch") {
            url.split("v=").nth(1)?.split('&').next()
        } else {
            None
        }
    }
    
    fn render_youtube_embed(&mut self, url: &str) {
        let video_id = self.extract_youtube_id(url);
        if let Some(video_id) = video_id {
            let class_attr = if !self.options.class_prefix.is_empty() { 
                format!(" class=\"{}youtube-embed\"", self.options.class_prefix) 
            } else { 
                String::new() 
            };
            
            write!(self.output, 
                r#"<div{}><iframe src="https://www.youtube.com/embed/{}" frameborder="0" allowfullscreen></iframe></div>"#,
                class_attr,
                video_id
            ).unwrap();
        } else {
            // Fallback to regular link
            write!(self.output, "<a href=\"{}\"", self.escape_html(url)).unwrap();
            self.write_class("link");
            write!(self.output, ">{}</a>", self.escape_html(url)).unwrap();
        }
    }
}

impl Visitor for HtmlRenderer {
    type Output = ();
    
    fn visit(&mut self, node: &Node) -> Self::Output {
        match node {
            Node::Document { children, .. } => self.visit_document(children),
            Node::Heading { level, content, .. } => self.visit_heading(*level, content),
            Node::Paragraph { content, .. } => self.visit_paragraph(content),
            Node::CodeBlock { language, content, .. } => self.visit_code_block(language, content),
            Node::MathBlock { content, .. } => {
                write!(self.output, "<div").unwrap();
                self.write_class("math-block");
                write!(self.output, ">{}</div>", self.escape_html(content)).unwrap();
            }
            Node::List { ordered, items, .. } => self.visit_list(*ordered, items),
            Node::ListItem { content, checked, .. } => {
                write!(self.output, "<li").unwrap();
                if checked.is_some() {
                    self.write_class("task-item");
                }
                write!(self.output, ">").unwrap();
                
                if let Some(checked) = checked {
                    write!(self.output, "<input type=\"checkbox\"").unwrap();
                    if *checked {
                        write!(self.output, " checked").unwrap();
                    }
                    write!(self.output, " disabled> ").unwrap();
                }
                
                for child in content {
                    self.visit(child);
                }
                write!(self.output, "</li>").unwrap();
            }
            Node::Table { headers, rows, .. } => self.visit_table(headers, rows),
            Node::Text { content, .. } => self.visit_text(content),
            Node::Emphasis { content, .. } => self.visit_emphasis(content),
            Node::Strong { content, .. } => self.visit_strong(content),
            Node::Strikethrough { content, .. } => {
                write!(self.output, "<s>").unwrap();
                for child in content {
                    self.visit(child);
                }
                write!(self.output, "</s>").unwrap();
            }
            Node::Highlight { content, .. } => {
                write!(self.output, "<mark>").unwrap();
                for child in content {
                    self.visit(child);
                }
                write!(self.output, "</mark>").unwrap();
            }
            Node::Superscript { content, .. } => {
                write!(self.output, "<sup>").unwrap();
                for child in content {
                    self.visit(child);
                }
                write!(self.output, "</sup>").unwrap();
            }
            Node::Subscript { content, .. } => {
                write!(self.output, "<sub>").unwrap();
                for child in content {
                    self.visit(child);
                }
                write!(self.output, "</sub>").unwrap();
            }
            Node::Code { content, .. } => self.visit_code(content),
            Node::MathInline { content, .. } => {
                write!(self.output, "<span class=\"math-inline\">\\({}\\)</span>", content).unwrap();
            }
            Node::Link { text, url, title, .. } => self.visit_link(text, url, title),
            Node::ReferenceLink { text, label, .. } => {
                write!(self.output, "<a href=\"#ref-{}\">", label).unwrap();
                for child in text {
                    self.visit(child);
                }
                write!(self.output, "</a>").unwrap();
            }
            Node::InlineFootnote { content, .. } => {
                write!(self.output, "<sup class=\"footnote\">").unwrap();
                for child in content {
                    self.visit(child);
                }
                write!(self.output, "</sup>").unwrap();
            }
            Node::Image { alt, url, title, .. } => self.visit_image(alt, url, title),
            Node::DefinitionList { items, .. } => {
                write!(self.output, "<dl>").unwrap();
                for child in items {
                    self.visit(child);
                }
                write!(self.output, "</dl>").unwrap();
            }
            Node::DefinitionTerm { content, .. } => {
                write!(self.output, "<dt>").unwrap();
                for child in content {
                    self.visit(child);
                }
                write!(self.output, "</dt>").unwrap();
            }
            Node::DefinitionDescription { content, .. } => {
                write!(self.output, "<dd>").unwrap();
                for child in content {
                    self.visit(child);
                }
                write!(self.output, "</dd>").unwrap();
            }
            Node::FootnoteDefinition { label, content, .. } => {
                write!(self.output, "<div id=\"footnote-{}\" class=\"footnote\">", label).unwrap();
                for child in content {
                    self.visit(child);
                }
                write!(self.output, "</div>").unwrap();
            }
            Node::ReferenceDefinition { label, url, title, .. } => {
                write!(self.output, "<div id=\"ref-{}\" class=\"reference\" data-url=\"{}\"", label, url).unwrap();
                if let Some(title) = title {
                    write!(self.output, " data-title=\"{}\"", title).unwrap();
                }
                write!(self.output, "></div>").unwrap();
            }
            Node::UserMention { username, platform, display_name, .. } => {
                let platform_str = platform.as_deref().unwrap_or("unknown");
                write!(self.output, "<span class=\"user-mention\" data-platform=\"{}\">@{}", platform_str, username).unwrap();
                if let Some(name) = display_name {
                    write!(self.output, " ({})", name).unwrap();
                }
                write!(self.output, "</span>").unwrap();
            }
            Node::Bookmark { label, path, line, .. } => {
                write!(self.output, "<a href=\"{}{}\" class=\"bookmark\">", path, 
                    line.map_or(String::new(), |l| format!("#{}", l))).unwrap();
                write!(self.output, "[bookmark:{}]", label).unwrap();
                write!(self.output, "</a>").unwrap();
            }
            Node::TabBlock { title, tabs, .. } => {
                write!(self.output, "<div class=\"tab-block\"").unwrap();
                if let Some(title) = title {
                    write!(self.output, " data-title=\"{}\"", title).unwrap();
                }
                write!(self.output, ">").unwrap();
                for child in tabs {
                    self.visit(child);
                }
                write!(self.output, "</div>").unwrap();
            }
            Node::Tab { name, content, .. } => {
                write!(self.output, "<div class=\"tab\"").unwrap();
                if let Some(name) = name {
                    write!(self.output, " data-name=\"{}\"", name).unwrap();
                }
                write!(self.output, ">").unwrap();
                for child in content {
                    self.visit(child);
                }
                write!(self.output, "</div>").unwrap();
            }
            Node::DiagramBlock { diagram_type, content, .. } => {
                write!(self.output, "<div class=\"diagram\" data-type=\"{}\">", diagram_type).unwrap();
                write!(self.output, "<pre><code>{}</code></pre>", content).unwrap();
                write!(self.output, "</div>").unwrap();
            }
            Node::RunInline { script_type, command, .. } => {
                write!(self.output, "<code class=\"run-inline\" data-lang=\"{}\">{}</code>", script_type, command).unwrap();
            }
            Node::RunBlock { script_type, content, .. } => {
                write!(self.output, "<div class=\"run-block\" data-lang=\"{}\">", script_type).unwrap();
                write!(self.output, "<pre><code>{}</code></pre>", content).unwrap();
                write!(self.output, "</div>").unwrap();
            }
            Node::PageTag { format, .. } => {
                let format_str = format.as_deref().unwrap_or("A4");
                write!(self.output, "<meta name=\"page-format\" content=\"{}\">", format_str).unwrap();
            }
            Node::DocumentReference { path, .. } => {
                write!(self.output, "<a href=\"{}\" class=\"doc-ref\">[@doc]({})</a>", path, path).unwrap();
            }
            Node::TableOfContents { depth, document, .. } => {
                write!(self.output, "<div class=\"toc\"").unwrap();
                if let Some(depth) = depth {
                    write!(self.output, " data-depth=\"{}\"", depth).unwrap();
                }
                if let Some(doc) = document {
                    write!(self.output, " data-document=\"{}\"", doc).unwrap();
                }
                write!(self.output, ">").unwrap();
                write!(self.output, "<!-- TOC will be generated here -->").unwrap();
                write!(self.output, "</div>").unwrap();
            }
            Node::Autolink { url, .. } => {
                write!(self.output, "<a href=\"{}\">{}</a>", url, url).unwrap();
            }
            Node::ReferenceImage { alt, label, .. } => {
                write!(self.output, "<img src=\"#ref-{}\" alt=\"{}\">", label, alt).unwrap();
            }
            Node::FootnoteRef { label, .. } => {
                write!(self.output, "<sup><a href=\"#footnote-{}\">{}</a></sup>", label, label).unwrap();
            }
            Node::Emoji { name, .. } => {
                write!(self.output, "<span class=\"emoji\">:{}</span>", name).unwrap();
            }
            Node::LineBreak { .. } => {
                write!(self.output, "<br>").unwrap();
            }
            Node::EscapedChar { character, .. } => {
                write!(self.output, "{}", character).unwrap();
            }
            Node::InlineHTML { content, .. } => {
                write!(self.output, "{}", content).unwrap();
            }
            Node::BlockHTML { content, .. } => {
                write!(self.output, "{}", content).unwrap();
            }
            Node::Macro { name, arguments, content, .. } => self.visit_macro(name, arguments, content),
            Node::HorizontalRule { .. } => self.visit_horizontal_rule(),
            Node::BlockQuote { content, .. } => self.visit_block_quote(content),
            Node::Admonition { kind, content, .. } => self.visit_admonition(kind, content),
            Node::TaskItem { checked, content, .. } => {
                let check_mark = if *checked { "checked" } else { "" };
                write!(self.output, "<input type=\"checkbox\" {} disabled> ", check_mark).unwrap();
                for child in content {
                    self.visit(child);
                }
            }
            Node::Unknown { content, rule, .. } => self.visit_unknown(content, rule),
            
            // New Node variants - provide default handling
            Node::SetextHeading { content, level, .. } => {
                // Treat setext headings same as regular headings
                self.visit_heading(*level, content)
            }
            Node::TableHeader { cells, .. } => {
                // Render table header cells
                write!(self.output, "<thead><tr>").unwrap();
                for cell in cells {
                    write!(self.output, "<th>").unwrap();
                    self.visit(cell);
                    write!(self.output, "</th>").unwrap();
                }
                write!(self.output, "</tr></thead>").unwrap();
            }
            Node::TableRow { cells, .. } => {
                // Render table row cells
                write!(self.output, "<tr>").unwrap();
                for cell in cells {
                    write!(self.output, "<td>").unwrap();
                    self.visit(cell);
                    write!(self.output, "</td>").unwrap();
                }
                write!(self.output, "</tr>").unwrap();
            }
            Node::TableCell { content, alignment, .. } => {
                // Render table cell content
                let align_attr = match alignment {
                    Some(align) => format!(" align=\"{}\"", align),
                    None => String::new(),
                };
                write!(self.output, "<td{}>", align_attr).unwrap();
                for node in content {
                    self.visit(node);
                }
                write!(self.output, "</td>").unwrap();
            }
            Node::ThematicBreak { .. } => {
                write!(self.output, "<hr>").unwrap();
            }
            Node::SoftLineBreak { .. } => {
                write!(self.output, " ").unwrap();
            }
            Node::HardLineBreak { .. } => {
                write!(self.output, "<br>").unwrap();
            }
            Node::HtmlBlock { content, .. } => {
                write!(self.output, "{}", content).unwrap();
            }
            Node::FencedCodeBlock { content, language, .. } => {
                match language {
                    Some(lang) => {
                        write!(self.output, "<pre><code class=\"language-{}\">{}</code></pre>", 
                               lang, self.escape_html(content)).unwrap();
                    }
                    None => {
                        write!(self.output, "<pre><code>{}</code></pre>", 
                               self.escape_html(content)).unwrap();
                    }
                }
            }
            Node::IndentedCodeBlock { content, .. } => {
                write!(self.output, "<pre><code>{}</code></pre>", 
                       self.escape_html(content)).unwrap();
            }
            Node::LinkReferenceDefinition { .. } => {
                // Usually not rendered
            }
            Node::MathBlockDisplay { content, .. } => {
                write!(self.output, "<div class=\"math-block\">{}</div>", content).unwrap();
            }
            Node::CodeSpan { content, .. } => {
                write!(self.output, "<code>{}</code>", self.escape_html(content)).unwrap();
            }
            Node::HtmlInlineTag { content, .. } => {
                if let Some(nodes) = content {
                    for node in nodes {
                        self.visit(node);
                    }
                }
            }
            Node::AutolinkUrl { url, .. } => {
                write!(self.output, "<a href=\"{}\">{}</a>", 
                       self.escape_html(url), self.escape_html(url)).unwrap();
            }
            Node::AutolinkEmail { email, .. } => {
                write!(self.output, "<a href=\"mailto:{}\">{}</a>", 
                       self.escape_html(email), self.escape_html(email)).unwrap();
            }
            Node::AdmonitionWithIcon { kind, content, .. } => {
                write!(self.output, "<div class=\"admonition {}\">", kind).unwrap();
                for node in content {
                    self.visit(node);
                }
                write!(self.output, "</div>").unwrap();
            }
            Node::TabWithMetadata { name, content, .. } => {
                let title = name.as_ref().map(|s| s.as_str()).unwrap_or("Tab");
                write!(self.output, "<div class=\"tab\" data-title=\"{}\">", title).unwrap();
                for node in content {
                    self.visit(node);
                }
                write!(self.output, "</div>").unwrap();
            }
            Node::UserMentionWithMetadata { username, platform, display_name, .. } => {
                let platform_str = platform.as_ref().map(|s| s.as_str()).unwrap_or("unknown");
                let display = display_name.as_ref().unwrap_or(username);
                write!(self.output, "<span class=\"user-mention\" data-platform=\"{}\">@{} ({})</span>", 
                       platform_str, username, display).unwrap();
            }
            Node::Citation { key, locator, .. } => {
                match locator {
                    Some(loc) => {
                        write!(self.output, "<cite data-key=\"{}\" data-locator=\"{}\">{}</cite>", 
                               key, loc, key).unwrap();
                    }
                    None => {
                        write!(self.output, "<cite data-key=\"{}\">{}</cite>", key, key).unwrap();
                    }
                }
            }
            Node::Keyboard { keys, .. } => {
                write!(self.output, "<kbd>{}</kbd>", keys.join("+")).unwrap();
            }
            Node::Mark { content, reason, .. } => {
                let class = reason.as_ref().map(|r| format!(" class=\"{}\"", r)).unwrap_or_default();
                write!(self.output, "<mark{}>", class).unwrap();
                for node in content {
                    self.visit(node);
                }
                write!(self.output, "</mark>").unwrap();
            }
            Node::Details { summary, content, open, .. } => {
                let open_attr = if *open { " open" } else { "" };
                write!(self.output, "<details{}>", open_attr).unwrap();
                write!(self.output, "<summary>").unwrap();
                for node in summary {
                    self.visit(node);
                }
                write!(self.output, "</summary>").unwrap();
                for node in content {
                    self.visit(node);
                }
                write!(self.output, "</details>").unwrap();
            }
        }
    }
    
    fn visit_document(&mut self, children: &[Node]) -> Self::Output {
        write!(self.output, "<div").unwrap();
        self.write_class("document");
        write!(self.output, ">").unwrap();
        
        for child in children {
            self.visit(child);
        }
        
        write!(self.output, "</div>").unwrap();
    }
    
    fn visit_heading(&mut self, level: u8, content: &[Node]) -> Self::Output {
        debug!("HtmlRenderer::visit_heading - level={}, content.len()={}", level, content.len());
        for (i, child) in content.iter().enumerate() {
            debug!("HtmlRenderer::visit_heading - child[{}]: {:?}", i, std::mem::discriminant(child));
        }
        
        let level = level.clamp(1, 6); // Ensure valid HTML heading level
        
        write!(self.output, "<h{}", level).unwrap();
        self.write_class(&format!("heading-{}", level));
        write!(self.output, ">").unwrap();
        
        for child in content {
            self.visit(child);
        }
        
        write!(self.output, "</h{}>", level).unwrap();
    }
    
    fn visit_paragraph(&mut self, content: &[Node]) -> Self::Output {
        write!(self.output, "<p").unwrap();
        self.write_class("paragraph");
        write!(self.output, ">").unwrap();
        
        for child in content {
            self.visit(child);
        }
        
        write!(self.output, "</p>").unwrap();
    }
    
    fn visit_code_block(&mut self, language: &Option<String>, content: &str) -> Self::Output {
        write!(self.output, "<pre").unwrap();
        self.write_class("code-block");
        write!(self.output, "><code").unwrap();
        
        if let Some(lang) = language {
            if self.options.syntax_highlighting {
                write!(self.output, " class=\"language-{}\"", self.escape_html(lang)).unwrap();
            }
        }
        
        write!(self.output, ">{}</code></pre>", self.escape_html(content)).unwrap();
    }
    
    fn visit_list(&mut self, ordered: bool, items: &[Node]) -> Self::Output {
        let tag = if ordered { "ol" } else { "ul" };
        
        write!(self.output, "<{}", tag).unwrap();
        self.write_class("list");
        write!(self.output, ">").unwrap();
        
        for item in items {
            self.visit(item);
        }
        
        write!(self.output, "</{}>", tag).unwrap();
    }
    
    fn visit_table(&mut self, headers: &[Node], rows: &[Vec<Node>]) -> Self::Output {
        write!(self.output, "<table").unwrap();
        self.write_class("table");
        write!(self.output, ">").unwrap();
        
        if !headers.is_empty() {
            write!(self.output, "<thead><tr>").unwrap();
            for header in headers {
                write!(self.output, "<th>").unwrap();
                self.visit(header);
                write!(self.output, "</th>").unwrap();
            }
            write!(self.output, "</tr></thead>").unwrap();
        }
        
        if !rows.is_empty() {
            write!(self.output, "<tbody>").unwrap();
            for row in rows {
                write!(self.output, "<tr>").unwrap();
                for cell in row {
                    write!(self.output, "<td>").unwrap();
                    self.visit(cell);
                    write!(self.output, "</td>").unwrap();
                }
                write!(self.output, "</tr>").unwrap();
            }
            write!(self.output, "</tbody>").unwrap();
        }
        
        write!(self.output, "</table>").unwrap();
    }
    
    fn visit_text(&mut self, content: &str) -> Self::Output {
        // First escape HTML, then convert newlines to <br> tags
        let escaped_content = self.escape_html(content);
        let html_content = escaped_content.replace('\n', "<br>");
        write!(self.output, "{}", html_content).unwrap();
    }
    
    fn visit_emphasis(&mut self, content: &[Node]) -> Self::Output {
        write!(self.output, "<em").unwrap();
        self.write_class("emphasis");
        write!(self.output, ">").unwrap();
        
        for child in content {
            self.visit(child);
        }
        
        write!(self.output, "</em>").unwrap();
    }
    
    fn visit_strong(&mut self, content: &[Node]) -> Self::Output {
        write!(self.output, "<strong").unwrap();
        self.write_class("strong");
        write!(self.output, ">").unwrap();
        
        for child in content {
            self.visit(child);
        }
        
        write!(self.output, "</strong>").unwrap();
    }
    
    fn visit_code(&mut self, content: &str) -> Self::Output {
        write!(self.output, "<code").unwrap();
        self.write_class("inline-code");
        write!(self.output, ">{}</code>", self.escape_html(content)).unwrap();
    }
    
    fn visit_link(&mut self, text: &[Node], url: &str, title: &Option<String>) -> Self::Output {
        // Check for YouTube embedding
        if self.options.youtube_embed && self.is_youtube_url(url) {
            self.render_youtube_embed(url);
            return;
        }
        
        write!(self.output, "<a href=\"{}\"", self.escape_html(url)).unwrap();
        self.write_class("link");
        
        if let Some(title) = title {
            write!(self.output, " title=\"{}\"", self.escape_html(title)).unwrap();
        }
        
        // Add target="_blank" for external links
        if url.starts_with("http://") || url.starts_with("https://") {
            write!(self.output, " target=\"_blank\" rel=\"noopener noreferrer\"").unwrap();
        }
        
        write!(self.output, ">").unwrap();
        
        for child in text {
            self.visit(child);
        }
        
        write!(self.output, "</a>").unwrap();
    }
    
    fn visit_image(&mut self, alt: &str, url: &str, title: &Option<String>) -> Self::Output {
        write!(self.output, "<img src=\"{}\" alt=\"{}\"", 
               self.escape_html(url), 
               self.escape_html(alt)).unwrap();
        
        self.write_class("image");
        
        if let Some(title) = title {
            write!(self.output, " title=\"{}\"", self.escape_html(title)).unwrap();
        }
        
        // Add loading="lazy" for better performance
        write!(self.output, " loading=\"lazy\"").unwrap();
        
        write!(self.output, " />").unwrap();
    }
    
    fn visit_macro(&mut self, name: &str, arguments: &[String], content: &Option<Vec<Node>>) -> Self::Output {
        write!(self.output, "<div").unwrap();
        self.write_class(&format!("macro-{}", name));
        write!(self.output, " data-macro=\"{}\"", self.escape_html(name)).unwrap();
        
        for (i, arg) in arguments.iter().enumerate() {
            write!(self.output, " data-arg-{}=\"{}\"", i, self.escape_html(arg)).unwrap();
        }
        
        write!(self.output, ">").unwrap();
        
        if let Some(content) = content {
            for child in content {
                self.visit(child);
            }
        }
        
        write!(self.output, "</div>").unwrap();
    }
    
    fn visit_horizontal_rule(&mut self) -> Self::Output {
        write!(self.output, "<hr").unwrap();
        self.write_class("horizontal-rule");
        write!(self.output, " />").unwrap();
    }
    
    fn visit_block_quote(&mut self, content: &[Node]) -> Self::Output {
        write!(self.output, "<blockquote").unwrap();
        self.write_class("blockquote");
        write!(self.output, ">").unwrap();
        
        for child in content {
            self.visit(child);
        }
        
        write!(self.output, "</blockquote>").unwrap();
    }
    
    fn visit_admonition(&mut self, kind: &str, content: &[Node]) -> Self::Output {
        write!(self.output, "<div").unwrap();
        self.write_class(&format!("admonition admonition-{}", kind));
        write!(self.output, " data-admonition=\"{}\">", self.escape_html(kind)).unwrap();
        
        // Add admonition header
        write!(self.output, "<div").unwrap();
        self.write_class("admonition-header");
        write!(self.output, ">{}</div>", self.escape_html(&kind.to_uppercase())).unwrap();
        
        // Add admonition content
        write!(self.output, "<div").unwrap();
        self.write_class("admonition-content");
        write!(self.output, ">").unwrap();
        
        for child in content {
            self.visit(child);
        }
        
        write!(self.output, "</div></div>").unwrap();
    }
    
    fn visit_unknown(&mut self, content: &str, rule: &str) -> Self::Output {
        write!(self.output, "<div").unwrap();
        self.write_class("unknown");
        write!(self.output, " data-rule=\"{}\" title=\"Unknown syntax: {}\">", 
               self.escape_html(rule), 
               self.escape_html(rule)).unwrap();
        write!(self.output, "{}", self.escape_html(content)).unwrap();
        write!(self.output, "</div>").unwrap();
    }
    
    fn default_output(&self) -> Self::Output {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::marco_engine::ast::{Node, Span};
    
    #[test]
    fn test_html_renderer() {
        let ast = Node::Document {
            children: vec![
                Node::heading(1, vec![Node::text("Hello World", Span::empty())], Span::empty()),
                Node::paragraph(vec![
                    Node::text("This is ", Span::empty()),
                    Node::Strong {
                        content: vec![Node::text("bold", Span::empty())],
                        span: Span::empty(),
                    },
                    Node::text(" text.", Span::empty()),
                ], Span::empty()),
            ],
            span: Span::empty(),
        };
        
        let options = HtmlOptions::default();
        let renderer = HtmlRenderer::new(options);
        let html = renderer.render(&ast);
        
        assert!(html.contains("<h1"));
        assert!(html.contains("marco-heading-1"));
        assert!(html.contains("<strong"));
        assert!(html.contains("marco-strong"));
        assert!(html.contains("Hello World"));
        assert!(html.contains("bold"));
    }
    
    #[test]
    fn test_youtube_embed() {
        let ast = Node::Document {
            children: vec![
                Node::Link {
                    text: vec![Node::text("Video", Span::empty())],
                    url: "https://youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
                    title: None,
                    span: Span::empty(),
                }
            ],
            span: Span::empty(),
        };
        
        let options = HtmlOptions::default();
        let renderer = HtmlRenderer::new(options);
        let html = renderer.render(&ast);
        
        assert!(html.contains("youtube.com/embed/dQw4w9WgXcQ"));
        assert!(html.contains("iframe"));
    }
}
