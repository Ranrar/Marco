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
        // Strikethrough: transform ~~text~~ into Strikethrough node (GFM 6.5)
        let mut i = 0;
        while i < ast.len() {
            if let InlineNode::Text { text, pos } = &ast[i] {
                // Only match ~~text~~, not ~text~ or ~~~
                if text.starts_with("~~") && text.ends_with("~~") && text.len() > 4 {
                    let strike = text[2..text.len()-2].to_string();
                    ast[i] = InlineNode::Strikethrough {
                        children: vec![InlineNode::Text { text: strike, pos: *pos }],
                        pos: *pos,
                    };
                }
            }
            i += 1;
        }

        // Task lists: transform [ ] and [x] at start of text into TaskListItem node (GFM 5.3)
        let mut i = 0;
        while i < ast.len() {
            if let InlineNode::Text { text, pos } = &ast[i] {
                if (text.starts_with("[ ] ") || text.starts_with("[x] ") || text.starts_with("[X] ")) && text.len() > 4 {
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

        // Autolinks: transform <http://...> or <https://...> into Link node (GFM 6.9)
        let mut i = 0;
        while i < ast.len() {
            if let InlineNode::Text { text, pos } = &ast[i] {
                if (text.starts_with("<http://") || text.starts_with("<https://")) && text.ends_with(">") {
                    let mut url = text.trim_start_matches('<').trim_end_matches('>').to_string();
                    // GFM: coerce http to https
                    if url.starts_with("http://") {
                        url = format!("https://{}", &url[7..]);
                    }
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
