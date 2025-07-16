// src/markdown/edit.rs

use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, ScrolledWindow};
use gtk_sourceview5::{Buffer as SourceBuffer, View as SourceView};
use crate::markdown::ast::MarkdownNode;

pub fn render_editor(ast: &MarkdownNode) -> GtkBox {
    let container = GtkBox::new(gtk::Orientation::Vertical, 6);

    match ast {
        MarkdownNode::Document(children) => {
            for node in children {
                let widget = render_editor(node);
                container.append(&widget);
            }
        }

        MarkdownNode::Heading { level, content } => {
            let text = content.iter().map(flatten_text).collect::<String>();
            let label = Label::new(Some(&text));
            label.set_margin_top(10);
            label.set_markup(&format!(
                "<span size=\"{}\">{}</span>",
                match level {
                    1 => "18000",
                    2 => "14000",
                    _ => "12000",
                },
                text
            ));
            container.append(&label);
        }

        MarkdownNode::Paragraph(children) => {
            let text = children.iter().map(flatten_text).collect::<String>();
            let label = Label::new(Some(&text));
            label.set_wrap(true);
            container.append(&label);
        }

        MarkdownNode::CodeBlock { language, code } => {
            let buffer = SourceBuffer::new(None);
            buffer.set_text(code);
            if let Some(lang_name) = language {
                if let Some(manager) = gtk_sourceview5::LanguageManager::default() {
                    if let Some(lang) = manager.language(lang_name) {
                        buffer.set_language(Some(&lang));
                    }
                }
            }

            let view = SourceView::new_with_buffer(&buffer);
            view.set_monospace(true);
            view.set_show_line_numbers(false);
            view.set_editable(false);

            let scrolled = ScrolledWindow::new();
            scrolled.set_child(Some(&view));
            scrolled.set_min_content_height(100);
            container.append(&scrolled);
        }

        _ => {}
    }

    container
}

fn flatten_text(node: &MarkdownNode) -> String {
    match node {
        MarkdownNode::Text(t) => t.clone(),
        MarkdownNode::Emphasis(inner)
        | MarkdownNode::Strong(inner)
        | MarkdownNode::Strikethrough(inner) => {
            inner.iter().map(flatten_text).collect()
        }
        MarkdownNode::Code(t) => t.clone(),
        _ => String::new(),
    }
}
