use crate::logic::ast::blocks_and_inlines::{Block, LeafBlock};
use crate::logic::ast::inlines::{Inline, Emphasis};

fn render_inlines(inlines: &[(Inline, crate::logic::parser::event::SourcePos)]) -> String {
    inlines.iter().map(|(inline, _pos)| match inline {
        Inline::Text(s) => {
            // Replace newlines with <br> for visibility
            html_escape::encode_text(s).replace("\n", "<br>")
        },
        Inline::Code(code) => format!("<code>{}</code>", html_escape::encode_text(&code.content)),
        Inline::Emphasis(e) => match e {
            Emphasis::Emph(children, _) => format!("<em>{}</em>", render_inlines(children)),
            Emphasis::Strong(children, _) => format!("<strong>{}</strong>", render_inlines(children)),
        },
        other => format!("[{:?}]", other), // Fallback: show debug output for unhandled variants
    }).collect::<Vec<_>>().join("")
}

pub fn render(ast: &Block) -> String {
    match ast {
        Block::Leaf(LeafBlock::Paragraph(inlines, _)) => {
            format!("<p>{}</p>", render_inlines(inlines))
        }
        Block::Leaf(LeafBlock::Heading { level, content, .. }) => {
            format!("<h{lvl}>{}</h{lvl}>", render_inlines(content), lvl = level)
        }
        Block::Leaf(LeafBlock::IndentedCodeBlock { content, .. }) => {
            format!("<pre><code>{}</code></pre>", html_escape::encode_text(content))
        }
        Block::Leaf(LeafBlock::FencedCodeBlock { content, .. }) => {
            format!("<pre><code>{}</code></pre>", html_escape::encode_text(content))
        }
        // Add more block types as needed
        _ => String::new(),
    }
}