use crate::errors::ValidationError;
use crate::types::{AstRoot, SyntaxRoot};
use regex::Regex;
use ron::ser::to_string as ron_to_string;
use ron::Value;
use serde_json;
use std::collections::HashSet;

pub fn validate(
    ast: &AstRoot,
    syntax: &SyntaxRoot,
    grammar_rules: &Vec<String>,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Collect kind names from syntax.ron: walk children and extract `type` or node variant names
    let syntax_kinds = collect_types_from_children(&syntax.children);

    // Collect node kinds from ast.ron children
    let ast_kinds = collect_types_from_children(&ast.children);

    // Invariant: every AST kind should exist in syntax kinds
    for k in &ast_kinds {
        if !syntax_kinds.contains(k) {
            errors.push(ValidationError::missing(format!(
                "AST kind '{}' not defined in syntax.ron",
                k
            )));
        }
    }

    // Grammar rule names
    // Note: grammar-based checks are handled by tests using a separate extractor. For now, skip.
    for child in &syntax.children {
        if let Some(rule_name) = extract_string_field(child, "grammar_rule") {
            if !grammar_rules.contains(&rule_name) {
                errors.push(ValidationError::Grammar(format!(
                    "syntax node references missing grammar rule '{}'",
                    rule_name
                )));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn collect_types_from_children(children: &Vec<Value>) -> HashSet<String> {
    let mut set = HashSet::new();
    let re = Regex::new(r#"type\s*:\s*\"([^\"]+)\""#).unwrap();
    for v in children {
        // if child is a plain string representing the kind
        if let Value::String(s) = v {
            set.insert(s.clone());
            continue;
        }
        // try direct structured extraction first
        if let Value::Map(map) = v {
            for (k, val) in map.iter() {
                if let Value::String(key) = k {
                    if key == "type" {
                        if let Value::String(s) = val {
                            set.insert(s.clone());
                        }
                    }
                }
            }
        }
        // fallback: try converting to serde_json value and read a "type" key
        if let Ok(json_val) = serde_json::to_value(v) {
            if let serde_json::Value::Object(map) = json_val {
                if let Some(serde_json::Value::String(sv)) = map.get("type") {
                    set.insert(sv.clone());
                }
            } else if let Ok(s) = ron_to_string(v) {
                if let Some(cap) = re.captures(&s) {
                    if let Some(m) = cap.get(1) {
                        set.insert(m.as_str().to_string());
                    }
                }
            }
        }
    }
    set
}

fn extract_string_field(v: &Value, field: &str) -> Option<String> {
    if let Value::Map(map) = v {
        for (k, val) in map.iter() {
            if let Value::String(key) = k {
                if key == field {
                    if let Value::String(s) = val {
                        return Some(s.clone());
                    }
                }
            }
        }
    }
    None
}
