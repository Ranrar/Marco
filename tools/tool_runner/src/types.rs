use ron::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AstRoot {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub children: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyntaxRoot {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub children: Vec<Value>,
}

// We intentionally keep Node definitions untyped (ron::Value) to avoid
// requiring exact schema knowledge from the RON files. Tests will operate
// on the Value shapes and extract fields logically.

pub type Node = Value;

impl AstRoot {
    pub fn from_value(v: &Value) -> Option<Self> {
        // Expect a tuple-like or map-like structure with children
        if let Value::Map(map) = v {
            // look for key "children"
            for (k, val) in map.iter() {
                if let Value::String(key) = k {
                    if key == "children" {
                        if let Value::Seq(seq) = val {
                            return Some(AstRoot {
                                type_: None,
                                children: seq.clone(),
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

impl SyntaxRoot {
    pub fn from_value(v: &Value) -> Option<Self> {
        if let Value::Map(map) = v {
            for (k, val) in map.iter() {
                if let Value::String(key) = k {
                    if key == "children" {
                        if let Value::Seq(seq) = val {
                            return Some(SyntaxRoot {
                                type_: None,
                                children: seq.clone(),
                            });
                        }
                    }
                }
            }
        }
        None
    }
}
