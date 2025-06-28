# Adding New Programming Languages to Marco

This guide explains how to add support for new programming languages to the Marco Markdown Editor, including syntax highlighting, menu integration, and fenced code block support.

## Overview

Marco uses a modular language system defined in `src/code_languages.rs` that supports:
- Syntax highlighting with regex patterns
- Language aliases and file extensions
- Color schemes for visual distinction
- Menu integration for quick access
- Fenced code block insertion

## Step 1: Define the Language

### 1.1 Create a Language Definition

Add your language to the `initialize_default_languages()` method in `src/code_languages.rs`:

```rust
// Example: Adding Swift support
self.add_language(CodeLanguage {
    name: "Swift".to_string(),
    aliases: vec!["swift".to_string(), "ios".to_string()],
    file_extensions: vec![".swift".to_string()],
    keywords: vec![
        "class", "struct", "enum", "protocol", "extension", "func", "var", "let",
        "if", "else", "switch", "case", "default", "for", "while", "repeat",
        "break", "continue", "return", "throw", "try", "catch", "guard",
        "defer", "import", "public", "private", "internal", "fileprivate",
        "open", "static", "final", "override", "required", "convenience",
        "lazy", "weak", "unowned", "mutating", "nonmutating", "dynamic",
        "inout", "associatedtype", "typealias", "true", "false", "nil"
    ].iter().map(|s| s.to_string()).collect(),
    comment_patterns: vec![
        "//.*$".to_string(),                    // Single-line comments
        "/\\*[\\s\\S]*?\\*/".to_string()       // Multi-line comments
    ],
    string_patterns: vec![
        "\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(),  // Double quotes
        "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string()       // Single quotes
    ],
    number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[eE][+-]?\\d+)?\\b".to_string(),
    function_pattern: Some("\\bfunc\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
    class_pattern: Some("\\b(class|struct|enum)\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
    color_scheme: LanguageColorScheme::default(),
});
```

### 1.2 Language Properties Explained

- **name**: Display name for the language
- **aliases**: Alternative names for recognition (e.g., "js" for "javascript")
- **file_extensions**: File extensions that auto-detect this language
- **keywords**: Language keywords for syntax highlighting
- **comment_patterns**: Regex patterns for single and multi-line comments
- **string_patterns**: Regex patterns for string literals
- **number_pattern**: Regex pattern for numeric literals
- **function_pattern**: Regex pattern for function declarations (optional)
- **class_pattern**: Regex pattern for class/struct declarations (optional)
- **color_scheme**: Color scheme for syntax highlighting

## Step 2: Add Menu Integration

### 2.1 Add Menu Item

In `src/menu.rs`, add your language to the fenced code submenu:

```rust
// In the fenced_code_menu section, add:
fenced_code_menu.append(Some("Swift"), Some("app.insert_fenced_swift"));
```

### 2.2 Create Action Handler

Add an action entry for your language:

```rust
let insert_fenced_swift_action = gio::ActionEntry::builder("insert_fenced_swift")
    .activate({
        let editor = editor.clone();
        move |_app: &Application, _action, _param| {
            editor.insert_fenced_code_with_language("swift");
        }
    })
    .build();
```

### 2.3 Register the Action

Add your action to the actions array:

```rust
app.add_action_entries([
    // ... existing actions ...
    insert_fenced_swift_action,
    // ... more actions ...
]);
```

## Step 3: Add CSS Styling (Optional)

### 3.1 Language-Specific Border Color

In `src/main.rs`, add a unique border color for your language:

```rust
// Add to the CSS section:
.code-block-swift {
    border-left: 4px solid #ff5722; /* Swift orange */
}
```

### 3.2 Custom Color Scheme

For a custom color scheme, modify the language definition:

```rust
color_scheme: LanguageColorScheme {
    keyword_color: "#ff5722".to_string(),     // Custom orange
    comment_color: "#6a737d".to_string(),     // Gray
    string_color: "#22863a".to_string(),      // Green
    number_color: "#005cc5".to_string(),      // Blue
    function_color: "#6f42c1".to_string(),    // Purple
    class_color: "#e36209".to_string(),       // Orange
    background_color: "#f6f8fa".to_string(),  // Light gray
    text_color: "#24292e".to_string(),        // Dark gray
},
```

## Step 4: Test Your Implementation

### 4.1 Compile and Test

```bash
cd /path/to/marco
cargo build
cargo run
```

### 4.2 Test the Menu

1. Open Marco
2. Go to **Format → Fenced Code Block**
3. Verify your language appears in the submenu
4. Test inserting a fenced code block

### 4.3 Test Auto-Detection

1. Create a file with your language's extension
2. Verify Marco detects the language automatically

## Step 5: Advanced Features

### 5.1 Complex Regex Patterns

For languages with complex syntax, use more sophisticated patterns:

```rust
// Example: Matching function calls with parameters
function_pattern: Some(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\([^)]*\)".to_string()),

// Example: Matching generics in class definitions
class_pattern: Some(r"\bclass\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:<[^>]*>)?".to_string()),
```

### 5.2 Language-Specific Validation

Add validation for your language patterns:

```rust
// Test your regex patterns
let manager = CodeLanguageManager::new();
if let Some(language) = manager.get_language("swift") {
    match manager.validate_language(language) {
        Ok(()) => println!("Language validation passed"),
        Err(e) => println!("Validation error: {}", e),
    }
}
```

## Examples of Supported Languages

Marco comes with built-in support for:

1. **JavaScript** (`js`, `javascript`, `node`)
2. **Python** (`py`, `python`, `python3`)
3. **Java** (`java`)
4. **TypeScript** (`ts`, `typescript`)
5. **C#** (`cs`, `csharp`, `c#`)
6. **C++** (`cpp`, `c++`, `cxx`)
7. **C** (`c`)
8. **PHP** (`php`)
9. **Go** (`go`, `golang`)
10. **Rust** (`rust`, `rs`)

Plus common markup languages:
- **HTML** (`html`)
- **CSS** (`css`)
- **JSON** (`json`)
- **XML** (`xml`)
- **SQL** (`sql`)
- **Bash** (`bash`)
- **YAML** (`yaml`)
- **Markdown** (`markdown`)

## API Reference

### CodeLanguage Struct

```rust
pub struct CodeLanguage {
    pub name: String,
    pub aliases: Vec<String>,
    pub file_extensions: Vec<String>,
    pub keywords: Vec<String>,
    pub comment_patterns: Vec<String>,
    pub string_patterns: Vec<String>,
    pub number_pattern: String,
    pub function_pattern: Option<String>,
    pub class_pattern: Option<String>,
    pub color_scheme: LanguageColorScheme,
}
```

### CodeLanguageManager Methods

- `add_language(language: CodeLanguage)` - Add a new language
- `get_language(name: &str) -> Option<&CodeLanguage>` - Get language by name/alias
- `get_language_names() -> Vec<String>` - List all languages
- `get_language_suggestions(partial: &str) -> Vec<String>` - Get autocomplete suggestions
- `validate_language(language: &CodeLanguage) -> Result<(), String>` - Validate regex patterns
- `highlight_code(code: &str, language: &str) -> String` - Apply syntax highlighting

### Utility Methods

- `create_custom_language()` - Helper to create languages with defaults
- `has_language(name: &str) -> bool` - Check if language exists
- `language_count() -> usize` - Get total language count
- `get_language_by_extension(ext: &str) -> Option<&CodeLanguage>` - Detect by file extension

## Best Practices

1. **Use Standard Aliases**: Include common abbreviations (e.g., "py" for Python)
2. **Comprehensive Keywords**: Include all language keywords for better highlighting
3. **Test Regex Patterns**: Validate patterns with `validate_language()`
4. **Unique Colors**: Choose distinctive border colors for visual identification
5. **File Extensions**: Include all common extensions for auto-detection
6. **Performance**: Keep regex patterns efficient for large files

## Troubleshooting

### Common Issues

1. **Regex Compilation Errors**: Test patterns with online regex validators
2. **Menu Not Appearing**: Check action registration in the actions array
3. **No Syntax Highlighting**: Verify regex patterns match your test code
4. **Wrong Language Detection**: Check file extension patterns

### Debugging

```rust
// Enable debug logging
let manager = CodeLanguageManager::new();
println!("Supported languages: {:?}", manager.get_language_names());

// Test language detection
if let Some(lang) = manager.get_language("your_language") {
    println!("Found language: {:?}", lang.name);
} else {
    println!("Language not found");
}
```

## Contributing

When adding a new language to Marco:

1. Follow this guide to implement the language
2. Test thoroughly with real code examples
3. Add unit tests for regex patterns
4. Update documentation
5. Submit a pull request with your changes

Marco's language system is designed to be extensible and maintainable. Following these guidelines ensures consistency and quality across all supported languages.
