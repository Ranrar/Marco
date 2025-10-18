// Block-level grammar: headings, paragraphs, lists, code blocks, blockquotes, tables

use nom::{IResult, bytes::complete::tag};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

// Heading parser (# Title)
pub fn heading(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing heading: {:?}", input.fragment());
    tag("#")(input)
}

// Paragraph parser
pub fn paragraph(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing paragraph: {:?}", input.fragment());
    // TODO: Implement paragraph parsing
    Ok((input, input))
}

// Code block parser (``` or indented)
pub fn code_block(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing code block");
    // TODO: Implement code block parsing
    Ok((input, input))
}

// List parser (ordered and unordered)
pub fn list(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing list");
    // TODO: Implement list parsing
    Ok((input, input))
}

// Blockquote parser (>)
pub fn blockquote(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing blockquote");
    // TODO: Implement blockquote parsing
    Ok((input, input))
}

// Table parser
pub fn table(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing table");
    // TODO: Implement table parsing
    Ok((input, input))
}
