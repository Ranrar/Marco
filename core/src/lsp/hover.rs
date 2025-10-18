// Hover information: show link URLs, image alt text, etc.

use crate::parser::Position;

#[derive(Debug, Clone)]
pub struct HoverInfo {
    pub contents: String,
    pub range: Option<crate::parser::Span>,
}

// Provide hover information at position
pub fn get_hover_info(position: Position, document: &crate::parser::Document) -> Option<HoverInfo> {
    log::debug!("Computing hover info at {:?}", position);
    
    // TODO: Find node at position and extract hover information
    
    None
}
