//! GFM extension logic stub

use crate::logic::core::options::ParserOptions;
use crate::logic::core::inline::types::InlineNode;
use super::MarkdownExtension;

pub struct GithubExtension;

impl MarkdownExtension for GithubExtension {
    fn apply(&self, ast: &mut Vec<InlineNode>, options: &ParserOptions) {
        if !options.gfm {
            return;
        }
        // Strikethrough: transform ~~text~~ into Strikethrough node
        let mut i = 0;
        while i < ast.len() {
            if let InlineNode::Text { text, pos } = &ast[i] {
                if text.contains("~~") {
                    let mut parts = text.splitn(3, "~~");
                    if let (Some(_), Some(strike), Some(_)) = (parts.next(), parts.next(), parts.next()) {
                        ast[i] = InlineNode::Strikethrough {
                            children: vec![InlineNode::Text { text: strike.to_string(), pos: *pos }],
                            pos: *pos,
                        };
                    }
                }
            }
            i += 1;
        }

        // Task lists: transform [ ] and [x] at start of text into TaskListItem node
        let mut i = 0;
        while i < ast.len() {
            if let InlineNode::Text { text, pos } = &ast[i] {
                if text.starts_with("[ ] ") || text.starts_with("[x] ") || text.starts_with("[X] ") {
                    let checked = text[1..2].eq_ignore_ascii_case("x");
                    let label = text[4..].to_string();
                    ast[i] = InlineNode::TaskListItem {
                        checked,
                        children: vec![InlineNode::Text { text: label, pos: *pos }],
                        pos: *pos,
                    };
                }
            }
            i += 1;
        }

        // Autolinks: transform <http://...> or <https://...> into Link node
        let mut i = 0;
        while i < ast.len() {
            if let InlineNode::Text { text, pos } = &ast[i] {
                if text.starts_with("<http") && text.ends_with(">") {
                    let url = text.trim_start_matches('<').trim_end_matches('>').to_string();
                    ast[i] = InlineNode::Link {
                        href: url.clone(),
                        title: String::new(),
                        children: vec![InlineNode::Text { text: url, pos: *pos }],
                        pos: *pos,
                    };
                }
            }
            i += 1;
        }
    }
}
