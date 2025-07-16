// src/markdown/renderer.rs

use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, ScrolledWindow, TextView};
use gtk_sourceview5::{Buffer as SourceBuffer, View as SourceView};
use webkit6::{WebView};

use crate::markdown::ast::MarkdownNode;

/// Render AST into an HTML string for WebKit6
pub fn render_html(ast: &MarkdownNode) -> String {
    match ast {
        MarkdownNode::Document(children) => {
            children.iter()
                .map(render_html)
                .collect::<Vec<_>>()
                .join("\n")
        }
        MarkdownNode::Heading { level, content } => {
            let inner = content.iter().map(render_html).collect::<Vec<_>>().join("");
            format!("<h{lvl}>{}</h{lvl}>", inner, lvl = level)
        }
        MarkdownNode::Paragraph(children) => {
            let inner = children.iter().map(render_html).collect::<Vec<_>>().join("");
            format!("<p>{}</p>", inner)
        }
        MarkdownNode::Text(text) => text.clone(),
        MarkdownNode::Emphasis(inner) => format!(
            "<em>{}</em>",
            inner.iter().map(render_html).collect::<String>()
        ),
        MarkdownNode::Strong(inner) => format!(
            "<strong>{}</strong>",
            inner.iter().map(render_html).collect::<String>()
        ),
        MarkdownNode::Strikethrough(inner) => format!(
            "<del>{}</del>",
            inner.iter().map(render_html).collect::<String>()
        ),
        MarkdownNode::Code(code) => format!("<code>{}</code>", html_escape::encode_text(code)),
        MarkdownNode::CodeBlock { language, code } => {
            let lang = language.clone().unwrap_or_else(|| "text".into());
            format!(
                "<pre><code class=\"language-{}\">{}</code></pre>",
                lang,
                html_escape::encode_text(code)
            )
        }
        MarkdownNode::List { ordered, items } => {
            let tag = if *ordered { "ol" } else { "ul" };
            let content = items.iter()
                .map(|item| render_html(item))
                .collect::<Vec<_>>()
                .join("");
            format!("<{tag}>{}</{tag}>", content, tag = tag)
        }
        MarkdownNode::ListItem(children) => {
            let inner = children.iter().map(render_html).collect::<String>();
            format!("<li>{}</li>", inner)
        }
        MarkdownNode::Link { label, destination, title } => {
            let label_html = label.iter().map(render_html).collect::<String>();
            let title_attr = title.as_ref().map_or(String::new(), |t| format!(" title=\"{}\"", t));
            format!("<a href=\"{}\"{}>{}</a>", destination, title_attr, label_html)
        }
        MarkdownNode::Image { alt, src, title } => {
            let alt_text = alt.iter().map(render_html).collect::<String>();
            let title_attr = title.as_ref().map_or(String::new(), |t| format!(" title=\"{}\"", t));
            format!("<img src=\"{}\" alt=\"{}\"{} />", src, alt_text, title_attr)
        }
        MarkdownNode::LineBreak => "<br/>".into(),
        MarkdownNode::ThematicBreak => "<hr/>".into(),
        _ => String::new(), // BlockQuote etc. not implemented yet
    }
}
