
use crate::editor::logic::parser::EventIter;
use crate::editor::logic::event::{Event, Tag, TagEnd};
use crate::editor::logic::ast::blocks_and_inlines::Block;

pub fn render_html(ast: &Block) -> String {
    let mut html = String::new();
    // TODO: Render custom attributes (classes, IDs, data-*) for each tag if present in attr
    // See parser and AST for propagation logic. This is not yet implemented.
    for event in EventIter::new(ast) {
        match event {
            Event::Start(Tag::Paragraph(attr), _, _) => html.push_str("<p>"),
            Event::End(TagEnd::Paragraph(attr), _, _) => html.push_str("</p>\n"),
            Event::Start(Tag::Heading(level, attr), _, _) => html.push_str(&format!("<h{}>", level)),
            Event::End(TagEnd::Heading(attr), _, _) => html.push_str("</h1>\n"),
            Event::Start(Tag::BlockQuote(attr), _, _) => html.push_str("<blockquote>"),
            Event::End(TagEnd::BlockQuote(attr), _, _) => html.push_str("</blockquote>\n"),
            Event::Start(Tag::List(attr), _, _) => html.push_str("<ul>"),
            Event::End(TagEnd::List(attr), _, _) => html.push_str("</ul>\n"),
            Event::Start(Tag::Item(attr), _, _) => html.push_str("<li>"),
            Event::End(TagEnd::Item(attr), _, _) => html.push_str("</li>\n"),
            Event::Start(Tag::CodeBlock(attr), _, _) => html.push_str("<pre><code>"),
            Event::End(TagEnd::CodeBlock(attr), _, _) => html.push_str("</code></pre>\n"),
            Event::Start(Tag::HtmlBlock(attr), _, _) => {},
            Event::End(TagEnd::HtmlBlock(attr), _, _) => {},
            Event::Text(text, _, _) => html.push_str(&text),
            Event::Code(code, _, _) => html.push_str(&code),
            Event::Html(html_block, _, _) => html.push_str(&html_block),
            Event::Start(Tag::Emphasis(attr), _, _) => html.push_str("<em>"),
            Event::End(TagEnd::Emphasis(attr), _, _) => html.push_str("</em>"),
            Event::Start(Tag::Strong(attr), _, _) => html.push_str("<strong>"),
            Event::End(TagEnd::Strong(attr), _, _) => html.push_str("</strong>"),
            Event::Start(Tag::Link(attr), _, _) => html.push_str("<a href='#'>"), // TODO: use actual href
            Event::End(TagEnd::Link(attr), _, _) => html.push_str("</a>"),
            Event::Start(Tag::Image(attr), _, _) => html.push_str("<img src='#' alt='' />"), // TODO: use actual src/alt
            Event::End(TagEnd::Image(attr), _, _) => {},
            Event::EmphasisStart(_, _) | Event::EmphasisEnd(_, _) | Event::StrongStart(_, _) | Event::StrongEnd(_, _) |
            Event::LinkStart { .. } | Event::LinkEnd(_, _) | Event::ImageStart { .. } | Event::ImageEnd(_, _) |
            Event::Autolink(_, _, _) | Event::RawHtml(_, _, _) | Event::HardBreak(_, _) | Event::SoftBreak(_, _) => {},
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
