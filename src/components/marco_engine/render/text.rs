use crate::components::marco_engine::ast::{Node, Visitor};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct TextOptions {
    pub line_width: Option<usize>,
    pub preserve_formatting: bool,
    pub bullet_char: char,
    pub number_format: String,
    pub heading_underline: bool,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            line_width: Some(80),
            preserve_formatting: true,
            bullet_char: '•',
            number_format: "{}.".to_string(),
            heading_underline: true,
        }
    }
}

pub struct TextRenderer {
    output: String,
    options: TextOptions,
    current_list_depth: usize,
    current_list_index: Vec<usize>,
}

impl TextRenderer {
    pub fn new(options: TextOptions) -> Self {
        Self {
            output: String::with_capacity(1024),
            options,
            current_list_depth: 0,
            current_list_index: Vec::new(),
        }
    }

    pub fn render(mut self, ast: &Node) -> String {
        self.visit(ast);
        self.output.trim().to_string()
    }

    fn write_line(&mut self, content: &str) {
        writeln!(self.output, "{}", content).unwrap();
    }

    fn write_indented(&mut self, content: &str, indent: usize) {
        let indent_str = "  ".repeat(indent);
        writeln!(self.output, "{}{}", indent_str, content).unwrap();
    }

    fn write_heading_underline(&mut self, level: u8, text_length: usize) {
        if !self.options.heading_underline {
            return;
        }

        let underline_char = match level {
            1 => '=',
            2 => '-',
            _ => '~',
        };

        let underline = underline_char.to_string().repeat(text_length);
        self.write_line(&underline);
    }

    fn wrap_text(&self, text: &str) -> String {
        if let Some(width) = self.options.line_width {
            if text.len() <= width {
                return text.to_string();
            }

            let mut wrapped = String::new();
            let mut current_line_length = 0;

            for word in text.split_whitespace() {
                if current_line_length + word.len() + 1 > width {
                    if !wrapped.is_empty() {
                        wrapped.push('\n');
                    }
                    wrapped.push_str(word);
                    current_line_length = word.len();
                } else {
                    if current_line_length > 0 {
                        wrapped.push(' ');
                        current_line_length += 1;
                    }
                    wrapped.push_str(word);
                    current_line_length += word.len();
                }
            }

            wrapped
        } else {
            text.to_string()
        }
    }

    fn extract_text_content(&self, nodes: &[Node]) -> String {
        let mut text = String::new();
        for node in nodes {
            match node {
                Node::Text { content, .. } => text.push_str(content),
                Node::Code { content, .. } => {
                    text.push('`');
                    text.push_str(content);
                    text.push('`');
                }
                Node::Emphasis { content, .. } => {
                    text.push('*');
                    text.push_str(&self.extract_text_content(content));
                    text.push('*');
                }
                Node::Strong { content, .. } => {
                    text.push_str("**");
                    text.push_str(&self.extract_text_content(content));
                    text.push_str("**");
                }
                Node::Link {
                    text: link_text,
                    url,
                    ..
                } => {
                    text.push_str(&self.extract_text_content(link_text));
                    text.push_str(" (");
                    text.push_str(url);
                    text.push(')');
                }
                Node::Image { alt, url, .. } => {
                    text.push_str("[Image: ");
                    text.push_str(alt);
                    text.push_str("](");
                    text.push_str(url);
                    text.push(')');
                }
                _ => {
                    // For other node types, try to extract their text recursively
                    // This is a simplified approach
                }
            }
        }
        text
    }
}

impl Visitor for TextRenderer {
    type Output = ();

    fn visit(&mut self, node: &Node) -> Self::Output {
        match node {
            Node::Document { children, .. } => self.visit_document(children),
            Node::Heading { level, content, .. } => self.visit_heading(*level, content),
            Node::Paragraph { content, .. } => self.visit_paragraph(content),
            Node::CodeBlock {
                language, content, ..
            } => self.visit_code_block(language, content),
            Node::MathBlock { content, .. } => {
                self.write_line(&format!("Math: {}", content));
                self.write_line("");
            }
            Node::List { ordered, items, .. } => self.visit_list(*ordered, items),
            Node::ListItem {
                content, checked, ..
            } => {
                let prefix = if let Some(checked) = checked {
                    if *checked {
                        "[x] ".to_string()
                    } else {
                        "[ ] ".to_string()
                    }
                } else if self.current_list_index.is_empty()
                    || self.current_list_index.last() == Some(&0)
                {
                    format!("{} ", self.options.bullet_char)
                } else {
                    let index = self.current_list_index.last().unwrap();
                    self.options.number_format.replace("{}", &index.to_string()) + " "
                };

                let content_text = self.extract_text_content(content);
                let wrapped = self.wrap_text(&content_text);
                let lines: Vec<&str> = wrapped.lines().collect();

                if let Some(first_line) = lines.first() {
                    self.write_indented(
                        &format!("{}{}", prefix, first_line),
                        self.current_list_depth,
                    );

                    for line in lines.iter().skip(1) {
                        let continuation_indent = " ".repeat(prefix.len());
                        self.write_indented(
                            &format!("{}{}", continuation_indent, line),
                            self.current_list_depth,
                        );
                    }
                }
            }
            Node::Table { headers, rows, .. } => self.visit_table(headers, rows),
            Node::Text { content, .. } => self.visit_text(content),
            Node::Emphasis { content, .. } => self.visit_emphasis(content),
            Node::Strong { content, .. } => self.visit_strong(content),
            Node::Code { content, .. } => self.visit_code(content),
            Node::Link {
                text, url, title, ..
            } => self.visit_link(text, url, title),
            Node::Image {
                alt, url, title, ..
            } => self.visit_image(alt, url, title),
            Node::Macro {
                name,
                arguments,
                content,
                ..
            } => self.visit_macro(name, arguments, content),
            Node::HorizontalRule { .. } => self.visit_horizontal_rule(),
            Node::BlockQuote { content, .. } => self.visit_block_quote(content),
            Node::Unknown { content, rule, .. } => self.visit_unknown(content, rule),
        }
    }

    fn visit_document(&mut self, children: &[Node]) -> Self::Output {
        for (i, child) in children.iter().enumerate() {
            self.visit(child);

            // Add spacing between blocks
            if i < children.len() - 1 {
                match child {
                    Node::Heading { .. }
                    | Node::Paragraph { .. }
                    | Node::CodeBlock { .. }
                    | Node::List { .. }
                    | Node::Table { .. }
                    | Node::BlockQuote { .. } => {
                        self.write_line("");
                    }
                    _ => {}
                }
            }
        }
    }

    fn visit_heading(&mut self, level: u8, content: &[Node]) -> Self::Output {
        let text = self.extract_text_content(content);
        let wrapped = self.wrap_text(&text);

        self.write_line(&wrapped);
        self.write_heading_underline(level, wrapped.len());
    }

    fn visit_paragraph(&mut self, content: &[Node]) -> Self::Output {
        let text = self.extract_text_content(content);
        let wrapped = self.wrap_text(&text);
        self.write_line(&wrapped);
    }

    fn visit_code_block(&mut self, language: &Option<String>, content: &str) -> Self::Output {
        if let Some(lang) = language {
            self.write_line(&format!("Code ({}): ", lang));
        } else {
            self.write_line("Code:");
        }

        for line in content.lines() {
            self.write_indented(line, 1);
        }

        self.write_line("");
    }

    fn visit_list(&mut self, ordered: bool, items: &[Node]) -> Self::Output {
        self.current_list_depth += 1;

        if ordered {
            self.current_list_index.push(1);
        } else {
            self.current_list_index.push(0);
        }

        for item in items {
            self.visit(item);

            if ordered {
                if let Some(index) = self.current_list_index.last_mut() {
                    *index += 1;
                }
            }
        }

        self.current_list_index.pop();
        self.current_list_depth -= 1;
    }

    fn visit_table(&mut self, headers: &[Node], rows: &[Vec<Node>]) -> Self::Output {
        // Simple table representation
        if !headers.is_empty() {
            let header_texts: Vec<String> = headers
                .iter()
                .map(|h| self.extract_text_content(&[h.clone()]))
                .collect();
            self.write_line(&header_texts.join(" | "));
            self.write_line(&"-".repeat(header_texts.join(" | ").len()));
        }

        for row in rows {
            let row_texts: Vec<String> = row
                .iter()
                .map(|cell| self.extract_text_content(&[cell.clone()]))
                .collect();
            self.write_line(&row_texts.join(" | "));
        }

        self.write_line("");
    }

    fn visit_text(&mut self, content: &str) -> Self::Output {
        write!(self.output, "{}", content).unwrap();
    }

    fn visit_emphasis(&mut self, content: &[Node]) -> Self::Output {
        if self.options.preserve_formatting {
            write!(self.output, "*").unwrap();
        }

        for child in content {
            self.visit(child);
        }

        if self.options.preserve_formatting {
            write!(self.output, "*").unwrap();
        }
    }

    fn visit_strong(&mut self, content: &[Node]) -> Self::Output {
        if self.options.preserve_formatting {
            write!(self.output, "**").unwrap();
        }

        for child in content {
            self.visit(child);
        }

        if self.options.preserve_formatting {
            write!(self.output, "**").unwrap();
        }
    }

    fn visit_code(&mut self, content: &str) -> Self::Output {
        if self.options.preserve_formatting {
            write!(self.output, "`{}`", content).unwrap();
        } else {
            write!(self.output, "{}", content).unwrap();
        }
    }

    fn visit_link(&mut self, text: &[Node], url: &str, _title: &Option<String>) -> Self::Output {
        for child in text {
            self.visit(child);
        }

        if self.options.preserve_formatting {
            write!(self.output, " ({})", url).unwrap();
        }
    }

    fn visit_image(&mut self, alt: &str, url: &str, _title: &Option<String>) -> Self::Output {
        if self.options.preserve_formatting {
            write!(self.output, "[Image: {}]({})", alt, url).unwrap();
        } else {
            write!(self.output, "Image: {}", alt).unwrap();
        }
    }

    fn visit_macro(
        &mut self,
        name: &str,
        arguments: &[String],
        content: &Option<Vec<Node>>,
    ) -> Self::Output {
        write!(self.output, "Macro: {} ", name).unwrap();

        if !arguments.is_empty() {
            write!(self.output, "({}) ", arguments.join(", ")).unwrap();
        }

        if let Some(content) = content {
            for child in content {
                self.visit(child);
            }
        }
    }

    fn visit_horizontal_rule(&mut self) -> Self::Output {
        self.write_line(&"-".repeat(self.options.line_width.unwrap_or(80)));
    }

    fn visit_block_quote(&mut self, content: &[Node]) -> Self::Output {
        let original_output = std::mem::take(&mut self.output);

        for child in content {
            self.visit(child);
        }

        let quote_content = std::mem::replace(&mut self.output, original_output);

        for line in quote_content.lines() {
            self.write_line(&format!("> {}", line));
        }
    }

    fn visit_unknown(&mut self, content: &str, rule: &str) -> Self::Output {
        write!(self.output, "[Unknown {}: {}]", rule, content).unwrap();
    }

    fn default_output(&self) -> Self::Output {
        ()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::marco_engine::ast::{Node, Span};

    #[test]
    fn test_text_renderer() {
        let ast = Node::Document {
            children: vec![
                Node::heading(
                    1,
                    vec![Node::text("Hello World", Span::empty())],
                    Span::empty(),
                ),
                Node::paragraph(
                    vec![
                        Node::text("This is ", Span::empty()),
                        Node::Strong {
                            content: vec![Node::text("bold", Span::empty())],
                            span: Span::empty(),
                        },
                        Node::text(" text.", Span::empty()),
                    ],
                    Span::empty(),
                ),
            ],
            span: Span::empty(),
        };

        let options = TextOptions::default();
        let renderer = TextRenderer::new(options);
        let text = renderer.render(&ast);

        assert!(text.contains("Hello World"));
        assert!(text.contains("==========="));
        assert!(text.contains("**bold**"));
    }

    #[test]
    fn test_list_rendering() {
        let ast = Node::Document {
            children: vec![Node::List {
                ordered: true,
                items: vec![
                    Node::ListItem {
                        content: vec![Node::text("First item", Span::empty())],
                        checked: None,
                        span: Span::empty(),
                    },
                    Node::ListItem {
                        content: vec![Node::text("Second item", Span::empty())],
                        checked: None,
                        span: Span::empty(),
                    },
                ],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };

        let options = TextOptions::default();
        let renderer = TextRenderer::new(options);
        let text = renderer.render(&ast);

        assert!(text.contains("1. First item"));
        assert!(text.contains("2. Second item"));
    }
}
