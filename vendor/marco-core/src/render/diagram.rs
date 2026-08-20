use std::borrow::Cow;
use std::panic;

use mermaid_rs_renderer::{
    render_with_options as mermaid_render_with_opts, RenderOptions as MermaidRenderOptions,
    Theme as MermaidTheme,
};

const MERMAID_MAX_CHARS: usize = 100_000;

/// Create Mermaid theme configuration from the active preview mode hint.
pub(crate) fn create_mermaid_theme(theme_hint: &str) -> MermaidTheme {
    let normalized = theme_hint.trim().to_ascii_lowercase();
    let is_dark = normalized == "dark"
        || normalized.ends_with("-dark")
        || normalized.contains("theme-dark")
        || normalized.contains("dark");

    if is_dark {
        let mut theme = MermaidTheme::modern();

        theme.background = "transparent".to_string();
        theme.text_color = "#f0f6fc".to_string();
        theme.primary_text_color = "#f0f6fc".to_string();
        theme.pie_title_text_color = "#f0f6fc".to_string();
        theme.pie_section_text_color = "#f0f6fc".to_string();
        theme.pie_legend_text_color = "#f0f6fc".to_string();
        theme.git_commit_label_color = "#f0f6fc".to_string();
        theme.git_tag_label_color = "#f0f6fc".to_string();

        theme.primary_color = "#161b22".to_string();
        theme.secondary_color = "#0d1117".to_string();
        theme.tertiary_color = "#1f2937".to_string();

        theme.primary_border_color = "#30363d".to_string();
        theme.line_color = "#8b949e".to_string();
        theme.edge_label_background = "rgba(13, 17, 23, 0.92)".to_string();

        theme.cluster_background = "#0d1117".to_string();
        theme.cluster_border = "#30363d".to_string();

        theme.sequence_actor_fill = "#161b22".to_string();
        theme.sequence_actor_border = "#30363d".to_string();
        theme.sequence_actor_line = "#8b949e".to_string();
        theme.sequence_note_fill = "#1f2937".to_string();
        theme.sequence_note_border = "#30363d".to_string();
        theme.sequence_activation_fill = "#21262d".to_string();
        theme.sequence_activation_border = "#30363d".to_string();

        theme.git_commit_label_background = "rgba(13, 17, 23, 0.9)".to_string();
        theme.git_tag_label_background = "rgba(13, 17, 23, 0.9)".to_string();
        theme.git_tag_label_border = "#30363d".to_string();
        theme.pie_colors = [
            "#58a6ff", "#3fb950", "#d29922", "#f778ba", "#a371f7", "#39c5cf", "#ff7b72", "#9e6a03",
            "#7ee787", "#79c0ff", "#ffa657", "#c9d1d9",
        ]
        .map(|c| c.to_string());
        theme.pie_stroke_color = "#30363d".to_string();
        theme.pie_outer_stroke_color = "#30363d".to_string();
        theme.pie_stroke_width = 1.5;
        theme.pie_outer_stroke_width = 1.5;
        theme.pie_opacity = 0.95;

        theme
    } else {
        let mut theme = MermaidTheme::mermaid_default();

        theme.background = "transparent".to_string();
        theme.text_color = "#24292f".to_string();
        theme.primary_text_color = "#24292f".to_string();
        theme.pie_title_text_color = "#24292f".to_string();
        theme.pie_section_text_color = "#24292f".to_string();
        theme.pie_legend_text_color = "#24292f".to_string();
        theme.git_commit_label_color = "#24292f".to_string();
        theme.git_tag_label_color = "#24292f".to_string();

        theme.primary_color = "#f6f8fa".to_string();
        theme.secondary_color = "#ffffff".to_string();
        theme.tertiary_color = "#f6f8fa".to_string();
        theme.primary_border_color = "#d0d7de".to_string();
        theme.line_color = "#57606a".to_string();
        theme.edge_label_background = "rgba(255, 255, 255, 0.92)".to_string();
        theme.cluster_background = "#f6f8fa".to_string();
        theme.cluster_border = "#d0d7de".to_string();

        theme.sequence_actor_fill = "#f6f8fa".to_string();
        theme.sequence_actor_border = "#d0d7de".to_string();
        theme.sequence_actor_line = "#57606a".to_string();
        theme.sequence_note_fill = "#ffffff".to_string();
        theme.sequence_note_border = "#d0d7de".to_string();
        theme.sequence_activation_fill = "#f6f8fa".to_string();
        theme.sequence_activation_border = "#d0d7de".to_string();

        theme.git_commit_label_background = "rgba(255, 255, 255, 0.92)".to_string();
        theme.git_tag_label_background = "rgba(255, 255, 255, 0.92)".to_string();
        theme.git_tag_label_border = "#d0d7de".to_string();
        theme.pie_colors = [
            "#1f6feb", "#2da44e", "#bf8700", "#cf4f8b", "#8250df", "#0a7d89", "#cf222e", "#9a6700",
            "#1a7f37", "#0550ae", "#bc4c00", "#57606a",
        ]
        .map(|c| c.to_string());
        theme.pie_stroke_color = "#57606a".to_string();
        theme.pie_outer_stroke_color = "#57606a".to_string();
        theme.pie_stroke_width = 1.25;
        theme.pie_outer_stroke_width = 1.25;
        theme.pie_opacity = 0.92;

        theme
    }
}

/// Normalize and validate Mermaid source before rendering.
fn normalize_mermaid_source(diagram: &str) -> Result<Cow<'_, str>, Box<dyn std::error::Error>> {
    let trimmed = diagram.trim();
    if trimmed.is_empty() {
        return Err("Empty Mermaid diagram".into());
    }

    if trimmed.len() > MERMAID_MAX_CHARS {
        return Err(format!(
            "Mermaid diagram too large ({} chars > max {})",
            trimmed.len(),
            MERMAID_MAX_CHARS
        )
        .into());
    }

    if let Some(first_line_end) = trimmed.find('\n') {
        let first_line = trimmed[..first_line_end].trim();
        if first_line.starts_with("```") || first_line.starts_with("~~~") {
            let fence = if first_line.starts_with("~~~") {
                "~~~"
            } else {
                "```"
            };
            let after_fence = first_line[fence.len()..].trim().to_ascii_lowercase();
            if after_fence.starts_with("mermaid") {
                let body = &trimmed[first_line_end + 1..];
                if let Some(last_nl) = body.rfind('\n') {
                    let last_line = body[last_nl + 1..].trim();
                    if last_line == fence {
                        let inner = body[..last_nl].trim();
                        if inner.is_empty() {
                            return Err("Empty Mermaid diagram".into());
                        }
                        if inner.len() > MERMAID_MAX_CHARS {
                            return Err(format!(
                                "Mermaid diagram too large ({} chars > max {})",
                                inner.len(),
                                MERMAID_MAX_CHARS
                            )
                            .into());
                        }
                        return Ok(Cow::Owned(inner.to_string()));
                    }
                }
            }
        }
    }

    Ok(Cow::Borrowed(trimmed))
}

/// Render Mermaid diagram to SVG.
pub fn render_mermaid_diagram(
    diagram: &str,
    theme_hint: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let normalized = normalize_mermaid_source(diagram)?;

    let diagram_owned = normalized.into_owned();
    let theme = create_mermaid_theme(theme_hint);
    let mut options = MermaidRenderOptions::modern();
    options.theme = theme;

    let result = panic::catch_unwind(|| mermaid_render_with_opts(&diagram_owned, options));

    match result {
        Ok(Ok(svg)) => Ok(svg),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Err("Mermaid rendering panicked (likely invalid diagram syntax)".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_create_mermaid_theme_light_dark() {
        let light = create_mermaid_theme("light");
        let dark = create_mermaid_theme("dark");

        assert_eq!(light.background, "transparent");
        assert_eq!(dark.background, "transparent");
        assert_eq!(light.text_color, "#24292f");
        assert_eq!(dark.text_color, "#f0f6fc");
        assert_eq!(light.pie_colors[0], "#1f6feb");
        assert_eq!(dark.pie_colors[0], "#58a6ff");
        assert_ne!(light.line_color, dark.line_color);
    }

    #[test]
    fn smoke_test_render_mermaid_diagram_mode_sensitive() {
        let diagram = "flowchart TD\nA[Start] --> B[End]";

        let svg_light = render_mermaid_diagram(diagram, "light").expect("light mermaid render");
        let svg_dark = render_mermaid_diagram(diagram, "dark").expect("dark mermaid render");

        assert!(svg_light.contains("<svg"));
        assert!(svg_dark.contains("<svg"));
        assert!(
            svg_light.contains("transparent")
                || svg_light.contains("rgba(255, 255, 255, 0.92)")
                || svg_light.contains("#24292f")
        );
        assert!(
            svg_dark.contains("transparent")
                || svg_dark.contains("rgba(34, 40, 49, 0.85)")
                || svg_dark.contains("#e6edf3")
                || svg_dark.contains("#f0f6fc")
        );
    }

    #[test]
    fn smoke_test_render_mermaid_diagram_rejects_empty() {
        let err = render_mermaid_diagram("   \n\t", "dark").expect_err("expected empty rejection");
        assert!(err.to_string().contains("Empty Mermaid diagram"));
    }

    #[test]
    fn smoke_test_render_mermaid_diagram_rejects_too_large() {
        let oversized = "A".repeat(MERMAID_MAX_CHARS + 1);
        let err = render_mermaid_diagram(&oversized, "light").expect_err("expected size rejection");
        assert!(err.to_string().contains("too large"));
    }

    #[test]
    fn smoke_test_render_mermaid_diagram_accepts_fenced_input() {
        let fenced = "```mermaid\nflowchart LR\nA[Start] --> B[End]\n```";
        let svg = render_mermaid_diagram(fenced, "light").expect("fenced mermaid should render");
        assert!(svg.contains("<svg"));
    }
}
