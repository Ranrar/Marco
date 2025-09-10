use super::{AstBuilder, BuilderHelpers, ErrorHandling};
use crate::components::marco_engine::{
    ast::{Node, Span},
    errors::MarcoResult,
    grammar::Rule,
};
use log::debug;
use pest::iterators::Pair;

// Constants for table alignments
const ALIGN_CENTER: &str = "center";
const ALIGN_RIGHT: &str = "right";
const ALIGN_LEFT: &str = "left";

/// Trait for building table AST nodes
pub trait TableBuilder: BuilderHelpers + ErrorHandling {
    /// Build table nodes
    fn build_table(pair: Pair<Rule>, span: Span) -> MarcoResult<Node> {
        debug!("TableBuilder::build_table - Processing table");
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        let table_text = pair.as_str(); // Store before consuming

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::table_header => {
                    headers = Self::build_table_row_cells(inner_pair)?;
                }
                Rule::table_sep => {
                    // Note: alignments are parsed but not stored in the current Table node structure
                    let _alignments = Self::parse_table_alignments(inner_pair);
                }
                Rule::table_row => {
                    let row_cells = Self::build_table_row_cells(inner_pair)?;
                    rows.push(row_cells);
                }
                _ => {
                    debug!(
                        "TableBuilder::build_table - Unexpected rule: {:?}",
                        inner_pair.as_rule()
                    );
                }
            }
        }

        // Fallback: parse table from raw text if needed
        if headers.is_empty() && rows.is_empty() {
            let (parsed_headers, _parsed_alignments, parsed_rows) =
                Self::parse_table_from_text(table_text, span.clone());
            headers = parsed_headers;
            rows = parsed_rows;
            // Note: _parsed_alignments are not stored in current Table node structure
        }

        // Validate table size for performance
        let total_cells = headers.len() + rows.len() * headers.len(); // Approximate cell count

        if let Err(e) = Self::validate_table_size(total_cells) {
            log::warn!("Table size validation failed: {}, using text fallback", e);
            return Ok(Node::text(table_text.to_string(), span));
        }

        Ok(Node::table(headers, rows, span))
    }

    /// Build table row cells
    fn build_table_row_cells(pair: Pair<Rule>) -> MarcoResult<Vec<Node>> {
        debug!("TableBuilder::build_table_row_cells - Processing table row");
        let mut cells = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::table_cell => {
                    let cell_text = inner_pair.as_str().trim();
                    let cell_span = Self::create_span(&inner_pair);
                    match Self::build_table_cell(inner_pair) {
                        Ok(cell) => cells.push(cell),
                        Err(e) => {
                            debug!(
                                "TableBuilder::build_table_row_cells - Error building cell: {}",
                                e
                            );
                            // Create fallback cell
                            cells.push(Self::create_text_node(cell_text, cell_span));
                        }
                    }
                }
                _ => {
                    debug!(
                        "TableBuilder::build_table_row_cells - Unexpected rule: {:?}",
                        inner_pair.as_rule()
                    );
                }
            }
        }

        Ok(cells)
    }

    /// Build individual table cell
    fn build_table_cell(pair: Pair<Rule>) -> MarcoResult<Node> {
        debug!("TableBuilder::build_table_cell - Processing table cell");
        let span = Self::create_span(&pair);
        let cell_text = pair.as_str().trim(); // Store before consuming
        let mut content = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::table_cell_content => {
                    // Process cell content which can contain inline elements
                    for content_pair in inner_pair.into_inner() {
                        let text = content_pair.as_str();
                        let span = Self::create_span(&content_pair);
                        match AstBuilder::build_node(content_pair) {
                            Ok(node) => content.push(node),
                            Err(_) => {
                                // Fallback to text node
                                content.push(Self::create_text_node(text, span));
                            }
                        }
                    }
                }
                Rule::table_safe_text => {
                    let text = inner_pair.as_str().trim();
                    if !text.is_empty() {
                        content.push(Self::create_text_node(text, Self::create_span(&inner_pair)));
                    }
                }
                _ => {
                    // Process other inline content
                    let text = inner_pair.as_str();
                    let span = Self::create_span(&inner_pair);
                    match AstBuilder::build_node(inner_pair) {
                        Ok(node) => content.push(node),
                        Err(_) => {
                            content.push(Self::create_text_node(text, span));
                        }
                    }
                }
            }
        }
        // Fallback: create text content from the entire cell if no content was found
        if content.is_empty() && !cell_text.is_empty() {
            content.push(Self::create_text_node(cell_text, span.clone()));
        }

        Ok(Node::table_cell(content, None, span))
    }

    /// Parse table alignments from separator row
    fn parse_table_alignments(pair: Pair<Rule>) -> Vec<&'static str> {
        debug!("TableBuilder::parse_table_alignments - Processing table separator");
        let mut alignments = Vec::new();

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::table_sep_cell => {
                    let sep_text = inner_pair.as_str().trim();
                    let alignment = if sep_text.starts_with(':') && sep_text.ends_with(':') {
                        ALIGN_CENTER
                    } else if sep_text.ends_with(':') {
                        ALIGN_RIGHT
                    } else {
                        ALIGN_LEFT
                    };
                    alignments.push(alignment);
                }
                _ => {
                    debug!(
                        "TableBuilder::parse_table_alignments - Unexpected rule: {:?}",
                        inner_pair.as_rule()
                    );
                }
            }
        }

        alignments
    }

    /// Parse table from raw text as fallback
    fn parse_table_from_text(
        text: &str,
        span: Span,
    ) -> (Vec<Node>, Vec<&'static str>, Vec<Vec<Node>>) {
        debug!("TableBuilder::parse_table_from_text - Parsing table from raw text");
        let lines: Vec<&str> = text.lines().collect();
        let mut headers = Vec::new();
        let mut alignments = Vec::new();
        let mut rows = Vec::new();

        if lines.len() < 2 {
            return (headers, alignments, rows);
        }

        // Parse header row
        if let Some(header_line) = lines.first() {
            headers = Self::parse_table_line_cells(header_line, span.clone());
        }

        // Parse separator row for alignments
        if lines.len() > 1 {
            if let Some(sep_line) = lines.get(1) {
                alignments = Self::parse_alignment_from_line(sep_line);
            }
        }

        // Parse data rows
        for line in lines.iter().skip(2) {
            let row_cells = Self::parse_table_line_cells(line, span.clone());
            if !row_cells.is_empty() {
                rows.push(row_cells);
            }
        }

        (headers, alignments, rows)
    }

    /// Parse cells from a table line
    fn parse_table_line_cells(line: &str, span: Span) -> Vec<Node> {
        debug!(
            "TableBuilder::parse_table_line_cells - Parsing table line: {}",
            line
        );
        let mut cells = Vec::new();

        // Split by | and clean up
        let parts: Vec<&str> = line.split('|').collect();

        for (i, part) in parts.iter().enumerate() {
            let trimmed = part.trim();

            // Skip empty parts at the beginning and end (from leading/trailing |)
            if (i == 0 || i == parts.len() - 1) && trimmed.is_empty() {
                continue;
            }

            if !trimmed.is_empty() {
                cells.push(Node::table_cell(
                    vec![Self::create_text_node(trimmed, span.clone())],
                    None, // Default alignment
                    span.clone(),
                ));
            }
        }

        cells
    }

    /// Parse alignment from separator line
    fn parse_alignment_from_line(line: &str) -> Vec<&'static str> {
        debug!(
            "TableBuilder::parse_alignment_from_line - Parsing alignment line: {}",
            line
        );
        let mut alignments = Vec::new();

        // Split by | and analyze each part
        let parts: Vec<&str> = line.split('|').collect();

        for (i, part) in parts.iter().enumerate() {
            let trimmed = part.trim();

            // Skip empty parts at the beginning and end
            if (i == 0 || i == parts.len() - 1) && trimmed.is_empty() {
                continue;
            }

            let alignment = if trimmed.starts_with(':') && trimmed.ends_with(':') {
                ALIGN_CENTER
            } else if trimmed.ends_with(':') {
                ALIGN_RIGHT
            } else {
                ALIGN_LEFT
            };

            alignments.push(alignment);
        }

        alignments
    }
}
