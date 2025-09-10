use crate::components::marco_engine::ast::Node;

/// Trait for visiting AST nodes
#[allow(unused_variables)]
pub trait Visitor {
    type Output;

    fn visit(&mut self, node: &Node) -> Self::Output {
        match node {
            // ===========================================
            // DOCUMENT STRUCTURE
            // ===========================================
            Node::Document { children, .. } => self.visit_document(children),

            // ===========================================
            // BLOCK ELEMENTS
            // ===========================================
            Node::Heading { level, content, .. } => self.visit_heading(*level, content),
            Node::SetextHeading {
                level,
                content,
                underline_char,
                ..
            } => self.visit_setext_heading(*level, content, *underline_char),
            Node::Paragraph { content, .. } => self.visit_paragraph(content),
            Node::FencedCodeBlock {
                language,
                info_string,
                content,
                fence_char,
                fence_length,
                ..
            } => self.visit_fenced_code_block(
                language,
                info_string,
                content,
                *fence_char,
                *fence_length,
            ),
            Node::IndentedCodeBlock { content, .. } => self.visit_indented_code_block(content),
            Node::CodeBlock {
                language, content, ..
            } => self.visit_code_block(language, content),
            Node::MathBlockDisplay {
                content, delimiter, ..
            } => self.visit_math_block_display(content, delimiter),
            Node::MathBlock { content, .. } => self.visit_math_block(content),
            Node::List { ordered, items, .. } => self.visit_list(*ordered, items),
            Node::ListItem {
                content, checked, ..
            } => self.visit_list_item(content, checked),
            Node::BlockQuote { content, .. } => self.visit_block_quote(content),
            Node::ThematicBreak { marker, .. } => self.visit_thematic_break(*marker),
            Node::HorizontalRule { .. } => self.visit_horizontal_rule(),
            Node::DefinitionList { items, .. } => self.visit_definition_list(items),
            Node::DefinitionTerm { content, .. } => self.visit_definition_term(content),
            Node::DefinitionDescription { content, .. } => {
                self.visit_definition_description(content)
            }

            // ===========================================
            // INLINE ELEMENTS
            // ===========================================
            Node::Text { content, .. } => self.visit_text(content),
            Node::Strong { content, .. } => self.visit_strong(content),
            Node::Emphasis { content, .. } => self.visit_emphasis(content),
            Node::Strikethrough { content, .. } => self.visit_strikethrough(content),
            Node::Highlight { content, .. } => self.visit_highlight(content),
            Node::Mark {
                content, reason, ..
            } => self.visit_mark(content, reason),
            Node::Superscript { content, .. } => self.visit_superscript(content),
            Node::Subscript { content, .. } => self.visit_subscript(content),
            Node::Code { content, .. } => self.visit_code(content),
            Node::CodeSpan {
                content,
                backtick_count,
                ..
            } => self.visit_code_span(content, *backtick_count),
            Node::MathInline { content, .. } => self.visit_math_inline(content),
            Node::HardLineBreak { .. } => self.visit_hard_line_break(),
            Node::SoftLineBreak { .. } => self.visit_soft_line_break(),
            Node::LineBreak { .. } => self.visit_line_break(),
            Node::EscapedChar { character, .. } => self.visit_escaped_char(*character),
            Node::Emoji { name, .. } => self.visit_emoji(name),
            Node::Keyboard { keys, .. } => self.visit_keyboard(keys),

            // ===========================================
            // LINK ELEMENTS
            // ===========================================
            Node::Link {
                text, url, title, ..
            } => self.visit_link(text, url, title),
            Node::Image {
                alt, url, title, ..
            } => self.visit_image(alt, url, title),
            Node::Autolink { url, .. } => self.visit_autolink(url),
            Node::AutolinkUrl { url, .. } => self.visit_autolink_url(url),
            Node::AutolinkEmail { email, .. } => self.visit_autolink_email(email),
            Node::ReferenceLink { text, label, .. } => self.visit_reference_link(text, label),
            Node::ReferenceImage { alt, label, .. } => self.visit_reference_image(alt, label),
            Node::ReferenceDefinition {
                label, url, title, ..
            } => self.visit_reference_definition(label, url, title),
            Node::LinkReferenceDefinition {
                label,
                destination,
                title,
                ..
            } => self.visit_link_reference_definition(label, destination, title),
            Node::FootnoteRef { label, .. } => self.visit_footnote_ref(label),
            Node::InlineFootnote { content, .. } => self.visit_inline_footnote(content),
            Node::FootnoteDefinition { label, content, .. } => {
                self.visit_footnote_definition(label, content)
            }

            // ===========================================
            // TABLE ELEMENTS
            // ===========================================
            Node::Table { headers, rows, .. } => self.visit_table(headers, rows),
            Node::TableHeader { cells, .. } => self.visit_table_header(cells),
            Node::TableRow { cells, .. } => self.visit_table_row(cells),
            Node::TableCell {
                content, alignment, ..
            } => self.visit_table_cell(content, alignment),

            // ===========================================
            // MARCO EXTENSIONS
            // ===========================================
            Node::UserMention {
                username,
                platform,
                display_name,
                ..
            } => self.visit_user_mention(username, platform, display_name),
            Node::UserMentionWithMetadata {
                username,
                platform,
                display_name,
                user_id,
                avatar_url,
                ..
            } => self.visit_user_mention_with_metadata(
                username,
                platform,
                display_name,
                user_id,
                avatar_url,
            ),
            Node::Bookmark {
                label, path, line, ..
            } => self.visit_bookmark(label, path, line),
            Node::PageTag { format, .. } => self.visit_page_tag(format),
            Node::DocumentReference { path, .. } => self.visit_document_reference(path),
            Node::TableOfContents {
                depth, document, ..
            } => self.visit_table_of_contents(depth, document),
            Node::RunInline {
                script_type,
                command,
                ..
            } => self.visit_run_inline(script_type, command),
            Node::RunBlock {
                script_type,
                content,
                ..
            } => self.visit_run_block(script_type, content),
            Node::DiagramBlock {
                diagram_type,
                content,
                ..
            } => self.visit_diagram_block(diagram_type, content),
            Node::TabBlock { title, tabs, .. } => self.visit_tab_block(title, tabs),
            Node::Tab { name, content, .. } => self.visit_tab(name, content),
            Node::TabWithMetadata {
                name,
                icon,
                active,
                content,
                ..
            } => self.visit_tab_with_metadata(name, icon, *active, content),
            Node::Admonition { kind, content, .. } => self.visit_admonition(kind, content),
            Node::AdmonitionWithIcon {
                kind,
                icon,
                title,
                content,
                ..
            } => self.visit_admonition_with_icon(kind, icon, title, content),
            Node::TaskItem {
                checked, content, ..
            } => self.visit_task_item(*checked, content),
            Node::Citation { key, locator, .. } => self.visit_citation(key, locator),
            Node::Details {
                summary,
                content,
                open,
                ..
            } => self.visit_details(summary, content, *open),
            Node::Macro {
                name,
                arguments,
                content,
                ..
            } => self.visit_macro(name, arguments, content),

            // ===========================================
            // HTML ELEMENTS
            // ===========================================
            Node::InlineHTML { content, .. } => self.visit_inline_html(content),
            Node::BlockHTML { content, .. } => self.visit_block_html(content),
            Node::HtmlBlock {
                html_type, content, ..
            } => self.visit_html_block(*html_type, content),
            Node::HtmlInlineTag {
                tag_name,
                attributes,
                content,
                is_self_closing,
                ..
            } => self.visit_html_inline_tag(tag_name, attributes, content, *is_self_closing),

            // ===========================================
            // ERROR RECOVERY
            // ===========================================
            Node::Unknown { content, rule, .. } => self.visit_unknown(content, rule),
        }
    }

    // Core document structure
    fn visit_document(&mut self, children: &[Node]) -> Self::Output {
        for child in children {
            self.visit(child);
        }
        self.default_output()
    }

    // Block elements
    fn visit_heading(&mut self, level: u8, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_paragraph(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_code_block(&mut self, language: &Option<String>, content: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_math_block(&mut self, content: &str) -> Self::Output {
        self.default_output()
    }

    // Lists
    fn visit_list(&mut self, ordered: bool, items: &[Node]) -> Self::Output {
        for item in items {
            self.visit(item);
        }
        self.default_output()
    }

    fn visit_list_item(&mut self, content: &[Node], checked: &Option<bool>) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    // Tables
    fn visit_table(&mut self, headers: &[Node], rows: &[Vec<Node>]) -> Self::Output {
        for header in headers {
            self.visit(header);
        }
        for row in rows {
            for cell in row {
                self.visit(cell);
            }
        }
        self.default_output()
    }

    // Definition lists
    fn visit_definition_list(&mut self, items: &[Node]) -> Self::Output {
        for item in items {
            self.visit(item);
        }
        self.default_output()
    }

    fn visit_definition_term(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_definition_description(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    // Footnotes
    fn visit_footnote_definition(&mut self, label: &str, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    // Reference definitions
    fn visit_reference_definition(
        &mut self,
        label: &str,
        url: &str,
        title: &Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    // HTML content
    fn visit_inline_html(&mut self, content: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_block_html(&mut self, content: &str) -> Self::Output {
        self.default_output()
    }

    // Inline elements
    fn visit_text(&mut self, content: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_emphasis(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_strong(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_strikethrough(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_highlight(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_superscript(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_subscript(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_code(&mut self, content: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_math_inline(&mut self, content: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_link(&mut self, text: &[Node], url: &str, title: &Option<String>) -> Self::Output {
        for child in text {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_image(&mut self, alt: &str, url: &str, title: &Option<String>) -> Self::Output {
        self.default_output()
    }

    fn visit_autolink(&mut self, url: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_reference_link(&mut self, text: &[Node], label: &str) -> Self::Output {
        for child in text {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_reference_image(&mut self, alt: &str, label: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_footnote_ref(&mut self, label: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_inline_footnote(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_emoji(&mut self, name: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_line_break(&mut self) -> Self::Output {
        self.default_output()
    }

    fn visit_escaped_char(&mut self, character: char) -> Self::Output {
        self.default_output()
    }

    // Marco-specific elements
    fn visit_macro(
        &mut self,
        name: &str,
        arguments: &[String],
        content: &Option<Vec<Node>>,
    ) -> Self::Output {
        if let Some(content) = content {
            for child in content {
                self.visit(child);
            }
        }
        self.default_output()
    }

    // Marco extensions
    fn visit_user_mention(
        &mut self,
        username: &str,
        platform: &Option<String>,
        display_name: &Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_bookmark(&mut self, label: &str, path: &str, line: &Option<u32>) -> Self::Output {
        self.default_output()
    }

    fn visit_page_tag(&mut self, format: &Option<String>) -> Self::Output {
        self.default_output()
    }

    fn visit_document_reference(&mut self, path: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_table_of_contents(
        &mut self,
        depth: &Option<u8>,
        document: &Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_run_inline(&mut self, script_type: &str, command: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_run_block(&mut self, script_type: &str, content: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_diagram_block(&mut self, diagram_type: &str, content: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_tab_block(&mut self, title: &Option<String>, tabs: &[Node]) -> Self::Output {
        for tab in tabs {
            self.visit(tab);
        }
        self.default_output()
    }

    fn visit_tab(&mut self, name: &Option<String>, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    // Additional elements
    fn visit_horizontal_rule(&mut self) -> Self::Output {
        self.default_output()
    }

    fn visit_block_quote(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_admonition(&mut self, kind: &str, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    // Task list item
    fn visit_task_item(&mut self, checked: bool, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    // Setext headings (CommonMark)
    fn visit_setext_heading(
        &mut self,
        level: u8,
        content: &[Node],
        underline_char: char,
    ) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    // Table components
    fn visit_table_header(&mut self, cells: &[Node]) -> Self::Output {
        for cell in cells {
            self.visit(cell);
        }
        self.default_output()
    }

    fn visit_table_row(&mut self, cells: &[Node]) -> Self::Output {
        for cell in cells {
            self.visit(cell);
        }
        self.default_output()
    }

    fn visit_table_cell(&mut self, content: &[Node], alignment: &Option<String>) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    // Thematic break
    fn visit_thematic_break(&mut self, marker: char) -> Self::Output {
        self.default_output()
    }

    // Soft line break
    fn visit_soft_line_break(&mut self) -> Self::Output {
        self.default_output()
    }

    // HTML blocks with type information
    fn visit_html_block(&mut self, html_type: u8, content: &str) -> Self::Output {
        self.default_output()
    }

    // Fenced code blocks with metadata
    fn visit_fenced_code_block(
        &mut self,
        language: &Option<String>,
        info_string: &Option<String>,
        content: &str,
        fence_char: char,
        fence_length: u8,
    ) -> Self::Output {
        self.default_output()
    }

    // Indented code block
    fn visit_indented_code_block(&mut self, content: &str) -> Self::Output {
        self.default_output()
    }

    // Link reference definition components
    fn visit_link_reference_definition(
        &mut self,
        label: &str,
        destination: &str,
        title: &Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    // Hard line break
    fn visit_hard_line_break(&mut self) -> Self::Output {
        self.default_output()
    }

    // Math blocks with different delimiters
    fn visit_math_block_display(&mut self, content: &str, delimiter: &str) -> Self::Output {
        self.default_output()
    }

    // Code spans with backtick count
    fn visit_code_span(&mut self, content: &str, backtick_count: u8) -> Self::Output {
        self.default_output()
    }

    // Raw HTML spans
    fn visit_html_inline_tag(
        &mut self,
        tag_name: &str,
        attributes: &[(String, Option<String>)],
        content: &Option<Vec<Node>>,
        is_self_closing: bool,
    ) -> Self::Output {
        if let Some(content) = content {
            for child in content {
                self.visit(child);
            }
        }
        self.default_output()
    }

    // Autolinks with type
    fn visit_autolink_url(&mut self, url: &str) -> Self::Output {
        self.default_output()
    }

    fn visit_autolink_email(&mut self, email: &str) -> Self::Output {
        self.default_output()
    }

    // Enhanced Marco elements
    fn visit_admonition_with_icon(
        &mut self,
        kind: &str,
        icon: &Option<String>,
        title: &Option<String>,
        content: &[Node],
    ) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_tab_with_metadata(
        &mut self,
        name: &Option<String>,
        icon: &Option<String>,
        active: bool,
        content: &[Node],
    ) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    fn visit_user_mention_with_metadata(
        &mut self,
        username: &str,
        platform: &Option<String>,
        display_name: &Option<String>,
        user_id: &Option<String>,
        avatar_url: &Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    // Bibliography and citations
    fn visit_citation(&mut self, key: &str, locator: &Option<String>) -> Self::Output {
        self.default_output()
    }

    // Keyboard input
    fn visit_keyboard(&mut self, keys: &[String]) -> Self::Output {
        self.default_output()
    }

    // Mark/highlight with reason
    fn visit_mark(&mut self, content: &[Node], reason: &Option<String>) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    // Details/summary disclosure
    fn visit_details(&mut self, summary: &[Node], content: &[Node], open: bool) -> Self::Output {
        for child in summary {
            self.visit(child);
        }
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }

    // Error recovery
    fn visit_unknown(&mut self, content: &str, rule: &str) -> Self::Output {
        self.default_output()
    }

    fn default_output(&self) -> Self::Output;
}

/// Trait for mutating visitors that can modify the AST
#[allow(unused_variables)]
pub trait VisitorMut {
    type Output;

    fn visit_mut(&mut self, node: &mut Node) -> Self::Output {
        match node {
            // ===========================================
            // DOCUMENT STRUCTURE
            // ===========================================
            Node::Document { children, .. } => self.visit_document_mut(children),

            // ===========================================
            // BLOCK ELEMENTS
            // ===========================================
            Node::Heading { level, content, .. } => self.visit_heading_mut(level, content),
            Node::SetextHeading {
                level,
                content,
                underline_char,
                ..
            } => self.visit_setext_heading_mut(level, content, underline_char),
            Node::Paragraph { content, .. } => self.visit_paragraph_mut(content),
            Node::FencedCodeBlock {
                language,
                info_string,
                content,
                fence_char,
                fence_length,
                ..
            } => self.visit_fenced_code_block_mut(
                language,
                info_string,
                content,
                fence_char,
                fence_length,
            ),
            Node::IndentedCodeBlock { content, .. } => self.visit_indented_code_block_mut(content),
            Node::CodeBlock {
                language, content, ..
            } => self.visit_code_block_mut(language, content),
            Node::MathBlockDisplay {
                content, delimiter, ..
            } => self.visit_math_block_display_mut(content, delimiter),
            Node::MathBlock { content, .. } => self.visit_math_block_mut(content),
            Node::List { ordered, items, .. } => self.visit_list_mut(ordered, items),
            Node::ListItem {
                content, checked, ..
            } => self.visit_list_item_mut(content, checked),
            Node::BlockQuote { content, .. } => self.visit_block_quote_mut(content),
            Node::ThematicBreak { marker, .. } => self.visit_thematic_break_mut(marker),
            Node::HorizontalRule { .. } => self.visit_horizontal_rule_mut(),
            Node::DefinitionList { items, .. } => self.visit_definition_list_mut(items),
            Node::DefinitionTerm { content, .. } => self.visit_definition_term_mut(content),
            Node::DefinitionDescription { content, .. } => {
                self.visit_definition_description_mut(content)
            }

            // ===========================================
            // INLINE ELEMENTS
            // ===========================================
            Node::Text { content, .. } => self.visit_text_mut(content),
            Node::Strong { content, .. } => self.visit_strong_mut(content),
            Node::Emphasis { content, .. } => self.visit_emphasis_mut(content),
            Node::Strikethrough { content, .. } => self.visit_strikethrough_mut(content),
            Node::Highlight { content, .. } => self.visit_highlight_mut(content),
            Node::Mark {
                content, reason, ..
            } => self.visit_mark_mut(content, reason),
            Node::Superscript { content, .. } => self.visit_superscript_mut(content),
            Node::Subscript { content, .. } => self.visit_subscript_mut(content),
            Node::Code { content, .. } => self.visit_code_mut(content),
            Node::CodeSpan {
                content,
                backtick_count,
                ..
            } => self.visit_code_span_mut(content, backtick_count),
            Node::MathInline { content, .. } => self.visit_math_inline_mut(content),
            Node::HardLineBreak { .. } => self.visit_hard_line_break_mut(),
            Node::SoftLineBreak { .. } => self.visit_soft_line_break_mut(),
            Node::LineBreak { .. } => self.visit_line_break_mut(),
            Node::EscapedChar { character, .. } => self.visit_escaped_char_mut(character),
            Node::Emoji { name, .. } => self.visit_emoji_mut(name),
            Node::Keyboard { keys, .. } => self.visit_keyboard_mut(keys),

            // ===========================================
            // LINK ELEMENTS
            // ===========================================
            Node::Link {
                text, url, title, ..
            } => self.visit_link_mut(text, url, title),
            Node::Image {
                alt, url, title, ..
            } => self.visit_image_mut(alt, url, title),
            Node::Autolink { url, .. } => self.visit_autolink_mut(url),
            Node::AutolinkUrl { url, .. } => self.visit_autolink_url_mut(url),
            Node::AutolinkEmail { email, .. } => self.visit_autolink_email_mut(email),
            Node::ReferenceLink { text, label, .. } => self.visit_reference_link_mut(text, label),
            Node::ReferenceImage { alt, label, .. } => self.visit_reference_image_mut(alt, label),
            Node::ReferenceDefinition {
                label, url, title, ..
            } => self.visit_reference_definition_mut(label, url, title),
            Node::LinkReferenceDefinition {
                label,
                destination,
                title,
                ..
            } => self.visit_link_reference_definition_mut(label, destination, title),
            Node::FootnoteRef { label, .. } => self.visit_footnote_ref_mut(label),
            Node::InlineFootnote { content, .. } => self.visit_inline_footnote_mut(content),
            Node::FootnoteDefinition { label, content, .. } => {
                self.visit_footnote_definition_mut(label, content)
            }

            // ===========================================
            // TABLE ELEMENTS
            // ===========================================
            Node::Table { headers, rows, .. } => self.visit_table_mut(headers, rows),
            Node::TableHeader { cells, .. } => self.visit_table_header_mut(cells),
            Node::TableRow { cells, .. } => self.visit_table_row_mut(cells),
            Node::TableCell {
                content, alignment, ..
            } => self.visit_table_cell_mut(content, alignment),

            // ===========================================
            // MARCO EXTENSIONS
            // ===========================================
            Node::UserMention {
                username,
                platform,
                display_name,
                ..
            } => self.visit_user_mention_mut(username, platform, display_name),
            Node::UserMentionWithMetadata {
                username,
                platform,
                display_name,
                user_id,
                avatar_url,
                ..
            } => self.visit_user_mention_with_metadata_mut(
                username,
                platform,
                display_name,
                user_id,
                avatar_url,
            ),
            Node::Bookmark {
                label, path, line, ..
            } => self.visit_bookmark_mut(label, path, line),
            Node::PageTag { format, .. } => self.visit_page_tag_mut(format),
            Node::DocumentReference { path, .. } => self.visit_document_reference_mut(path),
            Node::TableOfContents {
                depth, document, ..
            } => self.visit_table_of_contents_mut(depth, document),
            Node::RunInline {
                script_type,
                command,
                ..
            } => self.visit_run_inline_mut(script_type, command),
            Node::RunBlock {
                script_type,
                content,
                ..
            } => self.visit_run_block_mut(script_type, content),
            Node::DiagramBlock {
                diagram_type,
                content,
                ..
            } => self.visit_diagram_block_mut(diagram_type, content),
            Node::TabBlock { title, tabs, .. } => self.visit_tab_block_mut(title, tabs),
            Node::Tab { name, content, .. } => self.visit_tab_mut(name, content),
            Node::TabWithMetadata {
                name,
                icon,
                active,
                content,
                ..
            } => self.visit_tab_with_metadata_mut(name, icon, active, content),
            Node::Admonition { kind, content, .. } => self.visit_admonition_mut(kind, content),
            Node::AdmonitionWithIcon {
                kind,
                icon,
                title,
                content,
                ..
            } => self.visit_admonition_with_icon_mut(kind, icon, title, content),
            Node::TaskItem {
                checked, content, ..
            } => self.visit_task_item_mut(checked, content),
            Node::Citation { key, locator, .. } => self.visit_citation_mut(key, locator),
            Node::Details {
                summary,
                content,
                open,
                ..
            } => self.visit_details_mut(summary, content, open),
            Node::Macro {
                name,
                arguments,
                content,
                ..
            } => self.visit_macro_mut(name, arguments, content),

            // ===========================================
            // HTML ELEMENTS
            // ===========================================
            Node::InlineHTML { content, .. } => self.visit_inline_html_mut(content),
            Node::BlockHTML { content, .. } => self.visit_block_html_mut(content),
            Node::HtmlBlock {
                html_type, content, ..
            } => self.visit_html_block_mut(html_type, content),
            Node::HtmlInlineTag {
                tag_name,
                attributes,
                content,
                is_self_closing,
                ..
            } => self.visit_html_inline_tag_mut(tag_name, attributes, content, is_self_closing),

            // ===========================================
            // ERROR RECOVERY
            // ===========================================
            Node::Unknown { content, rule, .. } => self.visit_unknown_mut(content, rule),
        }
    }

    // Core document structure
    fn visit_document_mut(&mut self, children: &mut Vec<Node>) -> Self::Output {
        for child in children {
            self.visit_mut(child);
        }
        self.default_output()
    }

    // Block elements
    fn visit_heading_mut(&mut self, level: &mut u8, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_paragraph_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_code_block_mut(
        &mut self,
        language: &mut Option<String>,
        content: &mut String,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_math_block_mut(&mut self, content: &mut String) -> Self::Output {
        self.default_output()
    }

    // Lists
    fn visit_list_mut(&mut self, ordered: &mut bool, items: &mut Vec<Node>) -> Self::Output {
        for item in items {
            self.visit_mut(item);
        }
        self.default_output()
    }

    fn visit_list_item_mut(
        &mut self,
        content: &mut Vec<Node>,
        checked: &mut Option<bool>,
    ) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    // Tables
    fn visit_table_mut(
        &mut self,
        headers: &mut Vec<Node>,
        rows: &mut Vec<Vec<Node>>,
    ) -> Self::Output {
        for header in headers {
            self.visit_mut(header);
        }
        for row in rows {
            for cell in row {
                self.visit_mut(cell);
            }
        }
        self.default_output()
    }

    // All remaining visitor_mut methods following the same pattern as Visitor trait

    fn visit_definition_list_mut(&mut self, items: &mut Vec<Node>) -> Self::Output {
        for item in items {
            self.visit_mut(item);
        }
        self.default_output()
    }

    fn visit_definition_term_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_definition_description_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_footnote_definition_mut(
        &mut self,
        label: &mut String,
        content: &mut Vec<Node>,
    ) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_reference_definition_mut(
        &mut self,
        label: &mut String,
        url: &mut String,
        title: &mut Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_inline_html_mut(&mut self, content: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_block_html_mut(&mut self, content: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_text_mut(&mut self, content: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_emphasis_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_strong_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_strikethrough_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_highlight_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_superscript_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_subscript_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_code_mut(&mut self, content: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_math_inline_mut(&mut self, content: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_link_mut(
        &mut self,
        text: &mut Vec<Node>,
        url: &mut String,
        title: &mut Option<String>,
    ) -> Self::Output {
        for child in text {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_image_mut(
        &mut self,
        alt: &mut String,
        url: &mut String,
        title: &mut Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_autolink_mut(&mut self, url: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_reference_link_mut(
        &mut self,
        text: &mut Vec<Node>,
        label: &mut String,
    ) -> Self::Output {
        for child in text {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_reference_image_mut(&mut self, alt: &mut String, label: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_footnote_ref_mut(&mut self, label: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_inline_footnote_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_emoji_mut(&mut self, name: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_line_break_mut(&mut self) -> Self::Output {
        self.default_output()
    }

    fn visit_escaped_char_mut(&mut self, character: &mut char) -> Self::Output {
        self.default_output()
    }

    fn visit_macro_mut(
        &mut self,
        name: &mut String,
        arguments: &mut Vec<String>,
        content: &mut Option<Vec<Node>>,
    ) -> Self::Output {
        if let Some(content) = content {
            for child in content {
                self.visit_mut(child);
            }
        }
        self.default_output()
    }

    fn visit_user_mention_mut(
        &mut self,
        username: &mut String,
        platform: &mut Option<String>,
        display_name: &mut Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_bookmark_mut(
        &mut self,
        label: &mut String,
        path: &mut String,
        line: &mut Option<u32>,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_page_tag_mut(&mut self, format: &mut Option<String>) -> Self::Output {
        self.default_output()
    }

    fn visit_document_reference_mut(&mut self, path: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_table_of_contents_mut(
        &mut self,
        depth: &mut Option<u8>,
        document: &mut Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_run_inline_mut(
        &mut self,
        script_type: &mut String,
        command: &mut String,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_run_block_mut(
        &mut self,
        script_type: &mut String,
        content: &mut String,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_diagram_block_mut(
        &mut self,
        diagram_type: &mut String,
        content: &mut String,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_tab_block_mut(
        &mut self,
        title: &mut Option<String>,
        tabs: &mut Vec<Node>,
    ) -> Self::Output {
        for tab in tabs {
            self.visit_mut(tab);
        }
        self.default_output()
    }

    fn visit_tab_mut(
        &mut self,
        name: &mut Option<String>,
        content: &mut Vec<Node>,
    ) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_horizontal_rule_mut(&mut self) -> Self::Output {
        self.default_output()
    }

    fn visit_block_quote_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_admonition_mut(&mut self, kind: &mut String, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_task_item_mut(&mut self, checked: &mut bool, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_setext_heading_mut(
        &mut self,
        level: &mut u8,
        content: &mut Vec<Node>,
        underline_char: &mut char,
    ) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_table_header_mut(&mut self, cells: &mut Vec<Node>) -> Self::Output {
        for cell in cells {
            self.visit_mut(cell);
        }
        self.default_output()
    }

    fn visit_table_row_mut(&mut self, cells: &mut Vec<Node>) -> Self::Output {
        for cell in cells {
            self.visit_mut(cell);
        }
        self.default_output()
    }

    fn visit_table_cell_mut(
        &mut self,
        content: &mut Vec<Node>,
        alignment: &mut Option<String>,
    ) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_thematic_break_mut(&mut self, marker: &mut char) -> Self::Output {
        self.default_output()
    }

    fn visit_soft_line_break_mut(&mut self) -> Self::Output {
        self.default_output()
    }

    fn visit_html_block_mut(&mut self, html_type: &mut u8, content: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_fenced_code_block_mut(
        &mut self,
        language: &mut Option<String>,
        info_string: &mut Option<String>,
        content: &mut String,
        fence_char: &mut char,
        fence_length: &mut u8,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_indented_code_block_mut(&mut self, content: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_link_reference_definition_mut(
        &mut self,
        label: &mut String,
        destination: &mut String,
        title: &mut Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_hard_line_break_mut(&mut self) -> Self::Output {
        self.default_output()
    }

    fn visit_math_block_display_mut(
        &mut self,
        content: &mut String,
        delimiter: &mut String,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_code_span_mut(
        &mut self,
        content: &mut String,
        backtick_count: &mut u8,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_html_inline_tag_mut(
        &mut self,
        tag_name: &mut String,
        attributes: &mut Vec<(String, Option<String>)>,
        content: &mut Option<Vec<Node>>,
        is_self_closing: &mut bool,
    ) -> Self::Output {
        if let Some(content) = content {
            for child in content {
                self.visit_mut(child);
            }
        }
        self.default_output()
    }

    fn visit_autolink_url_mut(&mut self, url: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_autolink_email_mut(&mut self, email: &mut String) -> Self::Output {
        self.default_output()
    }

    fn visit_admonition_with_icon_mut(
        &mut self,
        kind: &mut String,
        icon: &mut Option<String>,
        title: &mut Option<String>,
        content: &mut Vec<Node>,
    ) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_tab_with_metadata_mut(
        &mut self,
        name: &mut Option<String>,
        icon: &mut Option<String>,
        active: &mut bool,
        content: &mut Vec<Node>,
    ) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_user_mention_with_metadata_mut(
        &mut self,
        username: &mut String,
        platform: &mut Option<String>,
        display_name: &mut Option<String>,
        user_id: &mut Option<String>,
        avatar_url: &mut Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_citation_mut(
        &mut self,
        key: &mut String,
        locator: &mut Option<String>,
    ) -> Self::Output {
        self.default_output()
    }

    fn visit_keyboard_mut(&mut self, keys: &mut Vec<String>) -> Self::Output {
        self.default_output()
    }

    fn visit_mark_mut(
        &mut self,
        content: &mut Vec<Node>,
        reason: &mut Option<String>,
    ) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_details_mut(
        &mut self,
        summary: &mut Vec<Node>,
        content: &mut Vec<Node>,
        open: &mut bool,
    ) -> Self::Output {
        for child in summary {
            self.visit_mut(child);
        }
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }

    fn visit_unknown_mut(&mut self, content: &mut String, rule: &mut String) -> Self::Output {
        self.default_output()
    }

    fn default_output(&self) -> Self::Output;
}
