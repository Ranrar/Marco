#[cfg(test)]
mod heading_test {
    use crate::components::marco_engine::{AstBuilder, MarcoParser, Rule};
    use pest::Parser;

    #[test]
    fn test_heading_ast_build() {
        let input = "# Hello World";

        // Parse with heading rule
        let pairs = MarcoParser::parse(Rule::heading, input).unwrap();
        let pair = pairs.into_iter().next().unwrap();

        println!("Raw parse structure:");
        print_pair(&pair, 0);

        // Build AST
        let ast = AstBuilder::build(pair).unwrap();
        println!("\nAST: {:?}", ast);
    }

    fn print_pair(pair: &pest::iterators::Pair<Rule>, indent: usize) {
        let indent_str = "  ".repeat(indent);
        println!(
            "{}Rule::{:?} -> {:?}",
            indent_str,
            pair.as_rule(),
            pair.as_str()
        );
        for inner in pair.clone().into_inner() {
            print_pair(&inner, indent + 1);
        }
    }
}
