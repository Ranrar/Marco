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

Edit `src/main.rs` in the `create_menu_actions()` function:

1. **Add language switching action**:
   ```rust
   // Add after existing language actions
   let set_language_it_action = gio::ActionEntry::builder("set_language_it")
       .activate({
           let editor = editor.clone();
           move |_app: &Application, _action, _param| {
               // Update settings and refresh UI
               crate::settings::SETTINGS.with(|settings| {
                   settings.borrow_mut().set_language("it");
               });
               // Refresh the menu bar to update checkmarks
               editor.rebuild_menu_bar();
           }
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