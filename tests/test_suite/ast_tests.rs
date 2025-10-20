// AST tests: validate tree structure and traversal

#[cfg(test)]
mod tests {
    use core::ast::{Document, Node, NodeKind, traversal};
    
    #[test]
    fn test_document_creation() {
        let doc = Document::new();
        assert_eq!(doc.len(), 0);
        log::info!("Document creation test passed");
    }
    
    #[test]
    fn test_ast_traversal_dfs() {
        let doc = Document::new();
        
        struct TestVisitor;
        impl traversal::Visitor for TestVisitor {
            fn visit_node(&mut self, _node: &Node) {
                log::trace!("Visiting node");
            }
        }
        
        let mut visitor = TestVisitor;
        traversal::walk_dfs(&doc, &mut visitor);
        log::info!("DFS traversal test passed");
    }
    
    #[test]
    fn test_ast_traversal_bfs() {
        let doc = Document::new();
        
        struct TestVisitor;
        impl traversal::Visitor for TestVisitor {
            fn visit_node(&mut self, _node: &Node) {
                log::trace!("Visiting node");
            }
        }
        
        let mut visitor = TestVisitor;
        traversal::walk_bfs(&doc, &mut visitor);
        log::info!("BFS traversal test passed");
    }
}
