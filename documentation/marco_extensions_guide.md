# Marco Markdown Extensions Guide

This document provides detailed explanations of Marco's custom markdown extensions, their syntax, use cases, and expected application behavior.

---

## Table of Contents

1. [Document References](#document-references)
2. [Executable Code](#executable-code)
3. [Page Formatting](#page-formatting)
4. [Bookmarks](#bookmarks)
5. [User Mentions](#user-mentions)
6. [Tab Blocks](#tab-blocks)

---

## Document References

### Syntax

```markdown
[@doc](./path/to/document.md)
[@doc](../relative/path/file.md)
```

### Description

Document references create navigational links between Marco documents, enabling cross-document workflows and multi-file project organization.

### Examples

```markdown
See the API reference: [@doc](./api_reference.md)

For installation instructions: [@doc](../setup/install.md)

Related concepts: [@doc](./concepts/architecture.md)
```

### Application Behavior

When the user clicks a document reference, the application should:

1. **Resolve Path**: Calculate the absolute path relative to the current document
2. **Validate Existence**: Check if the target document exists
3. **Open Document**: 
   - Load the referenced document in the editor
   - Switch focus to the new document
   - Preserve the current document in history/tabs
4. **Handle Errors**:
   - Show error message if document doesn't exist
   - Offer to create the document if it's missing
   - Handle permission errors gracefully

### UI Considerations

- **Visual Indicator**: Render with a distinct icon (e.g., document icon)
- **Hover Preview**: Show tooltip with full path or document title
- **Broken Links**: Highlight broken references in a different color
- **Navigation History**: Add to browser-style back/forward navigation

---

## Executable Code

Marco supports two forms of executable code: inline and fenced blocks.

### Inline Executable Code

#### Syntax

```markdown
run@bash(ls -la)
run@python(print('hello'))
run@zsh(echo $SHELL)
run@powershell(Get-Date)
```

#### Description

Inline executable code allows embedding small script commands directly within text paragraphs. Useful for documentation with interactive examples.

#### Examples

```markdown
Check your current directory with run@bash(pwd) to see where you are.

The current date is run@python(import datetime; print(datetime.date.today())).

List files: run@bash(ls -la | head -5)
```

#### Application Behavior

1. **Parsing**: Extract the script type and command
2. **Security Check**: 
   - Prompt user for permission before execution
   - Show the command that will be executed
   - Implement sandbox/whitelist for safe commands
3. **Execution**:
   - Run command in appropriate interpreter (bash, python, etc.)
   - Capture stdout and stderr
   - Apply timeout limits (e.g., 5 seconds for inline)
4. **Display**:
   - Show output inline, replacing or alongside the command
   - Use monospace font for output
   - Show error messages in red if command fails
5. **Caching**: Cache results to avoid re-running on every render

### Fenced Executable Code Block

#### Syntax

````markdown
```run@bash
ls -la
echo "Done listing files"
pwd
```

```run@python
def greet(name):
    return f"Hello, {name}!"

print(greet("Marco"))
```
````

#### Description

Fenced executable blocks allow multi-line scripts with full programming capabilities. Ideal for tutorials, demonstrations, and interactive documentation.

#### Examples

````markdown
```run@bash
# System information script
echo "System: $(uname -s)"
echo "User: $(whoami)"
echo "Date: $(date)"
```

```run@python
import sys
import platform

print(f"Python {sys.version}")
print(f"Platform: {platform.system()}")
```

```run@zsh
for i in {1..5}; do
  echo "Count: $i"
done
```
````

#### Application Behavior

1. **UI Presentation**:
   - Show "Run" button or icon in code block header
   - Display script type badge (bash, python, etc.)
   - Show execution status indicator
2. **Execution Model**:
   - Manual execution (user clicks "Run")
   - Optional auto-run mode with user permission
   - Working directory should be document's directory
3. **Output Handling**:
   - Create expandable output section below code
   - Show execution time and exit code
   - Stream output in real-time for long-running scripts
   - Preserve ANSI color codes if supported
4. **Security**:
   - Require explicit user permission
   - Sandbox execution environment
   - Limit resource usage (CPU, memory, time)
   - Warn about potentially dangerous commands
5. **Error Handling**:
   - Display stderr separately from stdout
   - Show exit codes and error messages
   - Provide option to copy error for debugging

#### Supported Script Types

- **Shell**: `bash`, `zsh`, `sh`
- **Windows**: `bat`, `powershell`, `ps`
- **Python**: `python`, `py`

---

## Page Formatting

### Syntax

```markdown
[page=A4]
[page=US]
[page=210]
```

### Description

Page tags define document formatting and pagination for print/export workflows. Helps create print-ready documents with proper page breaks.

### Examples

```markdown
[page=A4]
# Report Title

This document is formatted for A4 paper.

---

[page=US]
# US Letter Document

This section uses US Letter format (8.5" x 11").

---

[page=297]
# Custom Width

Custom page width of 297mm.
```

### Application Behavior

1. **Page Format Recognition**:
   - **A4**: 210mm × 297mm (ISO standard)
   - **US**: 8.5" × 11" (US Letter)
   - **Custom**: Numeric value in millimeters for width
2. **Visual Rendering**:
   - Show page boundaries in editor preview
   - Add subtle page break indicators
   - Display page dimensions in status bar
3. **Print/Export**:
   - Apply correct page size to PDF export
   - Insert proper page breaks
   - Adjust margins according to format
4. **Editor Features**:
   - Show ruler with page dimensions
   - Warn about content overflow
   - Preview page breaks in real-time
5. **Multiple Formats**:
   - Support switching formats within single document
   - Each `[page=...]` tag changes format from that point forward

---

## Bookmarks

### Syntax

```markdown
[bookmark: Label](./document.md)
[bookmark: Label](./document.md=42)
[bookmark: Section Name](../other/file.md=100)
```

### Description

Bookmarks create precise navigation points within documents, optionally targeting specific line numbers. Essential for code documentation and technical references.

### Examples

```markdown
See the main function: [bookmark: Main Entry Point](./src/main.rs=45)

Important configuration: [bookmark: Config Section](./config.md=12)

API endpoint definition: [bookmark: POST /users](./api_docs.md=234)

Jump to troubleshooting: [bookmark: Error Handling](./guide.md)
```

### Application Behavior

1. **Navigation**:
   - Open target document
   - Scroll to specified line number (if provided)
   - Highlight the target line temporarily
   - Focus cursor at the target location
2. **Line Number Handling**:
   - With `=42`: Jump to line 42
   - Without line: Jump to top of document or search for label
3. **Visual Indicators**:
   - Render with bookmark icon (🔖 or similar)
   - Show line number in tooltip
   - Different styling from regular links
4. **Validation**:
   - Check if document exists
   - Verify line number is within document bounds
   - Warn if target line is empty or out of range
5. **Smart Features**:
   - Update line numbers if document changes
   - Offer "Create Bookmark" UI action
   - Bookmark management panel showing all bookmarks
6. **Integration**:
   - Add to navigation history
   - Support back/forward navigation
   - Enable keyboard shortcuts (e.g., Ctrl+B for bookmarks panel)

---

## User Mentions

### Syntax

```markdown
@username[platform]
@username[platform](Display Name)
```

### Description

User mentions enable collaborative documentation by referencing team members, contributors, or authors with their social/platform identities.

### Examples

```markdown
Code review by @alice[github]

Documentation by @john_doe[gitlab](John Doe)

Design feedback: @sarah[twitter](Sarah Designer)

Contact @support[slack] for help

Follow @marco_editor[twitter](Marco Editor Official)
```

### Application Behavior

1. **Rendering**:
   - Display with @ symbol and username
   - Show platform badge/icon (GitHub, Twitter, etc.)
   - Use display name if provided, otherwise username
2. **Interactive Features**:
   - Clickable to open profile on platform
   - Hover tooltip showing full details:
     - Username
     - Platform
     - Display name
     - Profile link (if available)
3. **Platform Support**:
   - **Code Platforms**: github, gitlab, bitbucket
   - **Social Media**: twitter, linkedin, mastodon
   - **Communication**: slack, discord, teams
   - **Generic**: Custom platform names
4. **Link Generation**:
   - Automatically construct profile URLs:
     - `@user[github]` → `https://github.com/user`
     - `@user[twitter]` → `https://twitter.com/user`
   - Handle custom platforms gracefully
5. **Collaboration Features**:
   - Integrate with version control (Git blame)
   - Show user contributions in document
   - Enable @mentions in comments/notes
6. **Auto-completion**:
   - Suggest team members while typing @
   - Show recently mentioned users
   - Filter by platform

---

## Tab Blocks

### Syntax

````markdown
:::tab
@tab Tab 1 Name
Content for tab 1

@tab Tab 2 Name
Content for tab 2

@tab Tab 3 Name
Content for tab 3
:::

:::tab Optional Title
@tab JavaScript
```js
console.log('Hello');
```

@tab Python
```python
print('Hello')
```
:::
````

### Description

Tab blocks organize related content into tabbed interfaces, perfect for showing code examples in multiple languages, configuration options, or platform-specific instructions.

### Examples

````markdown
:::tab Installation Methods

@tab npm
```bash
npm install marco-editor
```

@tab yarn
```bash
yarn add marco-editor
```

@tab pnpm
```bash
pnpm add marco-editor
```
:::

:::tab Code Examples

@tab JavaScript
```js
function greet(name) {
  return `Hello, ${name}!`;
}
```

@tab Python
```python
def greet(name):
    return f"Hello, {name}!"
```

@tab Rust
```rust
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```
:::

:::tab Platform Instructions

@tab Windows
Run the installer from the downloads page.
Make sure to check "Add to PATH".

@tab macOS
```bash
brew install marco
```

@tab Linux
```bash
curl -sSL https://install.marco.dev | sh
```
:::
````

### Application Behavior

#### 1. Parsing and Structure

- **Header**: `:::tab` with optional title
- **Tabs**: Each `@tab` defines a new tab with optional name
- **Content**: Everything between `@tab` lines belongs to that tab
- **Closing**: `:::` marks the end of tab block

#### 2. Visual Rendering

- **Tab Bar**: Horizontal tabs at top of block
- **Active Tab**: Highlighted/underlined tab
- **Tab Names**: Display text after `@tab`
- **Content Area**: Shows content of active tab
- **Styling**: Match editor theme, clean borders

#### 3. Interactive Behavior

- **Click to Switch**: Click tab to show its content
- **Keyboard Navigation**: 
  - Arrow keys to move between tabs
  - Tab key to enter content area
- **Default Tab**: First tab active by default
- **Persistence**: Remember last active tab per block (optional)

#### 4. Content Handling

- **Full Markdown Support**: Each tab supports all Marco markdown
- **Code Blocks**: Syntax highlighting in tabs
- **Nested Blocks**: Support nested admonitions, tables, etc.
- **Mixed Content**: Text, code, images, lists all supported

#### 5. Advanced Features

- **Deep Linking**: URL parameters to open specific tabs
  - Example: `document.md#tab-block-2-python`
- **Copy Buttons**: Add copy button for code-heavy tabs
- **Expand All**: Option to view all tabs simultaneously
- **Tab Icons**: Optional icons based on language/platform
- **Search**: Search across all tabs in block

#### 6. Accessibility

- **ARIA Labels**: Proper roles for tabs and panels
- **Keyboard Support**: Full keyboard navigation
- **Screen Reader**: Announce tab changes
- **Focus Management**: Clear focus indicators

#### 7. Edge Cases

- **Empty Tabs**: Show placeholder for empty tab content
- **Single Tab**: Render without tab bar (just content)
- **No Content**: Default content before first `@tab`
- **Long Names**: Truncate or wrap long tab names

---

## Implementation Guidelines

### Security Considerations

1. **Executable Code**:
   - Always require user permission
   - Implement sandboxing
   - Validate script paths
   - Timeout long-running processes
   - Limit resource usage

2. **File References**:
   - Validate paths (prevent directory traversal)
   - Check file existence
   - Handle symbolic links carefully
   - Respect file permissions

3. **External Links**:
   - Sanitize user mention URLs
   - Warn before opening external links
   - Support HTTPS only for external resources

### Performance Optimization

1. **Lazy Loading**: Load tab content only when activated
2. **Caching**: Cache execution results and file lookups
3. **Debouncing**: Debounce real-time preview updates
4. **Virtual Scrolling**: For documents with many blocks

### User Experience

1. **Visual Feedback**: Show loading states, progress indicators
2. **Error Messages**: Clear, actionable error messages
3. **Undo/Redo**: Support undo for executable code changes
4. **Settings**: User preferences for auto-execution, themes, etc.

---

## Configuration Examples

### Settings File

```ron
// Example Marco configuration
(
    executable_code: (
        enabled: true,
        auto_run: false,
        timeout_seconds: 30,
        allowed_languages: ["bash", "python", "zsh"],
        sandbox: true,
    ),
    
    navigation: (
        track_history: true,
        max_history: 50,
        auto_validate_links: true,
    ),
    
    page_formatting: (
        default_format: "A4",
        show_page_breaks: true,
        print_margins: (20, 20, 20, 20), // top, right, bottom, left (mm)
    ),
    
    tabs: (
        remember_active: true,
        animation_speed: 200, // ms
        max_tabs_per_block: 10,
    ),
)
```

---

## Testing Checklist

- [ ] Document references resolve correctly
- [ ] Executable code runs with proper sandboxing
- [ ] Page formatting exports to PDF correctly
- [ ] Bookmarks navigate to precise line numbers
- [ ] User mentions generate correct profile URLs
- [ ] Tab blocks render and switch properly
- [ ] All features work with keyboard navigation
- [ ] Error handling is graceful and informative
- [ ] Security measures prevent malicious code execution
- [ ] Performance is acceptable with many blocks

---

## Future Enhancements

1. **Collaboration**: Real-time co-editing with user mentions
2. **Smart Bookmarks**: Auto-update on refactoring
3. **Tab Groups**: Nested tab blocks
4. **Code Execution**: Support more languages (Go, Ruby, etc.)
5. **Export**: Preserve all features in PDF/HTML export
6. **Templates**: Pre-defined tab block templates
7. **Analytics**: Track which tabs are most viewed

---

## Related Documentation

- [Marco Grammar Specification](../src/components/marco_engine/marco_grammar.pest)
- [User Guide](./user%20guide/user_guide.md)
- [Contributing Guidelines](../CONTRIBUTING.md)

---

**Last Updated**: October 2, 2025  
**Version**: 1.0  
**Status**: Draft
