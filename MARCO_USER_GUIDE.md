# Marco - Markdown Editor User Guide

## Table of Contents
1. [Getting Started](#getting-started)
2. [Interface Overview](#interface-overview)
3. [File Operations](#file-operations)
4. [Text Editing](#text-editing)
5. [Markdown Formatting](#markdown-formatting)
6. [Advanced Features](#advanced-features)
7. [View Options](#view-options)
8. [Settings & Preferences](#settings--preferences)
9. [Keyboard Shortcuts](#keyboard-shortcuts)
10. [Tips & Tricks](#tips--tricks)

## Getting Started

Welcome to **Marco**, a modern and powerful Markdown editor designed for writers, developers, and content creators. Marco provides an intuitive interface with live preview, syntax highlighting, and advanced formatting capabilities.

### First Launch
When you first open Marco, you'll see:
- A clean editing interface with syntax highlighting
- A toolbar with commonly used formatting options
- A status bar showing document statistics
- Side-by-side or single-pane view options

### Latest Features
Marco now includes several advanced features:
- **Smart Code Block Dialog** - Search through 100+ programming languages with aliases
- **Enhanced Theme System** - Automatic light/dark mode detection with 4 built-in CSS themes
- **Professional Dialogs** - Modal interfaces with real-time validation and preview

## Interface Overview

### Main Components

#### Menu Bar
- **File**: New, Open, Save, Save As, Recent Files, Quit
- **Edit**: Undo, Redo, Cut, Copy, Paste, Find, Replace
- **Insert**: Headings, Lists, Links, Images, Code Blocks
- **Format**: Text styling, Code formatting, Tables
- **Advanced**: Special formatting, Text transformations
- **View**: Themes, View modes, Preferences
- **Help**: User guide, Shortcuts, About

#### Toolbar
Quick access buttons for:
- **Headings dropdown**: H1-H6 heading levels
- **Text formatting**: Bold (𝐁), Italic (𝐼), Code ({}), Strikethrough (S̶)
- **Lists**: Bullet lists (•), Numbered lists (1.), Blockquotes (❝)
- **Insert elements**: Links (🔗), Images (🖼), Horizontal rules (—)

#### Status Bar
Shows real-time information:
- Word count
- Character count
- Current cursor position (line and column)
- Document status

## File Operations

### Creating New Documents
- **Menu**: File → New (`Ctrl+N`)
- **Action**: Creates a blank document ready for editing

### Opening Files
- **Menu**: File → Open (`Ctrl+O`)
- **Action**: Browse and open existing Markdown files
- **Supported formats**: `.md`, `.markdown`, `.txt`

### Saving Documents
- **Save**: File → Save (`Ctrl+S`)
- **Save As**: File → Save As (`Ctrl+Shift+S`)
- **Auto-save**: Marco automatically tracks changes and prompts before closing unsaved documents

### Recent Files
Access recently opened documents through File → Recent Files for quick editing.

## Text Editing

### Basic Operations
- **Undo**: `Ctrl+Z` or Edit → Undo
- **Redo**: `Ctrl+Shift+Z` or Edit → Redo
- **Cut**: `Ctrl+X` or Edit → Cut
- **Copy**: `Ctrl+C` or Edit → Copy
- **Paste**: `Ctrl+V` or Edit → Paste

### Find and Replace
- **Find**: `Ctrl+F` or Edit → Find
  - Search for text in your document
  - Case-sensitive option available
  - Navigate through search results
- **Replace**: `Ctrl+H` or Edit → Replace
  - Find and replace text
  - Replace individual instances or all occurrences
  - Smart replacement preserves formatting context

### Dialog Interface

Marco uses **modal dialogs** for advanced features that require user input. These dialogs provide a professional, consistent experience:

#### Dialog Behavior
- **Modal Windows**: All dialogs open as modal overlays attached to the main window
- **Focus Management**: You must interact with the dialog before returning to the editor
- **Consistent Design**: All dialogs feature the same header style with a simple X close button
- **Input Validation**: Real-time validation with clear error messages and user feedback
- **Preview Support**: Many dialogs include live preview of your changes

#### Types of Dialogs
- **Content Input**: Link insertion, image properties, code language selection
- **Text Styling**: Color selection, HTML formatting options, text alignment
- **Advanced Media**: Enhanced image options, YouTube embedding, custom links
- **System Information**: Keyboard shortcuts, about information, emoji picker

#### Tips for Dialog Use
- Use **Tab** to navigate between input fields
- Press **Enter** to confirm changes (equivalent to clicking OK)
- Press **Escape** to cancel and close the dialog
- All changes are previewed before being applied to your document

## Markdown Formatting

### Headings
Create headings using the toolbar dropdown or keyboard shortcuts:
- **H1**: `Ctrl+1` or `# Heading 1`
- **H2**: `Ctrl+2` or `## Heading 2`
- **H3**: `Ctrl+3` or `### Heading 3`
- **H4**: `Ctrl+4` or `#### Heading 4`
- **H5**: `Ctrl+5` or `##### Heading 5`
- **H6**: `Ctrl+6` or `###### Heading 6`

### Text Formatting
- **Bold**: `Ctrl+B` or surround text with `**bold**`
- **Italic**: `Ctrl+I` or surround text with `*italic*`
- **Inline Code**: `Ctrl+` ` or surround with `` `code` ``
- **Strikethrough**: Use toolbar or surround with `~~text~~`

### Lists
- **Bullet Lists**: Click toolbar button or start line with `- ` or `* `
- **Numbered Lists**: Click toolbar button or start line with `1. `
- **Nested Lists**: Indent with spaces or tabs

### Links and Images
- **Links**: `Ctrl+K` or use format `[text](URL)`
- **Images**: Use toolbar or format `![alt text](image_URL)`

### Code Blocks
- **Inline Code**: Use backticks `` `code` ``
- **Fenced Code Blocks**: Use the smart search dialog or type manually
  ```markdown
  ```python
  def hello():
      print("Hello, World!")
  ```
  ```

#### Smart Language Selection
Marco now features an advanced language picker for fenced code blocks:
- **Access**: Format → Fenced code block... (`Ctrl+Shift+C`)
- **Smart Search**: Type to search among 100+ supported programming languages
- **Alias Support**: Use shortcuts like "js" for JavaScript, "py" for Python
- **Popular Languages**: Shows commonly used languages first
- **Real-time Filtering**: Instant results as you type

**Supported Languages Include**: Rust, JavaScript, TypeScript, Python, Java, C++, C#, Go, PHP, Ruby, HTML, CSS, SQL, Bash, and many more!

### Blockquotes
- Use toolbar button or start line with `> `
- Can be nested with multiple `>` characters

### Horizontal Rules
- Use toolbar button or type `---` on its own line

## Advanced Features

### Text Styling (Requires Text Selection)
Access through Advanced menu when text is selected:

#### Underline Text
- Select text → Advanced → Text Styling → Underline
- Wraps selected text with HTML `<u>` tags

#### Center Text
- Select text → Advanced → Text Styling → Center Text
- Centers text using HTML `<center>` tags

#### Colored Text
- Select text → Advanced → Text Styling → Colored Text
- Opens modal color picker dialog attached to the main window
- Provides live preview of color changes
- Applies HTML color styling

#### Indent Text
- Select text → Advanced → Text Styling → Indent Text
- Adds proper indentation to selected text

### Special Elements
- **Tables**: Insert → Table (creates basic table structure)
- **Task Lists**: Insert → Task List (creates checkable items)
- **Definition Lists**: Insert → Definition List (creates term/definition pairs)

### HTML Integration
- **HTML Entities**: Insert → HTML Entity (special characters)
- **Custom HTML**: Direct HTML input is supported

## View Options

### View Modes
Choose your preferred editing experience:
- **Editor Only**: Focus on writing without distractions
- **Preview Only**: View formatted output
- **Split View**: Side-by-side editing and preview

### Themes
Customize the appearance:
- **Light Theme**: Clean, bright interface
- **Dark Theme**: Easy on the eyes for long writing sessions
- **System Theme**: Follows your system preferences

### CSS Themes
Apply different styling to your preview:
- **Standard**: Clean, professional styling
- **Academic**: Academic paper formatting with serif fonts
- **GitHub**: GitHub-style rendering
- **Minimal**: Clean, distraction-free appearance
- Themes affect how your Markdown renders in preview mode
- Automatic theme integration with light/dark mode detection

## Settings & Preferences

Access preferences through View → Preferences:

### Language Settings
- Switch between supported languages (English, German, Spanish, French)
- Interface updates dynamically

### UI Theme
- Choose between Light, Dark, or System theme
- Applies to the entire application interface

### View Mode
- Set default view mode preference
- Choose split ratio for split view

### CSS Theme
- Select default CSS styling for preview
- Affects document rendering appearance

## Keyboard Shortcuts

### File Operations
- `Ctrl+N` - New document
- `Ctrl+O` - Open file
- `Ctrl+S` - Save
- `Ctrl+Shift+S` - Save As
- `Ctrl+Q` - Quit application

### Editing
- `Ctrl+Z` - Undo
- `Ctrl+Shift+Z` - Redo
- `Ctrl+X` - Cut
- `Ctrl+C` - Copy
- `Ctrl+V` - Paste
- `Ctrl+F` - Find
- `Ctrl+H` - Replace

### Formatting
- `Ctrl+B` - Bold
- `Ctrl+I` - Italic
- `Ctrl+K` - Insert Link
- `Ctrl+` ` - Inline Code
- `Ctrl+Shift+C` - Fenced Code Block (opens smart language picker)

### Lists and Structure
- `Ctrl+L` - Bullet List
- `Ctrl+Shift+L` - Numbered List
- `Ctrl+Q` - Blockquote

### Headings
- `Ctrl+1` through `Ctrl+6` - Insert heading levels

### Help
- `Ctrl+?` - Show keyboard shortcuts
- Access this guide through Help → Markdown Guide

## Tips & Tricks

### Productivity Tips
1. **Use keyboard shortcuts** for faster editing
2. **Split view** is great for real-time preview while writing
3. **Find and Replace** with case sensitivity for precise editing
4. **Recent Files** menu for quick access to your documents

### Formatting Best Practices
1. **Consistent heading hierarchy** improves document structure
2. **Use code blocks** for multi-line code instead of inline code
3. **Alt text for images** improves accessibility
4. **Proper link text** makes documents more readable

### Advanced Usage
1. **Select text first** before using Advanced text styling features
2. **HTML mixing** - You can use HTML tags within Markdown for advanced formatting
3. **Theme switching** to match your working environment
4. **Language switching** for international collaboration
5. **Smart code search** - Use aliases like "js", "py", "rs" in the fenced code dialog
6. **100+ programming languages** supported with syntax highlighting

### Troubleshooting
- **Unsaved changes warning**: Marco will prompt you before closing unsaved documents
- **Text selection required**: Some advanced features require text selection first
- **Syntax highlighting**: Automatic highlighting helps identify formatting issues
- **Preview updates**: Split view shows changes in real-time
- **White preview background**: If preview appears white, try switching CSS themes in View menu
- **Code language not found**: Use the smart search in fenced code dialog - try aliases like "js", "py", "rs"
- **Theme not loading**: Restart application if theme changes don't apply immediately

## Getting Help

- **User Guide**: Help → Markdown Guide (this document)
- **Keyboard Shortcuts**: Help → Shortcuts (`Ctrl+?`)
- **About**: Help → About (version and license information)

---

**Marco** is designed to make Markdown editing efficient and enjoyable. Whether you're writing documentation, blog posts, or technical content, Marco provides the tools you need for professional results.

*Happy writing! 📝*
