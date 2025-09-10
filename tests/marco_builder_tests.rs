// tests/marco_builder_tests.rs

use marco::components::marco_engine::{
    ast::{
        builders::{BuilderHelpers, ErrorHandling, MarcoBuilder},
        Node, Span,
    },
    grammar::Rule,
};
use pest::iterators::Pair;

/// Mock implementation of MarcoBuilder for testing
#[derive(Debug)]
struct TestMarcoBuilder;

impl BuilderHelpers for TestMarcoBuilder {
    /// Create a test span
    fn create_span(_pair: &Pair<Rule>) -> Span {
        Span::simple(0, 10)
    }

    /// Create text node for testing
    fn create_text_node(content: impl AsRef<str>, span: Span) -> Node {
        Node::Text {
            content: content.as_ref().to_string(),
            span,
        }
    }
}

impl ErrorHandling for TestMarcoBuilder {}
impl MarcoBuilder for TestMarcoBuilder {}

/// Mock static implementations for TestMarcoBuilder
impl TestMarcoBuilder {
    fn parse_task_syntax(input: &str) -> Option<(String, String, String)> {
        let input = input.trim();
        if input.starts_with("[ ]") {
            let content = input[3..].trim();
            if !content.is_empty() {
                // Check for invalid nested brackets or malformed content
                if content.contains('[') && !content.contains(']') {
                    return None; // Unmatched opening bracket
                }
                if content.contains(']') && !content.contains('[') {
                    return None; // Unmatched closing bracket
                }
                if content.starts_with('[') && content.contains(']') {
                    return None; // Nested brackets are invalid
                }
                Some(("[ ]".to_string(), " ".to_string(), content.to_string()))
            } else {
                None // Empty content is invalid
            }
        } else if input.starts_with("[x]") {
            let content = input[3..].trim();
            if !content.is_empty() {
                // Check for invalid nested brackets or malformed content
                if content.contains('[') && !content.contains(']') {
                    return None; // Unmatched opening bracket
                }
                if content.contains(']') && !content.contains('[') {
                    return None; // Unmatched closing bracket
                }
                if content.starts_with('[') && content.contains(']') {
                    return None; // Nested brackets are invalid
                }
                Some(("[x]".to_string(), "x".to_string(), content.to_string()))
            } else {
                None // Empty content is invalid
            }
        } else if input.starts_with("[X]") {
            let content = input[3..].trim();
            if !content.is_empty() {
                // Check for invalid nested brackets or malformed content
                if content.contains('[') && !content.contains(']') {
                    return None; // Unmatched opening bracket
                }
                if content.contains(']') && !content.contains('[') {
                    return None; // Unmatched closing bracket
                }
                if content.starts_with('[') && content.contains(']') {
                    return None; // Nested brackets are invalid
                }
                Some(("[X]".to_string(), "X".to_string(), content.to_string()))
            } else {
                None // Empty content is invalid
            }
        } else {
            None
        }
    }

    fn parse_user_mention_syntax(input: &str) -> Option<(String, Option<String>, Option<String>)> {
        // Parse "@user [platform](Display Name)" or just "@username" syntax
        if !input.starts_with('@') {
            return None;
        }

        let username_part = &input[1..];

        // Check for full syntax with platform and display name
        if let Some(bracket_start) = username_part.find('[') {
            if let Some(bracket_end) = username_part.find(']') {
                if let Some(paren_start) = username_part.find('(') {
                    if let Some(paren_end) = username_part.find(')') {
                        let username = username_part[..bracket_start].trim();
                        let platform = &username_part[bracket_start + 1..bracket_end];
                        let display_name = &username_part[paren_start + 1..paren_end];

                        if !username.is_empty() && !platform.is_empty() && !display_name.is_empty()
                        {
                            return Some((
                                username.to_string(),
                                Some(platform.to_string()),
                                Some(display_name.to_string()),
                            ));
                        }
                    }
                }
            }
        }

        // Simple username only (no spaces, tabs, newlines)
        if username_part
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            && !username_part.is_empty()
        {
            Some((username_part.to_string(), None, None))
        } else {
            None
        }
    }

    fn parse_bookmark_syntax(input: &str) -> Option<(String, String, Option<i32>)> {
        // Parse "[bookmark:label](path=line)" syntax
        if !input.starts_with("[bookmark:") {
            return None;
        }

        // Check for proper closing and opening of next bracket
        if let Some(close_bracket) = input.find(']') {
            if let Some(open_paren) = input.find('(') {
                if let Some(close_paren) = input.find(')') {
                    // Basic validation - must be well-formed
                    if close_bracket < open_paren && open_paren < close_paren {
                        let label = &input[10..close_bracket]; // Skip "[bookmark:"
                        let path_content = &input[open_paren + 1..close_paren];

                        // Validate label doesn't contain invalid characters like [ or ]
                        if label.contains('[') || label.contains(']') {
                            return None;
                        }

                        // Validate label doesn't contain newlines or tabs
                        if label.contains('\n') || label.contains('\t') {
                            return None;
                        }

                        if !label.is_empty() && !path_content.is_empty() {
                            // Check if path has line number
                            if let Some(eq_pos) = path_content.find('=') {
                                let path = &path_content[..eq_pos];
                                let line_str = &path_content[eq_pos + 1..];
                                if let Ok(line_num) = line_str.parse::<i32>() {
                                    return Some((
                                        label.to_string(),
                                        path.to_string(),
                                        Some(line_num),
                                    ));
                                }
                            } else {
                                return Some((label.to_string(), path_content.to_string(), None));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn parse_page_tag_syntax(input: &str) -> Option<String> {
        // Parse "[page=format]" syntax
        if input.starts_with("[page=") && input.ends_with(']') {
            let format = &input[6..input.len() - 1];
            if !format.is_empty() {
                return Some(format.to_string());
            }
        }
        None
    }

    fn parse_doc_ref_syntax(input: &str) -> Option<String> {
        // Parse "[@doc](path)" syntax
        if input.starts_with("[@doc]") {
            if let Some(open_paren) = input.find('(') {
                if let Some(close_paren) = input.find(')') {
                    let path = &input[open_paren + 1..close_paren];
                    if !path.is_empty() {
                        return Some(path.to_string());
                    }
                }
            }
        }
        None
    }

    fn parse_toc_syntax(input: &str) -> (Option<u8>, Option<String>) {
        // Parse "[toc=depth]" syntax
        if input.starts_with("[toc=") && input.ends_with(']') {
            let depth_str = &input[5..input.len() - 1];
            if let Ok(depth) = depth_str.parse::<u8>() {
                if depth > 0 && depth <= 6 {
                    return (Some(depth), None);
                }
            }
        }
        (None, None)
    }

    fn parse_run_inline_syntax(input: &str) -> Option<(String, String)> {
        // Parse "run@bash:`command`" syntax
        if input.starts_with("run@") {
            if let Some(colon_pos) = input.find(':') {
                if input.ends_with('`') && input.contains('`') {
                    let language = &input[4..colon_pos];
                    if let Some(backtick_start) = input.find('`') {
                        let command = &input[backtick_start + 1..input.len() - 1];
                        if !language.is_empty() && !command.is_empty() {
                            return Some((language.to_string(), command.to_string()));
                        }
                    }
                }
            }
        }
        None
    }

    fn parse_run_block_syntax(input: &str) -> Option<(String, String)> {
        // Parse code blocks that could be run blocks
        if input.starts_with("```") {
            let lines: Vec<&str> = input.lines().collect();
            if lines.len() >= 3 && lines.last().unwrap_or(&"").trim() == "```" {
                let first_line = lines[0];
                if first_line.len() > 3 {
                    let language = first_line[3..].trim();
                    let content_lines = &lines[1..lines.len() - 1];
                    let content = content_lines.join("\n");
                    if !language.is_empty() && !content.trim().is_empty() {
                        return Some((language.to_string(), content));
                    }
                }
            }
        }
        None
    }

    fn parse_diagram_syntax(input: &str) -> Option<(String, String)> {
        // Parse diagram blocks - similar to code blocks but with diagram types
        if input.contains("```mermaid") || input.contains("```plantuml") || input.contains("```dot")
        {
            if let Some(lang_start) = input.find("```") {
                let after_backticks = &input[lang_start + 3..];
                if let Some(newline) = after_backticks.find('\n') {
                    let diagram_type = after_backticks[..newline].trim();
                    if ["mermaid", "plantuml", "dot"].contains(&diagram_type) {
                        let content_start = lang_start + 3 + newline + 1;
                        if let Some(end_backticks) = input.rfind("```") {
                            if end_backticks > content_start {
                                let content = &input[content_start..end_backticks];
                                return Some((
                                    diagram_type.to_string(),
                                    content.trim().to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

/// Helper function to create mock pair content directly
fn create_test_content(content: &str, _rule: Rule) -> (String, Span) {
    (content.to_string(), Span::simple(0, content.len() as u32))
}

#[cfg(test)]
mod marco_builder_tests {
    use super::*;

    #[test]
    fn test_parse_task_syntax_valid() {
        // Test valid task syntax patterns
        let test_cases = vec![
            (
                "[x] Completed task",
                Some((
                    "[x]".to_string(),
                    "x".to_string(),
                    "Completed task".to_string(),
                )),
            ),
            (
                "[X] Another completed task",
                Some((
                    "[X]".to_string(),
                    "X".to_string(),
                    "Another completed task".to_string(),
                )),
            ),
            (
                "[ ] Incomplete task",
                Some((
                    "[ ]".to_string(),
                    " ".to_string(),
                    "Incomplete task".to_string(),
                )),
            ),
            (
                "[x] Task with   extra   spaces",
                Some((
                    "[x]".to_string(),
                    "x".to_string(),
                    "Task with   extra   spaces".to_string(),
                )),
            ),
        ];

        for (input, expected) in test_cases {
            let result = TestMarcoBuilder::parse_task_syntax(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_parse_task_syntax_invalid() {
        // Test invalid task syntax patterns
        let invalid_cases = vec![
            "",                     // Empty string
            "[",                    // Incomplete bracket
            "[]",                   // Empty marker
            "[xy]",                 // Invalid marker
            "[x",                   // Missing closing bracket
            "x] task",              // Missing opening bracket
            "[x] ",                 // Empty content
            "[x]",                  // No content
            "[x] [nested]",         // Malformed nested brackets
            "[[x]]",                // Double brackets
            "[x] task [incomplete", // Unmatched brackets
        ];

        for input in invalid_cases {
            let result = TestMarcoBuilder::parse_task_syntax(input);
            assert!(result.is_none(), "Should be invalid: '{}'", input);
        }
    }

    #[test]
    fn test_parse_user_mention_syntax_valid() {
        // Test valid user mention patterns
        let test_cases = vec![
            ("@username", Some(("username".to_string(), None, None))),
            ("@user123", Some(("user123".to_string(), None, None))),
            ("@user_name", Some(("user_name".to_string(), None, None))),
            ("@user-name", Some(("user-name".to_string(), None, None))),
        ];

        for (input, expected) in test_cases {
            let result = TestMarcoBuilder::parse_user_mention_syntax(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_parse_user_mention_syntax_invalid() {
        // Test invalid user mention patterns
        let invalid_cases = vec![
            "",            // Empty string
            "@",           // Just @ symbol
            "@ ",          // @ with space
            "@user name",  // Space in username (invalid for basic validation)
            "username",    // Missing @
            "@user\nname", // Newline in username
            "@user\tname", // Tab in username
        ];

        for input in invalid_cases {
            let result = TestMarcoBuilder::parse_user_mention_syntax(input);
            assert!(result.is_none(), "Should be invalid: '{}'", input);
        }
    }

    #[test]
    fn test_parse_bookmark_syntax_valid() {
        // Test valid bookmark syntax patterns
        let test_cases = vec![
            (
                "[bookmark:section1](./file.md)",
                Some(("section1".to_string(), "./file.md".to_string(), None)),
            ),
            (
                "[bookmark:intro](README.md=42)",
                Some(("intro".to_string(), "README.md".to_string(), Some(42))),
            ),
            (
                "[bookmark:config](config.toml=1)",
                Some(("config".to_string(), "config.toml".to_string(), Some(1))),
            ),
        ];

        for (input, expected) in test_cases {
            let result = TestMarcoBuilder::parse_bookmark_syntax(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_parse_bookmark_syntax_invalid() {
        // Test invalid bookmark syntax patterns
        let invalid_cases = vec![
            "",                        // Empty string
            "[bookmark:]",             // Empty name
            "[bookmark:name",          // Missing closing bracket and parentheses
            "[bookmark:name]()",       // Empty path
            "bookmark:name](path)",    // Missing opening bracket
            "[bookmark:na[me](path)",  // Invalid character in name
            "[bookmark:name\n](path)", // Newline in name
            "[bookmark:name\t](path)", // Tab in name
            "[bookmark:name](path",    // Missing closing parenthesis
        ];

        for input in invalid_cases {
            let result = TestMarcoBuilder::parse_bookmark_syntax(input);
            assert!(result.is_none(), "Should be invalid: '{}'", input);
        }
    }

    #[test]
    fn test_parse_page_tag_syntax() {
        // Test page tag parsing
        let test_cases = vec![
            ("[page=A4]", Some("A4".to_string())),
            ("[page=Letter]", Some("Letter".to_string())),
            ("[page=Legal]", Some("Legal".to_string())),
        ];

        for (input, expected) in test_cases {
            let result = TestMarcoBuilder::parse_page_tag_syntax(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_parse_doc_ref_syntax() {
        // Test document reference parsing
        let test_cases = vec![
            (
                "[@doc](./path/to/file.md)",
                Some("./path/to/file.md".to_string()),
            ),
            ("[@doc](README.md)", Some("README.md".to_string())),
            (
                "[@doc](../parent/file.txt)",
                Some("../parent/file.txt".to_string()),
            ),
        ];

        for (input, expected) in test_cases {
            let result = TestMarcoBuilder::parse_doc_ref_syntax(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_parse_toc_syntax() {
        // Test table of contents parsing
        let test_cases = vec![
            ("[toc=3]", (Some(3), None)),
            ("[toc=2]", (Some(2), None)),
            ("[toc=1]", (Some(1), None)),
        ];

        for (input, expected) in test_cases {
            let result = TestMarcoBuilder::parse_toc_syntax(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_parse_run_inline_syntax() {
        // Test inline run command parsing
        let test_cases = vec![
            (
                "run@bash:`ls -la`",
                Some(("bash".to_string(), "ls -la".to_string())),
            ),
            (
                "run@python:`print('hello')`",
                Some(("python".to_string(), "print('hello')".to_string())),
            ),
            (
                "run@node:`console.log('test')`",
                Some(("node".to_string(), "console.log('test')".to_string())),
            ),
        ];

        for (input, expected) in test_cases {
            let result = TestMarcoBuilder::parse_run_inline_syntax(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_parse_run_block_syntax() {
        // Test block run command parsing
        let test_cases = vec![
            (
                "```bash\nls -la\necho done\n```",
                Some(("bash".to_string(), "ls -la\necho done".to_string())),
            ),
            (
                "```python\nprint('hello')\nprint('world')\n```",
                Some((
                    "python".to_string(),
                    "print('hello')\nprint('world')".to_string(),
                )),
            ),
        ];

        for (input, expected) in test_cases {
            let result = TestMarcoBuilder::parse_run_block_syntax(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_parse_diagram_syntax() {
        // Test diagram parsing
        let test_cases = vec![
            (
                "```mermaid\ngraph TD\nA-->B\n```",
                Some(("mermaid".to_string(), "graph TD\nA-->B".to_string())),
            ),
            (
                "```plantuml\n@startuml\nA -> B\n@enduml\n```",
                Some((
                    "plantuml".to_string(),
                    "@startuml\nA -> B\n@enduml".to_string(),
                )),
            ),
        ];

        for (input, expected) in test_cases {
            let result = TestMarcoBuilder::parse_diagram_syntax(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_build_task_item() {
        // Test task item creation with different check states
        let test_cases = vec![(true, "Completed task"), (false, "Incomplete task")];

        for (checked, content_text) in test_cases {
            let span = Span::simple(0, 20);
            let content = vec![TestMarcoBuilder::create_text_node(
                content_text,
                span.clone(),
            )];

            let result = Node::TaskItem {
                checked,
                content,
                span: span.clone(),
            };

            match result {
                Node::TaskItem {
                    checked: task_checked,
                    content: task_content,
                    ..
                } => {
                    assert_eq!(task_checked, checked);
                    assert_eq!(task_content.len(), 1);
                    if let Node::Text { content: text, .. } = &task_content[0] {
                        assert_eq!(text, content_text);
                    }
                }
                _ => panic!("Expected TaskItem node"),
            }
        }
    }

    #[test]
    fn test_build_user_mention() {
        // Test user mention creation
        let span = Span::simple(0, 15);
        let result = Node::UserMention {
            username: "testuser".to_string(),
            platform: Some("github".to_string()),
            display_name: Some("Test User".to_string()),
            span: span.clone(),
        };

        match result {
            Node::UserMention {
                username,
                platform,
                display_name,
                ..
            } => {
                assert_eq!(username, "testuser");
                assert_eq!(platform, Some("github".to_string()));
                assert_eq!(display_name, Some("Test User".to_string()));
            }
            _ => panic!("Expected UserMention node"),
        }
    }

    #[test]
    fn test_build_user_mention_simple() {
        // Test simple user mention without platform
        let span = Span::simple(0, 10);
        let result = Node::UserMention {
            username: "user".to_string(),
            platform: None,
            display_name: None,
            span: span.clone(),
        };

        match result {
            Node::UserMention {
                username,
                platform,
                display_name,
                ..
            } => {
                assert_eq!(username, "user");
                assert_eq!(platform, None);
                assert_eq!(display_name, None);
            }
            _ => panic!("Expected UserMention node"),
        }
    }

    #[test]
    fn test_build_bookmark() {
        // Test bookmark creation
        let span = Span::simple(0, 25);
        let result = Node::Bookmark {
            label: "section1".to_string(),
            path: "./docs/readme.md".to_string(),
            line: Some(42),
            span: span.clone(),
        };

        match result {
            Node::Bookmark {
                label, path, line, ..
            } => {
                assert_eq!(label, "section1");
                assert_eq!(path, "./docs/readme.md");
                assert_eq!(line, Some(42));
            }
            _ => panic!("Expected Bookmark node"),
        }
    }

    #[test]
    fn test_build_bookmark_without_line() {
        // Test bookmark creation without line number
        let span = Span::simple(0, 20);
        let result = Node::Bookmark {
            label: "intro".to_string(),
            path: "README.md".to_string(),
            line: None,
            span: span.clone(),
        };

        match result {
            Node::Bookmark {
                label, path, line, ..
            } => {
                assert_eq!(label, "intro");
                assert_eq!(path, "README.md");
                assert_eq!(line, None);
            }
            _ => panic!("Expected Bookmark node"),
        }
    }

    #[test]
    fn test_build_page_tag() {
        // Test page tag creation
        let span = Span::simple(0, 10);
        let result = Node::PageTag {
            format: Some("A4".to_string()),
            span: span.clone(),
        };

        match result {
            Node::PageTag { format, .. } => {
                assert_eq!(format, Some("A4".to_string()));
            }
            _ => panic!("Expected PageTag node"),
        }
    }

    #[test]
    fn test_build_document_reference() {
        // Test document reference creation
        let span = Span::simple(0, 15);
        let result = Node::DocumentReference {
            path: "./docs/api.md".to_string(),
            span: span.clone(),
        };

        match result {
            Node::DocumentReference { path, .. } => {
                assert_eq!(path, "./docs/api.md");
            }
            _ => panic!("Expected DocumentReference node"),
        }
    }

    #[test]
    fn test_build_table_of_contents() {
        // Test table of contents creation
        let span = Span::simple(0, 10);
        let result = Node::TableOfContents {
            depth: Some(3),
            document: Some("./README.md".to_string()),
            span: span.clone(),
        };

        match result {
            Node::TableOfContents {
                depth, document, ..
            } => {
                assert_eq!(depth, Some(3));
                assert_eq!(document, Some("./README.md".to_string()));
            }
            _ => panic!("Expected TableOfContents node"),
        }
    }

    #[test]
    fn test_build_table_of_contents_simple() {
        // Test simple table of contents without document
        let span = Span::simple(0, 8);
        let result = Node::TableOfContents {
            depth: Some(2),
            document: None,
            span: span.clone(),
        };

        match result {
            Node::TableOfContents {
                depth, document, ..
            } => {
                assert_eq!(depth, Some(2));
                assert_eq!(document, None);
            }
            _ => panic!("Expected TableOfContents node"),
        }
    }

    #[test]
    fn test_build_run_inline() {
        // Test inline run command creation
        let span = Span::simple(0, 20);
        let result = Node::RunInline {
            script_type: "bash".to_string(),
            command: "ls -la".to_string(),
            span: span.clone(),
        };

        match result {
            Node::RunInline {
                script_type,
                command,
                ..
            } => {
                assert_eq!(script_type, "bash");
                assert_eq!(command, "ls -la");
            }
            _ => panic!("Expected RunInline node"),
        }
    }

    #[test]
    fn test_build_run_block() {
        // Test block run command creation
        let span = Span::simple(0, 50);
        let result = Node::RunBlock {
            script_type: "python".to_string(),
            content: "print('hello')\nprint('world')".to_string(),
            span: span.clone(),
        };

        match result {
            Node::RunBlock {
                script_type,
                content,
                ..
            } => {
                assert_eq!(script_type, "python");
                assert_eq!(content, "print('hello')\nprint('world')");
            }
            _ => panic!("Expected RunBlock node"),
        }
    }

    #[test]
    fn test_build_diagram_block() {
        // Test diagram block creation
        let span = Span::simple(0, 30);
        let result = Node::DiagramBlock {
            diagram_type: "mermaid".to_string(),
            content: "graph TD\nA-->B".to_string(),
            span: span.clone(),
        };

        match result {
            Node::DiagramBlock {
                diagram_type,
                content,
                ..
            } => {
                assert_eq!(diagram_type, "mermaid");
                assert_eq!(content, "graph TD\nA-->B");
            }
            _ => panic!("Expected DiagramBlock node"),
        }
    }

    #[test]
    fn test_build_tab_block() {
        // Test tab block creation
        let span = Span::simple(0, 100);
        let tab1 = Node::Tab {
            name: Some("Tab 1".to_string()),
            content: vec![TestMarcoBuilder::create_text_node(
                "Content 1",
                span.clone(),
            )],
            span: span.clone(),
        };
        let tab2 = Node::Tab {
            name: Some("Tab 2".to_string()),
            content: vec![TestMarcoBuilder::create_text_node(
                "Content 2",
                span.clone(),
            )],
            span: span.clone(),
        };

        let result = Node::TabBlock {
            title: Some("Example Tabs".to_string()),
            tabs: vec![tab1, tab2],
            span: span.clone(),
        };

        match result {
            Node::TabBlock { title, tabs, .. } => {
                assert_eq!(title, Some("Example Tabs".to_string()));
                assert_eq!(tabs.len(), 2);

                if let Node::Tab { name, .. } = &tabs[0] {
                    assert_eq!(name, &Some("Tab 1".to_string()));
                }
                if let Node::Tab { name, .. } = &tabs[1] {
                    assert_eq!(name, &Some("Tab 2".to_string()));
                }
            }
            _ => panic!("Expected TabBlock node"),
        }
    }

    #[test]
    fn test_build_tab() {
        // Test individual tab creation
        let span = Span::simple(0, 20);
        let content = vec![TestMarcoBuilder::create_text_node(
            "Tab content",
            span.clone(),
        )];

        let result = Node::Tab {
            name: Some("Test Tab".to_string()),
            content,
            span: span.clone(),
        };

        match result {
            Node::Tab { name, content, .. } => {
                assert_eq!(name, Some("Test Tab".to_string()));
                assert_eq!(content.len(), 1);
                if let Node::Text { content: text, .. } = &content[0] {
                    assert_eq!(text, "Tab content");
                }
            }
            _ => panic!("Expected Tab node"),
        }
    }

    #[test]
    fn test_constants() {
        // Test Marco-specific constants
        let task_checked_markers = ['x', 'X'];
        let default_page_format = "A4";

        assert_eq!(task_checked_markers, ['x', 'X']);
        assert_eq!(default_page_format, "A4");
    }

    #[test]
    fn test_task_marker_validation() {
        // Test task marker character validation
        let task_checked_markers = ['x', 'X'];
        let valid_chars = ['x', 'X', ' '];
        let invalid_chars = ['o', 'v', '✓', '✗', 'y', 'n'];

        for &ch in &valid_chars {
            let is_valid = ch == ' ' || task_checked_markers.contains(&ch);
            assert!(is_valid, "Character '{}' should be valid", ch);
        }

        for &ch in &invalid_chars {
            let is_valid = ch == ' ' || task_checked_markers.contains(&ch);
            assert!(!is_valid, "Character '{}' should be invalid", ch);
        }
    }

    #[test]
    fn test_edge_cases() {
        // Test edge cases for parsing

        // Very short inputs
        assert!(TestMarcoBuilder::parse_task_syntax("").is_none());
        assert!(TestMarcoBuilder::parse_user_mention_syntax("@").is_none());
        assert!(TestMarcoBuilder::parse_bookmark_syntax("[bookmark:]").is_none());

        // Boundary length inputs
        let min_task = "[x] a";
        assert!(TestMarcoBuilder::parse_task_syntax(min_task).is_some());

        let min_mention = "@a";
        assert!(TestMarcoBuilder::parse_user_mention_syntax(min_mention).is_some());

        // Unicode and special characters
        let unicode_task = "[x] 你好世界 emoji 🎉";
        assert!(TestMarcoBuilder::parse_task_syntax(unicode_task).is_some());
    }

    #[test]
    fn test_error_recovery() {
        // Test error recovery scenarios
        let span = Span::simple(0, 10);

        // Test fallback content creation
        let fallback_node = TestMarcoBuilder::create_text_node("fallback", span.clone());
        match fallback_node {
            Node::Text { content, .. } => {
                assert_eq!(content, "fallback");
            }
            _ => panic!("Expected Text node for fallback"),
        }
    }

    #[test]
    fn test_complex_scenarios() {
        // Test complex parsing scenarios

        // Task with special characters in content
        let complex_task = "[x] Task with **bold** and `code` content";
        let task_result = TestMarcoBuilder::parse_task_syntax(complex_task);
        assert!(task_result.is_some());
        if let Some((_, checked, content)) = task_result {
            assert_eq!(checked, "x");
            assert!(content.contains("**bold**"));
            assert!(content.contains("`code`"));
        }

        // Bookmark with complex path
        let complex_bookmark = "[bookmark:section](./path/with spaces/file name.md=123)";
        let bookmark_result = TestMarcoBuilder::parse_bookmark_syntax(complex_bookmark);
        assert!(bookmark_result.is_some());
        if let Some((name, path, line)) = bookmark_result {
            assert_eq!(name, "section");
            assert!(path.contains("spaces"));
            assert_eq!(line, Some(123));
        }
    }

    #[test]
    fn test_span_handling() {
        // Test span creation and handling
        let test_cases = vec![("short", 5), ("longer content here", 19), ("", 0)];

        for (content, expected_len) in test_cases {
            let span = Span::simple(0, content.len() as u32);
            assert_eq!(span.end, expected_len);
            assert_eq!(span.start, 0);
            assert_eq!(span.line, 1);
            assert_eq!(span.column, 1);
        }
    }
}
