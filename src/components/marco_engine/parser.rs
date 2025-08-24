use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

/// Pest parser for markdown using reliable CommonMark grammar
#[derive(Parser)]
#[grammar = "assets/markdown_schema/Marco/markdown.pest"]
pub struct MarkdownParser;

/// AST node compatible with existing renderer  
#[derive(Debug, Clone)]
pub struct Node {
    pub node_type: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<Node>,
}

/// Syntax rule for markdown elements (compatibility with old parser)
#[derive(Debug, Clone)]
pub struct SyntaxRule {
    pub name: String,
    pub pattern: String,
    pub description: String,
}

// Legacy `BlockElement` and line-based `parse_document_blocks` scanner were
// removed as part of API cleanup. Use the full AST returned by
// `parse_markdown` and `Node` for structured analysis.

/// Markdown syntax map for compatibility with existing footer functionality
#[derive(Debug, Clone)]
pub struct MarkdownSyntaxMap {
    pub rules: HashMap<String, SyntaxRule>,
    pub display_hints: Option<HashMap<String, String>>,
}

impl MarkdownSyntaxMap {
    /// Load active schema (compatibility function)
    pub fn load_active_schema(
        _settings_path: &str,
        _schema_root: &str,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        // Create a basic syntax map for now
        let mut rules = HashMap::new();

        rules.insert(
            "heading".to_string(),
            SyntaxRule {
                name: "heading".to_string(),
                pattern: "^#{1,6} .+".to_string(),
                description: "Heading".to_string(),
            },
        );

        rules.insert(
            "paragraph".to_string(),
            SyntaxRule {
                name: "paragraph".to_string(),
                pattern: ".+".to_string(),
                description: "Paragraph".to_string(),
            },
        );

        Ok(Some(MarkdownSyntaxMap {
            rules,
            display_hints: None,
        }))
    }

    /// Build display hints map (compatibility function)
    pub fn build_display_hints(&self) -> HashMap<String, String> {
        // Return basic display hints
        let mut hints = HashMap::new();
        hints.insert("heading".to_string(), "📝".to_string());
        hints.insert("paragraph".to_string(), "📄".to_string());
        hints.insert("blockquote".to_string(), "📧".to_string());
        hints.insert("code_block".to_string(), "💻".to_string());
        hints.insert("list".to_string(), "📋".to_string());
        hints
    }
}

// The previous simple line-based scanner was removed to reduce duplicate
// parsing logic and to encourage using the canonical pest-based AST. If a
// lightweight, line-oriented summary is needed in future, we can reintroduce
// a dedicated function or provide `parse_markdown_with_summary` that returns
// both the AST and a compact summary.

impl Node {
    pub fn new(node_type: &str) -> Self {
        Self {
            node_type: node_type.to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn text_node(text: &str) -> Self {
        let mut node = Self::new("text");
        node.add_attribute("value", text);
        node
    }

    pub fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }

    pub fn add_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }
}

/// Convert markdown text to AST using pest parser
pub fn parse_markdown(input: &str) -> Result<Node, Box<dyn std::error::Error>> {
    // Run the full pest parser to build the detailed AST used by the
    // renderer and other components.
    let pairs = MarkdownParser::parse(Rule::document, input)?;

    let mut root = Node::new("root");

    for pair in pairs {
        if let Some(node) = parse_pair(pair) {
            root.add_child(node);
        }
    }

    // We intentionally do not attach the lightweight `blocks` node to the
    // returned AST. The call to `parse_document_blocks` above ensures
    // `BlockElement` values are constructed (satisfying static analysis and
    // legacy consumers that might call the function directly) without
    // altering the existing AST shape consumed elsewhere in the codebase.

    Ok(root)
}

/// Parse a pest pair into an AST node
fn parse_pair(pair: pest::iterators::Pair<Rule>) -> Option<Node> {
    match pair.as_rule() {
        Rule::document => {
            let mut root = Node::new("root");
            for inner_pair in pair.into_inner() {
                if let Some(child) = parse_pair(inner_pair) {
                    root.add_child(child);
                }
            }
            Some(root)
        }

        Rule::content => {
            // Content is a wrapper, process children
            let mut children = Vec::new();
            for inner_pair in pair.into_inner() {
                if let Some(child) = parse_pair(inner_pair) {
                    children.push(child);
                }
            }
            if children.len() == 1 {
                children.into_iter().next()
            } else {
                let mut node = Node::new("content");
                for child in children {
                    node.add_child(child);
                }
                Some(node)
            }
        }

        Rule::block => {
            // Block is just a wrapper, process its first child (if any)
            pair.into_inner().next().and_then(parse_pair)
        }

        // =============================================================================
        // HEADINGS
        // =============================================================================
        Rule::atx_heading => {
            let content = pair.as_str();
            let depth = content.chars().take_while(|&c| c == '#').count();
            let text = content[depth..].trim_start_matches(' ').trim().to_string();

            let mut node = Node::new("heading");
            node.add_attribute("depth", &depth.to_string());
            node.add_child(Node::text_node(&text));
            Some(node)
        }

        Rule::setext_heading => {
            let content = pair.as_str();
            let lines: Vec<&str> = content.lines().collect();

            if lines.len() >= 2 {
                let heading_text = lines[0].trim().to_string();
                let underline = lines[1];
                let depth = if underline.starts_with('=') { 1 } else { 2 };

                let mut node = Node::new("heading");
                node.add_attribute("depth", &depth.to_string());
                node.add_child(Node::text_node(&heading_text));
                Some(node)
            } else {
                None
            }
        }

        // =============================================================================
        // CODE BLOCKS
        // =============================================================================
        Rule::fenced_code_block => {
            let mut language = String::new();
            let mut code = String::new();

            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::language_info => language = inner_pair.as_str().to_string(),
                    Rule::code_content => code = inner_pair.as_str().to_string(),
                    _ => {}
                }
            }

            let mut node = Node::new("codeBlock");
            if !language.is_empty() {
                node.add_attribute("language", &language);
            }
            node.add_attribute("value", &code);
            Some(node)
        }

        Rule::indented_code_block => {
            let code = pair
                .as_str()
                .lines()
                .map(|line| line.trim_start_matches("    ").trim_start_matches("\t"))
                .collect::<Vec<_>>()
                .join("\n");

            let mut node = Node::new("codeBlock");
            node.add_attribute("value", &code);
            Some(node)
        }

        // =============================================================================
        // BLOCKQUOTES
        // =============================================================================
        Rule::blockquote => {
            let content = pair
                .as_str()
                .lines()
                .map(|line| line.trim_start_matches("> ").trim_start_matches(">"))
                .collect::<Vec<_>>()
                .join("\n");

            let mut node = Node::new("blockquote");
            node.add_child(Node::text_node(&content));
            Some(node)
        }

        // =============================================================================
        // LISTS
        // =============================================================================
        Rule::unordered_list => {
            let mut node = Node::new("list");
            node.add_attribute("ordered", "false");

            for inner_pair in pair.into_inner() {
                if let Some(item) = parse_pair(inner_pair) {
                    node.add_child(item);
                }
            }
            Some(node)
        }

        Rule::ordered_list => {
            let mut node = Node::new("list");
            node.add_attribute("ordered", "true");

            for inner_pair in pair.into_inner() {
                if let Some(item) = parse_pair(inner_pair) {
                    node.add_child(item);
                }
            }
            Some(node)
        }

        Rule::task_list => {
            let mut node = Node::new("list");
            node.add_attribute("task", "true");

            for inner_pair in pair.into_inner() {
                if let Some(item) = parse_pair(inner_pair) {
                    node.add_child(item);
                }
            }
            Some(node)
        }

        Rule::unordered_list_item | Rule::ordered_list_item => {
            // Capture raw content before consuming the pair so we can fallback to it
            // if inner parsing produces no children.
            let raw_content = pair.as_str().to_string();

            // Prefer to parse inner pairs so inline formatting (emphasis, strong, links, etc.)
            // inside list items is preserved in the AST instead of being collapsed to plain text.
            let mut node = Node::new("listItem");
            let mut had_child = false;
            for inner_pair in pair.into_inner() {
                if let Some(child) = parse_pair(inner_pair) {
                    node.add_child(child);
                    had_child = true;
                }
            }

            // Fallback: if no inner pairs were produced, use the raw text (preserve old behavior)
            if !had_child {
                let content = raw_content.as_str();
                let text = if let Some(space_pos) = content.find(' ') {
                    content[space_pos..].trim()
                } else {
                    content
                };
                node.add_child(Node::text_node(text));
            }

            Some(node)
        }

        Rule::task_list_item => {
            let content = pair.as_str();
            let checked = content.contains("[x]") || content.contains("[X]");

            // Find the text after the checkbox
            let text = if let Some(pos) = content.find("] ") {
                content[pos + 2..].trim()
            } else {
                content
            };

            let mut node = Node::new("listItem");
            node.add_attribute("checked", &checked.to_string());
            node.add_child(Node::text_node(text));
            Some(node)
        }

        // =============================================================================
        // LISTS (new combined list handler)
        // =============================================================================
        Rule::list => {
            // Process the inner list type (unordered_list, ordered_list, or task_list)
            let inner_pair = pair.into_inner().next()?;
            parse_pair(inner_pair)
        }

        // =============================================================================
        // TABLES
        // =============================================================================
        Rule::table => {
            let mut node = Node::new("table");

            for inner_pair in pair.into_inner() {
                match inner_pair.as_rule() {
                    Rule::table_header => {
                        if let Some(header) = parse_pair(inner_pair) {
                            let mut thead = Node::new("tableHead");
                            thead.add_child(header);
                            node.add_child(thead);
                        }
                    }
                    Rule::table_row => {
                        if let Some(row) = parse_pair(inner_pair) {
                            node.add_child(row);
                        }
                    }
                    _ => {}
                }
            }
            Some(node)
        }

        Rule::table_row => {
            let mut node = Node::new("tableRow");

            for inner_pair in pair.into_inner() {
                if inner_pair.as_rule() == Rule::table_cell {
                    let mut cell = Node::new("tableCell");
                    cell.add_child(Node::text_node(inner_pair.as_str().trim()));
                    node.add_child(cell);
                }
            }
            Some(node)
        }

        // =============================================================================
        // FRONTMATTER
        // =============================================================================
        Rule::frontmatter | Rule::yaml_frontmatter | Rule::toml_frontmatter => {
            let content = pair.as_str();
            let format = if content.starts_with("---") {
                "yaml"
            } else {
                "toml"
            };
            let data = content
                .lines()
                .skip(1)
                .take_while(|line| !line.starts_with("---") && !line.starts_with("+++"))
                .collect::<Vec<_>>()
                .join("\n");

            let mut node = Node::new("frontmatter");
            node.add_attribute("format", format);
            node.add_attribute("value", &data);
            Some(node)
        }

        // =============================================================================
        // MATH
        // =============================================================================
        Rule::math_block => {
            let content = pair
                .as_str()
                .trim_start_matches("$$")
                .trim_end_matches("$$")
                .trim();
            let mut node = Node::new("mathBlock");
            node.add_attribute("value", content);
            Some(node)
        }

        Rule::inline_math => {
            let content = pair.as_str().trim_start_matches('$').trim_end_matches('$');
            let mut node = Node::new("mathInline");
            node.add_attribute("value", content);
            Some(node)
        }

        // =============================================================================
        // HTML
        // =============================================================================
        Rule::html_block
        | Rule::html_tag_block
        | Rule::html_comment_block
        | Rule::html_cdata_block => {
            let mut node = Node::new("htmlBlock");
            node.add_attribute("value", pair.as_str());
            Some(node)
        }

        Rule::html_inline | Rule::html_tag => {
            let mut node = Node::new("htmlInline");
            node.add_attribute("value", pair.as_str());
            Some(node)
        }

        // =============================================================================
        // DEFINITIONS & REFERENCES
        // =============================================================================
        Rule::definition => {
            // Process the inner definition type - only the first relevant child is needed
            pair.into_inner().next().and_then(parse_pair)
        }

        Rule::link_definition => {
            let content = pair.as_str();
            let parts: Vec<&str> = content.split(':').collect();
            let label = parts[0].trim_matches(['[', ']']);
            let url = parts.get(1).unwrap_or(&"").trim();

            let mut node = Node::new("definition");
            node.add_attribute("identifier", label);
            node.add_attribute("url", url);
            Some(node)
        }

        Rule::footnote_definition => {
            let content = pair.as_str();
            let parts: Vec<&str> = content.split(':').collect();
            let label = parts[0].trim_matches(['[', '^', ']']);
            let text = parts.get(1).unwrap_or(&"").trim();

            let mut node = Node::new("footnoteDefinition");
            node.add_attribute("identifier", label);
            node.add_child(Node::text_node(text));
            Some(node)
        }

        // =============================================================================
        // INLINE FORMATTING
        // =============================================================================
        Rule::strong_emphasis | Rule::strong_asterisk | Rule::strong_underscore => {
            let content = pair
                .as_str()
                .trim_start_matches("**")
                .trim_start_matches("__")
                .trim_end_matches("**")
                .trim_end_matches("__");
            let mut node = Node::new("strong");
            node.add_child(Node::text_node(content));
            Some(node)
        }

        Rule::emphasis | Rule::emphasis_asterisk | Rule::emphasis_underscore => {
            let content = pair
                .as_str()
                .trim_start_matches('*')
                .trim_start_matches('_')
                .trim_end_matches('*')
                .trim_end_matches('_');
            let mut node = Node::new("emphasis");
            node.add_child(Node::text_node(content));
            Some(node)
        }

        Rule::strikethrough => {
            let content = pair
                .as_str()
                .trim_start_matches("~~")
                .trim_end_matches("~~");
            let mut node = Node::new("delete");
            node.add_child(Node::text_node(content));
            Some(node)
        }

        Rule::inline_code => {
            let content = pair.as_str().trim_start_matches('`').trim_end_matches('`');
            let mut node = Node::new("inlineCode");
            node.add_attribute("value", content);
            Some(node)
        }

        // =============================================================================
        // LINKS & IMAGES
        // =============================================================================
        Rule::link | Rule::link_inline => {
            let content = pair.as_str();
            // Parse [text](url) format
            if let Some(start) = content.find('[') {
                if let Some(middle) = content.find("](") {
                    if let Some(end) = content.rfind(')') {
                        let text = &content[start + 1..middle];
                        let url = &content[middle + 2..end];

                        let mut node = Node::new("link");
                        node.add_attribute("url", url);
                        node.add_child(Node::text_node(text));
                        return Some(node);
                    }
                }
            }
            None
        }

        Rule::image | Rule::image_inline => {
            let content = pair.as_str();
            // Parse ![alt](url) format
            if let Some(start) = content.find("![") {
                if let Some(middle) = content.find("](") {
                    if let Some(end) = content.rfind(')') {
                        let alt = &content[start + 2..middle];
                        let url = &content[middle + 2..end];

                        let mut node = Node::new("image");
                        node.add_attribute("url", url);
                        node.add_attribute("alt", alt);
                        return Some(node);
                    }
                }
            }
            None
        }

        Rule::autolink | Rule::autolink_url | Rule::autolink_email => {
            let url = pair.as_str().trim_start_matches('<').trim_end_matches('>');
            let mut node = Node::new("link");
            node.add_attribute("url", url);
            node.add_child(Node::text_node(url));
            Some(node)
        }

        // =============================================================================
        // LINE BREAKS
        // =============================================================================
        Rule::hard_break => Some(Node::new("break")),

        // =============================================================================
        // EXTENDED ELEMENTS
        // =============================================================================
        Rule::emoji => {
            let raw = pair.as_str();
            let name = raw.trim_start_matches(':').trim_end_matches(':');
            let mut node = Node::new("emoji");
            node.add_attribute("name", name);
            // include original shortcode form as `value` for compatibility with fixtures
            node.add_attribute("value", raw);
            Some(node)
        }

        Rule::mention => {
            let raw = pair.as_str();
            let username = raw.trim_start_matches('@');
            let mut node = Node::new("mention");
            node.add_attribute("username", username);
            // include original mention form as `value` (e.g. "@user") for fixture compatibility
            node.add_attribute("value", raw);
            Some(node)
        }

        Rule::entity | Rule::html_entity => {
            let mut node = Node::new("htmlInline");
            node.add_attribute("value", pair.as_str());
            Some(node)
        }

        // =============================================================================
        // THEMATIC BREAKS
        // =============================================================================
        Rule::thematic_break | Rule::hr_stars | Rule::hr_dashes | Rule::hr_underscores => {
            Some(Node::new("thematicBreak"))
        }

        // =============================================================================
        // PARAGRAPHS
        // =============================================================================
        Rule::paragraph => {
            let mut node = Node::new("paragraph");
            for inner_pair in pair.into_inner() {
                if let Some(child_node) = parse_pair(inner_pair) {
                    node.add_child(child_node);
                }
            }
            Some(node)
        }

        // =============================================================================
        // UTILITY RULES (usually not processed directly)
        // =============================================================================
        Rule::paragraph_content | Rule::inline_content => {
            // Store the text content before processing
            let text_content = pair.as_str();

            // For paragraph and inline content, process children while preserving
            // the plain text between inline elements. Use span offsets to slice
            // the parent text and insert text nodes for segments between children.
            let parent_span = pair.as_span();
            let parent_text = parent_span.as_str();
            let mut nodes: Vec<Node> = Vec::new();
            let mut cursor: usize = 0;

            for inner_pair in pair.into_inner() {
                let child_span = inner_pair.as_span();
                // Compute offsets relative to parent_text
                let start_off = child_span.start() - parent_span.start();
                let end_off = child_span.end() - parent_span.start();

                if start_off > cursor {
                    let segment = &parent_text[cursor..start_off];
                    if !segment.is_empty() {
                        nodes.push(Node::text_node(segment));
                    }
                }

                if let Some(child_node) = parse_pair(inner_pair) {
                    nodes.push(child_node);
                }

                cursor = end_off;
            }

            if cursor < parent_text.len() {
                let tail = &parent_text[cursor..];
                if !tail.is_empty() {
                    nodes.push(Node::text_node(tail));
                }
            }

            if !nodes.is_empty() {
                let mut container = Node::new("content");
                for n in nodes {
                    container.add_child(n);
                }
                Some(container)
            } else {
                let text = text_content.trim();
                if !text.is_empty() {
                    Some(Node::text_node(text))
                } else {
                    None
                }
            }
        }

        Rule::inline_element => {
            // Process the actual inline element
            // Find the first child that parses into a node and return it
            pair.into_inner().filter_map(parse_pair).next()
        }

        Rule::code_content
        | Rule::language_info
        | Rule::line_content
        | Rule::list_item_content
        | Rule::code_fence
        | Rule::blockquote_line
        | Rule::unordered_marker
        | Rule::ordered_marker
        | Rule::task_marker
        | Rule::checkbox
        | Rule::checkbox_state
        | Rule::indent_4
        | Rule::code_line
        | Rule::table_separator
        | Rule::table_cell
        | Rule::table_sep_cell => {
            // These are handled by their parent rules
            None
        }

        Rule::NEWLINE | Rule::WHITESPACE => {
            // Skip whitespace and blank lines
            None
        }

        // Skip other rules that don't need direct processing
        _ => None,
    }
}
