/// Additional SVG icon variants for layout and view controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutIcon {
    LayoutSwitcherButton,
    ViewOnly,
    EditorOnly,
    EditorAndViewSeparate,
    DualView,
}

/// Get the inline SVG string for a layout/view icon.
pub fn layout_icon_svg(icon: LayoutIcon) -> &'static str {
    match icon {
        LayoutIcon::LayoutSwitcherButton => r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M4 6a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v1a2 2 0 0 1 -2 2h-2a2 2 0 0 1 -2 -2l0 -1' /><path d='M4 15a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v3a2 2 0 0 1 -2 2h-2a2 2 0 0 1 -2 -2l0 -3' /><path d='M14 6a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-2a2 2 0 0 1 -2 -2l0 -12' /></svg>"#,
        LayoutIcon::ViewOnly => r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M3 7a3 3 0 0 1 3 -3h12a3 3 0 0 1 3 3v10a3 3 0 0 1 -3 3h-12a3 3 0 0 1 -3 -3l0 -10' /><path d='M7 10a2 2 0 1 0 4 0a2 2 0 1 0 -4 0' /><path d='M15 8l2 0' /><path d='M15 12l2 0' /><path d='M7 16l10 0' /></svg>"#,
        LayoutIcon::EditorOnly => r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M3 6a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2l0 -12' /><path d='M7 8h10' /><path d='M7 12h10' /><path d='M7 16h10' /></svg>"#,
        LayoutIcon::EditorAndViewSeparate => r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M3 17a1 1 0 0 1 1 -1h3a1 1 0 0 1 1 1v3a1 1 0 0 1 -1 1h-3a1 1 0 0 1 -1 -1l0 -3' /><path d='M4 12v-6a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-6' /><path d='M12 8h4v4' /><path d='M16 8l-5 5' /></svg>"#,
        LayoutIcon::DualView => r#"<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round' class='icon icon-tabler icons-tabler-outline icon-tabler-layout-columns'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M4 6a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2l0 -12' /><path d='M12 4l0 16' /></svg>"#,
    }
}
// Icon font support removed: we no longer bundle or use an icon font (IcoMoon).
// All UI icons should use inline SVGs via `layout_icon_svg` and `window_icon_svg`.

// Inline SVG definitions for window control icons. Colors can be applied by replacing
// `currentColor` in the returned string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowIcon {
    Close,
    Minimize,
    Maximize,
    Restore,
    Sun,
    Moon,
}

/// Get the inline SVG string for a window control icon with non-scaling strokes for crisp rendering.
pub fn window_icon_svg(icon: WindowIcon) -> &'static str {
    match icon {
        WindowIcon::Close => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M18 6l-12 12' vector-effect='non-scaling-stroke'/><path d='M6 6l12 12' vector-effect='non-scaling-stroke'/></svg>"#,
        WindowIcon::Minimize => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M5 12h14' vector-effect='non-scaling-stroke'/></svg>"#,
        WindowIcon::Maximize => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M5 7a2 2 0 0 1 2 -2h10a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2l0 -10' vector-effect='non-scaling-stroke'/></svg>"#,
        WindowIcon::Restore => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M8 6a2 2 0 0 1 2 -2h8a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2l0 -8' vector-effect='non-scaling-stroke'/><path d='M16 16v2a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2v-8a2 2 0 0 1 2 -2h2' vector-effect='non-scaling-stroke'/></svg>"#,
        WindowIcon::Sun => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><circle cx='12' cy='12' r='4' vector-effect='non-scaling-stroke'/><path d='M3 12h1m8 -9v1m8 8h1m-9 8v1m-6.4 -15.4l.7 .7m12.1 -.7l-.7 .7m0 11.4l.7 .7m-12.1 -.7l-.7 .7' vector-effect='non-scaling-stroke'/></svg>"#,
        WindowIcon::Moon => r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1' stroke-linecap='round' stroke-linejoin='round'><path stroke='none' d='M0 0h24v24H0z' fill='none'/><path d='M12 3c.132 0 .263 0 .393 0a7.5 7.5 0 0 0 7.92 12.446a9 9 0 1 1 -8.313 -12.454' vector-effect='non-scaling-stroke'/></svg>"#,
    }
}
