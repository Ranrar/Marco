use marco::components::marco_engine::ast::{Node, Span};
use std::mem;

#[cfg(test)]
mod memory_tests {
    use super::*;

    #[test]
    fn test_node_enum_size() {
        let node_size = mem::size_of::<Node>();
        let span_size = mem::size_of::<Span>();

        println!("Node enum size: {} bytes", node_size);
        println!("Span size: {} bytes", span_size);

        // Current baseline - we'll improve this
        // Target: reduce Node size to under 128 bytes
        println!(
            "Current Node size: {} bytes (target: <128 bytes)",
            node_size
        );

        // Print size of largest variants for analysis
        let document_size = mem::size_of::<(Vec<Node>, Span)>();
        let table_size = mem::size_of::<(Vec<Node>, Vec<Vec<Node>>, Span)>();

        println!("Document variant estimated size: {} bytes", document_size);
        println!("Table variant estimated size: {} bytes", table_size);
    }

    #[test]
    fn test_span_efficiency() {
        let span = Span::new(0, 100, 1, 1);
        let span_size = mem::size_of_val(&span);

        println!("Individual Span size: {} bytes", span_size);

        // Test if we can optimize with smaller integer types
        let u32_tuple_size = mem::size_of::<(u32, u32, u32, u32)>();
        println!("u32-based span would be: {} bytes", u32_tuple_size);

        assert!(span_size <= 32, "Span should not exceed 32 bytes");
    }

    #[test]
    fn test_node_variants_memory() {
        // Test memory usage of common node types
        let text_node = Node::text("Hello, World!", Span::simple(0, 13));
        let paragraph_node = Node::paragraph(vec![text_node.clone()], Span::simple(0, 13));
        let document_node = Node::document(vec![paragraph_node], Span::simple(0, 13));

        println!(
            "Text node size in memory: {} bytes",
            mem::size_of_val(&text_node)
        );
        println!(
            "Document node size in memory: {} bytes",
            mem::size_of_val(&document_node)
        );

        // All nodes should be the same size due to enum layout
        assert_eq!(
            mem::size_of_val(&text_node),
            mem::size_of_val(&document_node)
        );
    }
}
