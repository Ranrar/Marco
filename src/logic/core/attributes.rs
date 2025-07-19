use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attributes {
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub data: HashMap<String, String>,
    // Optionally: style, aria, etc. can be added later
}

impl Attributes {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Parses Pandoc/Maruku-style attribute blocks: `{.class #id key=value}`
/// Returns an Attributes struct
pub fn parse_attributes_block(s: &str) -> Attributes {
    let mut classes = Vec::new();
    let mut id = None;
    let mut data = HashMap::new();

    let tokens = s.trim_matches(|c| c == '{' || c == '}').split_whitespace();
    for token in tokens {
        if token.starts_with('.') {
            classes.push(token[1..].to_string());
        } else if token.starts_with('#') {
            id = Some(token[1..].to_string());
        } else if let Some(eq) = token.find('=') {
            let key = &token[..eq];
            let value = &token[eq+1..];
            data.insert(key.to_string(), value.to_string());
        }
    }
    Attributes { classes, id, data }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_attributes_block() {
        let attr = parse_attributes_block("{.foo .bar #main data-x=42 data-y='baz'}");
        assert_eq!(attr.classes, vec!["foo", "bar"]);
        assert_eq!(attr.id, Some("main".to_string()));
        assert_eq!(attr.data["data-x"], "42");
        assert_eq!(attr.data["data-y"], "'baz'");
    }

    #[test]
    fn test_markdown_event_stream_with_attributes() {
        use crate::logic::core::lexer::parse_phrases;
        use crate::logic::core::emitter::push_inline_events;
        let md = "*emph*{.important} and **strong**{#main} and [link](url){.external}";
        let (inlines, _diag_events) = parse_phrases(md);
        let mut events = Vec::new();
        push_inline_events(&mut events, inlines, &mut None);
        for event in &events {
            println!("{:?}", event);
        }
    }
}
