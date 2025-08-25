/// Helper to generate webkit scrollbar CSS given thumb/track colors
pub fn webkit_scrollbar_css(thumb: &str, track: &str) -> String {
    format!(
        r#"
        /* Match editor scrollbar styling for WebView */
        ::-webkit-scrollbar {{ width: 12px; height: 12px; background: {track}; }}
        ::-webkit-scrollbar-track {{ background: {track}; }}
    ::-webkit-scrollbar-thumb {{ background: {thumb}; border-radius: 0px; }}
    ::-webkit-scrollbar-thumb:hover {{ background: {thumb}; opacity: 0.9; }}
        "#,
        thumb = thumb,
        track = track
    )
}

/// Generate GTK CSS rules to style application scrollbars to match the editor
/// theme. Targets scrolled windows with the `.editor-scrolled` selector so the
/// same look applies to both the editor and source preview ScrolledWindow.
pub fn gtk_scrollbar_css(thumb: &str, track: &str) -> String {
    format!(
        r#"
        /* GTK scrollbar styling for in-app scrolled windows */
        /* Remove borders/spacing and make slider fill the trough to avoid
           visible gaps between slider and track (as shown in the screenshot). */
        .editor-scrolled scrollbar,
        .source-preview scrollbar {{
            -gtk-icon-transform: none;
            min-width: 12px;
            min-height: 12px;
            background: transparent;
            border: none;
            box-shadow: none;
            padding: 0;
            margin: 0;
        }}
        .editor-scrolled scrollbar trough,
        .source-preview scrollbar trough {{
            background-color: {track};
            border: none;
            box-shadow: none;
            min-width: 12px;
            min-height: 12px;
            padding: 0;
            margin: 0;
        }}
        .editor-scrolled scrollbar slider,
        .source-preview scrollbar slider {{
            background-color: {thumb};
            border-radius: 0px;
            border: none;
            box-shadow: none;
            min-width: 12px;
            min-height: 12px;
            margin: 0;
            padding: 0;
        }}
        "#,
        thumb = thumb,
        track = track
    )
}
