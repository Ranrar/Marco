use clap::Parser;
use std::path::PathBuf;

mod errors;
mod loader;
mod types;
mod validate;

#[derive(Parser)]
#[command(about = "Validate RON AST/syntax against a pest grammar")]
struct Cli {
    /// Path to ast.ron
    #[arg(long, default_value = "files/ast.ron")]
    ast: PathBuf,

    /// Path to syntax.ron
    #[arg(long, default_value = "files/syntax.ron")]
    syntax: PathBuf,

    /// Path to markdown.pest
    #[arg(long, default_value = "files/markdown.pest")]
    grammar: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let ast_val = match loader::load_ron_value(&cli.ast) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("WARN: failed to parse ast.ron as RON; using text-extraction fallback");
            let s = std::fs::read_to_string(&cli.ast)?;
            let kinds = loader::extract_kinds_from_ast_str(&s);
            loader::value_from_kinds(kinds)
        }
    };

    let syntax_val = match loader::load_ron_value(&cli.syntax) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("WARN: failed to parse syntax.ron as RON; using text-extraction fallback");
            let s = std::fs::read_to_string(&cli.syntax)?;
            let kinds = loader::extract_kinds_from_syntax_str(&s);
            loader::value_from_kinds(kinds)
        }
    };

    let ast = types::AstRoot::from_value(&ast_val)
        .ok_or_else(|| anyhow::anyhow!("failed to extract AstRoot children from RON"))?;
    let syntax = types::SyntaxRoot::from_value(&syntax_val)
        .ok_or_else(|| anyhow::anyhow!("failed to extract SyntaxRoot children from RON"))?;
    let grammar_text = std::fs::read_to_string(&cli.grammar)?;
    let rule_names = loader::extract_rule_names_from_str(&grammar_text);

    match validate::validate(&ast, &syntax, &rule_names) {
        Ok(()) => {
            println!("Validation passed");
            Ok(())
        }
        Err(errs) => {
            eprintln!("Validation failed with {} error(s)", errs.len());
            for e in errs {
                eprintln!("- {}", e);
            }
            std::process::exit(2);
        }
    }
}
// single main already defined above
