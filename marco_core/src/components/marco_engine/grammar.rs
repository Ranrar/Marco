// DEPRECATED: Old monolithic grammar system
//
// This file contains the legacy MarcoParser which used a single monolithic
// grammar file (marco_grammar.pest - now archived). 
//
// **DO NOT USE FOR NEW CODE** - Use the new two-stage parser instead:
//   - parsers::block_parser::BlockParser (block-level parsing)
//   - parsers::inline_parser::InlineParser (inline-level parsing)  
//   - parsers::orchestrator::parse_document (unified API)
//
// This legacy parser is retained temporarily for backward compatibility
// during the migration period. It will be removed in a future release.
//
// Archived grammar location:
//   marco_core/src/components/marco_engine/grammar/archive/marco_grammar.pest.old

use pest_derive::Parser;

#[deprecated(
    since = "0.2.0",
    note = "Use parsers::orchestrator::parse_document() with the new two-stage parser instead"
)]
#[derive(Parser)]
#[grammar = "components/marco_engine/grammar/archive/marco_grammar.pest.old"]
pub struct MarcoParser;
