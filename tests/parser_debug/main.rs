#[cfg(feature = "integration-tests")]
use marco::components::marco_engine::parser::parse_with_rule;
#[cfg(feature = "integration-tests")]
use marco::components::marco_engine::{build_ast, parse_markdown, parse_to_html_cached, Rule};
#[cfg(feature = "integration-tests")]
use pest::iterators::{Pair, Pairs};
#[cfg(feature = "integration-tests")]
use std::io::{self, Read};

#[cfg(feature = "integration-tests")]
use clap::{Arg, Command};

#[cfg(feature = "integration-tests")]
fn main() {
    // Initialize settings manager for shared settings
    let _settings_manager = match marco::logic::paths::get_settings_path() {
        Ok(settings_path) => {
            match marco::logic::swanson::SettingsManager::initialize(settings_path) {
                Ok(manager) => {
                    eprintln!("Settings initialized for parser debug");
                    Some(manager)
                },
                Err(e) => {
                    eprintln!("Warning: Failed to initialize settings: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            eprintln!("Warning: Failed to get settings path: {}", e);
            None
        }
    };

    let matches = Command::new("marco-parser-debug")
        .about("Debug Marco parser grammar and AST building")
        .version("0.1.0")
        .subcommand(
            Command::new("grammar")
                .about("Test grammar parsing with specific rules")
                .arg(
                    Arg::new("rule")
                        .short('r')
                        .long("rule")
                        .value_name("RULE")
                        .help("Specific grammar rule to test")
                        .required(true),
                )
                .arg(
                    Arg::new("input")
                        .help("Markdown input to parse")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("ast")
                .about("Debug AST building from parsed grammar")
                .arg(
                    Arg::new("input")
                        .help("Markdown input to build AST from")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("pipeline")
                .about("Debug full pipeline: grammar -> AST -> HTML")
                .arg(
                    Arg::new("input")
                        .help("Markdown input for full pipeline")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("setext")
                .about("Debug setext header parsing specifically")
                .arg(
                    Arg::new("input")
                        .help("Setext header input (optional, uses default if not provided)")
                        .required(false),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("grammar", sub_matches)) => {
            let rule_name = sub_matches.get_one::<String>("rule").unwrap();
            let input = get_input_text(sub_matches.get_one::<String>("input"));
            debug_grammar_rule(&input, rule_name);
        }
        Some(("ast", sub_matches)) => {
            let input = get_input_text(sub_matches.get_one::<String>("input"));
            debug_ast_building(&input);
        }
        Some(("pipeline", sub_matches)) => {
            let input = get_input_text(sub_matches.get_one::<String>("input"));
            debug_full_pipeline(&input);
        }
        Some(("setext", sub_matches)) => {
            let input = if let Some(provided_input) = sub_matches.get_one::<String>("input") {
                provided_input.clone()
            } else {
                get_input_text(None)
            };
            debug_setext_headers(&input);
        }
        _ => {
            println!("Use --help to see available commands");
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "integration-tests")]
fn get_input_text(provided: Option<&String>) -> String {
    match provided {
        Some(text) => text.clone(),
        None => {
            println!("Enter markdown input (press Ctrl+D when finished):");
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .expect("Failed to read from stdin");
            buffer.trim_end().to_string()
        }
    }
}

#[cfg(feature = "integration-tests")]
fn debug_grammar_rule(input: &str, rule_name: &str) {
    println!("=== Grammar Rule Debug: {} ===", rule_name);
    println!("Input: {:?}", input);
    println!();

    // Parse rule from string name to Rule enum
    let rule = match rule_name {
        "setext_h1" => Rule::setext_h1,
        "setext_h2" => Rule::setext_h2,
        "setext_content" => Rule::setext_content,
        "H1" => Rule::H1,
        "H2" => Rule::H2,
        "heading" => Rule::heading,
        "document" => Rule::document,
        _ => {
            println!("Unknown rule: {}. Trying to parse as document.", rule_name);
            Rule::document
        }
    };

    match parse_with_rule(input, rule) {
        Ok(pairs) => {
            println!("✓ Grammar parsing succeeded!");
            println!();
            print_parser_structure(pairs, 0);
        }
        Err(e) => {
            println!("✗ Grammar parsing failed: {}", e);
        }
    }
}

#[cfg(feature = "integration-tests")]
fn debug_ast_building(input: &str) {
    println!("=== AST Building Debug ===");
    println!("Input: {:?}", input);
    println!();

    // Parse with grammar
    match parse_markdown(input) {
        Ok(pairs) => {
            println!("✓ Grammar parsing succeeded");
            println!();

            // Show the grammar structure
            println!("Grammar structure:");
            print_parser_structure(pairs.clone(), 0);
            println!();

            // Build AST
            match build_ast(pairs) {
                Ok(ast) => {
                    println!("✓ AST building succeeded");
                    println!();
                    println!("AST structure:");
                    println!("{:#?}", ast);
                }
                Err(e) => {
                    println!("✗ AST building failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Grammar parsing failed: {}", e);
        }
    }
}

#[cfg(feature = "integration-tests")]
fn debug_full_pipeline(input: &str) {
    println!("=== Full Pipeline Debug ===");
    println!("Input: {:?}", input);
    println!();

    // Step 1: Grammar parsing
    match parse_markdown(input) {
        Ok(pairs) => {
            println!("✓ Step 1: Grammar parsing succeeded");
            print_parser_structure(pairs.clone(), 1);
            println!();

            // Step 2: AST building
            match build_ast(pairs) {
                Ok(ast) => {
                    println!("✓ Step 2: AST building succeeded");
                    println!("   AST: {:#?}", ast);
                    println!();

                    // Step 3: HTML rendering
                    match parse_to_html_cached(input) {
                        Ok(html) => {
                            println!("✓ Step 3: HTML rendering succeeded");
                            println!("   HTML: {}", html);
                            println!();

                            // Analyze the issue
                            if input.contains("=====") || input.contains("-----") {
                                if html.contains("=====") || html.contains("-----") {
                                    println!(
                                        "⚠️  ISSUE DETECTED: HTML contains underline markers!"
                                    );
                                    println!("   This indicates the setext parsing is not working correctly.");
                                } else {
                                    println!("✓ HTML looks correct - no underline markers found");
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ Step 3: HTML rendering failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Step 2: AST building failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Step 1: Grammar parsing failed: {}", e);
        }
    }
}

#[cfg(feature = "integration-tests")]
fn debug_setext_headers(input: &str) {
    let test_cases = if input.trim().is_empty() {
        vec![
            ("Simple H1", "Simple Header\n============="),
            ("Simple H2", "Simple Header\n-------------"),
            (
                "Complex H1",
                "Header with **bold** text\n========================",
            ),
            ("Multiline", "Line 1\nLine 2\n======"),
        ]
    } else {
        vec![("User Input", input)]
    };

    println!("=== Setext Header Debug ===");

    for (name, test_input) in test_cases {
        println!("\n--- Testing: {} ---", name);
        println!("Input: {:?}", test_input);

        // Test grammar parsing specifically for setext
        let setext_rule = if test_input.contains("=====") {
            Rule::setext_h1
        } else {
            Rule::setext_h2
        };

        match parse_with_rule(test_input, setext_rule) {
            Ok(pairs) => {
                println!("✓ Setext grammar parsing succeeded");
                print_parser_structure(pairs, 1);

                // Test full pipeline
                match parse_to_html_cached(test_input) {
                    Ok(html) => {
                        println!("HTML: {}", html);
                        if html.contains("=====") || html.contains("-----") {
                            println!("❌ PROBLEM: HTML contains underlines");
                        } else {
                            println!("✅ HTML looks clean");
                        }
                    }
                    Err(e) => println!("HTML generation failed: {}", e),
                }
            }
            Err(e) => {
                println!("✗ Setext grammar parsing failed: {}", e);
            }
        }
    }
}

#[cfg(feature = "integration-tests")]
fn print_parser_structure(pairs: Pairs<Rule>, base_indent: usize) {
    for pair in pairs {
        print_pair_structure(pair, base_indent);
    }
}

#[cfg(feature = "integration-tests")]
fn print_pair_structure(pair: Pair<Rule>, indent: usize) {
    let indent_str = "  ".repeat(indent);
    println!(
        "{}Rule: {:?}, Text: {:?}",
        indent_str,
        pair.as_rule(),
        pair.as_str()
    );

    for inner_pair in pair.into_inner() {
        print_pair_structure(inner_pair, indent + 1);
    }
}

#[cfg(not(feature = "integration-tests"))]
fn main() {
    eprintln!("This binary requires the 'integration-tests' feature to be enabled.");
    eprintln!("Run with: cargo run --bin marco-parser-debug --features integration-tests");
    std::process::exit(1);
}
