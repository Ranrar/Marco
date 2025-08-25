use proptest::prelude::*;
use ron::Value;
use tool_runner::loader;
use tool_runner::validate;
use tool_runner::types::{AstRoot, SyntaxRoot};

fn make_ron_ast_from_kinds(kinds: &[String]) -> AstRoot {
    // Build a RON string using the AstRoot struct with children as simple strings
    let mut s = String::from("AstRoot(type: Some(\"root\"), children: [");
    for (i, k) in kinds.iter().enumerate() {
        if i > 0 { s.push_str(", "); }
        s.push_str(&format!("\"{}\"", k));
    }
    s.push_str("])\n");
    loader::load_ron_from_str::<AstRoot>(&s).unwrap()
}

fn make_ron_syntax_from_kinds(kinds: &[String], grammar_refs: &[(String, Option<String>)]) -> SyntaxRoot {
    let mut s = String::from("SyntaxRoot(type: Some(\"root\"), children: [");
    let mut first = true;
    for k in kinds {
        if !first { s.push_str(", "); } else { first = false; }
        s.push_str(&format!("\"{}\"", k));
    }
    for (k, r) in grammar_refs {
        if !first { s.push_str(", "); } else { first = false; }
        if let Some(rule) = r {
                // represent a reference as a small inline map so we can include grammar_rule
                s.push_str(&format!("(type: \"{}\", grammar_rule: \"{}\")", k, rule));
        } else {
            s.push_str(&format!("\"{}\"", k));
        }
    }
    s.push_str("])\n");
    loader::load_ron_from_str::<SyntaxRoot>(&s).unwrap()
}

fn make_pest_grammar(rule_names: &[String]) -> String {
    // build a minimal grammar that defines rules with the given names
    let mut s = String::from("WHITESPACE = _{ \" \" | \"\t\" }\nNEWLINE = _{ \"\\n\" }\n");
    s.push_str("document = { SOI ~ NEWLINE* ~ EOI }\n");
    for r in rule_names {
        // simple rule: rule = @{ ASCII_ALPHANUMERIC+ }
        s.push_str(&format!("{} = @{{ ASCII_ALPHANUMERIC+ }}\n", r));
    }
    s
}

proptest! {
    #[test]
    fn prop_valid_ast_syntax((kinds, grammar_rules) in (prop::collection::vec("[a-z]{1,8}", 1..6), prop::collection::vec("[a-z]{1,8}", 0..4))) {
        // create overlapping sets
        let mut unique_kinds: Vec<String> = kinds.into_iter().collect();
        unique_kinds.sort(); unique_kinds.dedup();

        let grammar_rules: Vec<String> = grammar_rules.into_iter().collect();

        let ast = make_ron_ast_from_kinds(&unique_kinds);
        let syntax = make_ron_syntax_from_kinds(&unique_kinds, &[]);
    let grammar_text = make_pest_grammar(&grammar_rules);
    let rules = loader::extract_rule_names_from_str(&grammar_text);

    // Should succeed because AST kinds are present in syntax kinds
    let res = validate::validate(&ast, &syntax, &rules);
    match res {
        Ok(()) => prop_assert!(true),
        Err(e) => prop_assert!(false, "validation errors: {:?}", e),
    }
    }

    #[test]
    fn prop_missing_kind((ast_kinds, syntax_kinds) in (prop::collection::vec("[a-z]{1,8}", 1..6), prop::collection::vec("[a-z]{1,8}", 0..5))) {
        let mut a = ast_kinds.into_iter().collect::<Vec<_>>(); a.sort(); a.dedup();
        let mut s = syntax_kinds.into_iter().collect::<Vec<_>>(); s.sort(); s.dedup();

        // ensure at least one AST kind not in syntax
        if s.is_empty() || a.iter().all(|x| s.contains(x)) {
            // add a distinct element to AST
            a.push("zz_unique_kind".to_string());
        }

        let ast = make_ron_ast_from_kinds(&a);
        let syntax = make_ron_syntax_from_kinds(&s, &[]);
    let grammar_text = "WHITESPACE = _{ \" \" }\ndocument = { SOI ~ EOI }\n";
    let rules = loader::extract_rule_names_from_str(grammar_text);

    let res = validate::validate(&ast, &syntax, &rules);
        prop_assert!(res.is_err());
    }
}
