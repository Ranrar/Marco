// AST traversal utilities for depth-first and breadth-first operations

use super::{Document, Node};

// Depth-first traversal visitor pattern
pub trait Visitor {
    fn visit_node(&mut self, node: &Node);
}

// Traverse AST depth-first
pub fn walk_dfs(document: &Document, visitor: &mut dyn Visitor) {
    log::debug!("DFS traversal starting");
    for node in &document.children {
        visit_node(node, visitor);
    }
}

fn visit_node(node: &Node, visitor: &mut dyn Visitor) {
    visitor.visit_node(node);
    for child in &node.children {
        visit_node(child, visitor);
    }
}

// Breadth-first traversal
pub fn walk_bfs(document: &Document, visitor: &mut dyn Visitor) {
    log::debug!("BFS traversal starting");
    let mut queue: Vec<&Node> = document.children.iter().collect();
    
    while let Some(node) = queue.pop() {
        visitor.visit_node(node);
        queue.extend(&node.children);
    }
}
