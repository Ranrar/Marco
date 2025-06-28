# Marco Code Languages Quick Reference

## Supported Programming Languages

### Top 10 Languages (with aliases)
| Language   | Aliases              | Extension | Border Color |
|------------|---------------------|-----------|--------------|
| JavaScript | `js`, `node`        | `.js`     | 🟡 Yellow    |
| Python     | `py`, `python3`     | `.py`     | 🔵 Blue      |
| Java       | `java`              | `.java`   | 🟠 Orange    |
| TypeScript | `ts`                | `.ts`     | 🔵 Blue      |
| C#         | `cs`, `csharp`      | `.cs`     | 🟢 Green     |
| C++        | `cpp`, `c++`, `cxx` | `.cpp`    | 🔵 Blue      |
| C          | `c`                 | `.c`      | ⚪ Gray      |
| PHP        | `php`               | `.php`    | 🟣 Purple    |
| Go         | `golang`            | `.go`     | 🔵 Cyan      |
| Rust       | `rs`                | `.rs`     | 🔴 Red       |

### Markup & Data Languages
| Language | Extension | Use Case              |
|----------|----------|-----------------------|
| HTML     | `.html`  | Web markup            |
| CSS      | `.css`   | Styling               |
| JSON     | `.json`  | Data interchange      |
| XML      | `.xml`   | Structured data       |
| SQL      | `.sql`   | Database queries      |
| Bash     | `.sh`    | Shell scripts         |
| YAML     | `.yml`   | Configuration         |
| Markdown | `.md`    | Documentation         |

## How to Insert Code Blocks

### Method 1: Quick Selection
1. **Format** → **Fenced Code Block** → **[Language]**
2. Code block inserted instantly with language tag

### Method 2: Dialog Selection
1. **Format** → **Fenced Code Block** → **With Language Selection...**
2. Choose language from dropdown
3. Optionally add code sample
4. Click "Insert"

### Method 3: Keyboard Shortcuts
- **Ctrl+`**: Insert inline code
- Manual typing: ` ```language ` + Enter

## Syntax Highlighting Features

### Highlighted Elements
- **Keywords**: Language-specific keywords (bold, colored)
- **Strings**: Text in quotes (colored)
- **Comments**: Single/multi-line comments (italic, gray)
- **Numbers**: Numeric literals (colored)
- **Functions**: Function declarations (bold, colored)
- **Classes**: Class/struct definitions (bold, colored)

### Visual Indicators
- **Language borders**: Colored left border for quick identification
- **Monospace font**: Professional code appearance
- **Proper spacing**: Optimized line height and padding

## Examples

### JavaScript
```javascript
function greet(name) {
    return `Hello, ${name}!`;
}
```

### Python
```python
def greet(name):
    return f"Hello, {name}!"
```

### Rust
```rust
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

## Tips

1. **File Detection**: Marco auto-detects language from file extensions
2. **Aliases**: Use common abbreviations (e.g., `py` for Python)
3. **Mixed Languages**: Each code block can have a different language
4. **Custom Styling**: Languages have unique border colors for identification
5. **Professional Output**: All code blocks render with syntax highlighting in preview

## Adding New Languages

See `ADDING_LANGUAGES.md` for detailed instructions on extending Marco with additional programming languages.
