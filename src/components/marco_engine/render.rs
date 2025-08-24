// Renderer status and guidance
// - Implemented: headings, paragraphs, inline formatting (strong/emphasis/inlineCode/delete),
//   links, images, lists, tables (basic), code blocks, blockquote, thematic break, math
//   (inline/block), emoji nodes, and basic footnote collection.
// - Placeholders: figure, video, toc (table-of-contents), admonition/callout, and
//   definitionList currently render via fallbacks; see `render_node` for simple placeholder
//   behavior. These nodes include TODO notes where a richer HTML output can be added.
// - Emoji handling: the renderer accepts both `name` and `value` attributes on `emoji`
//   nodes. It supports shortcode expansion (e.g. `:smile:`) via the `emojis` crate and
//   falls back to escaping and inline shortcode replacement when necessary.
// - Extensions: `MarkdownOptions` controls feature toggles (table, autolink,
//   strikethrough, tasklist, footnotes). The options-aware renderer is implemented
//   via `render_with_options` and `render_node_with_opts` to allow testing and
//   compatibility with callers that expect configurable behavior.
// - Tests: Unit tests demonstrate core rendering and placeholder behavior. When
//   adding richer renderers for placeholder nodes, update or extend tests under
//   this module to ensure compatibility and expected HTML output.

use crate::components::marco_engine::parser::{parse_markdown, Node};
use emojis;
use once_cell::sync::Lazy;
use regex::Regex;

// Regex for URL detection used by autolink feature
static URL_RE: Lazy<Regex> = Lazy::new(|| {
    // match http(s)://... or www.... (stop at whitespace or <)
    Regex::new(r"(https?://[^\s<]+|www\.[^\s<]+)").unwrap()
});

pub struct MarkdownRenderer {
    // For now a simple implementation, can be extended to use the dynamic registry
}

impl MarkdownRenderer {
    fn render_with_options(node: &Node, opts: &MarkdownOptions) -> String {
        // allow passing options down to recursive rendering
        match node.node_type.as_str() {
            "root" => {
                // If footnotes extension is enabled, collect definitions first
                if opts.extension.footnotes {
                    let mut defs: Vec<(String, String)> = Vec::new();
                    Self::collect_footnote_definitions(node, opts, &mut defs);

                    let body = node
                        .children
                        .iter()
                        .map(|c| Self::render_with_options(c, opts))
                        .collect::<Vec<_>>()
                        .join("");

                    if defs.is_empty() {
                        return body;
                    }

                    // build footnotes section
                    let mut footnotes_html =
                        String::from("<section class=\"footnotes\">\n<hr>\n<ol>\n");
                    for (id, def_html) in defs {
                        footnotes_html.push_str(&format!(
                            "<li id=\"fn:{}\">{} <a href=\"#fnref:{}\">↩</a></li>\n",
                            html_escape(&id),
                            def_html,
                            html_escape(&id)
                        ));
                    }
                    footnotes_html.push_str("</ol>\n</section>");

                    return body + &footnotes_html;
                }

                node.children
                    .iter()
                    .map(|c| Self::render_with_options(c, opts))
                    .collect::<Vec<_>>()
                    .join("")
            }

            // delegate most cases to existing render_node variant by constructing a
            // temporary Node wrapper that uses Self::render_node for backwards compat
            _ => Self::render_node_with_opts(node, opts),
        }
    }

    // New core renderer that receives options and implements feature toggles
    fn render_node_with_opts(node: &Node, opts: &MarkdownOptions) -> String {
        match node.node_type.as_str() {
            "text" => {
                if let Some(value) = node.attributes.get("value") {
                    let escaped = html_escape_with_emoji(value);
                    if opts.extension.autolink {
                        // replace URLs with anchor tags
                        URL_RE
                            .replace_all(&escaped, |caps: &regex::Captures| {
                                let m = &caps[0];
                                let href = if m.starts_with("www.") {
                                    format!("http://{}", m)
                                } else {
                                    m.to_string()
                                };
                                format!("<a href=\"{}\">{}</a>", html_escape(&href), html_escape(m))
                            })
                            .to_string()
                    } else {
                        escaped
                    }
                } else {
                    String::new()
                }
            }

            // Tables: only render HTML table when table extension is enabled
            "table" => {
                if !opts.extension.table {
                    // render as plain text fallback
                    return node
                        .children
                        .iter()
                        .map(|c| Self::render_with_options(c, opts))
                        .collect::<String>();
                }
                let content = node
                    .children
                    .iter()
                    .map(|c| Self::render_with_options(c, opts))
                    .collect::<String>();
                format!("<table>{}</table>", content)
            }

            "tableRow" => {
                let content = node
                    .children
                    .iter()
                    .map(|c| Self::render_with_options(c, opts))
                    .collect::<String>();
                format!("<tr>{}</tr>", content)
            }

            "tableCell" => {
                let content = node
                    .children
                    .iter()
                    .map(|c| Self::render_with_options(c, opts))
                    .collect::<String>();
                format!("<td>{}</td>", content)
            }

            // Links: autolink extension - simple URL detection in text nodes
            "link" => {
                let url = node.attributes.get("url").map_or("#", |v| v);
                let content = node
                    .children
                    .iter()
                    .map(|c| Self::render_with_options(c, opts))
                    .collect::<String>();
                format!("<a href=\"{}\">{}</a>", html_escape(url), content)
            }

            // image remains unchanged
            "image" => {
                let url = node.attributes.get("url").map_or("", |v| v);
                let alt = node.attributes.get("alt").map_or("", |v| v);
                format!(
                    "<img src=\"{}\" alt=\"{}\">",
                    html_escape(url),
                    html_escape(alt)
                )
            }

            // strikethrough: only render <del> when enabled
            "delete" => {
                if opts.extension.strikethrough {
                    let children_content = node
                        .children
                        .iter()
                        .map(|c| Self::render_with_options(c, opts))
                        .collect::<String>();
                    let content = if children_content.is_empty() {
                        node.attributes
                            .get("value")
                            .map_or(String::new(), |v| html_escape_with_emoji(v))
                    } else {
                        children_content
                    };
                    format!("<del>{}</del>", content)
                } else {
                    // render inner content as plain text
                    node.children
                        .iter()
                        .map(|c| Self::render_with_options(c, opts))
                        .collect::<String>()
                }
            }

            // tasklist: if enabled, render checkboxes
            "listItem" => {
                let rendered_children = node
                    .children
                    .iter()
                    .map(|c| Self::render_with_options(c, opts))
                    .collect::<Vec<_>>();
                let mut content = rendered_children.join("");

                if opts.extension.tasklist {
                    if let Some(checked_attr) = node.attributes.get("checked") {
                        let checked = checked_attr == "true";
                        if checked {
                            return format!(
                                "<li><input type=\"checkbox\" checked disabled> {}</li>",
                                content
                            );
                        } else {
                            return format!(
                                "<li><input type=\"checkbox\" disabled> {}</li>",
                                content
                            );
                        }
                    }
                    if let Some(first) = rendered_children.first() {
                        let raw = first.trim_start();
                        if raw.starts_with("[ ] ") || raw.starts_with("[ ]") {
                            content = raw.replacen("[ ]", "", 1).trim_start().to_string()
                                + &rendered_children[1..].join("");
                            return format!(
                                "<li><input type=\"checkbox\" disabled> {}</li>",
                                content
                            );
                        } else if raw.to_lowercase().starts_with("[x] ")
                            || raw.to_lowercase().starts_with("[x]")
                        {
                            content = raw.replacen("[x]", "", 1).trim_start().to_string()
                                + &rendered_children[1..].join("");
                            return format!(
                                "<li><input type=\"checkbox\" checked disabled> {}</li>",
                                content
                            );
                        }
                    }
                }

                // default list item rendering
                format!("<li>{}</li>", content)
            }

            // footnotes: simple support - when enabled, collect definitions and append
            // a footnotes section when rendering root. Here we render footnoteReference
            // and footnoteDefinition inline; the root-level collection is handled below.
            "footnote" | "footnoteReference" => {
                // render as superscript reference linking to footnote list
                if let Some(id) = node.attributes.get("identifier") {
                    format!(
                        "<sup class=\"footnote-ref\"><a id=\"fnref:{}\" href=\"#fn:{}\">{}</a></sup>",
                        html_escape(id),
                        html_escape(id),
                        html_escape(id)
                    )
                } else {
                    node.children
                        .iter()
                        .map(|c| Self::render_with_options(c, opts))
                        .collect::<String>()
                }
            }

            // default fallback: delegate to existing render_node for other types
            _ => Self::render_node(node),
        }
    }

    fn render_node(node: &Node) -> String {
        match node.node_type.as_str() {
            "root" => node
                .children
                .iter()
                .map(Self::render_node)
                .collect::<Vec<_>>()
                .join(""),

            "text" => {
                if let Some(value) = node.attributes.get("value") {
                    html_escape_with_emoji(value)
                } else {
                    String::new()
                }
            }

            "heading" => {
                let depth = node
                    .attributes
                    .get("depth")
                    .and_then(|d| d.parse::<u32>().ok())
                    .unwrap_or(1);
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<h{}>{}</h{}>", depth, content, depth)
            }

            "paragraph" => {
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<p>{}</p>", content)
            }

            "strong" => {
                // First try to get content from children (nested nodes)
                let children_content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();

                // If no children content, try to get from attributes (direct text)
                let content = if children_content.is_empty() {
                    node.attributes
                        .get("value")
                        .map_or(String::new(), |v| html_escape_with_emoji(v))
                } else {
                    children_content
                };

                format!("<strong>{}</strong>", content)
            }

            "emphasis" => {
                // First try to get content from children (nested nodes)
                let children_content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();

                // If no children content, try to get from attributes (direct text)
                let content = if children_content.is_empty() {
                    node.attributes
                        .get("value")
                        .map_or(String::new(), |v| html_escape_with_emoji(v))
                } else {
                    children_content
                };

                format!("<em>{}</em>", content)
            }

            "inlineCode" => {
                if let Some(value) = node.attributes.get("value") {
                    format!("<code>{}</code>", html_escape(value))
                } else {
                    String::new()
                }
            }

            "delete" => {
                // First try to get content from children (nested nodes)
                let children_content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();

                // If no children content, try to get from attributes (direct text)
                let content = if children_content.is_empty() {
                    node.attributes
                        .get("value")
                        .map_or(String::new(), |v| html_escape_with_emoji(v))
                } else {
                    children_content
                };

                format!("<del>{}</del>", content)
            }

            "blockquote" => {
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<blockquote>{}</blockquote>", content)
            }

            "codeBlock" => {
                let code = node.attributes.get("value").map_or("", |v| v);
                let language = node.attributes.get("language");

                if let Some(lang) = language {
                    format!(
                        "<pre><code class=\"language-{}\">{}</code></pre>",
                        lang,
                        html_escape(code)
                    )
                } else {
                    format!("<pre><code>{}</code></pre>", html_escape(code))
                }
            }

            "thematicBreak" => "<hr>".to_string(),

            "list" => {
                let ordered = node
                    .attributes
                    .get("ordered")
                    .map(|o| o == "true")
                    .unwrap_or(false);
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();

                if ordered {
                    format!("<ol>{}</ol>", content)
                } else {
                    format!("<ul>{}</ul>", content)
                }
            }

            "listItem" => {
                // Render children to find task markers or content
                let rendered_children = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<Vec<_>>();
                let mut content = rendered_children.join("");

                // If the parser set a 'checked' attribute, honor it first
                if let Some(checked_attr) = node.attributes.get("checked") {
                    let checked = checked_attr == "true";
                    if checked {
                        return format!(
                            "<li><input type=\"checkbox\" checked disabled> {}</li>",
                            content
                        );
                    } else {
                        return format!("<li><input type=\"checkbox\" disabled> {}</li>", content);
                    }
                }

                // Otherwise, detect leading task marker in the raw text (e.g., "[ ] " or "[x] ")
                // Look at the first child rendered string
                if let Some(first) = rendered_children.first() {
                    let raw = first.trim_start();
                    if raw.starts_with("[ ] ") || raw.starts_with("[ ]") {
                        // remove the marker from content
                        content = raw.replacen("[ ]", "", 1).trim_start().to_string()
                            + &rendered_children[1..].join("");
                        return format!("<li><input type=\"checkbox\" disabled> {}</li>", content);
                    } else if raw.to_lowercase().starts_with("[x] ")
                        || raw.to_lowercase().starts_with("[x]")
                    {
                        content = raw.replacen("[x]", "", 1).trim_start().to_string()
                            + &rendered_children[1..].join("");
                        return format!(
                            "<li><input type=\"checkbox\" checked disabled> {}</li>",
                            content
                        );
                    }
                }

                // Default list item rendering
                format!("<li>{}</li>", content)
            }

            "table" => {
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<table>{}</table>", content)
            }

            "tableRow" => {
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<tr>{}</tr>", content)
            }

            "tableCell" => {
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<td>{}</td>", content)
            }

            "link" => {
                let url = node.attributes.get("url").map_or("#", |v| v);
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<a href=\"{}\">{}</a>", html_escape(url), content)
            }

            "image" => {
                let url = node.attributes.get("url").map_or("", |v| v);
                let alt = node.attributes.get("alt").map_or("", |v| v);
                format!(
                    "<img src=\"{}\" alt=\"{}\">",
                    html_escape(url),
                    html_escape(alt)
                )
            }

            // Accept multiple naming variants for breaks (parser may produce different names)
            "hardBreak" | "hard_break" | "break" | "br" => "<br>".to_string(),

            // Soft break variants map to a single newline or space depending on rendering policy
            "softBreak" | "soft_break" | "soft-break" => "\n".to_string(),

            // Accept HTML node variants produced by parser
            "html" | "htmlBlock" | "html_inline" | "htmlInline" | "htmlTag" | "html_tag" => {
                // Pass through HTML directly (be cautious about security in real applications)
                node.attributes.get("value").map_or("", |v| v).to_string()
            }

            // Frontmatter / yaml / toml nodes - skip in HTML output
            "yaml" | "frontmatter" | "yaml_frontmatter" | "toml_frontmatter" | "toml" => {
                String::new()
            }

            // Math node variants
            "inlineMath" | "mathInline" | "math_inline" => {
                if let Some(value) = node.attributes.get("value") {
                    format!(
                        "<span class=\"math-inline\">\\({}\\)</span>",
                        html_escape(value)
                    )
                } else {
                    String::new()
                }
            }

            "math" | "mathBlock" | "math_block" => {
                if let Some(value) = node.attributes.get("value") {
                    format!(
                        "<div class=\"math-block\">\\[{}\\]</div>",
                        html_escape(value)
                    )
                } else {
                    String::new()
                }
            }

            "emoji" => {
                // Render emoji nodes. Parser typically uses `name` attribute, fixtures sometimes
                // use `value`. Accept both. Also support shortcodes inside `value` (e.g. ":smile:")
                let key_opt = node
                    .attributes
                    .get("name")
                    .or_else(|| node.attributes.get("value"));

                if let Some(key) = key_opt {
                    // If the value looks like a :shortcode:, try shortcode replacement first
                    // Support both plain shortcodes ("smile") and wrapped ":smile:" in `value`.
                    let trimmed = key.trim();

                    // If value is wrapped as :shortcode:, try direct shortcode lookup without colons
                    if trimmed.starts_with(':') && trimmed.ends_with(':') {
                        let inner = trimmed.trim_matches(':');
                        if let Some(e) = emojis::get_by_shortcode(inner) {
                            return e.as_str().to_string();
                        }
                    }

                    // Try shortcode lookup (without colons)
                    if let Some(e) = emojis::get_by_shortcode(trimmed) {
                        e.as_str().to_string()
                    // Try direct name/unicode lookup
                    } else if let Some(e) = emojis::get(trimmed) {
                        e.as_str().to_string()
                    } else {
                        // As a last resort run shortcode replacement inside the text and escape
                        html_escape_with_emoji(key)
                    }
                } else {
                    String::new()
                }
            }

            // Figure element: try to extract image src/alt/title and optional caption
            "figure" => {
                // Look for an image child or attributes on the figure node
                let mut src: Option<String> = None;
                let mut alt: Option<String> = None;
                let mut title: Option<String> = None;

                for child in &node.children {
                    if child.node_type == "image" {
                        if let Some(u) = child.attributes.get("url") {
                            src = Some(u.clone());
                        }
                        if let Some(a) = child.attributes.get("alt") {
                            alt = Some(a.clone());
                        }
                        if let Some(t) = child.attributes.get("title") {
                            title = Some(t.clone());
                        }
                    }
                }

                // fallback to attributes on the figure node
                if src.is_none() {
                    if let Some(u) = node.attributes.get("src") {
                        src = Some(u.clone());
                    } else if let Some(u) = node.attributes.get("url") {
                        src = Some(u.clone());
                    }
                }
                if alt.is_none() {
                    if let Some(a) = node.attributes.get("alt") {
                        alt = Some(a.clone());
                    }
                }
                if title.is_none() {
                    if let Some(t) = node.attributes.get("title") {
                        title = Some(t.clone());
                    }
                }

                // Build caption from remaining text children (non-image)
                let caption_parts: Vec<String> = node
                    .children
                    .iter()
                    .filter(|c| c.node_type != "image")
                    .map(Self::render_node)
                    .filter(|s| !s.trim().is_empty())
                    .collect();
                let caption = if !caption_parts.is_empty() {
                    Some(caption_parts.join(" "))
                } else {
                    node.attributes.get("caption").cloned()
                };

                if let Some(u) = src {
                    let alt_attr = alt.unwrap_or_default();
                    let title_attr = title.unwrap_or_default();
                    let mut figure_html = format!(
                        "<figure><img src=\"{}\" alt=\"{}\"",
                        html_escape(&u),
                        html_escape(&alt_attr)
                    );
                    if !title_attr.is_empty() {
                        figure_html.push_str(&format!(" title=\"{}\"", html_escape(&title_attr)));
                    }
                    figure_html.push_str(" />");
                    if let Some(c) = caption {
                        figure_html.push_str(&format!("<figcaption>{}</figcaption>", c));
                    }
                    figure_html.push_str("</figure>");
                    figure_html
                } else {
                    // fallback: render children inline inside a figure
                    let content = node
                        .children
                        .iter()
                        .map(Self::render_node)
                        .collect::<String>();
                    format!("<figure>{}</figure>", content)
                }
            }

            // Placeholder: video embedding (could be <video> or iframe depending on source)
            "video" => {
                // TODO: implement video rendering (attrs: src, poster, controls)
                if let Some(src) = node.attributes.get("src") {
                    format!(
                        "<video controls src=\"{}\">Your browser does not support the video tag.</video>",
                        html_escape(src)
                    )
                } else {
                    node.children
                        .iter()
                        .map(Self::render_node)
                        .collect::<String>()
                }
            }

            // Placeholder: admonition / callout block (note/warning/tip)
            "admonition" | "callout" => {
                // TODO: render admonitions with role and title
                let role = node
                    .attributes
                    .get("role")
                    .map(|r| r.as_str())
                    .unwrap_or("note");
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!(
                    "<div class=\"admonition {}\">{}</div>",
                    html_escape(role),
                    content
                )
            }

            // Placeholder: table of contents node
            "toc" | "tableOfContents" => {
                // TODO: generate a nested TOC from children or headings
                node.children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>()
            }

            // Placeholder: definition list
            "definitionList" | "defList" => {
                // TODO: render definition lists (<dl><dt><dd>) properly
                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                format!("<dl>{}</dl>", content)
            }

            // Unknown node types - render children
            _ => node
                .children
                .iter()
                .map(Self::render_node)
                .collect::<String>(),
        }
    }

    // Walk the tree and collect footnote definitions (identifier -> rendered HTML)
    fn collect_footnote_definitions(
        node: &Node,
        opts: &MarkdownOptions,
        out: &mut Vec<(String, String)>,
    ) {
        // If this node is a footnoteDefinition, render its children into HTML and store
        if node.node_type == "footnoteDefinition" || node.node_type == "footnote_def" {
            if let Some(id) = node.attributes.get("identifier") {
                let content = node
                    .children
                    .iter()
                    .map(|c| Self::render_with_options(c, opts))
                    .collect::<String>();
                out.push((id.clone(), content));
            }
        }

        for child in &node.children {
            Self::collect_footnote_definitions(child, opts, out);
        }
    }
}

/// Simple HTML escape function
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

// Replace :shortcode: patterns with actual emoji characters using the `emojis` crate.
fn replace_shortcodes_with_emoji(input: &str) -> String {
    // regex for :shortcode:
    static SHORTCODE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r":([a-z0-9_+-]+):").unwrap());

    SHORTCODE_RE
        .replace_all(input, |caps: &regex::Captures| {
            let key = &caps[1];
            if let Some(emoji) = emojis::get_by_shortcode(key) {
                emoji.as_str().to_string()
            } else {
                caps[0].to_string()
            }
        })
        .to_string()
}

// HTML-escape and then run emoji shortcode replacement on the result's text nodes.
fn html_escape_with_emoji(input: &str) -> String {
    let escaped = html_escape(input);
    replace_shortcodes_with_emoji(&escaped)
}

// Note: `build_renderer_compatibility_node` was removed and its construction
// is now inlined into the test below to avoid an unused-public warning while
// keeping the test coverage.

/// Compatibility function that matches comrak's signature  
pub fn markdown_to_html(input: &str, _options: &MarkdownOptions) -> String {
    match parse_markdown(input) {
        Ok(ast) => MarkdownRenderer::render_with_options(&ast, _options),
        Err(err) => {
            eprintln!("Markdown parsing error: {}", err);
            format!(
                "<p>Error parsing markdown: {}</p>",
                html_escape(&err.to_string())
            )
        }
    }
}

/// Compatibility struct to replace ComrakOptions
#[derive(Default)]
pub struct MarkdownOptions {
    pub extension: MarkdownExtensions,
}

#[derive(Default)]
pub struct MarkdownExtensions {
    pub table: bool,
    pub autolink: bool,
    pub strikethrough: bool,
    pub tasklist: bool,
    pub footnotes: bool,
    pub tagfilter: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let input = "# Hello World\n\nThis is a **bold** text.\n";
        let opts = MarkdownOptions::default();
        let output = markdown_to_html(input, &opts);
        println!("Input: {:?}", input);
        println!("Output: {:?}", output);
        assert!(output.contains("<h1>Hello World</h1>"));
        assert!(output.contains("<strong>bold</strong>"));
    }

    #[test]
    fn test_code_block() {
        let input = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n";
        let opts = MarkdownOptions::default();
        let output = markdown_to_html(input, &opts);
        assert!(output.contains("<pre><code class=\"language-rust\">"));
        assert!(output.contains("fn main()"));
    }

    #[test]
    fn test_list() {
        let input = "- Item 1\n- Item 2\n- [ ] Task\n- [x] Done\n";
        let opts = MarkdownOptions::default();
        let output = markdown_to_html(input, &opts);
        assert!(output.contains("<ul>"));
        assert!(output.contains("<li>"));
        assert!(output.contains("checkbox"));
    }

    #[test]
    fn test_placeholder_nodes_from_ast() {
        // Figure
        let mut fig = crate::components::marco_engine::parser::Node::new("figure");
        let mut img = crate::components::marco_engine::parser::Node::new("image");
        img.add_attribute("url", "/assets/images/albuquerque.jpg");
        img.add_attribute("alt", "Albuquerque");
        fig.add_child(img);
        fig.add_child(crate::components::marco_engine::parser::Node::text_node(
            "A single track trail outside of Albuquerque, New Mexico.",
        ));
        let out_fig = MarkdownRenderer::render_node(&fig);
        assert!(out_fig.contains("<figure>"));
        assert!(out_fig.contains("/assets/images/albuquerque.jpg"));
        // caption should be present as figcaption
        assert!(out_fig.contains("figcaption") || out_fig.contains("A single track trail"));

        // Video
        let mut vid = crate::components::marco_engine::parser::Node::new("video");
        vid.add_attribute("src", "https://www.youtube.com/watch?v=YOUTUBE-ID");
        let out_vid = MarkdownRenderer::render_node(&vid);
        assert!(out_vid.contains("<video") || out_vid.contains("youtube"));

        // Admonition
        let mut adm = crate::components::marco_engine::parser::Node::new("admonition");
        adm.add_attribute("role", "warning");
        adm.add_child(crate::components::marco_engine::parser::Node::text_node(
            "Warning: Something important.",
        ));
        let out_adm = MarkdownRenderer::render_node(&adm);
        assert!(out_adm.contains("admonition"));
        assert!(out_adm.contains("warning"));

        // TOC
        let mut toc = crate::components::marco_engine::parser::Node::new("toc");
        let mut heading = crate::components::marco_engine::parser::Node::new("heading");
        heading.add_attribute("depth", "4");
        heading.add_child(crate::components::marco_engine::parser::Node::text_node(
            "Table of Contents",
        ));
        toc.add_child(heading);
        let out_toc = MarkdownRenderer::render_node(&toc);
        assert!(out_toc.contains("Table of Contents"));

        // Definition list
        let mut dlist = crate::components::marco_engine::parser::Node::new("definitionList");
        let mut term = crate::components::marco_engine::parser::Node::new("term");
        term.add_child(crate::components::marco_engine::parser::Node::text_node(
            "Term",
        ));
        let mut desc = crate::components::marco_engine::parser::Node::new("description");
        let mut para = crate::components::marco_engine::parser::Node::new("paragraph");
        para.add_child(crate::components::marco_engine::parser::Node::text_node(
            "Definition text.",
        ));
        desc.add_child(para);
        dlist.add_child(term);
        dlist.add_child(desc);
        let out_dl = MarkdownRenderer::render_node(&dlist);
        assert!(out_dl.contains("<dl>") || out_dl.contains("Definition text."));

        // Emoji node (value shortcode)
        let mut emoji = crate::components::marco_engine::parser::Node::new("emoji");
        emoji.add_attribute("value", ":smile:");
        let out_emoji = MarkdownRenderer::render_node(&emoji);
        // Expect shortcode to be expanded to a unicode char (no colon left)
        assert!(!out_emoji.contains(':'));
        assert!(!out_emoji.is_empty());
    }

    #[test]
    fn test_build_renderer_compatibility_node() {
        let mut node = crate::components::marco_engine::parser::Node::new("compatibility_notes");
        let notes = "Renderer gaps / compatibility notes:\n- figure: placeholder\n- video: placeholder\n- toc: placeholder\n- admonition: placeholder\n- definitionList: placeholder\n- emoji: accepts `name` and `value`, supporting shortcodes";
        node.add_child(crate::components::marco_engine::parser::Node::text_node(
            notes,
        ));
        let out = MarkdownRenderer::render_node(&node);
        assert!(out.contains("Renderer gaps"));
    }
}
