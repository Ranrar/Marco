use super::{BuilderHelpers, ErrorHandling};
use crate::components::marco_engine::{
    ast::{Node, Span},
    errors::MarcoResult,
    grammar::Rule,
};
use log::debug;
use pest::iterators::Pair;

/// Trait for building inline AST nodes
pub trait InlineBuilder: BuilderHelpers + ErrorHandling {
    /// Build inline wrapper nodes
    fn build_inline(pair: Pair<Rule>) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_inline - Processing inline");
        Self::build_wrapper_node(pair)
    }

    /// Build inline core nodes
    fn build_inline_core(pair: Pair<Rule>) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_inline_core - Processing inline core");
        Self::build_wrapper_node(pair)
    }

    /// Build strong/bold text nodes
    fn build_strong(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_strong - Processing strong text");
        Self::build_inline_formatting(pair, span, Node::strong)
    }

    /// Build emphasis/italic text nodes
    fn build_emphasis(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_emphasis - Processing emphasis text");
        Self::build_inline_formatting(pair, span, Node::emphasis)
    }

    /// Build strikethrough text nodes
    fn build_strikethrough(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_strikethrough - Processing strikethrough text");
        Self::build_inline_formatting(pair, span, Node::strikethrough)
    }

    /// Build highlight text nodes
    fn build_highlight(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_highlight - Processing highlight text");
        Self::build_inline_formatting(pair, span, Node::highlight)
    }

    /// Build superscript text nodes
    fn build_superscript(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_superscript - Processing superscript text");
        Self::build_inline_formatting(pair, span, Node::superscript)
    }

    /// Build subscript text nodes
    fn build_subscript(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_subscript - Processing subscript text");
        Self::build_inline_formatting(pair, span, Node::subscript)
    }

    /// Build inline math expressions
    fn build_inline_math(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_inline_math - Processing inline math");
        let content = pair.as_str();

        // Validate math expression length
        if let Err(e) = Self::validate_math_expression(content) {
            log::warn!(
                "Math expression validation failed: {}, using text fallback",
                e
            );
            return Ok(Node::text(content.to_string(), span));
        }

        // Use proper MathInline node for inline math expressions
        Ok(Node::math_inline(content.to_string(), span))
    }

    /// Build emoji nodes
    fn build_emoji(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_emoji - Processing emoji");
        let emoji_str = pair.as_str();
        let emoji_name = emoji_str.trim_matches(':');

        Ok(Node::emoji(emoji_name.to_string(), span))
    }

    /// Build line break nodes
    fn build_line_break(span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_line_break - Processing line break");
        Ok(Node::line_break(span))
    }

    /// Build escaped character nodes
    fn build_escaped_char(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_escaped_char - Processing escaped character");
        let escaped_str = pair.as_str();
        let char_content = if escaped_str.len() > 1 {
            // Remove the escape backslash
            escaped_str[1..].to_string()
        } else {
            escaped_str.to_string()
        };

        Ok(Node::text(char_content, span))
    }

    /// Build inline code nodes
    fn build_code_inline(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_code_inline - Processing inline code");
        let content = pair.as_str().trim_matches('`');
        Ok(Node::code(content.to_string(), span))
    }

    /// Build inline HTML nodes
    fn build_inline_html(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("InlineBuilder::build_inline_html - Processing inline HTML");
        Ok(Node::inline_html(pair.as_str().to_string(), span))
    }
}
