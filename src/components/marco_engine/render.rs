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

// Helper: extract YouTube video id from common URL forms and build auxiliary URLs
fn youtube_id_from_url(url: &str) -> Option<String> {
    // Try common patterns without heavy deps
    // youtu.be/ID
    if let Some(rest) = url.strip_prefix("https://youtu.be/") {
        return rest.split(&['?', '&'][..]).next().map(|s| s.to_string());
    }
    if let Some(rest) = url.strip_prefix("http://youtu.be/") {
        return rest.split(&['?', '&'][..]).next().map(|s| s.to_string());
    }
    // youtube.com/watch?v=ID
    if url.contains("youtube.com/watch") || url.contains("youtube.com/") {
        if let Some(qpos) = url.find('?') {
            let query = &url[qpos + 1..];
            for pair in query.split('&') {
                let mut parts = pair.splitn(2, '=');
                if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                    if k == "v" {
                        return Some(v.to_string());
                    }
                }
            }
        }
    }
    // embed/ID
    if let Some(pos) = url.find("/embed/") {
        return url[pos + 7..]
            .split(&['?', '&'][..])
            .next()
            .map(|s| s.to_string());
    }
    // img.youtube.com/vi/ID/... (thumbnail link)
    if let Some(pos) = url.find("img.youtube.com/vi/") {
        let tail = &url[pos + "img.youtube.com/vi/".len()..];
        return tail
            .split(&['/', '?', '&'][..])
            .next()
            .map(|s| s.to_string());
    }
    None
}

fn youtube_watch_url_for_id(id: &str) -> String {
    format!("https://www.youtube.com/watch?v={}", id)
}

fn youtube_thumbnail_url_for_id(id: &str) -> String {
    format!("https://img.youtube.com/vi/{}/0.jpg", id)
}

// Build an embeddable YouTube iframe for a video id and title (no autoplay)
fn build_youtube_iframe(id: &str, title: &str) -> String {
    // include controls and disable related videos (rel=0); no autoplay param
    // Keep an iframe fallback for non-JS environments and tests, but also
    // include a small initialization script that loads the YouTube IFrame
    // API and creates a YT.Player instance targeting a sanitized container id.
    fn sanitize_for_id(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect()
    }

    let safe = sanitize_for_id(id);
    let embed_src = format!(
        "https://www.youtube.com/embed/{}?controls=1&rel=0",
        html_escape(id)
    );

    // The div with id `yt-player-{safe}` will be used by the IFrame API to
    // instantiate a player. We keep an iframe inside as fallback for tests and
    // non-JS clients.
    let container_start = format!(
            "<div class=\"yt-embed\" style=\"position:relative;padding-bottom:56.25%;height:0;overflow:hidden;max-width:100%;\">\n  <div id=\"yt-player-{safe}\" class=\"yt-player\" data-ytid=\"{}\" title=\"{}\" style=\"position:absolute;top:0;left:0;width:100%;height:100%;\">\n",
            html_escape(id),
            html_escape(title)
        );

    let iframe_html = format!(
            "  <iframe src=\"{}\" title=\"{}\" frameborder=\"0\" allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture\" allowfullscreen style=\"width:100%;height:100%;border:0;\"></iframe>\n",
            embed_src,
            html_escape(title)
        );

    // Script: ensure iframe has an origin param appended at runtime (helps when referer is absent)
    let script = "<script>(function(){\n  try {\n    // If an iframe exists, append an origin query param so YouTube can identify the embedder\n    var playerContainer = document.getElementById('%ID%');\n    if (playerContainer) {\n      var ifr = playerContainer.querySelector('iframe');\n      if (ifr && ifr.src) {\n        try {\n          var sep = ifr.src.indexOf('?') !== -1 ? '&' : '?';\n          var origin = (typeof location !== 'undefined' && location.origin) ? location.origin : '';\n          if (origin) { ifr.src = ifr.src + sep + 'origin=' + encodeURIComponent(origin); }\n        } catch (e) { /* swallow */ }\n      }\n    }\n  } catch (e) { /* ignore */ }\n  if (!window.YT) {\n    var tag = document.createElement('script');\n    tag.src = 'https://www.youtube.com/iframe_api';\n    var firstScript = document.getElementsByTagName('script')[0];\n    firstScript.parentNode.insertBefore(tag, firstScript);\n  }\n  function createPlayer() { try { if (typeof YT === 'object' && YT && YT.Player) { new YT.Player('%ID%', { videoId: '%VID%', playerVars: { controls: 1, rel: 0 } }); } } catch (e) { console && console.error && console.error(e); } }\n  if (window.YT && YT.Player) { createPlayer(); } else { var prev = window.onYouTubeIframeAPIReady; window.onYouTubeIframeAPIReady = function() { if (prev) try { prev(); } catch(e){} createPlayer(); }; }\n})();</script>";

    // replace placeholders
    let script_inst = script
        .replace("%ID%", &format!("yt-player-{}", safe))
        .replace("%VID%", id);

    format!(
        "{}{}  </div>\n</div>\n{}",
        container_start, iframe_html, script_inst
    )
}

// Small helper: render an <a><img></a> thumbnail link for a YouTube id and optional alt
fn render_youtube_thumbnail_link(id: &str, alt: &str, href_override: Option<&str>) -> String {
    let href = if let Some(h) = href_override {
        h.to_string()
    } else {
        youtube_watch_url_for_id(id)
    };
    let alt_escaped = html_escape(alt);
    let thumb = youtube_thumbnail_url_for_id(id);
    format!(
            "<div class=\"yt-embed\" style=\"position:relative;padding-bottom:56.25%;height:0;overflow:hidden;max-width:100%;\">\n  <a href=\"{}\" style=\"position:absolute;top:0;left:0;width:100%;height:100%;display:block;\">\n    <img src=\"{}\" alt=\"{}\" style=\"width:100%;height:100%;object-fit:cover;border:0;\">\n  </a>\n</div>",
            html_escape(&href),
            html_escape(&thumb),
            alt_escaped
        )
}

// Crude helper to extract alt text from a text node that starts with a markdown image
// shorthand like "![alt](url)" or "![alt]". Returns the alt or default.
fn extract_alt_from_shorthand(text: &str) -> String {
    let trimmed = text.trim_start();
    if let Some(start) = trimmed.find("![") {
        let rest = &trimmed[start + 2..];
        if let Some(end) = rest.find(']') {
            return rest[..end].to_string();
        } else if let Some(paren) = rest.find('(') {
            return rest[..paren].to_string();
        } else {
            return rest.to_string();
        }
    }
    String::from("YouTube video")
}

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

                // If the link wraps a single image child, handle YouTube thumbnail patterns
                if node.children.len() == 1 && node.children[0].node_type == "image" {
                    let img = &node.children[0];
                    let img_url_opt = img.attributes.get("url");

                    // Try to get youtube id from the link first (preferred)
                    if let Some(link_id) = youtube_id_from_url(url) {
                        let title = img
                            .attributes
                            .get("alt")
                            .map(|s| s.as_str())
                            .unwrap_or("YouTube video");
                        return build_youtube_iframe(&link_id, title);
                    }

                    // Otherwise, if the image is a YouTube thumbnail or youtu link, render a
                    // clickable thumbnail that links out to the watch page for that id.
                    if let Some(img_url) = img_url_opt {
                        if let Some(img_id) = youtube_id_from_url(img_url) {
                            let alt = img.attributes.get("alt").map(|s| s.as_str()).unwrap_or("");
                            return render_youtube_thumbnail_link(&img_id, alt, None);
                        }
                    }
                }

                // Handle malformed/missing image AST cases where the link child is a
                // text node containing an image-like shorthand (e.g. starts with "![alt]").
                // If the link URL is a YouTube URL, prefer embedding the iframe using
                // the link URL and extract an alt/title from the shorthand when possible.
                if node.children.len() == 1 && node.children[0].node_type == "text" {
                    if let Some(text_val) = node.children[0].attributes.get("value") {
                        let trimmed = text_val.trim_start();
                        if trimmed.starts_with("![") {
                            if let Some(link_id) = youtube_id_from_url(url) {
                                // crude alt extraction: between ![ and ], or until ( if no ]
                                let mut alt = "YouTube video".to_string();
                                if let Some(start) = trimmed.find("![") {
                                    let rest = &trimmed[start + 2..];
                                    if let Some(end) = rest.find(']') {
                                        alt = rest[..end].to_string();
                                    } else if let Some(paren) = rest.find('(') {
                                        alt = rest[..paren].to_string();
                                    } else {
                                        alt = rest.to_string();
                                    }
                                }
                                let title = alt.trim();
                                return build_youtube_iframe(&link_id, title);
                            }
                        }
                    }
                }

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
                // If the image url itself is a YouTube link (e.g., markdown like
                // `![alt](https://youtu.be/ID)` or direct youtu.be in image src),
                // prefer embedding the YouTube iframe so single-image video links
                // produce the expected player rather than a bare link.
                if let Some(id) = youtube_id_from_url(url) {
                    let title = if !alt.is_empty() {
                        alt
                    } else {
                        "YouTube video"
                    };
                    return build_youtube_iframe(&id, title);
                }

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

                if node.children.len() == 1 && node.children[0].node_type == "image" {
                    let img = &node.children[0];
                    if let Some(img_url) = img.attributes.get("url") {
                        if let Some(id) = youtube_id_from_url(img_url) {
                            let href = if let Some(link_id) = youtube_id_from_url(url) {
                                youtube_watch_url_for_id(&link_id)
                            } else {
                                url.to_string()
                            };
                            let alt = img.attributes.get("alt").map(|s| s.as_str()).unwrap_or("");
                            return render_youtube_thumbnail_link(&id, alt, Some(&href));
                        }
                    }
                    if let Some(img_url) = img.attributes.get("url") {
                        if let Some(id) = youtube_id_from_url(img_url) {
                            let alt = img.attributes.get("alt").map(|s| s.as_str()).unwrap_or("");
                            return render_youtube_thumbnail_link(&id, alt, None);
                        }
                    }
                }

                let content = node
                    .children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>();
                // Handle malformed shorthand where the child is a text node starting
                // with "!["; prefer embedding if the link URL is YouTube.
                if node.children.len() == 1 && node.children[0].node_type == "text" {
                    if let Some(text_val) = node.children[0].attributes.get("value") {
                        let trimmed = text_val.trim_start();
                        if trimmed.starts_with("![") {
                            if let Some(link_id) = youtube_id_from_url(url) {
                                let alt = extract_alt_from_shorthand(trimmed);
                                return build_youtube_iframe(&link_id, alt.trim());
                            }
                        }
                    }
                }

                format!("<a href=\"{}\">{}</a>", html_escape(url), content)
            }

            "image" => {
                let url = node.attributes.get("url").map_or("", |v| v);
                let alt = node.attributes.get("alt").map_or("", |v| v);

                // If an image's src is actually a YouTube URL (e.g. `![alt](https://youtu.be/ID)`),
                // embed the YouTube iframe instead of outputting a bare <img> tag.
                if let Some(id) = youtube_id_from_url(url) {
                    let title = if !alt.is_empty() {
                        alt
                    } else {
                        "YouTube video"
                    };
                    return build_youtube_iframe(&id, title);
                }

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

            // Video embedding: prefer YouTube iframe when `url` or `src` points to YouTube,
            // otherwise fall back to an HTML5 <video> tag or render children.
            "video" => {
                // Prefer `url` (parser-produced) but accept legacy `src`
                let src_opt = node
                    .attributes
                    .get("url")
                    .or_else(|| node.attributes.get("src"));

                if let Some(src) = src_opt {
                    // If it's a YouTube URL, embed using the central helper
                    if let Some(vid) = youtube_id_from_url(src) {
                        let title_attr = node
                            .attributes
                            .get("title")
                            .map(|s| s.as_str())
                            .or_else(|| node.attributes.get("alt").map(|s| s.as_str()))
                            .unwrap_or("YouTube video");

                        return build_youtube_iframe(&vid, title_attr);
                    }

                    // Not a recognized YouTube URL - if it's an actual media file, use <video>
                    let lower = src.to_lowercase();
                    if lower.ends_with(".mp4")
                        || lower.ends_with(".webm")
                        || lower.ends_with(".ogg")
                    {
                        let mut attrs = String::new();
                        if node
                            .attributes
                            .get("controls")
                            .map(|v| v == "false")
                            .unwrap_or(false)
                        {
                            // omit controls when explicitly set to false
                        } else {
                            attrs.push_str(" controls");
                        }
                        if let Some(poster) = node.attributes.get("poster") {
                            attrs.push_str(&format!(" poster=\"{}\"", html_escape(poster)));
                        }
                        return format!(
                            "<video src=\"{}\"{}>Your browser does not support the video tag.</video>",
                            html_escape(src),
                            attrs
                        );
                    }

                    // Fallback: render children if present, otherwise render the raw src as a link
                    if !node.children.is_empty() {
                        return node
                            .children
                            .iter()
                            .map(Self::render_node)
                            .collect::<String>();
                    }

                    return format!("<a href=\"{}\">{}</a>", html_escape(src), html_escape(src));
                }

                node.children
                    .iter()
                    .map(Self::render_node)
                    .collect::<String>()
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

    #[test]
    fn test_image_youtube_embeds_iframe() {
        // Create an image node with a youtu.be URL as src and ensure the renderer
        // produces an iframe embed rather than an <img> or plain link.
        let mut img = crate::components::marco_engine::parser::Node::new("image");
        img.add_attribute("url", "https://youtu.be/H8c1ObYSlQI?si=SC_SV3bB7fT1gvN7");
        img.add_attribute("alt", "Test");

        let out = MarkdownRenderer::render_node(&img);
        // Should contain the embed URL format
        assert!(
            out.contains("https://www.youtube.com/embed/H8c1ObYSlQI"),
            "out={}",
            out
        );
        // Prefer iframe output
        assert!(out.contains("<iframe"), "out={}", out);
    }

    #[test]
    fn test_link_with_text_shorthand_embeds_iframe() {
        // Simulate parser producing a link whose child is a text node that contains
        // an image shorthand (malformed/partial AST). The link url is a youtu.be link.
        let mut link = crate::components::marco_engine::parser::Node::new("link");
        link.add_attribute("url", "https://youtu.be/H8c1ObYSlQI?si=SC_SV3bB7fT1gvN7");
        let txt = crate::components::marco_engine::parser::Node::text_node(
            "![Test](https://youtu.be/H8c1ObYSlQI?si=SC_SV3bB7fT1gvN7)",
        );
        link.add_child(txt);

        let out = MarkdownRenderer::render_node(&link);
        assert!(
            out.contains("https://www.youtube.com/embed/H8c1ObYSlQI"),
            "out={}",
            out
        );
        assert!(out.contains("<iframe"), "out={}", out);
    }

    // test_print_youtube_render was used during development to inspect output and
    // was removed to keep the test suite focused. Manual smoke runs confirmed
    // the renderer outputs iframe HTML for the canonical thumbnail-wrapped link.
}
