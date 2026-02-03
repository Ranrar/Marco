/// Format HTML string with indentation for debugging/display
/// Adds newlines between tags and indents nested elements
pub fn pretty_print_html(input: &str) -> String {
    let with_newlines = input.replace(
        "><", ">
<",
    );
    let mut out = String::new();
    let mut indent: usize = 0;
    for raw_line in with_newlines.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("</") {
            indent = indent.saturating_sub(1);
        }
        out.push_str(&"    ".repeat(indent));
        out.push_str(line);
        out.push('\n');
        let has_closing_after = line.find("</").is_some_and(|i| i > 0);
        if line.starts_with('<')
            && !line.starts_with("</")
            && !line.ends_with("/>")
            && !line.starts_with("<!")
            && !has_closing_after
        {
            indent += 1;
        }
    }
    out
}

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

/// Generate CSS for indentation levels
/// Creates classes .marco-indent-level-1, .marco-indent-level-2, etc. with appropriate margins
pub fn indentation_css() -> String {
    let mut css = String::new();
    css.push_str("/* Marco Indentation Levels */\n");

    // Generate CSS for indentation levels 1-10 (should be enough for most use cases)
    for level in 1..=10 {
        let indent_size = level * 2; // 2em per indentation level
        css.push_str(&format!(
            ".marco-indent-level-{} {{\n    margin-left: {}em;\n}}\n",
            level, indent_size
        ));
    }

    css
}

/// Generate CSS for list item indentation with proper task list support
pub fn list_indentation_css() -> String {
    let mut css = String::new();
    css.push_str("/* Marco List Indentation with Task List Support */\n");

    for level in 1..=10 {
        let indent_size = level * 2; // 2em per indentation level
        css.push_str(&format!(
            "li.marco-indent-level-{} {{\n    margin-left: {}em;\n}}\n",
            level, indent_size
        ));

        // Special handling for task list items
        css.push_str(&format!(
            "li.marco-task-item.marco-indent-level-{} {{\n    margin-left: {}em;\n}}\n",
            level, indent_size
        ));
    }

    css
}

/// Generate CSS for code block indentation
pub fn code_indentation_css() -> String {
    let mut css = String::new();
    css.push_str("/* Marco Code Block Indentation */\n");

    for level in 1..=10 {
        let indent_size = level * 2; // 2em per indentation level
        css.push_str(&format!(
            "pre.marco-indent-level-{} {{\n    margin-left: {}em;\n}}\n",
            level, indent_size
        ));
    }

    css
}

/// Generate complete indentation CSS for all elements
pub fn complete_indentation_css() -> String {
    let mut css = String::new();
    css.push_str(&indentation_css());
    css.push('\n');
    css.push_str(&list_indentation_css());
    css.push('\n');
    css.push_str(&code_indentation_css());
    css
}
