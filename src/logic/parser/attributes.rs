use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attributes {
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub data: HashMap<String, String>,
    // Optionally: style, aria, etc. can be added later
}

impl Attributes {
    pub fn new() -> Self {
        Self::default()
    }
}
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attributes {
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub data: HashMap<String, String>,
    // Optionally: style, aria, etc. can be added later
}

impl Attributes {
    pub fn new() -> Self {
        Self::default()
    }
}
