use crate::logic::parser::EventIter;
use crate::logic::event::{Event, Tag, TagEnd};
use crate::logic::ast::blocks_and_inlines::Block;

pub fn render_html(ast: &Block) -> String {
    let mut html = String::new();
    // TODO: Render custom attributes (classes, IDs, data-*) for each tag if present in attr
    // See parser and AST for propagation logic. This is not yet implemented.
    let mut diagnostics = crate::logic::parser::diagnostics::Diagnostics::new();
    for event in EventIter::new(ast, Some(&mut diagnostics)) {
        match event {
            Event::Profile(profile_type, value, timestamp) => {
                // Profiling event: log, ignore, or provide plugin hook
                // For now, add as HTML comment for diagnostics
                html.push_str(&format!("<!-- Profile: {:?} value={} timestamp={} -->", profile_type, value, timestamp));
            }
            Event::GroupStart(group_type, _, _) => {
                // Fallback: wrap group in a div with group type as class
                let class = match group_type {
                    crate::logic::parser::event::GroupType::List => "group-list",
                    crate::logic::parser::event::GroupType::TableRow => "group-table-row",
                    crate::logic::parser::event::GroupType::BlockGroup => "group-block",
                };
                html.push_str(&format!("<div class='{}'>", class));
            }
            Event::GroupEnd(_, _, _) => {
                // Fallback: close the group div
                html.push_str("</div>");
            }
            Event::Start(Tag::Paragraph(_attr), _, _) => html.push_str("<p>"),
            Event::End(TagEnd::Paragraph(_attr), _, _) => html.push_str("</p>\n"),
            Event::Start(Tag::Heading(level, _attr), _, _) => html.push_str(&format!("<h{}>", level)),
            Event::End(TagEnd::Heading(_attr), _, _) => html.push_str("</h1>\n"),
            Event::Start(Tag::BlockQuote(_attr), _, _) => html.push_str("<blockquote>"),
            Event::End(TagEnd::BlockQuote(_attr), _, _) => html.push_str("</blockquote>\n"),
            Event::Start(Tag::List(_attr), _, _) => html.push_str("<ul>"),
            Event::End(TagEnd::List(_attr), _, _) => html.push_str("</ul>\n"),
            Event::Start(Tag::Item(_attr), _, _) => html.push_str("<li>"),
            Event::End(TagEnd::Item(_attr), _, _) => html.push_str("</li>\n"),
            Event::Start(Tag::CodeBlock(_attr), _, _) => html.push_str("<pre><code>"),
            Event::End(TagEnd::CodeBlock(_attr), _, _) => html.push_str("</code></pre>\n"),
            Event::Start(Tag::HtmlBlock(_attr), _, _) => {},
            Event::End(TagEnd::HtmlBlock(_attr), _, _) => {},
            Event::Start(Tag::MathBlock(content, math_type, _attr), _, _) => {
                let typ = math_type.as_ref().map(|t| format!(" class='math-{:?}'", t)).unwrap_or_default();
                html.push_str(&format!("<div class='math-block'{}>{}</div>", typ, content));
            },
            Event::Start(Tag::TableCaption(content, _attr), _, _) => {
                html.push_str(&format!("<caption>{}</caption>", content));
            },
            Event::Start(Tag::TaskListMeta(group, _attr), _, _) => {
                let group_str = group.as_ref().map(|g| format!(" data-group='{}'", g)).unwrap_or_default();
                html.push_str(&format!("<div class='task-list-meta'{}></div>", group_str));
            },
            Event::Text(text, _, _) => html.push_str(&text),
            Event::Code(code, _, _) => html.push_str(&code),
            Event::Html(html_block, _, _) => html.push_str(&html_block),
            Event::Start(Tag::Emphasis(_attr), _, _) => html.push_str("<em>"),
            Event::End(TagEnd::Emphasis(_attr), _, _) => html.push_str("</em>"),
            Event::Start(Tag::Strong(_attr), _, _) => html.push_str("<strong>"),
            Event::End(TagEnd::Strong(_attr), _, _) => html.push_str("</strong>"),
            Event::Start(Tag::Link(_attr), _, _) => html.push_str("<a href='#'>"), // TODO: use actual href
            Event::End(TagEnd::Link(_attr), _, _) => html.push_str("</a>"),
            Event::Start(Tag::Image(_attr), _, _) => html.push_str("<img src='#' alt='' />"), // TODO: use actual src/alt
            Event::End(TagEnd::Image(_attr), _, _) => {},
            Event::Math { content, .. } => {
                html.push_str(&format!("<span class='math'>{}</span>", content));
            },
            Event::MathBlock { content, math_type, .. } => {
                // Render block math with type info
                let typ = math_type.as_ref().map(|t| format!(" class='math-{:?}'", t)).unwrap_or_default();
                html.push_str(&format!("<div class='math-block'{}>{}</div>", typ, content));
            },
            Event::Emoji(shortcode, unicode, _) => {
                // Render emoji as unicode with tooltip
                html.push_str(&format!("<span class='emoji' title=':{}:'>{}</span>", shortcode, unicode));
            },
            Event::Mention(username, _) => {
                // Render mention as a link
                html.push_str(&format!("<a class='mention' href='https://github.com/{}'>@{}</a>", username, username));
            },
            Event::EmphasisStart(_, _) | Event::EmphasisEnd(_, _) | Event::StrongStart(_, _) | Event::StrongEnd(_, _) |
            Event::LinkStart { .. } | Event::LinkEnd(_, _) | Event::ImageStart { .. } | Event::ImageEnd(_, _) |
            Event::Autolink(_, _, _) | Event::RawHtml(_, _, _) | Event::HardBreak(_, _) | Event::SoftBreak(_, _) => {},
            Event::Error(msg, pos) => {
                html.push_str(&format!("<span class='error'>Error: {} at {:?}</span>", msg, pos));
            },
            Event::Warning(msg, pos) => {
                html.push_str(&format!("<span class='warning'>Warning: {} at {:?}</span>", msg, pos));
            },
            Event::Unsupported(msg, pos) => {
                html.push_str(&format!("<span class='unsupported'>Unsupported: {} at {:?}</span>", msg, pos));
            },
            Event::Start(Tag::CustomTag { name, data, attributes }, _, _) => {
                // Fallback: render as a div with data attributes, or call plugin hook
                html.push_str(&format!(
                    "<div class='custom-tag' data-name='{}'{}{}>",
                    name,
                    data.as_ref().map(|d| format!(" data-data='{}'", d)).unwrap_or_default(),
                    attributes.as_ref().map(|a| format!(" data-attrs='{:?}'", a)).unwrap_or_default()
                ));
            }
            Event::End(TagEnd::CustomTagEnd { name, attributes }, _, _) => {
                // Fallback: close the custom tag div
                html.push_str("</div>");
            }
        }
    }
    html
}

pub fn wrap_html_document(body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <style>body {{ font-family: sans-serif; }}</style>
  </head>
  <body>{}</body>
</html>"#,
        body
    )
}
