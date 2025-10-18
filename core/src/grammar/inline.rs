// Inline-level grammar: emphasis, strong, links, images, code spans, inline HTML

use nom::{IResult, bytes::complete::tag};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

// Emphasis parser (*text* or _text_)
pub fn emphasis(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing emphasis: {:?}", input.fragment());
    tag("*")(input)
}

// Strong emphasis parser (**text** or __text__)
pub fn strong(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing strong emphasis");
    // TODO: Implement strong parsing
    Ok((input, input))
}

// Link parser ([text](url))
pub fn link(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing link");
    // TODO: Implement link parsing
    Ok((input, input))
}

// Image parser (![alt](url))
pub fn image(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing image");
    // TODO: Implement image parsing
    Ok((input, input))
}

// Code span parser (`code`)
pub fn code_span(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing code span");
    // TODO: Implement code span parsing
    Ok((input, input))
}

// Inline HTML parser
pub fn inline_html(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing inline HTML");
    // TODO: Implement inline HTML parsing
    Ok((input, input))
}
