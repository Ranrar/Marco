# Unified Theme System

The Marco markdown editor now features a comprehensive theme system that applies to both the preview pane and the source editor, providing a consistent visual experience throughout the application.

## Components

### 1. Preview Pane Themes
The preview uses a unified CSS file (`css/standard.css`) that automatically adapts to both light and dark themes without requiring separate CSS files.

### 2. Source Editor Themes  
The source editor (where you type markdown) now also responds to theme changes, using appropriate SourceView syntax highlighting schemes for light and dark modes.

## How It Works

### Preview Pane (CSS-based)
The CSS file defines color variables for all theme-dependent properties:
```css
:root {
  --text-color: #1a1a1a;      /* Main text color */
  --bg-color: #fff;           /* Background color */
  --bg-code: #f5f5f5;         /* Code background */
  /* ... more variables ... */
}
```

**Automatic Dark Mode Detection:**
```css
@media (prefers-color-scheme: dark) {
  :root {
    --text-color: #e1e1e1;
    --bg-color: #1a1a1a;
    /* ... dark theme variables ... */
  }
}
```

**Manual Theme Override:**
- `.theme-light` - Forces light theme
- `.theme-dark` - Forces dark theme
- No class - Uses system preference (`prefers-color-scheme`)

### Source Editor (SourceView-based)
The source editor uses GTK SourceView syntax highlighting schemes:

**Light Theme Schemes (in priority order):**
- Adwaita (default GTK light scheme)
- classic, tango, kate, solarized-light

**Dark Theme Schemes (in priority order):**
- Adwaita-dark (default GTK dark scheme)  
- oblivion, cobalt, monokai, solarized-dark

**Automatic Fallback:**
If preferred schemes aren't available, the system automatically tries alternative schemes and falls back to the basic Adwaita scheme.

## Theme Switching

Users can switch themes via the **View > Theme** menu:
1. **Light** - Forces light theme for both editor and preview
2. **Dark** - Forces dark theme for both editor and preview  
3. **System** - Automatically detects OS theme preference

**Instant Updates:**
Both the source editor and preview update immediately when themes are changed, providing seamless visual feedback.

## Implementation Details

### Rust Code Structure
```rust
// Theme manager handles theme detection and switching
let theme_manager = ThemeManager::new();

// Editor receives theme manager and applies to both views
editor.set_theme_manager(theme_manager);

// Both HTML preview and source editor update on theme changes
editor.refresh_html_view(); // Updates both preview and editor
```

### Editor Theme Application
1. **HTML Preview**: Adds CSS class to body element (`theme-light`, `theme-dark`, or none)
2. **Source Editor**: Selects appropriate SourceView syntax highlighting scheme
3. **Fallback Handling**: Graceful degradation if preferred schemes aren't available

## Benefits

✅ **Consistent Experience** - Both editor and preview use coordinated themes
✅ **Automatic Detection** - Respects OS dark/light preference by default
✅ **Manual Override** - Users can force specific themes
✅ **Smooth Transitions** - Elegant theme switching animations in preview
✅ **Robust Fallbacks** - Works even if some style schemes are missing
✅ **Single Maintenance** - Unified CSS system reduces code duplication

## Theme Variables (Preview)

| Variable | Light Theme | Dark Theme | Usage |
|----------|-------------|------------|-------|
| `--text-color` | `#1a1a1a` | `#e1e1e1` | Main text |
| `--bg-color` | `#fff` | `#1a1a1a` | Background |
| `--bg-code` | `#f5f5f5` | `#2a2a2a` | Code background |
| `--border-color` | `#ddd` | `#333` | Borders |
| `--link-color` | `#0066cc` | `#4da6ff` | Links |

## Style Schemes (Editor)

| Theme | Primary Scheme | Fallback Schemes |
|-------|----------------|------------------|
| Light | Adwaita | classic, tango, kate, solarized-light |
| Dark | Adwaita-dark | oblivion, cobalt, monokai, solarized-dark |

This unified theme system provides a modern, maintainable, and user-friendly theming experience that automatically adapts to user preferences while offering manual control when needed.
