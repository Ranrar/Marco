//! Parser entry points and AST-facing parser modules.
//!
//! The parser layer consumes grammar outputs and builds the crate AST.

/// AST node and document types.
pub mod ast;
/// Position and span utilities.
pub mod position;
/// Shared parser span conversion helpers.
pub mod shared;

/// Block-level parser modules.
pub mod blocks;
/// Inline-level parser modules.
pub mod inlines;

/// Re-export AST types.
pub use ast::*;
/// Re-export block parser entry point.
pub use blocks::parse_blocks;
/// Re-export inline parser entry point.
pub use inlines::parse_inlines;
/// Re-export position and span types.
pub use position::*;

/// Runtime configuration for the Markdown parser.
///
/// Pass to [`parse_with_options`] to skip expensive hot-path work in
/// performance-sensitive pipelines.
///
/// All fields default to `true` (full-featured parse). Set individual fields
/// to `false` to opt out of that work at runtime (not just at compile time).
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// Track source positions (line/column spans) on every AST node.
    ///
    /// When `false`, the O(n) string scans inside span conversion are skipped
    /// and all node `span` fields will be `None`. Use this for render-only
    /// pipelines that never inspect positions.
    ///
    /// Default: `true`.
    pub track_positions: bool,

    /// Parse inline `$...$` / `$$...$$` math and fenced ` ```math ` blocks.
    ///
    /// When `false`, math syntax falls through to plain text or regular code
    /// blocks. Skips the math parser attempts in the inline hot loop.
    ///
    /// Default: `true`.
    pub parse_math: bool,

    /// Parse fenced ` ```mermaid ` code blocks into `NodeKind::MermaidDiagram`.
    ///
    /// When `false`, mermaid blocks are emitted as regular `NodeKind::CodeBlock`
    /// nodes. Skips the diagram branch in the fenced-code-block parser.
    ///
    /// Default: `true`.
    pub parse_diagrams: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            track_positions: true,
            parse_math: true,
            parse_diagrams: true,
        }
    }
}

/// Parse Markdown text with runtime options controlling which work is performed.
///
/// This is the high-performance entry point. Pass a [`ParseOptions`] with
/// fields set to `false` to skip expensive hot-path work at runtime.
///
/// For the default full-featured parse, use [`parse`] instead.
///
/// # Example
/// ```rust
/// let opts = marco_core::ParseOptions {
///     track_positions: false,
///     ..Default::default()
/// };
/// let doc = marco_core::parse_with_options("# Hello", opts)?;
/// // All node spans are None — position computation was skipped.
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_with_options(
    input: &str,
    opts: ParseOptions,
) -> Result<Document, Box<dyn std::error::Error>> {
    log::info!("Starting parse: {} bytes", input.len());

    // Set thread-local options for the duration of this parse call.
    // The guard restores previous values on drop (including on error).
    let _guard =
        shared::ParseOptionsGuard::new(opts.track_positions, opts.parse_math, opts.parse_diagrams);

    let mut document = parse_blocks(input)?;
    log::debug!("Parsed {} blocks", document.children.len());

    resolve_reference_links(&mut document);
    blocks::gfm_admonitions::apply_gfm_admonitions(&mut document);

    Ok(document)
}

/// Parse Markdown text into Document AST using default options (full-featured).
pub fn parse(input: &str) -> Result<Document, Box<dyn std::error::Error>> {
    parse_with_options(input, ParseOptions::default())
}

fn resolve_reference_links(document: &mut Document) {
    resolve_reference_links_in_nodes(&mut document.children, &document.references);
}

fn unescape_commonmark_backslash_escapes(input: &str) -> String {
    // CommonMark escapable punctuation set.
    const ESCAPABLE: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next) = chars.peek() {
                if ESCAPABLE.contains(next) {
                    out.push(next);
                    chars.next();
                    continue;
                }
            }
        }

        out.push(ch);
    }

    out
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
                kind: NodeKind::Text(unescape_commonmark_backslash_escapes(&suffix)),
                span: None,
                children: Vec::new(),
            });
        }

        let replacement_len = replacement.len();
        nodes.splice(i..i + 1, replacement);
        i += replacement_len;
    }
}
