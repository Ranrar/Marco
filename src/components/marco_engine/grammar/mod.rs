use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "components/marco_engine/grammar/marco.pest"]
pub struct MarcoParser;
