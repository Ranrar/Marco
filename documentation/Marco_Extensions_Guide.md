# Marco Extensions Documentation

## Overview

Marco extends standard Markdown with powerful features for technical documentation, collaboration, and advanced content organization. This guide covers all Marco-specific extensions and their usage.

## Table of Contents

1. [User Mentions](#user-mentions)
2. [Bookmarks](#bookmarks)
3. [Tabs Blocks](#tabs-blocks)
4. [Admonitions](#admonitions)
5. [Run Blocks](#run-blocks)
6. [Document References](#document-references)
7. [Table of Contents](#table-of-contents)
8. [Page Tags](#page-tags)
9. [Task Lists](#task-lists)
10. [Implementation Examples](#implementation-examples)

## User Mentions

Reference users across different platforms with optional display names.

### Syntax

```markdown
@username [platform](Display Name)
@username [platform]
@username
```

### Examples

```markdown
@john [github](John Doe)
@alice [slack]
@bob
```

### Implementation

```rust
// AST Structure
pub struct UserMention {
    pub username: String,
    pub platform: Option<String>,
    pub display_name: Option<String>,
    pub span: Span,
}

// Parser implementation
fn build_user_mention(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut username = String::new();
    let mut platform = None;
    let mut display_name = None;
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::username => {
                username = inner_pair.as_str().trim_start_matches('@').to_string();
                Self::validate_username(&username)?;
            }
            Rule::platform => {
                let platform_str = inner_pair.as_str();
                Self::validate_platform(platform_str)?;
                platform = Some(platform_str.to_string());
            }
            Rule::display_name => {
                let name = inner_pair.as_str();
                Self::validate_display_name(name)?;
                display_name = Some(name.to_string());
            }
            _ => continue,
        }
    }
    
    Ok(Node::UserMention {
        username,
        platform,
        display_name,
        span,
    })
}
```

### Validation Rules

- Username: 1-32 characters, alphanumeric + underscore + hyphen
- Platform: 1-20 characters, alphanumeric + underscore + hyphen
- Display Name: 0-64 characters, no control characters

## Bookmarks

Create navigable references to specific locations in documents.

### Syntax

```markdown
[bookmark:label](path=line)
[bookmark:label](path)
```

### Examples

```markdown
[bookmark:introduction](./README.md=15)
[bookmark:config](./config.md)
[bookmark:section](../docs/guide.md=42)
```

### Implementation

```rust
// AST Structure
pub struct Bookmark {
    pub label: String,
    pub path: String,
    pub line: Option<u32>,
    pub span: Span,
}

// Parser implementation
fn build_bookmark(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut label = String::new();
    let mut path = String::new();
    let mut line = None;
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::bookmark_label => {
                label = inner_pair.as_str().to_string();
                Self::validate_label(&label)?;
            }
            Rule::bookmark_path => {
                let path_content = inner_pair.as_str();
                
                if let Some((file_path, line_str)) = path_content.split_once('=') {
                    path = file_path.to_string();
                    line = Some(Self::validate_line_number(line_str)?);
                } else {
                    path = path_content.to_string();
                }
                
                Self::validate_path(&path)?;
            }
            _ => continue,
        }
    }
    
    Ok(Node::Bookmark { label, path, line, span })
}
```

### Use Cases

- Cross-referencing sections in documentation
- Creating navigation aids in large documents
- Linking to specific code locations
- Building interactive documentation indices

## Tabs Blocks

Organize content into tabbed interfaces for better organization.

### Syntax

```markdown
:::
tabs [Title]
@tab Tab1
Content for tab 1

@tab Tab2
Content for tab 2
:::
```

### Examples

```markdown
:::
tabs API Examples
@tab Python
```python
import requests
response = requests.get('https://api.example.com')
```

@tab JavaScript
```javascript
fetch('https://api.example.com')
  .then(response => response.json())
```

@tab curl
```bash
curl https://api.example.com
```
:::
```

### Implementation

```rust
// AST Structure
pub struct TabBlock {
    pub title: Option<String>,
    pub tabs: Vec<TabContent>,
    pub span: Span,
}

pub struct TabContent {
    pub name: String,
    pub content: Vec<Node>,
}

// Parser implementation
fn build_tab_block(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut title = None;
    let mut tabs = Vec::new();
    let mut current_tab = None;
    let mut current_content = Vec::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::tabs_title => {
                title = Some(inner_pair.as_str().to_string());
            }
            Rule::tab_marker => {
                // Save previous tab if exists
                if let Some(tab_name) = current_tab.take() {
                    tabs.push(TabContent {
                        name: tab_name,
                        content: std::mem::take(&mut current_content),
                    });
                }
                
                // Start new tab
                current_tab = Some(inner_pair.as_str().trim_start_matches('@').trim_start_matches("tab").trim().to_string());
            }
            Rule::tab_content => {
                let content = Self::build_content_with_fallback(inner_pair, span.clone())?;
                current_content.extend(content);
            }
            _ => continue,
        }
    }
    
    // Save final tab
    if let Some(tab_name) = current_tab {
        tabs.push(TabContent {
            name: tab_name,
            content: current_content,
        });
    }
    
    Ok(Node::TabBlock { title, tabs, span })
}
```

### Use Cases

- API documentation with multiple language examples
- Step-by-step tutorials with different approaches
- Configuration examples for different environments
- Code samples in multiple frameworks

## Admonitions

Create attention-grabbing callout boxes for important information.

### Syntax

```markdown
:::
note|tip|warning|danger|info [Title]
Content here
:::
```

### Examples

```markdown
:::
note Important Security Notice
Always validate user input before processing.
:::

:::
warning
This operation cannot be undone!
:::

:::
tip Pro Tip
Use keyboard shortcuts to improve your workflow.
:::
```

### Implementation

```rust
// AST Structure
pub struct Admonition {
    pub admonition_type: AdmonitionType,
    pub title: Option<String>,
    pub content: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdmonitionType {
    Note,
    Tip,
    Warning,
    Danger,
    Info,
}

// Parser implementation
fn build_admonition(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut admonition_type = AdmonitionType::Note;
    let mut title = None;
    let mut content = Vec::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::admonition_type => {
                let type_str = inner_pair.as_str().to_lowercase();
                admonition_type = match type_str.as_str() {
                    "note" => AdmonitionType::Note,
                    "tip" => AdmonitionType::Tip,
                    "warning" => AdmonitionType::Warning,
                    "danger" => AdmonitionType::Danger,
                    "info" => AdmonitionType::Info,
                    _ => return Err(MarcoError::invalid_admonition_type(type_str)),
                };
            }
            Rule::admonition_title => {
                title = Some(inner_pair.as_str().to_string());
            }
            Rule::admonition_content => {
                content = Self::build_content_with_fallback(inner_pair, span.clone())?;
            }
            _ => continue,
        }
    }
    
    Ok(Node::Admonition {
        admonition_type,
        title,
        content,
        span,
    })
}
```

### Styling Guidelines

- **Note**: Blue background, information icon
- **Tip**: Green background, lightbulb icon
- **Warning**: Yellow background, caution icon
- **Danger**: Red background, warning icon
- **Info**: Light blue background, info icon

## Run Blocks

Execute code and display results inline with documentation.

### Syntax

```markdown
`run@language command`

```
run@language
command here
```
```

### Examples

```markdown
`run@bash echo "Hello World"`

```
run@python
print("Current time:", datetime.now())
import datetime
```

```
run@javascript
console.log("JavaScript execution");
const result = Math.PI * 2;
```
```

### Implementation

```rust
// AST Structure
pub struct RunBlock {
    pub language: String,
    pub code: String,
    pub inline: bool,
    pub span: Span,
}

// Parser implementation
fn build_run_inline(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut language = String::new();
    let mut code = String::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::run_language => {
                language = inner_pair.as_str().to_string();
            }
            Rule::run_code => {
                code = inner_pair.as_str().to_string();
                Self::validate_code_block(&code)?;
            }
            _ => continue,
        }
    }
    
    Ok(Node::RunBlock {
        language,
        code,
        inline: true,
        span,
    })
}

fn build_run_block(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut language = String::new();
    let mut code = String::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::run_language => {
                language = inner_pair.as_str().trim_start_matches("run@").to_string();
            }
            Rule::run_code => {
                code = inner_pair.as_str().to_string();
                Self::validate_code_block(&code)?;
            }
            _ => continue,
        }
    }
    
    Ok(Node::RunBlock {
        language,
        code,
        inline: false,
        span,
    })
}
```

### Security Considerations

- Sandbox execution environment
- Limit execution time and resources
- Whitelist allowed languages and commands
- Validate and sanitize all code input

## Document References

Create references to other documents with automatic title resolution.

### Syntax

```markdown
[@doc](path)
[@doc](path "Custom Title")
```

### Examples

```markdown
[@doc](./user-guide.md)
[@doc](../api/reference.md "API Reference")
[@doc](https://example.com/docs.md)
```

### Implementation

```rust
// AST Structure
pub struct DocumentReference {
    pub path: String,
    pub title: Option<String>,
    pub resolved_title: Option<String>,
    pub span: Span,
}

// Parser implementation
fn build_document_reference(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut path = String::new();
    let mut title = None;
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::doc_path => {
                path = inner_pair.as_str().to_string();
                Self::validate_path(&path)?;
            }
            Rule::doc_title => {
                title = Some(inner_pair.as_str().to_string());
                Self::validate_title(title.as_ref().unwrap())?;
            }
            _ => continue,
        }
    }
    
    // Title resolution happens during rendering
    Ok(Node::DocumentReference {
        path,
        title,
        resolved_title: None,
        span,
    })
}
```

## Table of Contents

Generate automatic table of contents from document headings.

### Syntax

```markdown
[toc]
[toc=depth]
[toc depth=3 title="Custom Title"]
```

### Examples

```markdown
[toc]
[toc=2]
[toc depth=3 title="Contents"]
```

### Implementation

```rust
// AST Structure
pub struct TableOfContents {
    pub depth: Option<u8>,
    pub title: Option<String>,
    pub entries: Vec<TocEntry>,
    pub span: Span,
}

pub struct TocEntry {
    pub level: u8,
    pub title: String,
    pub anchor: String,
    pub children: Vec<TocEntry>,
}

// Parser implementation
fn build_table_of_contents(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut depth = None;
    let mut title = None;
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::toc_depth => {
                let depth_str = inner_pair.as_str();
                depth = Some(depth_str.parse::<u8>().map_err(|_| {
                    MarcoError::parse_error(format!("Invalid TOC depth: {}", depth_str))
                })?);
            }
            Rule::toc_title => {
                title = Some(inner_pair.as_str().to_string());
            }
            _ => continue,
        }
    }
    
    // TOC entries are populated during post-processing
    Ok(Node::TableOfContents {
        depth,
        title,
        entries: Vec::new(),
        span,
    })
}
```

## Page Tags

Add metadata tags to documents for organization and processing.

### Syntax

```markdown
[page=format]
[tag=value]
[meta key=value]
```

### Examples

```markdown
[page=A4]
[tag=documentation]
[meta author="John Doe"]
[meta version="1.0"]
```

### Implementation

```rust
// AST Structure
pub struct PageTag {
    pub tag_type: PageTagType,
    pub key: String,
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PageTagType {
    Page,
    Tag,
    Meta,
}

// Parser implementation
fn build_page_tag(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut tag_type = PageTagType::Tag;
    let mut key = String::new();
    let mut value = String::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::page_tag_type => {
                let type_str = inner_pair.as_str();
                tag_type = match type_str {
                    "page" => PageTagType::Page,
                    "tag" => PageTagType::Tag,
                    "meta" => PageTagType::Meta,
                    _ => PageTagType::Tag,
                };
            }
            Rule::page_tag_key => {
                key = inner_pair.as_str().to_string();
            }
            Rule::page_tag_value => {
                value = inner_pair.as_str().to_string();
            }
            _ => continue,
        }
    }
    
    Ok(Node::PageTag {
        tag_type,
        key,
        value,
        span,
    })
}
```

## Task Lists

Enhanced task lists with additional metadata and styling.

### Syntax

```markdown
- [ ] Uncompleted task
- [x] Completed task
- [!] Important task
- [?] Question/uncertain task
```

### Examples

```markdown
- [x] Complete basic documentation
- [ ] Add advanced examples
- [!] Fix security vulnerability
- [?] Consider alternative approach
```

### Implementation

```rust
// AST Structure
pub struct TaskItem {
    pub checked: Option<bool>,
    pub priority: Option<TaskPriority>,
    pub content: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskPriority {
    Normal,
    Important,
    Question,
}

// Parser implementation
fn build_task_item(pair: Pair<Rule>, span: Span) -> Result<Node> {
    let mut checked = None;
    let mut priority = Some(TaskPriority::Normal);
    let mut content = Vec::new();
    
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::task_marker => {
                let marker = inner_pair.as_str();
                match marker {
                    "[ ]" => {
                        checked = Some(false);
                        priority = Some(TaskPriority::Normal);
                    }
                    "[x]" | "[X]" => {
                        checked = Some(true);
                        priority = Some(TaskPriority::Normal);
                    }
                    "[!]" => {
                        checked = Some(false);
                        priority = Some(TaskPriority::Important);
                    }
                    "[?]" => {
                        checked = Some(false);
                        priority = Some(TaskPriority::Question);
                    }
                    _ => {
                        checked = None;
                        priority = None;
                    }
                }
            }
            Rule::task_content => {
                content = Self::build_content_with_fallback(inner_pair, span.clone())?;
            }
            _ => continue,
        }
    }
    
    Ok(Node::TaskItem {
        checked,
        priority,
        content,
        span,
    })
}
```

## Implementation Examples

### Complete Marco Document

```markdown
# Project Documentation

[meta author="Development Team"]
[meta version="2.0"]
[page=A4]

[toc depth=3 title="Table of Contents"]

## Introduction

This document demonstrates Marco extensions for enhanced documentation.

:::
note Project Status
This project is actively maintained by @alice [github](Alice Smith).
See [bookmark:changelog](./CHANGELOG.md=15) for recent updates.
:::

## API Examples

:::
tabs Programming Languages
@tab Python
```python
import marco
parser = marco.Parser()
result = parser.parse(document)
```

@tab JavaScript
```javascript
const marco = require('marco');
const parser = new marco.Parser();
const result = parser.parse(document);
```
:::

## Tasks

Project roadmap:

- [x] Complete core parser
- [!] Implement security audit
- [ ] Add more language support
- [?] Consider WebAssembly compilation

## Execution Examples

Check system status: `run@bash date && uptime`

```
run@python
import sys
print(f"Python version: {sys.version}")
print("Marco parser ready!")
```

## References

For more information, see [@doc](./advanced-guide.md "Advanced Features Guide").

:::
tip
Use bookmarks like [bookmark:examples](./examples.md) for quick navigation!
:::
```

### Testing Marco Extensions

```rust
#[test]
fn test_complete_marco_document() {
    let input = include_str!("test_documents/complete_marco.md");
    let node = AstBuilder::build_from_string(input).unwrap();
    
    match node {
        Node::Document { children, .. } => {
            // Verify all Marco extensions are parsed correctly
            let mut found_user_mention = false;
            let mut found_bookmark = false;
            let mut found_tabs = false;
            let mut found_admonition = false;
            let mut found_run_block = false;
            let mut found_doc_ref = false;
            let mut found_toc = false;
            let mut found_page_tag = false;
            let mut found_task_item = false;
            
            for child in &children {
                match child {
                    Node::UserMention { .. } => found_user_mention = true,
                    Node::Bookmark { .. } => found_bookmark = true,
                    Node::TabBlock { .. } => found_tabs = true,
                    Node::Admonition { .. } => found_admonition = true,
                    Node::RunBlock { .. } => found_run_block = true,
                    Node::DocumentReference { .. } => found_doc_ref = true,
                    Node::TableOfContents { .. } => found_toc = true,
                    Node::PageTag { .. } => found_page_tag = true,
                    Node::TaskItem { .. } => found_task_item = true,
                    _ => continue,
                }
            }
            
            assert!(found_user_mention, "User mention not found");
            assert!(found_bookmark, "Bookmark not found");
            assert!(found_tabs, "Tabs block not found");
            assert!(found_admonition, "Admonition not found");
            assert!(found_run_block, "Run block not found");
            assert!(found_doc_ref, "Document reference not found");
            assert!(found_toc, "Table of contents not found");
            assert!(found_page_tag, "Page tag not found");
            assert!(found_task_item, "Task item not found");
        }
        _ => panic!("Expected document node"),
    }
}
```

## Best Practices

### 1. User Mentions
- Always provide platform context for clarity
- Use consistent username formats across platforms
- Consider privacy implications

### 2. Bookmarks
- Use descriptive labels for better navigation
- Keep paths relative when possible
- Update line numbers when documents change

### 3. Tabs Blocks
- Limit number of tabs for usability (3-6 recommended)
- Use consistent naming conventions
- Provide meaningful titles

### 4. Admonitions
- Choose appropriate types for the content
- Keep titles concise and descriptive
- Don't overuse - reserve for important information

### 5. Run Blocks
- Always validate and sanitize code input
- Implement proper security measures
- Provide clear error messages

### 6. Document References
- Use absolute paths for external documents
- Provide fallback titles for broken references
- Consider link checking in CI/CD

This comprehensive guide covers all Marco extensions and their implementation details. The modular architecture makes it easy to extend Marco with additional features while maintaining consistency and performance.
