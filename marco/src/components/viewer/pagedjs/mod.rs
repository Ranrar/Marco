//! Embedded paged.js polyfill for page view simulation.
//!
//! paged.js implements the CSS Paged Media specification (W3C), restructuring
//! the document DOM into fixed-size page boxes with accurate page breaks,
//! margins, running headers/footers, and page numbering.
//!
//! Version: 0.5.0-beta.2
//! License: MIT — see LICENSE.md in this directory.
pub const PAGED_POLYFILL_JS: &str = include_str!("paged.polyfill.min.js");
