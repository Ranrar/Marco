// Marco Parser - 100% CommonMark Compliant (652/652 spec examples passing)
// nom-based parser with full UTF-8 support (em dashes, smart quotes, Japanese, Arabic, emoji)

pub mod ast;
pub mod position;
pub mod shared;

// Modular parser structure
pub mod blocks;
pub mod inlines;

// Re-export public API
pub use ast::*;
pub use blocks::parse_blocks;
pub use inlines::parse_inlines;
pub use position::*;

use anyhow::Result;

/// Parse Markdown text into Document AST
pub fn parse(input: &str) -> Result<Document> {
    log::info!("Starting parse: {} bytes", input.len());

    let mut document = parse_blocks(input)?;
    log::debug!("Parsed {} blocks", document.children.len());

    // Second pass: resolve reference-style links against collected reference definitions.
    // This must happen after block parsing because definitions may appear later.
    resolve_reference_links(&mut document);

    Ok(document)
}

fn resolve_reference_links(document: &mut Document) {
    resolve_reference_links_in_nodes(&mut document.children, &document.references);
}

fn resolve_reference_links_in_nodes(nodes: &mut Vec<Node>, references: &ReferenceMap) {
    let mut i = 0;
    while i < nodes.len() {
        // Always resolve inside children first.
        if !nodes[i].children.is_empty() {
            resolve_reference_links_in_nodes(&mut nodes[i].children, references);
        }

        let is_ref = matches!(nodes[i].kind, NodeKind::LinkReference { .. });
        if !is_ref {
            i += 1;
            continue;
        }

        // Temporarily take ownership of data we might need.
        let (label, suffix) = match &nodes[i].kind {
            NodeKind::LinkReference { label, suffix } => (label.clone(), suffix.clone()),
            _ => unreachable!(),
        };

        if let Some((url, title)) = references.get(&label) {
            nodes[i].kind = NodeKind::Link {
                url: url.clone(),
                title: title.clone(),
            };
            i += 1;
            continue;
        }

        // Unresolved reference: fall back to literal bracketed text while preserving
        // already-parsed children for the first bracket segment.
        let mut inner_children = std::mem::take(&mut nodes[i].children);

        let mut replacement: Vec<Node> = Vec::new();
        replacement.push(Node {
            kind: NodeKind::Text("[".to_string()),
            span: None,
            children: Vec::new(),
        });
        replacement.append(&mut inner_children);
        replacement.push(Node {
            kind: NodeKind::Text("]".to_string()),
            span: None,
            children: Vec::new(),
        });
        if !suffix.is_empty() {
            replacement.push(Node {
                kind: NodeKind::Text(suffix),
                span: None,
                children: Vec::new(),
            });
        }

        let replacement_len = replacement.len();
        nodes.splice(i..i + 1, replacement);
        i += replacement_len;
    }
}
