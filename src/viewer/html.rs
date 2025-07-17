
use crate::editor::logic::parser::EventIter;
use crate::editor::logic::ast::blocks_and_inlines::Block;

pub fn render_html(ast: &Block) -> String {
    let mut html = String::new();
    for event in EventIter::new(ast) {
        match event {
            crate::editor::logic::parser::Event::Start(crate::editor::logic::parser::Tag::Paragraph) => html.push_str("<p>"),
            crate::editor::logic::parser::Event::End(crate::editor::logic::parser::TagEnd::Paragraph) => html.push_str("</p>\n"),
            crate::editor::logic::parser::Event::Start(crate::editor::logic::parser::Tag::Heading(level)) => html.push_str(&format!("<h{}>", level)),
            crate::editor::logic::parser::Event::End(crate::editor::logic::parser::TagEnd::Heading) => html.push_str("</h1>\n"),
            crate::editor::logic::parser::Event::Start(crate::editor::logic::parser::Tag::BlockQuote) => html.push_str("<blockquote>"),
            crate::editor::logic::parser::Event::End(crate::editor::logic::parser::TagEnd::BlockQuote) => html.push_str("</blockquote>\n"),
            crate::editor::logic::parser::Event::Start(crate::editor::logic::parser::Tag::List) => html.push_str("<ul>"),
            crate::editor::logic::parser::Event::End(crate::editor::logic::parser::TagEnd::List) => html.push_str("</ul>\n"),
            crate::editor::logic::parser::Event::Start(crate::editor::logic::parser::Tag::Item) => html.push_str("<li>"),
            crate::editor::logic::parser::Event::End(crate::editor::logic::parser::TagEnd::Item) => html.push_str("</li>\n"),
            crate::editor::logic::parser::Event::Start(crate::editor::logic::parser::Tag::CodeBlock) => html.push_str("<pre><code>"),
            crate::editor::logic::parser::Event::End(crate::editor::logic::parser::TagEnd::CodeBlock) => html.push_str("</code></pre>\n"),
            crate::editor::logic::parser::Event::Start(crate::editor::logic::parser::Tag::HtmlBlock) => {},
            crate::editor::logic::parser::Event::End(crate::editor::logic::parser::TagEnd::HtmlBlock) => {},
            crate::editor::logic::parser::Event::Text(text) => html.push_str(text),
            crate::editor::logic::parser::Event::Code(code) => html.push_str(code),
            crate::editor::logic::parser::Event::Html(html_block) => html.push_str(html_block),
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
