# Adding Languages to Marco

This guide covers how to add two types of languages to Marco:
1. **UI Languages** (Internationalization) - Adding support for new interface languages
2. **Programming Languages** - Adding syntax highlighting for new programming languages

## Adding UI Languages (Internationalization)

Marco uses a YAML-based translation system that allows easy addition of new interface languages.

### Step 1: Create Translation Files

1. **Create language directory**:
   ```bash
   mkdir locales/[language_code]/
   ```
   Use ISO 639-1 language codes (e.g., `it` for Italian, `pt` for Portuguese, `nl` for Dutch)

2. **Copy base translation file**:
   ```bash
   cp locales/en/main.yml locales/[language_code]/main.yml
   ```

3. **Translate all text values** in the new `main.yml` file:
   ```yaml
   [language_code]:
     app:
       title: "Marco - Editor de Markdown"  # Translate this
     menu:
       file: "Archivo"                      # Translate this
       new: "Nuevo"                         # Translate this
       # ... translate all other values
   ```

### Step 2: Update Code Files

#### A. Add Language to Localization System

Edit `src/localization.rs`:

1. **Add language to `get_available_locales()` function**:
   ```rust
   pub fn get_available_locales() -> Vec<(String, &'static str)> {
       vec![
           ("en".to_string(), "English"),
           ("es".to_string(), "Español"),
           ("fr".to_string(), "Français"),
           ("de".to_string(), "Deutsch"),
           ("it".to_string(), "Italiano"),    // Add your language here
           // Add more languages as needed
       ]
   }
   ```

#### B. Add Menu Action for Language

Edit `src/menu.rs` in the `create_menu_actions()` function:

1. **Add language switching action**:
   ```rust
   // Add after existing language actions
   let set_language_it_action = gio::ActionEntry::builder("set_language_it")
       .activate(|_app: &Application, _action, _param| {
           localization::set_locale("it");
           println!("Language changed to Italian");
       })
       .build();
   ```

2. **Add action to the actions array**:
   ```rust
   app.add_action_entries([
       // ... existing actions ...
       set_language_en_action, set_language_es_action, set_language_fr_action, 
       set_language_de_action, set_language_it_action,  // Add your action here
   ]);
   ```

### Step 3: Translation Keys Reference

Here are the key sections you need to translate in `main.yml`:

#### Application and Menus
```yaml
[language_code]:
  app:
    title: "Application title"
  menu:
    file: "File menu"
    edit: "Edit menu"
    insert: "Insert menu"
    format: "Format menu"
    view: "View menu"
    help: "Help menu"
    language: "Language submenu"
```

#### File Operations
```yaml
  file:
    new: "New document"
    open: "Open file"
    save: "Save"
    save_as: "Save As"
    quit: "Quit application"
```

#### Insert Menu Items
```yaml
  insert:
    heading1: "Heading 1"
    bold: "Bold text"
    italic: "Italic text"
    # ... and many more formatting options
```

#### Dialog Text
```yaml
  dialog:
    ok: "OK button"
    cancel: "Cancel button"
    insert: "Insert button"
```

#### Keyboard Shortcuts
```yaml
  shortcuts:
    title: "Keyboard Shortcuts dialog title"
    ctrl_b: "Ctrl+B"
    bold_text: "Bold text description"
    # ... all shortcut descriptions
```

### Step 4: Testing Your Translation

1. **Build and run Marco**:
   ```bash
   cargo run
   ```

2. **Test language switching**:
   - Go to **View → Language**
   - Select your new language
   - Verify all UI elements are translated

3. **Check for missing translations**:
   - If text appears in English instead of your language, check for missing or incorrect keys in your `main.yml`

---

## Adding Programming Languages (Code Syntax Highlighting)

Marco supports syntax highlighting for code blocks. You can add new programming languages to the fenced code block system.

### Step 1: Update Code Languages Definition

Edit `src/code_languages.rs`:

1. **Add language to `CodeLanguageManager::new()`**:
   ```rust
   pub fn new() -> Self {
       let mut manager = Self {
           languages: HashMap::new(),
       };
       
       // ... existing language definitions ...
       
       // Add your new language
       manager.add_language(CodeLanguage {
           name: "kotlin".to_string(),
           display_name: "Kotlin".to_string(),
           file_extensions: vec!["kt".to_string(), "kts".to_string()],
           comment_style: CommentStyle::DoubleSlash,
           keywords: vec![
               // Add Kotlin keywords
               "fun", "val", "var", "class", "object", "interface",
               "if", "else", "when", "for", "while", "do",
               // ... more keywords
           ].iter().map(|s| s.to_string()).collect(),
           color_scheme: ColorScheme {
               keyword: "#CF8E6D".to_string(),      // Orange for keywords
               string: "#6AAB73".to_string(),       // Green for strings
               comment: "#7A7E85".to_string(),      // Gray for comments
               number: "#2AACB8".to_string(),       // Cyan for numbers
               operator: "#89DDFF".to_string(),     // Light blue for operators
           },
       });
       
       manager
   }
   ```

### Step 2: Add Menu Actions

Edit `src/menu.rs`:

1. **Add menu item to fenced code submenu** in `create_menu_bar()`:
   ```rust
   // In the fenced_code_menu section, add your language
   fenced_code_menu.append(Some("Kotlin"), Some("app.insert_fenced_kotlin"));
   ```

2. **Add action handler** in `create_menu_actions()`:
   ```rust
   let insert_fenced_kotlin_action = gio::ActionEntry::builder("insert_fenced_kotlin")
       .activate({
           let editor = editor.clone();
           move |_app: &Application, _action, _param| {
               editor.insert_fenced_code_with_language("kotlin");
           }
       })
       .build();
   ```

3. **Add to actions array**:
   ```rust
   app.add_action_entries([
       // ... existing actions ...
       insert_fenced_kotlin_action,  // Add your action here
   ]);
   ```

### Step 3: Language Definition Details

#### Comment Styles
Choose the appropriate comment style for your language:
```rust
pub enum CommentStyle {
    DoubleSlash,    // // comment (C++, Java, JavaScript, etc.)
    Hash,           // # comment (Python, Ruby, Shell, etc.)
    Dash,           // -- comment (SQL, Haskell, etc.)
    Semicolon,      // ; comment (Assembly, Lisp, etc.)
    Percent,        // % comment (LaTeX, MATLAB, etc.)
}
```

#### Color Schemes
Define colors for different syntax elements:
```rust
color_scheme: ColorScheme {
    keyword: "#CF8E6D".to_string(),      // Language keywords (if, for, class)
    string: "#6AAB73".to_string(),       // String literals
    comment: "#7A7E85".to_string(),      // Comments
    number: "#2AACB8".to_string(),       // Numeric literals
    operator: "#89DDFF".to_string(),     // Operators (+, -, =, etc.)
},
```

Use hex color codes. Here are some popular color scheme inspirations:
- **Material Theme**: Dark background with vibrant colors
- **Monokai**: Dark theme with warm colors
- **Solarized**: Balanced light/dark themes
- **GitHub**: Light theme similar to GitHub's syntax highlighting

### Step 4: Adding Advanced Language Features

For more sophisticated language support, you can extend the `CodeLanguage` struct:

1. **Add regex patterns for advanced syntax**:
   ```rust
   pub struct CodeLanguage {
       // ... existing fields ...
       pub syntax_patterns: Vec<SyntaxPattern>,
   }
   
   pub struct SyntaxPattern {
       pub name: String,
       pub regex: String,
       pub color: String,
   }
   ```

2. **Example advanced patterns**:
   ```rust
   syntax_patterns: vec![
       SyntaxPattern {
           name: "function_call".to_string(),
           regex: r"\b\w+(?=\s*\()".to_string(),  // Function calls
           color: "#DCDCAA".to_string(),           // Yellow for functions
       },
       SyntaxPattern {
           name: "type_annotation".to_string(),
           regex: r":\s*\w+".to_string(),          // Type annotations
           color: "#4EC9B0".to_string(),           // Teal for types
       },
   ],
   ```

### Step 5: Testing Your Programming Language

1. **Build and run Marco**:
   ```bash
   cargo run
   ```

2. **Test the new language**:
   - Go to **Format → Fenced Code Block**
   - Select your new language from the submenu
   - Or use the dialog and type your language name
   - Verify the code block is inserted with correct language tag

3. **Test syntax highlighting**:
   - Insert a fenced code block with your language
   - Add some sample code
   - Check that keywords, strings, and comments are highlighted correctly

---

## Language File Structure Reference

### UI Language File (`locales/[code]/main.yml`)
```yaml
[language_code]:
  app:
    title: "Marco - Markdown Composer"
  menu:
    file: "File"
    edit: "Edit"
    insert: "Insert"
    format: "Format"
    view: "View"
    help: "Help"
    language: "Language"
    new: "New"
    open: "Open"
    save: "Save"
    save_as: "Save As"
    quit: "Quit"
    # ... many more keys (see existing files for complete list)
```

### Programming Language Integration Points

1. **`src/code_languages.rs`** - Main language definitions
2. **`src/menu.rs`** - Menu integration and actions
3. **`src/editor.rs`** - Editor integration (usually no changes needed)
4. **`src/syntax_extended.rs`** - Advanced syntax highlighting (if needed)

---

## Best Practices

### For UI Languages:
1. **Keep translations consistent** - Use the same terminology throughout
2. **Consider text length** - Some languages are longer/shorter than English
3. **Test with your target audience** - Native speakers can catch nuances
4. **Follow platform conventions** - Use standard terms for your platform

### For Programming Languages:
1. **Include popular keywords** - Focus on the most commonly used language features
2. **Choose readable colors** - Ensure good contrast and accessibility
3. **Test with real code** - Use actual code samples to verify highlighting
4. **Document language features** - Update this guide when adding new capabilities

### For Both:
1. **Test thoroughly** - Verify everything works before committing
2. **Update documentation** - Keep this guide updated with new languages
3. **Follow naming conventions** - Use standard language codes and names
4. **Consider maintenance** - Languages you add will need updates over time

---

## Troubleshooting

### UI Language Issues:
- **Text not changing**: Check language code matches exactly in all files
- **Missing translations**: Verify all keys exist in your YAML file
- **YAML parsing errors**: Check syntax, indentation, and special characters

### Programming Language Issues:
- **Language not appearing in menu**: Check action is added to actions array
- **Syntax highlighting not working**: Verify color scheme format (hex colors)
- **Keywords not highlighted**: Check keyword list and regex patterns

### General Issues:
- **Compilation errors**: Run `cargo check` to see detailed error messages
- **Runtime errors**: Check console output for panic messages
- **UI not updating**: Try restarting the application

---

## Contributing Your Languages

Once you've successfully added a language, consider contributing it back to the Marco project:

1. **Fork the repository**
2. **Create a feature branch** for your language addition
3. **Test thoroughly** on different systems
4. **Submit a pull request** with:
   - Complete translation files
   - Updated code files
   - Test results
   - Documentation updates

Your contributions help make Marco accessible to more users worldwide!
