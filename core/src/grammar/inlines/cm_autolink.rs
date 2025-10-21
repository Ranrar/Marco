//! Autolink grammar - <url> or <email>
use nom::{IResult, bytes::complete::{tag, take_while1}};
use super::Span;

pub fn autolink(input: Span) -> IResult<Span, (Span, bool)> {
    log::debug!("Parsing autolink at: {:?}", input.fragment());
    let (input, _) = tag("<")(input)?;
    let (input, url) = take_while1(|c: char| c != '>')(input)?;
    let (input, _) = tag(">")(input)?;
    let is_email = url.fragment().contains('@');
    Ok((input, (url, is_email)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smoke_test_autolink() {
        let input = Span::new("<http://example.com>");
        let result = autolink(input);
        assert!(result.is_ok());
    }
}
