# Themes

- **Themes and assets**: `marco-shared/src/assets/themes/`.
- The application uses a `ThemeManager` to map editor schemes to preview theme modes. Changing themes from the settings dialog calls back into functions returned by `create_editor_with_preview_and_buffer`.

Both HTML preview themes and editor style schemes are **auto-discovered** by scanning their asset folder — there is no registry/list in code to update by hand.

## HTML preview themes (Markdown rendering)

1. Add a token-only CSS file under `marco-shared/src/assets/themes/html_viever/` (see any existing theme, e.g. `neutral.css`, for the full set of `--*` tokens to define, plus `.theme-light` / `.theme-dark` overrides).
2. Declare theme metadata in the `:root` block using the `--theme-*` custom properties supported by `marco-core` 1.2.0+'s `parse_theme_metadata` (`marco-core/src/render/theme_meta.rs`):
   ```css
   :root {
     --theme-name: 'My Theme';
     --theme-author: 'Your Name';
     --theme-license: 'MIT';
     --theme-version: '1.0.0';
     --theme-description: 'One-line description of the look and intent';
     /* ...colour/font tokens... */
   }
   ```
   `--theme-name` is what the Settings and Export dialog dropdowns display; if omitted, the picker falls back to a title-cased version of the filename.
3. That's it — `list_html_view_themes()` (`marco-shared/src/logic/loaders/theme_loader.rs`) picks up any `*.css` file in the folder automatically, in both the Settings appearance picker and the Export dialog.
4. If the theme is a port of an existing named colour scheme (Nord, Gruvbox, Solarized, etc.), credit the original author/project and license in `--theme-author` / `--theme-license`, not yourself — reserve your own name for genuinely original Marco themes.

## Editor style schemes (SourceView5 syntax highlighting)

1. Add a GtkSourceView style-scheme XML file under `marco-shared/src/assets/themes/editor/`.
2. `list_editor_style_schemes()` (same file) scans this folder automatically and prefers the scheme's own `name`/`_name` XML attribute for the display label, falling back to the filename.
