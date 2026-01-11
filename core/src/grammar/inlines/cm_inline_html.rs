//! Inline HTML grammar
use super::Span;
use nom::IResult;
use nom::{Input, Parser};

pub fn inline_html(input: Span) -> IResult<Span, Span> {
    use nom::{
        bytes::complete::{tag, take_while},
        character::complete::{alpha1, char},
        combinator::{opt, recognize},
        sequence::pair,
    };
    log::debug!("Parsing inline HTML at: {:?}", input.fragment());
    fn is_tagname_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-'
    }
    // Whitelist of valid HTML tags
    const HTML_TAGS: &[&str] = &[
        "img",
        "span",
        "div",
        "br",
        "hr",
        "table",
        "tr",
        "td",
        "th",
        "thead",
        "tbody",
        "tfoot",
        "ul",
        "ol",
        "li",
        "a",
        "p",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "pre",
        "code",
        "blockquote",
        "em",
        "strong",
        "b",
        "i",
        "u",
        "s",
        "del",
        "ins",
        "sup",
        "sub",
        "form",
        "input",
        "label",
        "select",
        "option",
        "textarea",
        "button",
        "script",
        "style",
        "link",
        "meta",
        "head",
        "body",
        "html",
        "section",
        "article",
        "aside",
        "footer",
        "header",
        "nav",
        "main",
        "figure",
        "figcaption",
        "canvas",
        "svg",
        "video",
        "audio",
        "source",
        "iframe",
        "object",
        "embed",
        "param",
        "picture",
        "map",
        "area",
        "details",
        "summary",
        "mark",
        "cite",
        "q",
        "abbr",
        "address",
        "small",
        "big",
        "center",
        "font",
        "base",
        "col",
        "colgroup",
        "datalist",
        "fieldset",
        "legend",
        "optgroup",
        "output",
        "progress",
        "meter",
        "noscript",
        "template",
        "time",
        "var",
        "wbr",
    ];
    let original = input;
    let (input, _) = tag("<")(input)?;
    let (input, _) = opt(char('/')).parse(input)?;
    let (input, tagname_span) =
        recognize(pair(alpha1, take_while(is_tagname_char))).parse(input)?;
    let tagname = tagname_span.fragment();
    // Check whitelist
    let is_valid = HTML_TAGS.iter().any(|&t| t.eq_ignore_ascii_case(tagname));
    if !is_valid {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )));
    }
    let (input, _) = take_while(|c: char| c != '>' && c != '/')(input)?;
    let (input, _) = opt(tag("/")).parse(input)?;
    let (input, _) = tag(">")(input)?;
    // Return the span from original to input
    let len = original.fragment().len() - input.fragment().len();
    let tag_span = original.take(len);
    Ok((input, tag_span))
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

    #[test]
    fn test_valid_html_tags() {
        let valid_tags = [
            "<img src=\"test.png\" />",
            "<span class=\"highlight\">",
            "<div id=\"container\">",
            "<br />",
            "<hr />",
            "</div>",
            "<table>",
            "<tr>",
            "<td>",
            "</td>",
            "</tr>",
            "</table>",
        ];
        for tag in valid_tags.iter() {
            let input = Span::new(tag);
            let result = inline_html(input);
            assert!(result.is_ok(), "Should match valid HTML tag: {}", tag);
        }
    }

    #[test]
    fn test_invalid_html_tags() {
        let invalid_tags = [
            "<notaurl>",
            "<x:something>",
            "<1http://example.com>",
            "<not an email>",
            "<foo bar>", // not a valid tagname
            "<@invalid>",
            "<123>",
            "<>",
        ];
        for tag in invalid_tags.iter() {
            let input = Span::new(tag);
            let result = inline_html(input);
            if result.is_ok() {
                println!("FAIL: Matched invalid HTML tag: {}", tag);
            }
            assert!(
                result.is_err(),
                "Should NOT match invalid HTML tag: {}",
                tag
            );
        }
    }
}
