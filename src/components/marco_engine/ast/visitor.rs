use crate::components::marco_engine::ast::Node;

/// Trait for visiting AST nodes
pub trait Visitor {
    type Output;
    
    fn visit(&mut self, node: &Node) -> Self::Output;
    
    fn visit_document(&mut self, children: &[Node]) -> Self::Output {
        for child in children {
            self.visit(child);
        }
        self.default_output()
    }
    
    fn visit_heading(&mut self, level: u8, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }
    
    fn visit_paragraph(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }
    
    fn visit_code_block(&mut self, language: &Option<String>, content: &str) -> Self::Output {
        self.default_output()
    }
    
    fn visit_math_block(&mut self, content: &str) -> Self::Output {
        self.default_output()
    }
    
    fn visit_list(&mut self, ordered: bool, items: &[Node]) -> Self::Output {
        for item in items {
            self.visit(item);
        }
        self.default_output()
    }
    
    fn visit_list_item(&mut self, content: &[Node], checked: &Option<bool>) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }
    
    fn visit_table(&mut self, headers: &[Node], rows: &[Vec<Node>]) -> Self::Output {
        for header in headers {
            self.visit(header);
        }
        for row in rows {
            for cell in row {
                self.visit(cell);
            }
        }
        self.default_output()
    }
    
    fn visit_text(&mut self, content: &str) -> Self::Output {
        self.default_output()
    }
    
    fn visit_emphasis(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }
    
    fn visit_strong(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }
    
    fn visit_code(&mut self, content: &str) -> Self::Output {
        self.default_output()
    }
    
    fn visit_link(&mut self, text: &[Node], url: &str, title: &Option<String>) -> Self::Output {
        for child in text {
            self.visit(child);
        }
        self.default_output()
    }
    
    fn visit_image(&mut self, alt: &str, url: &str, title: &Option<String>) -> Self::Output {
        self.default_output()
    }
    
    fn visit_macro(&mut self, name: &str, arguments: &[String], content: &Option<Vec<Node>>) -> Self::Output {
        if let Some(content) = content {
            for child in content {
                self.visit(child);
            }
        }
        self.default_output()
    }
    
    fn visit_horizontal_rule(&mut self) -> Self::Output {
        self.default_output()
    }
    
    fn visit_block_quote(&mut self, content: &[Node]) -> Self::Output {
        for child in content {
            self.visit(child);
        }
        self.default_output()
    }
    
    fn visit_unknown(&mut self, content: &str, rule: &str) -> Self::Output {
        self.default_output()
    }
    
    fn default_output(&self) -> Self::Output;
}

/// Trait for mutating visitors that can modify the AST
pub trait VisitorMut {
    type Output;
    
    fn visit_mut(&mut self, node: &mut Node) -> Self::Output;
    
    fn visit_document_mut(&mut self, children: &mut Vec<Node>) -> Self::Output {
        for child in children {
            self.visit_mut(child);
        }
        self.default_output()
    }
    
    fn visit_heading_mut(&mut self, level: &mut u8, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }
    
    fn visit_paragraph_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }
    
    fn visit_code_block_mut(&mut self, language: &mut Option<String>, content: &mut String) -> Self::Output {
        self.default_output()
    }
    
    fn visit_math_block_mut(&mut self, content: &mut String) -> Self::Output {
        self.default_output()
    }
    
    fn visit_list_mut(&mut self, ordered: &mut bool, items: &mut Vec<Node>) -> Self::Output {
        for item in items {
            self.visit_mut(item);
        }
        self.default_output()
    }
    
    fn visit_text_mut(&mut self, content: &mut String) -> Self::Output {
        self.default_output()
    }
    
    fn visit_emphasis_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }
    
    fn visit_strong_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }
    
    fn visit_code_mut(&mut self, content: &mut String) -> Self::Output {
        self.default_output()
    }
    
    fn visit_link_mut(&mut self, text: &mut Vec<Node>, url: &mut String, title: &mut Option<String>) -> Self::Output {
        for child in text {
            self.visit_mut(child);
        }
        self.default_output()
    }
    
    fn visit_image_mut(&mut self, alt: &mut String, url: &mut String, title: &mut Option<String>) -> Self::Output {
        self.default_output()
    }
    
    fn visit_macro_mut(&mut self, name: &mut String, arguments: &mut Vec<String>, content: &mut Option<Vec<Node>>) -> Self::Output {
        if let Some(content) = content {
            for child in content {
                self.visit_mut(child);
            }
        }
        self.default_output()
    }
    
    fn visit_block_quote_mut(&mut self, content: &mut Vec<Node>) -> Self::Output {
        for child in content {
            self.visit_mut(child);
        }
        self.default_output()
    }
    
    fn visit_unknown_mut(&mut self, content: &mut String, rule: &mut String) -> Self::Output {
        self.default_output()
    }
    
    fn default_output(&self) -> Self::Output;
}
