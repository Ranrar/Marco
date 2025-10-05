use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "components/marco_engine/marco_grammar.pest"]
pub struct MarcoParser;
