use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "components/marco_engine/grammar/marco.pest"]
pub struct MarcoParser;

// Re-export pest types for convenience
pub use pest::error::Error as PestError;
pub use pest::iterators::{Pair, Pairs};

pub type ParseResult<T> = Result<T, PestError<Rule>>;

/// Parse input text with the specified rule
pub fn parse_with_rule(rule: Rule, input: &str) -> ParseResult<Pairs<Rule>> {
    MarcoParser::parse(rule, input)
}

/// Parse a complete document
pub fn parse_document(input: &str) -> ParseResult<Pairs<Rule>> {
    MarcoParser::parse(Rule::document, input)
}

/// Helper function to print parse tree for debugging
pub fn print_pairs(pair: Pair<Rule>, depth: usize) {
    for _ in 0..depth {
        print!("  ");
    }
    println!("{:?}: {}", pair.as_rule(), pair.as_str());
    for inner_pair in pair.into_inner() {
        print_pairs(inner_pair, depth + 1);
    }
}
