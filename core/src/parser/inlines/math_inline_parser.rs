//! Math inline parser - converts grammar output to InlineMath AST node
//!
//! Parses inline math delimited by `$...$` using KaTeX.

use super::shared::{to_parser_span, GrammarSpan};
use crate::grammar::inlines::inline_math;
use crate::parser::ast::{Node, NodeKind};
use nom::IResult;

/// Parse an inline math expression `$...$` to an InlineMath AST node.
///
/// # Example
/// ```ignore
/// let input = GrammarSpan::new("$E = mc^2$ text");
/// let (rest, node) = parse_inline_math(input).unwrap();
/// // node.kind == NodeKind::InlineMath { content: "E = mc^2" }
/// ```
pub fn parse_inline_math(input: GrammarSpan) -> IResult<GrammarSpan, Node> {
    let (rest, content_span) = inline_math(input)?;

    let node = Node {
        kind: NodeKind::InlineMath {
            content: content_span.fragment().to_string(),
        },
        span: Some(to_parser_span(content_span)),
        children: Vec::new(),
    };

    Ok((rest, node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_inline_math() {
        let input = GrammarSpan::new("$E = mc^2$ text");
        let result = parse_inline_math(input);
        assert!(result.is_ok());

        let (_, node) = result.unwrap();
        match node.kind {
            NodeKind::InlineMath { content } => {
                assert_eq!(content, "E = mc^2");
            }
            _ => panic!("Expected InlineMath node"),
        }
    }
}
