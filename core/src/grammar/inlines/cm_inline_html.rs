//! Inline HTML grammar
use nom::{IResult, bytes::complete::{tag, take_until}};
use super::Span;

pub fn inline_html(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing inline HTML at: {:?}", input.fragment());
    let (input, _) = tag("<")(input)?;
    let (input, content) = take_until(">")(input)?;
    let (input, _) = tag(">")(input)?;
    Ok((input, content))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smoke_test_inline_html() {
        let input = Span::new("<span>text</span>");
        let result = inline_html(input);
        assert!(result.is_ok());
    }
}
