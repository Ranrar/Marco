// Block-level grammar: headings, paragraphs, lists, code blocks, blockquotes, tables

use nom::{
    IResult,
    bytes::complete::{tag, take_while, take_until},
    character::complete::line_ending,
    multi::many1_count,
    combinator::{opt, recognize},
    branch::alt,
    sequence::tuple,
    Slice,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

// HTML Comment parser
// Parses <!-- ... --> on its own line(s)
pub fn html_comment(input: Span) -> IResult<Span, Span> {
    log::debug!("Trying HTML comment at: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    
    // Optional leading spaces (0-3 allowed, just like other block elements)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Parse <!-- ... -->
    let (input, (_, _content, _)) = tuple((
        tag("<!--"),
        take_until("-->"),
        tag("-->"),
    ))(input)?;
    
    // Must be followed by newline or EOF
    let (input, _) = opt(line_ending)(input)?;
    
    // Return the whole comment including markers
    let consumed_len = input.location_offset() - start.location_offset();
    let comment_span = start.slice(..consumed_len);
    
    log::debug!("Parsed HTML comment: {:?}", crate::logic::logger::safe_preview(comment_span.fragment(), 40));
    Ok((input, comment_span))
}

// HTML Block Type 3: Processing Instructions
// Parses <?...?>
pub fn html_processing_instruction(input: Span) -> IResult<Span, Span> {
    log::debug!("Trying processing instruction at: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    
    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Must start with <?
    let (input, _) = tag("<?")(input)?;
    
    // Consume until ?>
    let (input, _content) = take_until("?>")(input)?;
    let (input, _) = tag("?>")(input)?;
    
    // Must be followed by newline or EOF
    let (input, _) = opt(line_ending)(input)?;
    
    let consumed_len = input.location_offset() - start.location_offset();
    let pi_span = start.slice(..consumed_len);
    
    log::debug!("Parsed processing instruction: {:?}", crate::logic::logger::safe_preview(pi_span.fragment(), 40));
    Ok((input, pi_span))
}

// HTML Block Type 4: Declarations
// Parses <!X...> where X is ASCII letter
pub fn html_declaration(input: Span) -> IResult<Span, Span> {
    log::debug!("Trying HTML declaration at: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    
    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Must start with <! followed by ASCII letter
    let (input, _) = tag("<!")(input)?;
    
    // Next character must be ASCII letter
    let bytes = input.fragment().as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_alphabetic() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Alpha)));
    }
    
    // Consume until >
    let (input, _content) = take_until(">")(input)?;
    let (input, _) = tag(">")(input)?;
    
    // Must be followed by newline or EOF
    let (input, _) = opt(line_ending)(input)?;
    
    let consumed_len = input.location_offset() - start.location_offset();
    let decl_span = start.slice(..consumed_len);
    
    log::debug!("Parsed HTML declaration: {:?}", crate::logic::logger::safe_preview(decl_span.fragment(), 40));
    Ok((input, decl_span))
}

// HTML Block Type 5: CDATA Sections
// Parses <![CDATA[...]]>
pub fn html_cdata(input: Span) -> IResult<Span, Span> {
    log::debug!("Trying CDATA section at: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    
    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Must start with <![CDATA[
    let (input, _) = tag("<![CDATA[")(input)?;
    
    // Consume until ]]>
    let (input, _content) = take_until("]]>")(input)?;
    let (input, _) = tag("]]>")(input)?;
    
    // Must be followed by newline or EOF
    let (input, _) = opt(line_ending)(input)?;
    
    let consumed_len = input.location_offset() - start.location_offset();
    let cdata_span = start.slice(..consumed_len);
    
    log::debug!("Parsed CDATA section: {:?}", crate::logic::logger::safe_preview(cdata_span.fragment(), 40));
    Ok((input, cdata_span))
}

// HTML Block Type 1: Special Raw Content Tags (script, pre, style, textarea)
// These consume content until closing tag, can contain blank lines
pub fn html_special_tag(input: Span) -> IResult<Span, Span> {
    log::debug!("Trying special HTML tag at: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    
    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Try to parse opening tag: <pre, <script, <style, <textarea (case-insensitive)
    let lower_input = input.fragment().to_lowercase();
    let tag_name = if lower_input.starts_with("<script") {
        "script"
    } else if lower_input.starts_with("<pre") {
        "pre"
    } else if lower_input.starts_with("<style") {
        "style"
    } else if lower_input.starts_with("<textarea") {
        "textarea"
    } else {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    };
    
    // Check that after tag name there's space, tab, >, or EOL
    let tag_len = tag_name.len() + 1; // +1 for '<'
    if input.fragment().len() > tag_len {
        let next_char = input.fragment().chars().nth(tag_len);
        match next_char {
            Some(' ') | Some('\t') | Some('>') | Some('\n') | Some('\r') => {},
            Some(_) => return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
            None => {}, // EOF is OK
        }
    }
    
    // Build closing tag pattern (case-insensitive)
    let closing_tag = format!("</{}>", tag_name);
    
    // Consume until we find the closing tag
    let mut remaining = input;
    
    while !remaining.fragment().is_empty() {
        // Check if current position contains closing tag (case-insensitive)
        if remaining.fragment().to_lowercase().contains(&closing_tag) {
            // Find exact position of closing tag
            if let Some(pos) = remaining.fragment().to_lowercase().find(&closing_tag) {
                // Advance to after the closing tag
                let bytes_to_consume = pos + closing_tag.len();
                remaining = remaining.slice(bytes_to_consume..);
                break;
            }
        }
        
        // Advance to next line
        if let Some(newline_pos) = remaining.fragment().find('\n') {
            remaining = remaining.slice((newline_pos + 1)..);
        } else {
            // No more newlines, consume rest
            remaining = remaining.slice(remaining.fragment().len()..);
            break;
        }
    }
    
    // Return the entire block (from start to after closing tag or EOF)
    let consumed_len = remaining.location_offset() - start.location_offset();
    let block_span = start.slice(..consumed_len);
    
    log::debug!("Parsed special HTML tag ({}): {:?}", tag_name, crate::logic::logger::safe_preview(block_span.fragment(), 40));
    Ok((remaining, block_span))
}

// HTML Block Type 6: Standard Block Tags
// CommonMark spec lists these specific tags
const BLOCK_TAGS: &[&str] = &[
    "address", "article", "aside", "base", "basefont", "blockquote", "body",
    "caption", "center", "col", "colgroup", "dd", "details", "dialog", "dir",
    "div", "dl", "dt", "fieldset", "figcaption", "figure", "footer", "form",
    "frame", "frameset", "h1", "h2", "h3", "h4", "h5", "h6", "head", "header",
    "hr", "html", "iframe", "legend", "li", "link", "main", "menu", "menuitem",
    "nav", "noframes", "ol", "optgroup", "option", "p", "param", "search",
    "section", "summary", "table", "tbody", "td", "tfoot", "th", "thead",
    "title", "tr", "track", "ul",
];

pub fn html_block_tag(input: Span) -> IResult<Span, Span> {
    log::debug!("Trying block HTML tag at: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    
    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Must start with < or </
    let (input, _opening) = alt((tag("</"), tag("<")))(input)?;
    
    // Try to match one of the block tag names (case-insensitive)
    let lower_input = input.fragment().to_lowercase();
    let mut matched_tag: Option<&str> = None;
    
    for tag_name in BLOCK_TAGS {
        if lower_input.starts_with(tag_name) {
            // Check what follows the tag name
            let tag_len = tag_name.len();
            if lower_input.len() == tag_len {
                // Tag name at EOF is valid
                matched_tag = Some(tag_name);
                break;
            }
            
            let next_char = lower_input.chars().nth(tag_len);
            match next_char {
                // Valid: space, tab, >, newline, or / followed by >
                Some(' ') | Some('\t') | Some('>') | Some('\n') | Some('\r') => {
                    matched_tag = Some(tag_name);
                    break;
                },
                Some('/') => {
                    // Check if followed by >
                    if lower_input.len() > tag_len + 1 {
                        if lower_input.chars().nth(tag_len + 1) == Some('>') {
                            matched_tag = Some(tag_name);
                            break;
                        }
                    }
                },
                _ => continue, // Not a match, try next tag
            }
        }
    }
    
    let tag_name = matched_tag.ok_or_else(|| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
    })?;
    
    // Consume rest of current line
    let mut remaining = input;
    if let Some(newline_pos) = remaining.fragment().find('\n') {
        remaining = remaining.slice((newline_pos + 1)..);
    } else {
        // No newline, consume rest
        remaining = remaining.slice(remaining.fragment().len()..);
    }
    
    // Type 6 blocks end at next blank line
    // Consume lines until blank line or EOF
    while !remaining.fragment().is_empty() {
        // Check if this line is blank
        let line_content = if let Some(newline_pos) = remaining.fragment().find('\n') {
            &remaining.fragment()[..newline_pos]
        } else {
            remaining.fragment()
        };
        
        // If line is blank (only whitespace), end here
        if line_content.trim().is_empty() {
            break;
        }
        
        // Not blank, consume this line
        if let Some(newline_pos) = remaining.fragment().find('\n') {
            remaining = remaining.slice((newline_pos + 1)..);
        } else {
            // No more newlines, consume rest
            remaining = remaining.slice(remaining.fragment().len()..);
            break;
        }
    }
    
    // Return block from start to current position (before blank line)
    let consumed_len = remaining.location_offset() - start.location_offset();
    let block_span = start.slice(..consumed_len);
    
    log::debug!("Parsed block HTML tag ({}): {:?}", tag_name, crate::logic::logger::safe_preview(block_span.fragment(), 40));
    Ok((remaining, block_span))
}

// HTML Block Type 7: Complete Tags
// Must be a complete open or closing tag on a line by itself (followed only by spaces/tabs)
// Cannot interrupt paragraphs (must be handled specially by caller)
// IMPORTANT: This must validate that the tag is well-formed per CommonMark spec
pub fn html_complete_tag(input: Span) -> IResult<Span, Span> {
    log::debug!("Trying complete HTML tag at: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    
    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Try to parse complete tag (open or closing)
    // For Type 7, tag must be complete on this line
    
    let line_content = if let Some(newline_pos) = input.fragment().find('\n') {
        &input.fragment()[..newline_pos]
    } else {
        input.fragment()
    };
    
    // Must start with < and contain >
    if !line_content.starts_with('<') || !line_content.contains('>') {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // Find the > position
    let gt_pos = line_content.find('>').unwrap();
    
    // After >, rest of line must be only spaces/tabs
    let after_tag = &line_content[(gt_pos + 1)..];
    if !after_tag.chars().all(|c| c == ' ' || c == '\t') {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // Check if it's a closing tag or opening tag
    let tag_content = &line_content[..=gt_pos];
    let is_closing = tag_content.starts_with("</");
    
    if is_closing {
        // Closing tag: </tagname>
        // Must have form </[a-zA-Z][a-zA-Z0-9-]*>
        // No attributes allowed in closing tags
        if !tag_content.starts_with("</") || tag_content.contains(' ') || tag_content.contains('\t') {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        
        // Extract tag name (between </ and >)
        let tag_name = &tag_content[2..(tag_content.len()-1)];
        
        // Tag name must start with ASCII letter
        if tag_name.is_empty() || !tag_name.chars().next().unwrap().is_ascii_alphabetic() {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        
        // Rest of tag name must be alphanumeric or hyphen
        if !tag_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
    } else {
        // Opening tag: <tagname ...>
        // Exclude special tags (pre, script, style, textarea) - those are Type 1
        let lower_tag = tag_content.to_lowercase();
        if lower_tag.starts_with("<pre") || lower_tag.starts_with("<script") 
           || lower_tag.starts_with("<style") || lower_tag.starts_with("<textarea") {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        
        // Must be a valid opening tag per CommonMark spec
        // Tag must start with <, then ASCII letter, then alphanumeric/hyphen for tag name
        // Then optional attributes, then optional /, then >
        
        // Extract tag name (from < to first space, /, or >)
        let after_lt = &tag_content[1..];
        let tag_name_end = after_lt.find(|c| c == ' ' || c == '\t' || c == '/' || c == '>')
            .unwrap_or(after_lt.len());
        let tag_name = &after_lt[..tag_name_end];
        
        // Tag name must start with ASCII letter
        if tag_name.is_empty() || !tag_name.chars().next().unwrap().is_ascii_alphabetic() {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        
        // Rest of tag name must be alphanumeric or hyphen
        if !tag_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        
        // If there are attributes, validate them strictly per CommonMark spec
        let after_tag_name = &after_lt[tag_name_end..];
        if !after_tag_name.is_empty() && !after_tag_name.starts_with('>') && !after_tag_name.starts_with("/>") {
            // There are attributes - validate they're well-formed
            // For Type 7, the tag must be "complete" - attributes must be well-formed
            let trimmed = after_tag_name.trim_start();
            if trimmed.starts_with('/') && trimmed.len() == 2 && trimmed == "/>" {
                // Self-closing tag, OK
            } else {
                // Validate attribute format strictly
                // CommonMark requires: whitespace before each attribute, proper quoting, no malformed values
                
                // Reject invalid characters in attributes
                if after_tag_name.contains("*") || after_tag_name.contains("#") {
                    return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
                }
                
                // Must start with whitespace (space or tab) before attribute name
                if !after_tag_name.starts_with(' ') && !after_tag_name.starts_with('\t') {
                    return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
                }
                
                // Check for common malformed patterns
                // Pattern 1: Missing space between attributes (e.g., "href='x'title=y")
                // Look for quote followed directly by letter (no space)
                if after_tag_name.contains("'") {
                    let parts: Vec<&str> = after_tag_name.split('\'').collect();
                    // Check each transition between quoted sections
                    for i in 1..parts.len() {
                        if i % 2 == 0 && !parts[i].is_empty() {
                            // After closing quote, must have space or > or /
                            let first_char = parts[i].chars().next().unwrap();
                            if first_char.is_alphabetic() {
                                // Attribute name directly after quote - missing space!
                                return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
                            }
                        }
                    }
                }
                
                // Pattern 2: Malformed quotes (e.g., href=\"\\\"\")
                // Check for escaped quotes or quote mismatches
                if after_tag_name.contains("\\\"") {
                    // Has escaped quotes - this is invalid in HTML attributes for Type 7
                    return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
                }
                
                // Basic attribute name validation (must start with letter after whitespace)
                if !trimmed.starts_with(char::is_alphabetic) && !trimmed.starts_with('/') && !trimmed.starts_with('>') {
                    return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
                }
            }
        }
    }
    
    // Consume current line
    let mut remaining = input;
    if let Some(newline_pos) = remaining.fragment().find('\n') {
        remaining = remaining.slice((newline_pos + 1)..);
    } else {
        // No newline, consume rest
        remaining = remaining.slice(remaining.fragment().len()..);
    }
    
    // Type 7 blocks end at next blank line
    // Consume lines until blank line or EOF
    while !remaining.fragment().is_empty() {
        // Check if this line is blank
        let line_content = if let Some(newline_pos) = remaining.fragment().find('\n') {
            &remaining.fragment()[..newline_pos]
        } else {
            remaining.fragment()
        };
        
        // If line is blank (only whitespace), end here (don't include the blank line)
        if line_content.trim().is_empty() {
            break;
        }
        
        // Not blank, consume this line
        if let Some(newline_pos) = remaining.fragment().find('\n') {
            remaining = remaining.slice((newline_pos + 1)..);
        } else {
            // No more newlines, consume rest
            remaining = remaining.slice(remaining.fragment().len()..);
            break;
        }
    }
    
    // Return block from start to current position (before blank line)
    let consumed_len = remaining.location_offset() - start.location_offset();
    let block_span = start.slice(..consumed_len);
    
    log::debug!("Parsed complete HTML tag: {:?}", crate::logic::logger::safe_preview(block_span.fragment(), 40));
    Ok((remaining, block_span))
}

// Helper: Count spaces considering tab expansion (1 tab = 4 spaces)
// Returns the number of space characters to skip for a given indentation level
fn count_indentation(input: &str) -> usize {
    let mut spaces = 0;
    for ch in input.chars() {
        match ch {
            ' ' => spaces += 1,
            '\t' => spaces += 4 - (spaces % 4), // Expand to next tab stop
            _ => break,
        }
    }
    spaces
}

// Helper: Skip indentation characters (spaces and tabs) up to a certain number of effective spaces
fn skip_indentation(input: Span, max_spaces: usize) -> IResult<Span, usize> {
    let mut spaces = 0;
    let mut bytes = 0;
    
    for ch in input.fragment().chars() {
        if spaces >= max_spaces {
            break;
        }
        match ch {
            ' ' => {
                spaces += 1;
                bytes += 1;
            }
            '\t' => {
                let tab_width = 4 - (spaces % 4);
                if spaces + tab_width <= max_spaces {
                    spaces += tab_width;
                    bytes += 1;
                } else {
                    // Tab would exceed max, stop here
                    break;
                }
            }
            _ => break,
        }
    }
    
    if bytes == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Space)));
    }
    
    // Use nom's `take` combinator to skip bytes while preserving location information
    use nom::bytes::complete::take;
    let (remaining, _skipped) = take(bytes as u32)(input)?;
    Ok((remaining, spaces))
}

// ATX Heading parser (# through ######)
// Returns (level, content) where level is 1-6
pub fn heading(input: Span) -> IResult<Span, (u8, Span)> {
    log::debug!("Parsing ATX heading: {:?}", input.fragment());
    
    let start = input;
    
    // 1. Optional leading spaces (0-3 spaces allowed)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        // 4+ spaces means indented code block, not heading
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // 2. Count opening # symbols (1-6)
    let (input, hashes) = recognize(many1_count(tag("#")))(input)?;
    let level = hashes.fragment().len();
    
    if level > 6 {
        // 7+ hashes is not a valid heading
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // 3. Require at least one space/tab or end of line after hashes
    // Check what's next without consuming
    let next_char = input.fragment().chars().next();
    let is_valid_separator = match next_char {
        None => true,  // EOF
        Some('\n') | Some('\r') => true,  // Newline
        Some(' ') | Some('\t') => true,  // Whitespace
        Some(_) => false,  // Other character - not valid
    };
    
    if !is_valid_separator {
        // No valid separator - not a valid heading (e.g., "#hashtag")
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Consume whitespace (but not newlines)
    let (input, _) = take_while(|c| c == ' ' || c == '\t')(input)?;
    
    // 4. Parse content until end of line
    let (input, content) = take_while(|c| c != '\n' && c != '\r')(input)?;
    
    // 5. Trim trailing spaces and optional closing hashes
    let content_str = content.fragment();
    let trimmed = content_str.trim_end();
    
    // Remove trailing hashes if they're preceded by a space
    let final_content = if let Some(hash_pos) = trimmed.rfind(|c: char| c != '#' && c != ' ') {
        let after_content = &trimmed[hash_pos + 1..];
        // If everything after is spaces and hashes, remove them
        if after_content.chars().all(|c| c == ' ' || c == '#') {
            let before_trailing = &trimmed[..=hash_pos];
            before_trailing.trim_end()
        } else {
            trimmed
        }
    } else {
        // Content is all hashes/spaces or empty
        ""
    };
    
    // Create a span for the final content
    let content_span = LocatedSpan::new(final_content);
    
    // Consume the newline if present
    let (remaining, _) = opt(line_ending)(input)?;
    
    log::debug!("Parsed heading level {}: {:?}", level, final_content);
    Ok((remaining, (level as u8, content_span)))
}

// Setext heading parser (underline style)
// Level 1: ===
// Level 2: ---
pub fn setext_heading(input: Span) -> IResult<Span, (u8, Span)> {
    log::debug!("Parsing Setext heading: {:?}", input.fragment());
    
    let start = input;
    let start_offset = start.location_offset();
    
    // CRITICAL: Setext headings cannot have link reference definitions as content
    // Check if the first line matches the link reference pattern: ^\[.*\]:\s
    // We do a quick check before full parsing
    let first_line_end = input.fragment().find('\n').unwrap_or(input.fragment().len());
    let first_line = &input.fragment()[..first_line_end];
    let trimmed_first = first_line.trim_start_matches(' ');
    
    // If first line looks like a link reference definition, reject immediately
    // Pattern: optional spaces (0-3) + '[' + text + ']:' 
    if trimmed_first.starts_with('[') && trimmed_first.contains("]:") {
        // Do a more precise check using the link_reference_definition parser
        if link_reference_definition(input).is_ok() {
            log::debug!("Setext heading rejected: content is a link reference definition");
            return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
        }
    }
    
    // Helper: Check if a line starts with blockquote marker ('>') with 0-3 leading spaces
    fn has_blockquote_marker(line: &str) -> bool {
        let trimmed = line.trim_start_matches(' ');
        let leading_spaces = line.len() - trimmed.len();
        leading_spaces <= 3 && trimmed.starts_with('>')
    }
    
    // 1. Parse the content lines (heading text) - can be multiple lines
    // Each line cannot be indented more than 3 spaces
    // CRITICAL: All lines (content + underline) must be in the same block context
    let mut content_end_offset;
    let mut current_input = input;
    let mut has_content = false;
    let mut first_line_in_blockquote: Option<bool> = None;
    
    // Parse at least one line of content
    loop {
        let (after_spaces, leading_spaces) = take_while(|c| c == ' ')(current_input)?;
        if leading_spaces.fragment().len() > 3 {
            if !has_content {
                return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
            }
            break;
        }
        
        // Get the text line
        let (after_line, text_line) = take_while(|c| c != '\n' && c != '\r')(after_spaces)?;
        
        // Text cannot be empty on first line
        if !has_content && text_line.fragment().trim().is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
        }
        
        // Check block context: is this line in a blockquote?
        let line_start = current_input.location_offset();
        let line_end = text_line.location_offset() + text_line.fragment().len();
        let full_line_len = line_end - line_start;
        let full_line = &current_input.fragment()[..full_line_len.min(current_input.fragment().len())];
        let this_line_in_blockquote = has_blockquote_marker(full_line);
        
        // First content line sets the block context
        if first_line_in_blockquote.is_none() {
            first_line_in_blockquote = Some(this_line_in_blockquote);
        } else {
            // Subsequent lines must match first line's block context
            if first_line_in_blockquote.unwrap() != this_line_in_blockquote {
                // Block context boundary crossed - setext heading cannot span this
                log::debug!("Setext heading rejected: content crosses blockquote boundary");
                return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
            }
        }
        
        has_content = true;
        content_end_offset = text_line.location_offset() + text_line.fragment().len();
        
        // Must have line ending after content (setext needs underline on next line)
        let (after_newline, _) = line_ending(after_line)?;
        
        // Check if next line is blank - if so, this is NOT a setext heading
        if after_newline.fragment().starts_with('\n') || after_newline.fragment().starts_with('\r') {
            // Blank line - setext heading must have underline immediately after content
            return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
        }
        
        // Check if next line is the underline or another content line
        // Peek at next line to see if it's an underline
        let (peek_after_spaces, underline_spaces) = take_while(|c| c == ' ')(after_newline)?;
        if underline_spaces.fragment().len() > 3 {
            // Too much indentation for underline, continue as content
            current_input = after_newline;
            continue;
        }
        
        // Check block context for the potential underline line
        let underline_line_start = after_newline.location_offset();
        let underline_peek_len = after_newline.fragment().find('\n').unwrap_or(after_newline.fragment().len());
        let underline_full_line = &after_newline.fragment()[..underline_peek_len.min(after_newline.fragment().len())];
        let underline_in_blockquote = has_blockquote_marker(underline_full_line);
        
        // Underline MUST be in same block context as content
        if first_line_in_blockquote.unwrap() != underline_in_blockquote {
            log::debug!("Setext heading rejected: underline crosses blockquote boundary");
            return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
        }
        
        // Check if we have an underline character
        if let Ok((peek_after_char, first_char)) = nom::character::complete::one_of::<_, _, nom::error::Error<_>>("=-")(peek_after_spaces) {
            // Check if rest of line is all the same character (valid underline)
            let (after_underline, _) = take_while(|c| c == first_char)(peek_after_char)?;
            
            // Count underline characters (must be at least 1, and no spaces allowed)
            let underline_offset = peek_after_spaces.location_offset();
            let underline_len = after_underline.location_offset() - underline_offset;
            let underline_str = &peek_after_spaces.fragment()[..underline_len];
            
            // Verify underline is solid (no spaces)
            if underline_str.chars().all(|c| c == first_char) && !underline_str.is_empty() {
                // Valid underline - check it ends properly (trailing spaces/tabs allowed)
                let (after_trailing_ws, _) = take_while(|c| c == ' ' || c == '\t')(after_underline)?;
                
                // Must end with line ending or EOF
                if let Ok((remaining, _)) = alt::<_, _, nom::error::Error<Span>, _>((
                    recognize(line_ending),
                    recognize(nom::combinator::eof),
                ))(after_trailing_ws) {
                    // This is a valid setext heading!
                    let level = if first_char == '=' { 1 } else { 2 };
                    
                    // Extract content from original input
                    let content_len = content_end_offset - start_offset;
                    let content_str = &start.fragment()[..content_len];
                    let content_span = LocatedSpan::new(content_str);
                    
                    log::debug!("Setext heading parsed: level={}, text={:?}", level, content_str);
                    
                    return Ok((remaining, (level, content_span)));
                }
            }
        }
        
        // Not an underline, continue parsing content lines
        current_input = after_newline;
    }
    
    // If we get here, we didn't find a valid underline
    Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)))
}

// Link reference definition parser
// Parses [label]: destination "optional title"
// Returns (label, url, optional_title)
pub fn link_reference_definition(input: Span) -> IResult<Span, (String, String, Option<String>)> {
    use nom::bytes::complete::{take_while1, take_till};
    use nom::character::complete::{space0, space1, char};
    
    log::debug!("Trying link reference definition at: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    
    // Optional leading spaces (0-3)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Parse [label]:
    let (input, _) = char('[')(input)?;
    let (input, label) = take_till(|c| c == ']' || c == '\n')(input)?;
    
    // Label must not be empty
    if label.fragment().is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    let (input, _) = char(']')(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;
    
    // Optional newline and indentation after colon
    let (input, _) = opt(tuple((line_ending, take_while(|c| c == ' '))))(input)?;
    
    // Parse destination (URL) - can be <url> or bare url
    let (input, url_str) = if input.fragment().starts_with('<') {
        let (input, _) = char('<')(input)?;
        let (input, url) = take_till(|c| c == '>' || c == '\n')(input)?;
        let (input, _) = char('>')(input)?;
        (input, url)
    } else {
        take_while1(|c: char| !c.is_whitespace())(input)?
    };
    
    let url = url_str.fragment().to_string();
    
    // Optional title (must have whitespace before it)
    let (input, title) = if let Ok((i, _)) = space1::<Span, nom::error::Error<Span>>(input) {
        // Optional newline before title
        let (i, _) = opt(tuple((line_ending, take_while(|c| c == ' '))))(i)?;
        
        // Title can be in "...", '...', or (...)
        let (i, title_str) = if i.fragment().starts_with('"') {
            let (i, _) = char('"')(i)?;
            let (i, t) = take_till(|c| c == '"' || c == '\n')(i)?;
            let (i, _) = char('"')(i)?;
            (i, t)
        } else if i.fragment().starts_with('\'') {
            let (i, _) = char('\'')(i)?;
            let (i, t) = take_till(|c| c == '\'' || c == '\n')(i)?;
            let (i, _) = char('\'')(i)?;
            (i, t)
        } else if i.fragment().starts_with('(') {
            let (i, _) = char('(')(i)?;
            let (i, t) = take_till(|c| c == ')' || c == '\n')(i)?;
            let (i, _) = char(')')(i)?;
            (i, t)
        } else {
            return Err(nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Char)));
        };
        
        (i, Some(title_str.fragment().to_string()))
    } else {
        (input, None)
    };
    
    // Consume optional trailing spaces
    let (input, _) = space0(input)?;
    
    // Must end with newline or EOF
    let (input, _) = if input.fragment().is_empty() {
        (input, ())
    } else {
        line_ending(input).map(|(i, _)| (i, ()))?
    };
    
    let label_str = label.fragment().to_string();
    
    log::debug!("Parsed link reference: [{}] -> {}", label_str, url);
    
    Ok((input, (label_str, url, title)))
}

// Paragraph parser
// Parses a paragraph as a sequence of non-blank lines.
// A paragraph ends at a blank line or end of input.
// Leading spaces (0-3) are allowed, 4+ spaces means code block.
// Returns the paragraph content with internal newlines but no trailing newline.
pub fn paragraph(input: Span) -> IResult<Span, Span> {
    use nom::character::complete::not_line_ending;
    
    log::debug!("Parsing paragraph from: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let original_input = input;
    
    // Check for leading indentation (4+ effective spaces = code block)
    let indentation = count_indentation(input.fragment());
    if indentation >= 4 {
        return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
    }
    
    // Skip the actual leading whitespace
    let (after_ws, _) = take_while(|c| c == ' ' || c == '\t')(original_input)?;
    
    // Parse at least one line of text
    let (after_first, first_line) = not_line_ending(after_ws)?;
    
    // First line must not be empty (blank lines don't start paragraphs)
    if first_line.fragment().trim().is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
    }
    
    // Track the end of content (last non-blank line)
    let mut last_line_end = first_line.location_offset() + first_line.fragment().len();
    
    // Consume the newline after first line if present
    let (mut input, _) = opt(line_ending)(after_first)?;
    
    // Continue parsing lines until we hit a blank line or end of input
    loop {
        // Try to parse leading spaces
        let (after_spaces, spaces) = match take_while::<_, _, nom::error::Error<Span>>(|c| c == ' ')(input) {
            Ok(result) => result,
            Err(_) => break,
        };
        
        // Check if line starts with ATX heading (# through ######)
        // ATX headings can interrupt paragraphs, but only with 0-3 leading spaces
        if spaces.fragment().len() <= 3 {
            let trimmed = after_spaces.fragment().trim_start();
            if trimmed.starts_with('#') {
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                if (1..=6).contains(&hash_count) {
                    // Check if followed by space or end of line (valid ATX heading)
                    if hash_count == trimmed.len() || 
                       trimmed.chars().nth(hash_count).map(|c| c.is_whitespace()).unwrap_or(false) {
                        // This is an ATX heading, stop paragraph here
                        break;
                    }
                }
            }
            
            // Check if line starts with fenced code block (``` or ~~~)
            // Fenced code blocks can interrupt paragraphs with 0-3 leading spaces
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                let fence_char = trimmed.chars().next().unwrap();
                let fence_count = trimmed.chars().take_while(|&c| c == fence_char).count();
                if fence_count >= 3 {
                    // This is a fenced code block, stop paragraph here
                    break;
                }
            }
            
            // Check if line starts with blockquote (>)
            // Block quotes can interrupt paragraphs
            if trimmed.starts_with('>') {
                // This is a blockquote, stop paragraph here
                break;
            }
            
            // Check if line starts with list marker
            // Unordered lists can always interrupt paragraphs
            // Ordered lists can only interrupt if they start with "1"
            if let Ok(_) = detect_list_marker(after_spaces) {
                // Check if it's unordered or ordered starting with 1
                let marker_chars: Vec<char> = trimmed.chars().take(5).collect();
                if marker_chars.first().map(|c| *c == '-' || *c == '*' || *c == '+').unwrap_or(false) {
                    // Unordered list, can interrupt
                    break;
                } else if marker_chars.first().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    // Ordered list - check if starts with "1"
                    if trimmed.starts_with("1.") || trimmed.starts_with("1)") {
                        // Can interrupt
                        break;
                    }
                    // Other numbers can't interrupt paragraphs
                }
            }
        }
        
        // Note: We allow indented lines as lazy continuation per CommonMark spec
        // Indented code blocks can only interrupt paragraphs if preceded by blank line
        
        // Try to parse the line content
        let (after_line, line) = match not_line_ending::<Span, nom::error::Error<Span>>(after_spaces) {
            Ok(result) => result,
            Err(_) => break,
        };
        
        // Check if line is blank (only whitespace or empty)
        if line.fragment().trim().is_empty() {
            // Blank line ends the paragraph
            break;
        }
        
        // This is a valid continuation line
        // Update the end position to include this line
        last_line_end = line.location_offset() + line.fragment().len();
        
        // Try to consume newline
        match line_ending::<Span, nom::error::Error<Span>>(after_line) {
            Ok((after_newline, _)) => {
                input = after_newline;
            }
            Err(_) => {
                // No newline, we're at end of input
                input = after_line;
                break;
            }
        }
    }
    
    // Calculate paragraph content from original input
    // Figure out how many bytes the leading whitespace was (from original_input to after_ws)
    let leading_ws_len = original_input.fragment().len() - after_ws.fragment().len();
    let start_offset = original_input.location_offset() + leading_ws_len;
    let content_len = last_line_end - start_offset;
    let para_content = &original_input.fragment()[leading_ws_len..leading_ws_len + content_len];
    let para_span = Span::new(para_content);
    
    log::debug!("Parsed paragraph: {:?}", crate::logic::logger::safe_preview(para_content, 40));
    
    Ok((input, para_span))
}

// Fenced code block parser
// Parses ``` or ~~~ code blocks with optional language info string.
// Returns (language, content) where language is the first word of info string.
pub fn fenced_code_block(input: Span) -> IResult<Span, (Option<String>, Span)> {
    use nom::character::complete::{char as nom_char, not_line_ending};
    
    log::debug!("Parsing fenced code block from: {:?}", crate::logic::logger::safe_preview(input.fragment(), 20));
    
    let original_input = input;
    
    // Parse optional leading spaces (0-3 allowed)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
    }
    
    // Parse the opening fence (``` or ~~~)
    let (input, fence_char) = alt((nom_char('`'), nom_char('~')))(input)?;
    
    // Count the fence delimiters (must be at least 3)
    let (input, fence_count) = {
        let mut count = 1; // We already parsed one
        let mut current = input;
        
        while let Ok((remaining, _)) = nom_char::<_, nom::error::Error<Span>>(fence_char)(current) {
            count += 1;
            current = remaining;
        }
        
        if count < 3 {
            return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
        }
        
        (current, count)
    };
    
    // Parse optional info string (rest of the line after fence)
    let (input, info_line) = not_line_ending(input)?;
    let info_string = info_line.fragment().trim();
    
    // CommonMark spec: info string cannot contain backticks if fence uses backticks
    if fence_char == '`' && info_string.contains('`') {
        return Err(nom::Err::Error(nom::error::Error::new(original_input, nom::error::ErrorKind::Tag)));
    }
    
    // Extract language (first word of info string)
    let language = if !info_string.is_empty() {
        Some(info_string.split_whitespace().next().unwrap_or("").to_string())
    } else {
        None
    };
    
    // Consume newline after opening fence
    let (mut input, _) = line_ending(input)?;
    
    // Track content start and end positions
    let content_start = input.location_offset();
    let mut content_end = content_start;
    
    // Collect code block content lines until we find closing fence
    let mut found_closing = false;
    
    loop {
        // Check for closing fence
        let check_input = input;
        
        // Try to parse optional leading spaces (0-3)
        if let Ok((after_spaces, spaces)) = take_while::<_, _, nom::error::Error<Span>>(|c| c == ' ')(check_input) {
            if spaces.fragment().len() <= 3 {
                // Try to match the fence character
                if let Ok((after_fence_start, _)) = nom_char::<_, nom::error::Error<Span>>(fence_char)(after_spaces) {
                    // Count closing fence delimiters
                    let mut close_count = 1;
                    let mut current = after_fence_start;
                    
                    while let Ok((remaining, _)) = nom_char::<_, nom::error::Error<Span>>(fence_char)(current) {
                        close_count += 1;
                        current = remaining;
                    }
                    
                    // Closing fence must have at least as many delimiters as opening
                    if close_count >= fence_count {
                        // Check that rest of line is whitespace only
                        if let Ok((after_line, rest)) = not_line_ending::<_, nom::error::Error<Span>>(current) {
                            if rest.fragment().trim().is_empty() {
                                // Valid closing fence!
                                found_closing = true;
                                // Consume the closing fence line and optional newline
                                input = after_line;
                                let _ = opt::<_, _, nom::error::Error<Span>, _>(line_ending)(input)?;
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        // Not a closing fence, so this line is content
        // Parse the line
        match not_line_ending::<Span, nom::error::Error<Span>>(input) {
            Ok((after_line, line)) => {
                // Update content end to include this line
                content_end = line.location_offset() + line.fragment().len();
                
                // Try to consume newline
                match line_ending::<Span, nom::error::Error<Span>>(after_line) {
                    Ok((after_newline, _)) => {
                        content_end += 1; // Include newline in content
                        input = after_newline;
                    }
                    Err(_) => {
                        // No newline, end of input
                        input = after_line;
                        break;
                    }
                }
            }
            Err(_) => {
                // Can't parse line, end of input
                break;
            }
        }
    }
    
    if !found_closing {
        // Unclosed code block is still valid in CommonMark (content goes to end of document)
        log::debug!("Unclosed fenced code block");
    }
    
    // Calculate content length and create span from original input
    let content_len = content_end.saturating_sub(content_start);
    
    // Find the content in the original input
    // We need to calculate offset from original_input start
    let offset_from_original = content_start - original_input.location_offset();
    let content_fragment = if content_len > 0 && offset_from_original + content_len <= original_input.fragment().len() {
        &original_input.fragment()[offset_from_original..offset_from_original + content_len]
    } else {
        ""
    };
    
    // Remove trailing newline if present (CommonMark doesn't include trailing newline in content)
    let content_fragment = content_fragment.strip_suffix('\n').unwrap_or(content_fragment);
    let content_span = Span::new(content_fragment);
    
    log::debug!("Parsed fenced code block with language={:?}, content length={}", language, content_fragment.len());
    
    Ok((input, (language, content_span)))
}

// Indented code block parser (4 spaces or 1 tab indentation)
// CommonMark spec: Lines indented with 4 spaces or 1 tab
pub fn indented_code_block(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing indented code block: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    let mut remaining = input;
    let start_offset = start.location_offset();
    let mut last_content_offset = start_offset;
    
    // Parse consecutive indented lines (need at least 4 effective spaces)
    loop {
        // Try to skip at least 4 spaces of indentation (counting tab expansion)
        let indent_result = skip_indentation(remaining, 4);
        
        match indent_result {
            Ok((after_indent, effective_spaces)) if effective_spaces >= 4 => {
                // Get the rest of the line
                let (after_line, line) = take_while(|c| c != '\n' && c != '\r')(after_indent)?;
                
                // Update last_content_offset to include this line
                // Important: we want the offset of the END of the line content
                let line_end_offset = after_line.location_offset();
                last_content_offset = line_end_offset;
                
                log::debug!("Indented code line parsed: {:?}, line_end_offset={}", line.fragment(), line_end_offset);
                
                // Try to consume line ending
                match line_ending::<Span, nom::error::Error<Span>>(after_line) {
                    Ok((after_newline, newline)) => {
                        last_content_offset += newline.fragment().len();
                        remaining = after_newline;
                        
                        // Peek ahead: is next line blank or indented?
                        if remaining.fragment().starts_with('\n') || remaining.fragment().starts_with('\r') {
                            // Blank line - consume it and continue
                            if let Ok((after_blank, blank)) = line_ending::<Span, nom::error::Error<Span>>(remaining) {
                                last_content_offset = blank.location_offset() + blank.fragment().len();
                                remaining = after_blank;
                                continue;
                            }
                        }
                        // Continue to next iteration to check for indentation
                        continue;
                    }
                    Err(_) => {
                        // No newline, end of input
                        log::debug!("No trailing newline, end of code block");
                        break;
                    }
                }
            }
            _ => {
                // Line doesn't have 4 spaces of indentation, end of code block
                break;
            }
        }
    }
    
    // Calculate content length
    let content_len = last_content_offset.saturating_sub(start_offset);
    
    log::debug!("Indented code block: start_offset={}, last_content_offset={}, content_len={}", start_offset, last_content_offset, content_len);
    
    if content_len == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Extract content from input
    let content_fragment = &start.fragment()[..content_len.min(start.fragment().len())];
    
    // Create span for the content (will be processed by parser to remove indentation)
    let content_span = Span::new(content_fragment);
    
    log::debug!("Indented code block parsed: {} bytes", content_fragment.len());
    
    Ok((remaining, content_span))
}

// Legacy function name for compatibility
pub fn code_block(input: Span) -> IResult<Span, Span> {
    // Try fenced code block first
    if let Ok((remaining, (_, content))) = fenced_code_block(input) {
        Ok((remaining, content))
    } else {
        // TODO: Implement indented code blocks
        Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
    }
}

// List marker types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMarker {
    Bullet(char),  // -, +, or *
    Ordered { number: u32, delimiter: char },  // 1. or 1)
}

/// Detect list marker at start of line
/// Returns: (marker, content_indent)
/// 
/// CommonMark rules:
/// - Max 3 leading spaces before marker
/// - Bullet markers: -, +, *
/// - Ordered markers: 1-9 digits followed by . or )
/// - Must have at least 1 space or tab after marker
/// - Content indent = position after marker + spaces (capped at marker position + 4)
pub fn detect_list_marker(input: Span) -> IResult<Span, (ListMarker, usize)> {
    use nom::character::complete::{digit1, one_of};
    use nom::bytes::complete::take;
    
    let start = input;
    
    // 1. Optional leading spaces (0-3 max)
    // Manually count leading spaces since skip_indentation fails on 0 spaces
    let leading_spaces = input.fragment()
        .chars()
        .take_while(|&c| c == ' ' || c == '\t')
        .take(3)
        .fold(0, |acc, c| {
            if c == ' ' { acc + 1 }
            else { acc + 4 - (acc % 4) } // tab expansion
        });
    
    // Skip the leading space bytes
    let space_bytes = input.fragment()
        .chars()
        .take_while(|&c| c == ' ' || c == '\t')
        .take(3)
        .count();
    
    let (input, _) = if space_bytes > 0 {
        take(space_bytes)(input)?
    } else {
        (input, Span::new(""))
    };
    
    // 2. Try to parse marker
    // Try ordered list marker first (e.g., "1." or "1)")
    if let Ok((after_marker, digits)) = digit1::<Span, nom::error::Error<Span>>(input) {
        let number_str = digits.fragment();
        
        // Must be 1-9 digits
        if number_str.len() > 9 {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::TooLarge)));
        }
        
        // Parse the number
        let number: u32 = number_str.parse().map_err(|_| {
            nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
        })?;
        
        // Must be followed by . or )
        if let Ok((after_delim, delimiter)) = one_of::<_, _, nom::error::Error<Span>>(".)")(after_marker) {
            // Must have at least 1 space/tab after delimiter, OR end of line/input (for empty items)
            let after_delim_fragment = after_delim.fragment();
            let has_space_or_tab = !after_delim_fragment.is_empty() && 
                                   (after_delim_fragment.starts_with(' ') || after_delim_fragment.starts_with('\t'));
            let is_end_of_line = after_delim_fragment.is_empty() || 
                                after_delim_fragment.starts_with('\n') || 
                                after_delim_fragment.starts_with('\r');
            
            if has_space_or_tab || is_end_of_line {
                // Calculate content indent
                let marker_width = leading_spaces + number_str.len() + 1; // +1 for delimiter
                
                // Skip spaces/tabs after delimiter (up to 4 effective spaces)
                // Need to count with tab expansion: tab goes to next multiple of 4
                let mut spaces_after = 0;
                let current_column = marker_width; // Column position after delimiter
                
                for ch in after_delim_fragment.chars() {
                    if ch != ' ' && ch != '\t' {
                        break;
                    }
                    
                    // Calculate how many spaces this character adds
                    let space_width = if ch == ' ' {
                        1
                    } else {
                        // Tab: advance to next multiple of 4
                        4 - ((current_column + spaces_after) % 4)
                    };
                    
                    if spaces_after + space_width > 4 {
                        // Would exceed the 4-space limit, stop here
                        break;
                    }
                    
                    spaces_after += space_width;
                }
                
                let content_indent = marker_width + spaces_after;
                
                let marker = ListMarker::Ordered { number, delimiter };
                
                // Return position immediately after marker (NOT after spaces)
                // The spaces are part of the content and will be dedented later
                return Ok((after_delim, (marker, content_indent)));
            }
        }
    }
    
    // Try bullet marker (-, +, *)
    if let Ok((after_marker, bullet_char)) = one_of::<_, _, nom::error::Error<Span>>("-+*")(input) {
        // Must have at least 1 space/tab after bullet, OR end of line/input (for empty items)
        let after_marker_fragment = after_marker.fragment();
        let has_space_or_tab = !after_marker_fragment.is_empty() && 
                               (after_marker_fragment.starts_with(' ') || after_marker_fragment.starts_with('\t'));
        let is_end_of_line = after_marker_fragment.is_empty() || 
                            after_marker_fragment.starts_with('\n') || 
                            after_marker_fragment.starts_with('\r');
        
        if has_space_or_tab || is_end_of_line {
            // Calculate content indent
            let marker_width = leading_spaces + 1; // +1 for bullet
            
            // Skip spaces/tabs after marker (up to 4 effective spaces)
            // Need to count with tab expansion: tab goes to next multiple of 4
            let mut spaces_after = 0;
            let current_column = marker_width; // Column position after marker
            
            for ch in after_marker_fragment.chars() {
                if ch != ' ' && ch != '\t' {
                    break;
                }
                
                // Calculate how many spaces this character adds
                let space_width = if ch == ' ' {
                    1
                } else {
                    // Tab: advance to next multiple of 4
                    4 - ((current_column + spaces_after) % 4)
                };
                
                if spaces_after + space_width > 4 {
                    // Would exceed the 4-space limit, stop here
                    break;
                }
                
                spaces_after += space_width;
            }
            
            let content_indent = marker_width + spaces_after;
            
            let marker = ListMarker::Bullet(bullet_char);
            
            // Return position immediately after marker (NOT after spaces)
            // The spaces are part of the content and will be dedented later
            return Ok((after_marker, (marker, content_indent)));
        }
    }
    
    // No valid marker found
    Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)))
}

/// Parse a single list item
/// Returns: (marker, content_span, has_blank_lines)
/// 
/// The content_span includes all content belonging to this item (may span multiple lines).
/// has_blank_lines indicates whether there are blank lines within the item (affects tight/loose).
pub fn list_item(input: Span, expected_marker_type: Option<ListMarker>) -> IResult<Span, (ListMarker, Span, bool, usize)> {
    use nom::bytes::complete::take;
    
    // 1. Parse the list marker
    let (after_marker, (marker, content_indent)) = detect_list_marker(input)?;
    
    // Calculate the marker's indentation (distance from start of input to start of marker character)
    let marker_indent = count_indentation(input.fragment());
    
    // 2. Check if marker type matches expected (if specified)
    if let Some(expected) = expected_marker_type {
        let matches = matches!((&marker, &expected),
            (ListMarker::Bullet(_), ListMarker::Bullet(_)) |
            (ListMarker::Ordered { .. }, ListMarker::Ordered { .. })
        );
        if !matches {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
    }
    
    // 3. Collect all lines belonging to this item
    let content_start = after_marker;  // Content starts after the marker
    let content_start_offset = content_start.location_offset();
    let mut remaining = after_marker;
    let mut content_end_offset = remaining.location_offset();
    let mut has_blank_lines = false;
    let mut last_was_blank = false;
    let mut is_first_line = true;
    
    // Track fenced code blocks to avoid counting their blank lines
    let mut in_fenced_code = false;
    let mut fence_char: Option<char> = None;
    let mut fence_indent: usize = 0;
    
    // Safety: prevent infinite loops
    const MAX_LINES: usize = 10000;
    let mut line_count = 0;
    
    loop {
        line_count += 1;
        if line_count > MAX_LINES {
            log::warn!("List item exceeded MAX_LINES");
            break;
        }
        
        // Check if we've reached the end of input
        if remaining.fragment().is_empty() {
            break;
        }
        
        // Find the next newline
        let current_line_end = remaining.fragment().find('\n').unwrap_or(remaining.fragment().len());
        let current_line = &remaining.fragment()[..current_line_end];
        
        // Check if this line is blank
        let is_blank = current_line.trim().is_empty();
        
        // Special case: First line handling (even if blank - for empty list items)
        if is_first_line {
            is_first_line = false;
            
            if is_blank {
                // Empty list item (just marker + newline/whitespace)
                // Include just the newline if present, then stop
                let skip_len = if current_line_end < remaining.fragment().len() {
                    current_line_end + 1  // Include newline
                } else {
                    current_line_end  // No newline at end of input
                };
                
                if skip_len > 0 {
                    let (new_remaining, _) = take(skip_len)(remaining)?;
                    content_end_offset = new_remaining.location_offset();
                    remaining = new_remaining;
                }
                
                // Check if next line is a new list marker - if so, stop here (empty item)
                if !remaining.fragment().is_empty() {
                    let next_line_end = remaining.fragment().find('\n').unwrap_or(remaining.fragment().len());
                    let next_line = &remaining.fragment()[..next_line_end];
                    let next_indent = count_indentation(next_line);
                    
                    if next_indent < 4 {
                        use nom_locate::LocatedSpan;
                        let next_span = LocatedSpan::new(*remaining.fragment());
                        if detect_list_marker(next_span).is_ok() {
                            // Next line is a new marker, this is an empty item
                            break;
                        }
                    }
                }
                
                // Otherwise continue to see if there's indented continuation
                continue;
            }
            
            // Non-blank first line - include it
            last_was_blank = false;
            
            // Check if first line starts a fenced code block
            let line_indent = count_indentation(current_line);
            let trimmed_line = current_line.trim_start();
            if (trimmed_line.starts_with("```") || trimmed_line.starts_with("~~~")) && trimmed_line.len() >= 3 {
                let ch = trimmed_line.chars().next().unwrap();
                let fence_len = trimmed_line.chars().take_while(|&c| c == ch).count();
                if fence_len >= 3 {
                    log::debug!("list_item: first line starts fenced code block");
                    in_fenced_code = true;
                    fence_char = Some(ch);
                    fence_indent = line_indent;
                }
            }
            
            let skip_len = if current_line_end < remaining.fragment().len() {
                current_line_end + 1  // Include newline
            } else {
                current_line_end
            };
            
            let (new_remaining, _) = take(skip_len)(remaining)?;
            content_end_offset = new_remaining.location_offset();
            remaining = new_remaining;
            continue;
        }
        
        // Now handle subsequent lines (not first line)
        if is_blank {
            // Blank line - could continue the item if followed by indented content
            // But if the next non-blank line is a new list marker, stop here
            
            // Look ahead to see what comes after this blank line
            let skip_len = if current_line_end < remaining.fragment().len() {
                current_line_end + 1
            } else {
                current_line_end
            };
            
            if skip_len < remaining.fragment().len() {
                // Check what's on the next line
                let after_blank = &remaining.fragment()[skip_len..];
                let next_line_end = after_blank.find('\n').unwrap_or(after_blank.len());
                let next_line = &after_blank[..next_line_end];
                
                // If next line is a list marker or HTML comment, stop before this blank line
                let next_line_indent = count_indentation(next_line);
                if next_line_indent < 4 {
                    use nom_locate::LocatedSpan;
                    let next_line_span = LocatedSpan::new(after_blank);
                    if detect_list_marker(next_line_span).is_ok() {
                        // Next line is a new list marker, don't include this blank line
                        break;
                    }
                    // Also check for HTML comments which interrupt lists
                    if html_comment(next_line_span).is_ok() {
                        // Next line is an HTML comment, don't include this blank line
                        break;
                    }
                }
            }
            
            // CRITICAL: Determine if we should include this blank line.
            // A blank line should only be included if the next non-blank line will continue the item.
            // This prevents marking single-item lists as "loose" when they have a trailing blank line.
            // Per cmark reference implementation, continuation requires >= content_indent spaces.
            let should_include_blank = if skip_len < remaining.fragment().len() {
                // Look ahead to find the next NON-BLANK line
                let mut search_offset = skip_len;
                let mut found_non_blank = false;
                let mut next_non_blank_indent = 0;
                
                while search_offset < remaining.fragment().len() {
                    let search_text = &remaining.fragment()[search_offset..];
                    let line_end = search_text.find('\n').unwrap_or(search_text.len());
                    let line = &search_text[..line_end];
                    
                    if !line.trim().is_empty() {
                        // Found a non-blank line
                        found_non_blank = true;
                        next_non_blank_indent = count_indentation(line);
                        break;
                    }
                    
                    // Move to next line
                    search_offset += line_end + 1;
                    if search_offset > remaining.fragment().len() {
                        break;  // Reached end
                    }
                }
                
                if !found_non_blank {
                    false  // No non-blank line found, don't include trailing blanks
                } else {
                    // Next non-blank line continues if it has at least content_indent spaces
                    next_non_blank_indent >= content_indent
                }
            } else {
                false  // End of input, don't include trailing blank
            };
            
            if !should_include_blank {
                // Don't include this blank line, stop here
                break;
            }
            
            // Only count as "has blank lines" if NOT inside a fenced code block
            // Blank lines inside code blocks don't affect tight/loose list detection
            if !in_fenced_code {
                has_blank_lines = true;
            }
            last_was_blank = true;
            
            // Include the blank line and continue
            let skip_len = if current_line_end < remaining.fragment().len() {
                current_line_end + 1  // Include newline
            } else {
                current_line_end
            };
            
            let (new_remaining, _) = take(skip_len)(remaining)?;
            content_end_offset = new_remaining.location_offset();
            remaining = new_remaining;
            continue;
        }
        
        // Non-blank line - check indentation
        let line_indent = count_indentation(current_line);
        
        // Check for fenced code block markers (``` or ~~~)
        // This helps us track whether blank lines are inside code blocks
        let trimmed_line = current_line.trim_start();
        if !in_fenced_code {
            // Not in a code block, check if this line starts one
            if (trimmed_line.starts_with("```") || trimmed_line.starts_with("~~~")) && trimmed_line.len() >= 3 {
                let ch = trimmed_line.chars().next().unwrap();
                let fence_len = trimmed_line.chars().take_while(|&c| c == ch).count();
                if fence_len >= 3 {
                    // Entering a fenced code block
                    log::debug!("list_item: entering fenced code block at line: {:?}", current_line);
                    in_fenced_code = true;
                    fence_char = Some(ch);
                    fence_indent = line_indent;
                }
            }
        } else {
            // Already in a code block, check if this closes it
            if let Some(fc) = fence_char {
                if trimmed_line.starts_with(fc) {
                    let close_fence_len = trimmed_line.chars().take_while(|&c| c == fc).count();
                    // Closing fence must be at least as long as opening fence and properly indented
                    if close_fence_len >= 3 && line_indent <= fence_indent + content_indent {
                        // Exiting the fenced code block
                        log::debug!("list_item: exiting fenced code block at line: {:?}", current_line);
                        in_fenced_code = false;
                        fence_char = None;
                    }
                }
            }
        }
        
        // Check if this starts a new list item (not first line, we handled that above)
        // A line is a new SIBLING item if:
        // 1. It has a list marker
        // 2. The marker is at the same or lesser indentation as the current item's marker
        if line_indent < 4 {  // Could be a new marker (markers need < 4 spaces)
            if detect_list_marker(remaining).is_ok() {
                // Check if this marker is at greater indentation (nested) or same/lesser (sibling)
                if line_indent <= marker_indent {
                    // This is a sibling list item, stop here
                    break;
                }
                // Otherwise, marker is indented more than current marker, so it's nested content
                // Fall through to continue collecting
            }
        }
        
        // Check if line is indented enough to continue the item
        // CommonMark simply requires that continuation lines have at least content_indent spaces.
        // This is true both for normal lines and lines after blank lines.
        // The cmark reference implementation does NOT add extra requirements after blank lines.
        let min_indent = content_indent;
        
        if line_indent >= min_indent {
            // Line is indented, it continues the item
            last_was_blank = false;
            
            let skip_len = if current_line_end < remaining.fragment().len() {
                current_line_end + 1  // Include newline
            } else {
                current_line_end
            };
            
            let (new_remaining, _) = take(skip_len)(remaining)?;
            content_end_offset = new_remaining.location_offset();
            remaining = new_remaining;
            continue;
        }
        
        // Line is not indented enough
        // Check for lazy continuation (only if not after blank line AND not a new list marker)
        if !last_was_blank {
            // Check if this could be a new list marker
            if line_indent < 4 && detect_list_marker(remaining).is_ok() {
                // This is a new list item, don't lazy continue
                break;
            }
            
            // Check if this line is a block structure that interrupts lazy continuation
            // Thematic breaks, ATX headings, and fenced code blocks cannot be lazy continuation
            if thematic_break(remaining).is_ok() {
                // This is a thematic break, stop here
                break;
            }
            
            if heading(remaining).is_ok() {
                // This is an ATX heading, stop here
                break;
            }
            
            if fenced_code_block(remaining).is_ok() {
                // This is a fenced code block, stop here
                break;
            }
            
            // Lazy continuation: any non-blank line that's not a new list marker or block structure
            last_was_blank = false;
            
            let skip_len = if current_line_end < remaining.fragment().len() {
                current_line_end + 1
            } else {
                current_line_end
            };
            
            let (new_remaining, _) = take(skip_len)(remaining)?;
            content_end_offset = new_remaining.location_offset();
            remaining = new_remaining;
            continue;
        }
        
        // Line doesn't continue the item (would only reach here after blank line)
        break;
    }
    
    // Extract the content span (from content_start to content_end)
    let content_length = content_end_offset - content_start_offset;
    let (after_content, content) = take(content_length)(content_start)?;
    
    Ok((after_content, (marker, content, has_blank_lines, content_indent)))
}

/// Parse a complete list (ordered or unordered)
/// Returns: Vec of (marker, content_span, has_blank_lines_in_item, has_blank_before_next, content_indent)
/// The 4th boolean indicates if there's a blank line BETWEEN this item and the next
pub fn list(input: Span) -> IResult<Span, Vec<(ListMarker, Span, bool, bool, usize)>> {
    use nom::bytes::complete::take;
    
    log::debug!("Parsing list");
    
    // 1. Parse first item to determine list type
    let (mut remaining, (first_marker, first_content, first_has_blank, first_indent)) = list_item(input, None)?;
    
    let mut items = vec![(first_marker, first_content, first_has_blank, false, first_indent)];
    
    // Safety: prevent infinite loops
    const MAX_ITEMS: usize = 1000;
    let mut item_count = 1;
    let mut last_offset = 0;  // Initialize to 0, will be set properly in loop
    
    // 2. Continue parsing items with matching marker type
    loop {
        item_count += 1;
        if item_count > MAX_ITEMS {
            log::warn!("List exceeded MAX_ITEMS");
            break;
        }
        
        // Check if we've reached end of input
        if remaining.fragment().is_empty() {
            break;
        }
        
        // Check for blank lines before next item
        let mut has_blank_before_next = false;
        let mut temp_remaining = remaining;
        
        // Skip blank lines and track if we found any
        loop {
            if temp_remaining.fragment().is_empty() {
                remaining = temp_remaining;
                break;
            }
            
            let first_line_end = temp_remaining.fragment().find('\n').unwrap_or(temp_remaining.fragment().len());
            let first_line = &temp_remaining.fragment()[..first_line_end];
            
            if first_line.trim().is_empty() {
                has_blank_before_next = true;
                
                // Skip this blank line
                let skip_len = if first_line_end < temp_remaining.fragment().len() {
                    first_line_end + 1
                } else {
                    first_line_end
                };
                
                let (new_remaining, _) = take(skip_len)(temp_remaining)?;
                temp_remaining = new_remaining;
            } else {
                // Non-blank line found
                remaining = temp_remaining;
                break;
            }
        }
        
        // Safety check: ensure progress AFTER skipping blank lines
        let current_offset = remaining.location_offset();
        if current_offset == last_offset {
            log::error!("List parser stuck at offset {}", current_offset);
            break;
        }
        last_offset = current_offset;
        
        // Check if we've reached end after skipping blanks
        if remaining.fragment().is_empty() {
            break;
        }
        
        // Try to parse next item with expected marker type
        match list_item(remaining, Some(first_marker)) {
            Ok((new_remaining, (marker, content, has_blank, item_content_indent))) => {
                log::debug!("Parsed list item: {:?}", content.fragment());
                
                // Only set blank-before-next flag if we successfully parsed another item
                // This prevents blank lines at the END of a list from making it loose
                if has_blank_before_next {
                    let last_idx = items.len() - 1;
                    items[last_idx].3 = true;
                }
                
                items.push((marker, content, has_blank, false, item_content_indent));
                remaining = new_remaining;
            }
            Err(e) => {
                // No more items of this type - blank lines after last item don't count
                log::debug!("Failed to parse next list item: {:?}", e);
                break;
            }
        }
    }
    
    log::debug!("List parsing complete, {} items", items.len());
    Ok((remaining, items))
}

// Block quote parser
// CommonMark spec: Lines starting with `>` (with optional space after)
// Supports lazy continuation and nesting
pub fn block_quote(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing block quote: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    let start_offset = start.location_offset();
    let mut remaining = input;
    let mut last_content_end_offset = start_offset;  // Track end of content
    let mut last_offset;
    
    // Must start with a block quote marker
    let (after_marker, _) = parse_block_quote_marker(remaining)?;
    remaining = after_marker;
    last_offset = start_offset - 1;  // Initialize to before start to allow first iteration!
    
    // Parse lines until we hit a non-continuation line
    const MAX_ITERATIONS: usize = 50;  // Safety: prevent infinite loops (reduced to prevent memory issues)
    let mut iteration_count = 0;
    loop {
        iteration_count += 1;
        if iteration_count > MAX_ITERATIONS {
            log::warn!("Block quote parser exceeded MAX_ITERATIONS, breaking");
            break;
        }
        
        // Safety check: if remaining is empty, break
        if remaining.fragment().is_empty() {
            // Update last_content_end_offset to current position
            last_content_end_offset = remaining.location_offset();
            break;
        }
        
        // Safety check: ensure we're making progress
        let current_offset = remaining.location_offset();
        if current_offset == last_offset {
            log::warn!("Block quote parser not making progress at offset {}, breaking", current_offset);
            break;
        }
        last_offset = current_offset;
        
        // Get the rest of the current line
        let (after_line, line) = take_while(|c| c != '\n' && c != '\r')(remaining)?;
        
        // Try to consume line ending
        match line_ending::<Span, nom::error::Error<Span>>(after_line) {
            Ok((after_newline, newline)) => {
                // Update end position to include line + newline
                last_content_end_offset = line.location_offset() + line.fragment().len() + newline.fragment().len();
                remaining = after_newline;
                
                // Safety check after consuming newline
                if remaining.fragment().is_empty() {
                    break;
                }
                
                // Check if next line continues the block quote
                // Try to parse block quote marker on next line
                if let Ok((after_marker, _)) = parse_block_quote_marker(remaining) {
                    remaining = after_marker;
                    continue;
                } else {
                    // No block quote marker, end here
                    break;
                }
            }
            Err(_) => {
                // No newline, end of input - include the line
                last_content_end_offset = line.location_offset() + line.fragment().len();
                break;
            }
        }
    }
    
    // Calculate content length
    let content_len = last_content_end_offset.saturating_sub(start_offset);
    
    if content_len == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Extract content from input
    let content_fragment = &start.fragment()[..content_len.min(start.fragment().len())];
    let content_span = Span::new(content_fragment);
    
    log::debug!("Block quote parsed: {} bytes", content_fragment.len());
    
    Ok((remaining, content_span))
}

// Helper: Parse block quote marker (> with optional leading spaces and optional trailing space)
fn parse_block_quote_marker(input: Span) -> IResult<Span, ()> {
    // Optional leading spaces (0-3)
    let (input, leading) = take_while(|c| c == ' ')(input)?;
    if leading.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
    }
    
    // The `>` character
    let (input, _) = nom::character::complete::char('>')(input)?;
    
    // Optional single space or tab after >
    let (input, _) = opt(nom::character::complete::one_of(" \t"))(input)?;
    
    Ok((input, ()))
}

// Thematic break parser (---, ***, ___)
// CommonMark spec: 3+ matching characters (-, *, or _) with optional spaces between
// Can have 0-3 leading spaces
pub fn thematic_break(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing thematic break: {:?}", input.fragment());
    
    let start = input;
    
    // 1. Optional leading spaces (0-3 spaces allowed)
    let (input, leading_spaces) = take_while(|c| c == ' ')(input)?;
    if leading_spaces.fragment().len() > 3 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // 2. Determine which character is used (-, *, or _)
    let (input, first_char) = nom::character::complete::one_of("-*_")(input)?;
    
    // 3. Count matching characters with optional spaces/tabs between
    let mut remaining = input;
    let mut char_count = 1; // Already found first char
    
    loop {
        // Try to consume optional spaces and tabs
        let (input_after_space, _) = take_while(|c| c == ' ' || c == '\t')(remaining)?;
        
        // Try to match the same character
        if let Ok((input_after_char, _matched_char)) = nom::character::complete::char::<_, nom::error::Error<Span>>(first_char)(input_after_space) {
            char_count += 1;
            remaining = input_after_char;
        } else {
            // No more matching chars, check if we're at end of line
            remaining = input_after_space;
            break;
        }
    }
    
    // 4. Must have at least 3 matching characters
    if char_count < 3 {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // 5. Must be followed by whitespace or end of input (nothing else on the line)
    let (remaining, _) = take_while(|c| c == ' ' || c == '\t')(remaining)?;
    
    // Check for end of line or end of input
    let (remaining, _) = alt((
        recognize(line_ending),
        recognize(nom::combinator::eof),
    ))(remaining)?;
    
    log::debug!("Thematic break parsed: {} matching '{}' chars", char_count, first_char);
    
    Ok((remaining, LocatedSpan::new("---")))
}

// Blockquote parser (>)
/// Parse a block quote (lines starting with >)
/// Returns the content of the blockquote (with > markers still present for recursive parsing)
pub fn blockquote(input: Span) -> IResult<Span, Span> {
    use nom::bytes::complete::take;
    
    log::debug!("Parsing blockquote from: {:?}", crate::logic::logger::safe_preview(input.fragment(), 40));
    
    let start = input;
    let start_offset = start.location_offset();
    let mut remaining = input;
    let mut last_content_offset = start_offset;
    
    // Safety: prevent infinite loops
    const MAX_LINES: usize = 10000;
    let mut line_count = 0;
    let mut has_parsed_line = false;
    let mut last_line_opened_fence = false;  // Track if previous line opened fenced code block
    
    loop {
        line_count += 1;
        if line_count > MAX_LINES {
            log::warn!("Blockquote exceeded MAX_LINES");
            break;
        }
        
        // Check if we've reached the end
        if remaining.fragment().is_empty() {
            break;
        }
        
        // Check leading spaces
        let leading_spaces = remaining.fragment().chars()
            .take_while(|&c| c == ' ')
            .count();
        
        // Try to match '>' marker
        let after_spaces = if leading_spaces > 0 && leading_spaces < remaining.fragment().len() {
            &remaining.fragment()[leading_spaces..]
        } else if leading_spaces > 0 {
            ""
        } else {
            remaining.fragment()
        };
        
        // Check if this line has a > marker
        let has_marker = after_spaces.starts_with('>');
        
        // If line has > marker, it can only have 0-3 leading spaces
        if has_marker && leading_spaces > 3 {
            // Too much indentation before >, not valid blockquote line
            break;
        }
        
        if !has_marker {
            // No '>' marker
            // Lazy continuation: if we already have content, non-blank lines can continue
            if has_parsed_line {
                // Check if line is blank
                let line_end = after_spaces.find('\n').unwrap_or(after_spaces.len());
                let line = &after_spaces[..line_end];
                
                if line.trim().is_empty() {
                    // Blank line ends blockquote
                    break;
                }
                
                // Check if this could be another block element starting
                // (ATX heading, fenced code, etc.)
                if line.starts_with('#') {
                    // ATX heading - stop blockquote
                    break;
                }
                
                // CRITICAL FIX FOR EXAMPLE 237:
                // If previous line opened a fenced code block, lazy continuation is NOT allowed
                if last_line_opened_fence {
                    log::debug!("Blockquote stopping: previous line opened fenced code, lazy continuation not allowed");
                    break;
                }
                
                // Check for actual thematic break using parser (not just "---" prefix)
                let line_span = LocatedSpan::new(line);
                if thematic_break(line_span).is_ok() {
                    // This is a thematic break, stop blockquote
                    break;
                }
                
                // Lazy continuation - include this line
                let skip_len = if line_end < after_spaces.len() {
                    leading_spaces + line_end + 1  // Include newline
                } else {
                    leading_spaces + line_end
                };
                
                if let Ok((new_remaining, _)) = take::<_, _, nom::error::Error<Span>>(skip_len)(remaining) {
                    last_content_offset = new_remaining.location_offset();
                    remaining = new_remaining;
                    last_line_opened_fence = false;  // Reset flag after consuming lazy continuation
                    continue;
                } else {
                    break;
                }
            } else {
                // Haven't parsed any blockquote lines yet, this is not a blockquote
                return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
            }
        }
        
        // We have a '>' marker
        has_parsed_line = true;
        
        // Skip the '>' and optional space after it
        let after_marker = &after_spaces[1..];
        let after_optional_space = after_marker.strip_prefix(' ').unwrap_or(after_marker);
        
        // Get the rest of the line
        let line_end = after_optional_space.find('\n').unwrap_or(after_optional_space.len());
        let line_content = &after_optional_space[..line_end];
        
        // Check if this line opens a fenced code block
        let line_trimmed = line_content.trim_start();
        last_line_opened_fence = line_trimmed.starts_with("```") || line_trimmed.starts_with("~~~");
        
        // Calculate how much to skip (leading spaces + '>' + optional space + line content + newline)
        let skip_len = if line_end < after_optional_space.len() {
            leading_spaces + 1 + (after_marker.len() - after_optional_space.len()) + line_end + 1
        } else {
            leading_spaces + 1 + (after_marker.len() - after_optional_space.len()) + line_end
        };
        
        // Use nom's take to consume the line
        if let Ok((new_remaining, _)) = take::<_, _, nom::error::Error<Span>>(skip_len)(remaining) {
            last_content_offset = new_remaining.location_offset();
            remaining = new_remaining;
        } else {
            // Couldn't consume, this shouldn't happen but break to be safe
            log::warn!("Failed to consume blockquote line");
            break;
        }
    }
    
    // Calculate content length
    let content_len = last_content_offset.saturating_sub(start_offset);
    
    if content_len == 0 || !has_parsed_line {
        return Err(nom::Err::Error(nom::error::Error::new(start, nom::error::ErrorKind::Tag)));
    }
    
    // Extract content using nom's take to preserve position information
    let (_, content_span) = take(content_len)(start)?;
    
    log::debug!("Blockquote parsed: {} bytes", content_span.fragment().len());
    
    Ok((remaining, content_span))
}

// Table parser
pub fn table(input: Span) -> IResult<Span, Span> {
    log::debug!("Parsing table");
    // TODO: Implement table parsing
    Ok((input, input))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_heading_levels() {
        // Test all heading levels 1-6
        for level in 1..=6 {
            let hashes = "#".repeat(level);
            let input_str = format!("{} Heading level {}", hashes, level);
            let input = Span::new(&input_str);
            
            let result = heading(input);
            assert!(result.is_ok(), "Failed to parse heading level {}", level);
            
            let (_, (parsed_level, content)) = result.unwrap();
            assert_eq!(parsed_level, level as u8);
            assert!(content.fragment().contains(&format!("Heading level {}", level)));
        }
    }
    
    #[test]
    fn smoke_test_heading_basic() {
        let input = Span::new("# Hello World");
        let result = heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(content.fragment(), &"Hello World");
    }
    
    #[test]
    fn smoke_test_heading_trailing_hashes() {
        // Example 71: trailing hashes should be removed
        let input = Span::new("## foo ##");
        let result = heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 2);
        assert_eq!(content.fragment(), &"foo");
    }
    
    #[test]
    fn smoke_test_heading_trim_whitespace() {
        // Example 67: multiple spaces should be trimmed
        let input = Span::new("#                  foo                     ");
        let result = heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(content.fragment(), &"foo");
    }
    
    #[test]
    fn smoke_test_heading_leading_spaces() {
        // Example 68: 0-3 leading spaces allowed
        let input = Span::new("   # foo");
        let result = heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(content.fragment(), &"foo");
    }
    
    #[test]
    fn smoke_test_heading_invalid_seven_hashes() {
        // Example 63: 7+ hashes is not a heading
        let input = Span::new("####### foo");
        let result = heading(input);
        
        assert!(result.is_err(), "Should fail with 7 hashes");
    }
    
    #[test]
    fn smoke_test_heading_invalid_no_space() {
        // Example 64: No space after # means not a heading
        let input = Span::new("#5 bolt");
        let result = heading(input);
        
        assert!(result.is_err(), "Should fail without space after #");
    }
    
    #[test]
    fn smoke_test_heading_invalid_four_space_indent() {
        // Example 69: 4+ spaces means code block
        let input = Span::new("    # foo");
        let result = heading(input);
        
        assert!(result.is_err(), "Should fail with 4+ leading spaces");
    }
    
    #[test]
    fn smoke_test_heading_empty_content() {
        // Example 79: empty heading is valid
        let input = Span::new("## ");
        let result = heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 2);
        assert_eq!(content.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_paragraph_simple() {
        // Example 219: Simple single-line paragraph
        let input = Span::new("aaa");
        let result = paragraph(input);
        
        assert!(result.is_ok());
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"aaa");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_paragraph_multiline() {
        // Example 220: Multi-line paragraph
        let input = Span::new("aaa\nbbb");
        let result = paragraph(input);
        
        assert!(result.is_ok());
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"aaa\nbbb");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_paragraph_blank_line_terminator() {
        // Example 219: Paragraph ends at blank line
        let input = Span::new("aaa\n\nbbb");
        let result = paragraph(input);
        
        assert!(result.is_ok());
        let (remaining, content) = result.unwrap();
        assert_eq!(content.fragment(), &"aaa");
        assert!(remaining.fragment().starts_with("\n"));
    }
    
    #[test]
    fn smoke_test_paragraph_leading_spaces() {
        // Example 222: 0-3 leading spaces allowed
        let input = Span::new("  aaa\n bbb");
        let result = paragraph(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        // Content should include the first line with its spaces
        assert!(content.fragment().contains("aaa"));
        assert!(content.fragment().contains("bbb"));
    }
    
    #[test]
    fn smoke_test_paragraph_four_spaces_rejected() {
        // Example 225: 4+ leading spaces means code block
        let input = Span::new("    aaa");
        let result = paragraph(input);
        
        assert!(result.is_err(), "Should fail with 4+ leading spaces");
    }
    
    #[test]
    fn smoke_test_paragraph_blank_line_rejected() {
        // Blank lines don't start paragraphs
        let input = Span::new("\n\naaa");
        let result = paragraph(input);
        
        assert!(result.is_err(), "Should fail on blank line");
    }
    
    #[test]
    fn smoke_test_paragraph_whitespace_only_rejected() {
        // Lines with only whitespace don't start paragraphs
        let input = Span::new("   \n");
        let result = paragraph(input);
        
        assert!(result.is_err(), "Should fail on whitespace-only line");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_basic_backticks() {
        // Example 119: Basic fenced code block with backticks
        let input = Span::new("```\n<\n >\n```\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (language, content)) = result.unwrap();
        assert_eq!(language, None);
        assert_eq!(content.fragment(), &"<\n >");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_basic_tildes() {
        // Example 120: Basic fenced code block with tildes
        let input = Span::new("~~~\n<\n >\n~~~\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (language, content)) = result.unwrap();
        assert_eq!(language, None);
        assert_eq!(content.fragment(), &"<\n >");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_with_language() {
        // Example 142: Code block with language info string
        let input = Span::new("```ruby\ndef foo(x)\n  return 3\nend\n```\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (language, content)) = result.unwrap();
        assert_eq!(language, Some("ruby".to_string()));
        assert_eq!(content.fragment(), &"def foo(x)\n  return 3\nend");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_longer_closing() {
        // Closing fence can be longer than opening
        let input = Span::new("```\ncode\n````\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (language, content)) = result.unwrap();
        assert_eq!(language, None);
        assert_eq!(content.fragment(), &"code");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_nested_fences() {
        // Example 122: Different fence chars can be nested
        let input = Span::new("```\naaa\n~~~\n```\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (_, content)) = result.unwrap();
        assert_eq!(content.fragment(), &"aaa\n~~~");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_less_than_three() {
        // Example 121: Less than 3 delimiters is not a code block
        let input = Span::new("``\nfoo\n``\n");
        let result = fenced_code_block(input);
        
        assert!(result.is_err(), "Should fail with less than 3 delimiters");
    }
    
    #[test]
    fn smoke_test_fenced_code_block_unclosed() {
        // Unclosed code blocks are valid - go to end of input
        let input = Span::new("```\ncode\nmore code");
        let result = fenced_code_block(input);
        
        assert!(result.is_ok());
        let (_, (_, content)) = result.unwrap();
        assert_eq!(content.fragment(), &"code\nmore code");
    }
    
    // Setext heading smoke tests
    
    #[test]
    fn smoke_test_setext_heading_level_1() {
        // Level 1 setext heading with = underline
        let input = Span::new("Heading\n=======\n");
        let result = setext_heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(content.fragment().trim(), "Heading");
    }
    
    #[test]
    fn smoke_test_setext_heading_level_2() {
        // Level 2 setext heading with - underline
        let input = Span::new("Heading\n-------\n");
        let result = setext_heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 2);
        assert_eq!(content.fragment().trim(), "Heading");
    }
    
    #[test]
    fn smoke_test_setext_heading_minimal_underline() {
        // Single character underline is valid
        let input = Span::new("Title\n=\n");
        let result = setext_heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(content.fragment().trim(), "Title");
    }
    
    #[test]
    fn smoke_test_setext_heading_multiline_text() {
        // Setext heading only captures first line of text before underline
        let input = Span::new("Line one\n========\n");
        let result = setext_heading(input);
        
        assert!(result.is_ok());
        let (_, (level, content)) = result.unwrap();
        assert_eq!(level, 1);
        assert_eq!(content.fragment().trim(), "Line one");
    }
    
    // Indented code block smoke tests
    
    #[test]
    fn smoke_test_indented_code_block_basic() {
        // Four spaces = indented code block
        // Returns raw content WITH indentation
        let input = Span::new("    code line 1\n    code line 2\n");
        let result = indented_code_block(input);
        
        assert!(result.is_ok(), "Should parse indented code block");
        let (_, content) = result.unwrap();
        // Just verify we got some content back
        assert!(!content.fragment().is_empty(), "Content should not be empty");
    }
    
    #[test]
    fn smoke_test_indented_code_block_single_line() {
        let input = Span::new("    single code line\n");
        let result = indented_code_block(input);
        
        assert!(result.is_ok(), "Should parse single line");
        let (_, content) = result.unwrap();
        assert!(!content.fragment().is_empty(), "Content should not be empty");
    }
    
    #[test]
    fn smoke_test_indented_code_block_tab() {
        // Tab counts as 4 spaces
        let input = Span::new("\tcode with tab\n");
        let result = indented_code_block(input);
        
        assert!(result.is_ok(), "Should parse tab-indented code");
        let (_, content) = result.unwrap();
        assert!(!content.fragment().is_empty(), "Content should not be empty");
    }
    
    #[test]
    fn smoke_test_indented_code_block_blank_lines() {
        // Blank lines within code block are preserved
        let input = Span::new("    line 1\n\n    line 2\n");
        let result = indented_code_block(input);
        
        assert!(result.is_ok(), "Should parse with blank lines");
        let (_, content) = result.unwrap();
        assert!(!content.fragment().is_empty(), "Content should not be empty");
    }
    
    #[test]
    fn smoke_test_indented_code_block_three_spaces_fails() {
        // Only 3 spaces - not enough for indented code
        let input = Span::new("   not code\n");
        let result = indented_code_block(input);
        
        assert!(result.is_err(), "Three spaces should not be indented code");
    }
    
    // Block quote smoke tests
    
    #[test]
    fn smoke_test_block_quote_basic() {
        // Simple block quote with > marker
        // Returns raw content WITH the > marker
        let input = Span::new("> This is a quote\n");
        let result = block_quote(input);
        
        assert!(result.is_ok(), "block_quote should succeed");
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains(">"), "Should contain >");
        assert!(content.fragment().contains("This is a quote"), "Should contain text");
    }
    
    #[test]
    fn smoke_test_block_quote_multiline() {
        // Multiple lines with > marker
        // Returns raw content WITH > markers
        let input = Span::new("> Line 1\n> Line 2\n> Line 3\n");
        let result = block_quote(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        let fragment = content.fragment();
        assert!(fragment.contains("Line 1"));
        assert!(fragment.contains("Line 2"));
        assert!(fragment.contains("Line 3"));
        assert_eq!(fragment, &"> Line 1\n> Line 2\n> Line 3\n");
    }
    
    #[test]
    fn smoke_test_block_quote_lazy_continuation() {
        // Lazy continuation - only first line has >, parser returns just first line
        let input = Span::new("> Line 1\nLine 2\n");
        let result = block_quote(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        // Grammar returns only the line with > marker
        assert_eq!(content.fragment(), &"> Line 1\n");
    }
    
    #[test]
    fn smoke_test_block_quote_nested() {
        // Nested block quotes with >>
        let input = Span::new("> Outer\n>> Inner\n> Outer again\n");
        let result = block_quote(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        // Returns all lines with > markers
        assert!(content.fragment().contains("Outer"));
        assert!(content.fragment().contains(">> Inner"));
    }
    
    #[test]
    fn smoke_test_block_quote_with_leading_spaces() {
        // Up to 3 leading spaces before > marker
        let input = Span::new("  > Quoted text\n");
        let result = block_quote(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("Quoted text"));
        assert_eq!(content.fragment(), &"  > Quoted text\n");
    }
    
    #[test]
    fn smoke_test_block_quote_empty_line() {
        // Block quote with empty line (just > marker)
        let input = Span::new("> Line 1\n>\n> Line 3\n");
        let result = block_quote(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains("Line 1"));
        assert!(content.fragment().contains("Line 3"));
        assert_eq!(content.fragment(), &"> Line 1\n>\n> Line 3\n");
    }
    
    // Thematic break smoke tests
    
    #[test]
    fn smoke_test_thematic_break_asterisks() {
        // Three or more asterisks
        // Always returns "---" as canonical representation
        let input = Span::new("***\n");
        let result = thematic_break(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(content.fragment(), &"---");
    }
    
    #[test]
    fn smoke_test_thematic_break_hyphens() {
        // Three or more hyphens
        // Always returns "---" as canonical representation
        let input = Span::new("---\n");
        let result = thematic_break(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(content.fragment(), &"---");
    }
    
    #[test]
    fn smoke_test_thematic_break_underscores() {
        // Three or more underscores
        // Always returns "---" as canonical representation
        let input = Span::new("___\n");
        let result = thematic_break(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(content.fragment(), &"---");
    }
    
    #[test]
    fn smoke_test_thematic_break_with_spaces() {
        // Spaces between characters are allowed
        // Always returns "---" as canonical representation
        let input = Span::new("* * *\n");
        let result = thematic_break(input);
        
        assert!(result.is_ok());
        let (_, content) = result.unwrap();
        assert_eq!(content.fragment(), &"---");
    }
    
    #[test]
    fn smoke_test_thematic_break_many_chars() {
        // More than 3 characters is valid
        let input = Span::new("*****\n");
        let result = thematic_break(input);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn smoke_test_thematic_break_leading_spaces() {
        // Up to 3 leading spaces allowed
        let input = Span::new("  ***\n");
        let result = thematic_break(input);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn smoke_test_thematic_break_two_chars_fails() {
        // Only 2 characters - not enough
        let input = Span::new("**\n");
        let result = thematic_break(input);
        
        assert!(result.is_err(), "Two characters should not be a thematic break");
    }
    
    #[test]
    fn smoke_test_thematic_break_mixed_chars_fails() {
        // Mixed characters are not allowed
        let input = Span::new("*-*\n");
        let result = thematic_break(input);
        
        assert!(result.is_err(), "Mixed characters should fail");
    }
    
    // Code block wrapper smoke test
    
    #[test]
    fn smoke_test_code_block_wrapper_fenced() {
        // code_block() should accept fenced code blocks
        let input = Span::new("```\ntest\n```\n");
        let result = code_block(input);
        
        // Just verify it parses successfully
        assert!(result.is_ok(), "Should parse fenced code block");
        let (_, content) = result.unwrap();
        assert!(!content.fragment().is_empty(), "Content should not be empty");
    }
    
    #[test]
    fn smoke_test_code_block_wrapper_indented() {
        // code_block() currently only tries fenced code blocks
        // Indented code blocks are not yet integrated (TODO in code)
        let input = Span::new("    test\n");
        let result = code_block(input);
        
        // Currently returns error because indented code blocks not integrated
        assert!(result.is_err(), "code_block() doesn't handle indented blocks yet");
    }
    
    // Indented code block smoke tests
    
    #[test]
    fn smoke_test_indented_code_simple() {
        // Simple indented code block with trailing newline
        let input = Span::new("    foo\n");
        let result = indented_code_block(input);
        
        assert!(result.is_ok(), "Should parse indented code block");
        let (_, content) = result.unwrap();
        assert_eq!(content.fragment(), &"    foo\n");
    }
    
    #[test]
    fn smoke_test_indented_code_no_trailing_newline() {
        // Example 117 from CommonMark spec
        // Just "    foo" with no trailing newline
        let input = Span::new("    foo");
        let result = indented_code_block(input);
        
        assert!(result.is_ok(), "Should parse indented code block without trailing newline");
        let (_, content) = result.unwrap();
        assert_eq!(content.fragment(), &"    foo");
    }
    
    // === List Marker Detection Tests ===
    
    #[test]
    fn smoke_test_bullet_marker_dash() {
        let input = Span::new("- item");
        let result = detect_list_marker(input);
        
        assert!(result.is_ok(), "Should detect dash bullet marker");
        let (remaining, (marker, content_indent)) = result.unwrap();
        assert_eq!(marker, ListMarker::Bullet('-'));
        assert_eq!(content_indent, 2); // 1 for marker + 1 for space
        assert_eq!(remaining.fragment(), &" item"); // Now includes the space
    }
    
    #[test]
    fn smoke_test_bullet_marker_plus() {
        let input = Span::new("+ item");
        let result = detect_list_marker(input);
        
        assert!(result.is_ok(), "Should detect plus bullet marker");
        let (remaining, (marker, content_indent)) = result.unwrap();
        assert_eq!(marker, ListMarker::Bullet('+'));
        assert_eq!(content_indent, 2);
        assert_eq!(remaining.fragment(), &" item"); // Now includes the space
    }
    
    #[test]
    fn smoke_test_bullet_marker_asterisk() {
        let input = Span::new("* item");
        let result = detect_list_marker(input);
        
        assert!(result.is_ok(), "Should detect asterisk bullet marker");
        let (remaining, (marker, content_indent)) = result.unwrap();
        assert_eq!(marker, ListMarker::Bullet('*'));
        assert_eq!(content_indent, 2);
        assert_eq!(remaining.fragment(), &" item"); // Now includes the space
    }
    
    #[test]
    fn smoke_test_ordered_marker_period() {
        let input = Span::new("1. item");
        let result = detect_list_marker(input);
        
        assert!(result.is_ok(), "Should detect ordered marker with period");
        let (remaining, (marker, content_indent)) = result.unwrap();
        assert_eq!(marker, ListMarker::Ordered { number: 1, delimiter: '.' });
        assert_eq!(content_indent, 3); // 1 digit + 1 period + 1 space
        assert_eq!(remaining.fragment(), &" item"); // Now includes the space (will be dedented later)
    }
    
    #[test]
    fn smoke_test_ordered_marker_paren() {
        let input = Span::new("1) item");
        let result = detect_list_marker(input);
        
        assert!(result.is_ok(), "Should detect ordered marker with paren");
        let (remaining, (marker, content_indent)) = result.unwrap();
        assert_eq!(marker, ListMarker::Ordered { number: 1, delimiter: ')' });
        assert_eq!(content_indent, 3);
        assert_eq!(remaining.fragment(), &" item"); // Now includes the space
    }
    
    #[test]
    fn smoke_test_ordered_marker_multidigit() {
        let input = Span::new("123. item");
        let result = detect_list_marker(input);
        
        assert!(result.is_ok(), "Should detect multi-digit ordered marker");
        let (remaining, (marker, content_indent)) = result.unwrap();
        assert_eq!(marker, ListMarker::Ordered { number: 123, delimiter: '.' });
        assert_eq!(content_indent, 5); // 3 digits + 1 period + 1 space
        assert_eq!(remaining.fragment(), &" item"); // Now includes the space
    }
    
    #[test]
    fn smoke_test_marker_with_leading_spaces() {
        let input = Span::new("  - item");
        let result = detect_list_marker(input);
        
        assert!(result.is_ok(), "Should detect marker with leading spaces");
        let (remaining, (marker, content_indent)) = result.unwrap();
        assert_eq!(marker, ListMarker::Bullet('-'));
        assert_eq!(content_indent, 4); // 2 leading + 1 marker + 1 space
        assert_eq!(remaining.fragment(), &" item"); // Now includes the space
    }
    
    #[test]
    fn smoke_test_marker_with_multiple_spaces_after() {
        let input = Span::new("-   item");
        let result = detect_list_marker(input);
        
        assert!(result.is_ok(), "Should detect marker with multiple spaces after");
        let (remaining, (marker, content_indent)) = result.unwrap();
        assert_eq!(marker, ListMarker::Bullet('-'));
        assert_eq!(content_indent, 4); // 1 marker + 3 spaces (capped at 4 total)
        assert_eq!(remaining.fragment(), &"   item"); // Now includes all the spaces
    }
    
    #[test]
    fn smoke_test_marker_fails_no_space_after() {
        let input = Span::new("-item");
        let result = detect_list_marker(input);
        
        assert!(result.is_err(), "Should fail without space after marker");
    }
    
    #[test]
    fn smoke_test_marker_fails_too_many_leading_spaces() {
        let input = Span::new("    - item");
        let result = detect_list_marker(input);
        
        assert!(result.is_err(), "Should fail with 4+ leading spaces");
    }
    
    #[test]
    fn smoke_test_marker_fails_ordered_no_space() {
        let input = Span::new("1.item");
        let result = detect_list_marker(input);
        
        assert!(result.is_err(), "Should fail ordered marker without space");
    }
    
    #[test]
    fn smoke_test_marker_with_tab_after() {
        let input = Span::new("-\titem");
        let result = detect_list_marker(input);
        
        assert!(result.is_ok(), "Should detect marker with tab after");
        let (remaining, (marker, content_indent)) = result.unwrap();
        assert_eq!(marker, ListMarker::Bullet('-'));
        assert_eq!(content_indent, 4); // 1 marker + tab (column 1→4 = 3 spaces)
        assert_eq!(remaining.fragment(), &"\titem"); // Now includes the tab
    }
    
    // === List Item Parser Tests ===
    
    #[test]
    fn smoke_test_list_item_simple_single_line() {
        let input = Span::new("- item one\n");
        let result = list_item(input, None);
        
        assert!(result.is_ok(), "Should parse simple single-line list item");
        let (remaining, (marker, content, has_blank, _content_indent)) = result.unwrap();
        eprintln!("Content: {:?}", content.fragment());
        eprintln!("Remaining: {:?}", remaining.fragment());
        assert_eq!(marker, ListMarker::Bullet('-'));
        assert!(content.fragment().contains("item one"), "Content was: {:?}", content.fragment());
        assert!(!has_blank, "Single line should not have blank lines");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_list_item_stops_at_next_marker() {
        let input = Span::new("- item one\n- item two\n");
        let result = list_item(input, None);
        
        assert!(result.is_ok(), "Should parse first item and stop at second marker");
        let (remaining, (marker, content, _, _)) = result.unwrap();
        assert_eq!(marker, ListMarker::Bullet('-'));
        assert!(content.fragment().contains("item one"));
        assert!(!content.fragment().contains("item two"));
        assert!(remaining.fragment().starts_with("- item two"));
    }
    
    #[test]
    fn smoke_test_list_item_multiline_indented() {
        let input = Span::new("- item one\n  continuation\n");
        let result = list_item(input, None);
        
        assert!(result.is_ok(), "Should parse multi-line item with indented continuation");
        let (remaining, (marker, content, _, _)) = result.unwrap();
        assert_eq!(marker, ListMarker::Bullet('-'));
        assert!(content.fragment().contains("item one"));
        assert!(content.fragment().contains("continuation"));
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_list_item_with_blank_line() {
        let input = Span::new("- item one\n\n  continuation\n");
        let result = list_item(input, None);
        
        assert!(result.is_ok(), "Should parse item with blank line and continuation");
        let (remaining, (marker, content, has_blank, _content_indent)) = result.unwrap();
        assert_eq!(marker, ListMarker::Bullet('-'));
        assert!(content.fragment().contains("item one"));
        assert!(content.fragment().contains("continuation"));
        assert!(has_blank, "Should detect blank line in content");
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_list_item_ordered() {
        let input = Span::new("1. first item\n");
        let result = list_item(input, None);
        
        assert!(result.is_ok(), "Should parse ordered list item");
        let (remaining, (marker, content, _, _)) = result.unwrap();
        assert_eq!(marker, ListMarker::Ordered { number: 1, delimiter: '.' });
        assert!(content.fragment().contains("first item"));
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_list_item_type_mismatch() {
        let input = Span::new("- bullet item\n");
        let expected = ListMarker::Ordered { number: 1, delimiter: '.' };
        let result = list_item(input, Some(expected));
        
        assert!(result.is_err(), "Should fail when marker type doesn't match expected");
    }
    
    #[test]
    fn smoke_test_list_item_lazy_continuation() {
        let input = Span::new("- item\ncontinuation without indent\n");
        let result = list_item(input, None);
        
        assert!(result.is_ok(), "Should handle lazy continuation");
        let (_remaining, (_, content, _, _)) = result.unwrap();
        eprintln!("Lazy content: {:?}", content.fragment());
        assert!(content.fragment().contains("item"));
        assert!(content.fragment().contains("continuation"), "Content was: {:?}", content.fragment());
    }
    
    // === List Parser Tests ===
    
    #[test]
    fn smoke_test_list_simple_bullet() {
        let input = Span::new("- item 1\n- item 2\n- item 3\n");
        let result = list(input);
        
        assert!(result.is_ok(), "Should parse simple bullet list");
        let (remaining, items) = result.unwrap();
        eprintln!("Items parsed: {}", items.len());
        for (i, item) in items.iter().enumerate() {
            eprintln!("Item {}: {:?}", i, item.1.fragment());
        }
        eprintln!("Remaining: {:?}", remaining.fragment());
        assert_eq!(items.len(), 3, "Should have 3 items");
        assert_eq!(items[0].0, ListMarker::Bullet('-'));
        assert!(items[0].1.fragment().contains("item 1"));
        assert!(items[1].1.fragment().contains("item 2"));
        assert!(items[2].1.fragment().contains("item 3"));
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_list_simple_ordered() {
        let input = Span::new("1. first\n2. second\n3. third\n");
        let result = list(input);
        
        assert!(result.is_ok(), "Should parse simple ordered list");
        let (remaining, items) = result.unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].0, ListMarker::Ordered { number: 1, delimiter: '.' });
        assert_eq!(items[1].0, ListMarker::Ordered { number: 2, delimiter: '.' });
        assert_eq!(items[2].0, ListMarker::Ordered { number: 3, delimiter: '.' });
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_list_tight() {
        let input = Span::new("- item 1\n- item 2\n");
        let result = list(input);
        
        assert!(result.is_ok(), "Should parse tight list");
        let (_remaining, items) = result.unwrap();
        assert_eq!(items.len(), 2);
        // Check that there are no blank lines between items
        assert!(!items[0].3, "Should not have blank before next item");
        assert!(!items[0].2, "First item should not have internal blank lines");
        assert!(!items[1].2, "Second item should not have internal blank lines");
    }
    
    #[test]
    fn smoke_test_list_loose() {
        let input = Span::new("- item 1\n\n- item 2\n");
        let result = list(input);
        
        assert!(result.is_ok(), "Should parse loose list");
        let (_remaining, items) = result.unwrap();
        eprintln!("Loose list items: {}", items.len());
        for (i, item) in items.iter().enumerate() {
            eprintln!("Item {}: has_blank_in={}, has_blank_before_next={}", i, item.2, item.3);
        }
        assert_eq!(items.len(), 2);
        // Check that there's a blank line between items
        assert!(items[0].3, "Should have blank before next item");
    }
    
    #[test]
    fn smoke_test_list_stops_at_different_marker() {
        let input = Span::new("- bullet item\n1. ordered item\n");
        let result = list(input);
        
        assert!(result.is_ok(), "Should stop at different marker type");
        let (remaining, items) = result.unwrap();
        assert_eq!(items.len(), 1, "Should only parse bullet list");
        assert_eq!(items[0].0, ListMarker::Bullet('-'));
        assert!(remaining.fragment().starts_with("1. ordered"));
    }
    
    #[test]
    fn smoke_test_list_multiline_items() {
        let input = Span::new("- item 1\n  continuation\n- item 2\n");
        let result = list(input);
        
        assert!(result.is_ok(), "Should parse list with multi-line items");
        let (_remaining, items) = result.unwrap();
        assert_eq!(items.len(), 2);
        assert!(items[0].1.fragment().contains("continuation"));
    }
    
    #[test]
    fn smoke_test_list_internal_blank_lines() {
        let input = Span::new("- item 1\n\n  continuation\n- item 2\n");
        let result = list(input);
        
        assert!(result.is_ok(), "Should parse list with internal blank lines");
        let (_remaining, items) = result.unwrap();
        assert_eq!(items.len(), 2);
        assert!(items[0].2, "First item should have internal blank lines");
    }
    
    // === Blockquote Parser Tests ===
    
    #[test]
    fn smoke_test_blockquote_basic() {
        let input = Span::new("> quote\n");
        let result = blockquote(input);
        
        assert!(result.is_ok(), "Should parse basic blockquote");
        let (remaining, content) = result.unwrap();
        assert!(content.fragment().contains("> quote"));
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_blockquote_multiline() {
        let input = Span::new("> line one\n> line two\n");
        let result = blockquote(input);
        
        assert!(result.is_ok(), "Should parse multi-line blockquote");
        let (remaining, content) = result.unwrap();
        assert!(content.fragment().contains("> line one"));
        assert!(content.fragment().contains("> line two"));
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_blockquote_lazy_continuation() {
        let input = Span::new("> line one\ncontinuation\n");
        let result = blockquote(input);
        
        assert!(result.is_ok(), "Should parse blockquote with lazy continuation");
        let (_remaining, content) = result.unwrap();
        assert!(content.fragment().contains("> line one"));
        assert!(content.fragment().contains("continuation"));
    }
    
    #[test]
    fn smoke_test_blockquote_lazy_with_equals() {
        // Example 93 from CommonMark spec - note NO trailing newline
        let input = Span::new("> foo\nbar\n===");
        let result = blockquote(input);
        
        assert!(result.is_ok(), "Should parse blockquote with === lazy continuation");
        let (remaining, content) = result.unwrap();
        eprintln!("Content: {:?}", content.fragment());
        eprintln!("Remaining: {:?}", remaining.fragment());
        assert!(content.fragment().contains("> foo"));
        assert!(content.fragment().contains("bar"), "Should include 'bar' as lazy continuation");
        assert!(content.fragment().contains("==="), "Should include '===' as lazy continuation");
        assert_eq!(remaining.fragment(), &"", "Should consume all input");
    }
    
    #[test]
    fn smoke_test_blockquote_with_space_after_marker() {
        let input = Span::new(">  text with space\n");
        let result = blockquote(input);
        
        assert!(result.is_ok(), "Should parse blockquote with space after >");
        let (_remaining, content) = result.unwrap();
        assert!(content.fragment().contains(">"));
    }
    
    #[test]
    fn smoke_test_blockquote_empty() {
        let input = Span::new(">\n");
        let result = blockquote(input);
        
        assert!(result.is_ok(), "Should parse empty blockquote");
        let (_, content) = result.unwrap();
        assert_eq!(content.fragment().trim(), ">");
    }
    
    #[test]
    fn smoke_test_blockquote_leading_spaces() {
        let input = Span::new("   > quote with leading spaces\n");
        let result = blockquote(input);
        
        assert!(result.is_ok(), "Should parse blockquote with 0-3 leading spaces");
        let (_, content) = result.unwrap();
        assert!(content.fragment().contains(">"));
    }
    
    #[test]
    fn smoke_test_blockquote_stops_at_blank() {
        let input = Span::new("> quote\n\nnot quote\n");
        let result = blockquote(input);
        
        assert!(result.is_ok(), "Should stop at blank line");
        let (remaining, content) = result.unwrap();
        assert!(content.fragment().contains("> quote"));
        assert!(!content.fragment().contains("not quote"));
        assert!(remaining.fragment().starts_with("\nnot quote"));
    }
    
    #[test]
    fn smoke_test_list_item_empty() {
        let input = Span::new("-\n");
        let result = list_item(input, None);
        
        assert!(result.is_ok(), "Should parse empty list item");
        let (_remaining, (marker, content, _, _)) = result.unwrap();
        assert_eq!(marker, ListMarker::Bullet('-'));
        eprintln!("Empty item content: {:?}", content.fragment());
        // Content should be empty or just whitespace
        assert!(content.fragment().trim().is_empty(), "Content should be empty, was: {:?}", content.fragment());
    }
    
    #[test]
    fn smoke_test_list_empty_items() {
        let input = Span::new("- foo\n-\n- bar\n");
        let result = list(input);
        
        assert!(result.is_ok(), "Should parse list with empty item");
        let (_remaining, items) = result.unwrap();
        eprintln!("Number of items: {}", items.len());
        for (i, item) in items.iter().enumerate() {
            eprintln!("Item {}: {:?}", i, item.1.fragment());
        }
        assert_eq!(items.len(), 3, "Should have 3 items");
        assert!(items[0].1.fragment().contains("foo"));
        assert!(items[1].1.fragment().trim().is_empty(), "Second item should be empty");
        assert!(items[2].1.fragment().contains("bar"));
    }
    
    // === Link Reference Definition Tests ===
    
    #[test]
    fn smoke_test_link_ref_basic() {
        let input = Span::new("[foo]: /url\n");
        let result = link_reference_definition(input);
        
        assert!(result.is_ok(), "Should parse basic link reference");
        let (remaining, (label, url, title)) = result.unwrap();
        assert_eq!(label, "foo");
        assert_eq!(url, "/url");
        assert_eq!(title, None);
        assert_eq!(remaining.fragment(), &"");
    }
    
    #[test]
    fn smoke_test_link_ref_with_title() {
        let input = Span::new("[foo]: /url \"title\"\n");
        let result = link_reference_definition(input);
        
        assert!(result.is_ok(), "Should parse link reference with title");
        let (_remaining, (label, url, title)) = result.unwrap();
        assert_eq!(label, "foo");
        assert_eq!(url, "/url");
        assert_eq!(title, Some("title".to_string()));
    }
    
    #[test]
    fn smoke_test_link_ref_angle_brackets() {
        let input = Span::new("[foo]: <http://example.com>\n");
        let result = link_reference_definition(input);
        
        assert!(result.is_ok(), "Should parse link reference with angle brackets");
        let (_remaining, (label, url, _)) = result.unwrap();
        assert_eq!(label, "foo");
        assert_eq!(url, "http://example.com");
    }
    
    #[test]
    fn smoke_test_link_ref_multiline() {
        let input = Span::new("[foo]:\n  /url\n");
        let result = link_reference_definition(input);
        
        assert!(result.is_ok(), "Should parse link reference across lines");
        let (_remaining, (label, url, _)) = result.unwrap();
        assert_eq!(label, "foo");
        assert_eq!(url, "/url");
    }
    
    #[test]
    fn smoke_test_link_ref_title_parens() {
        let input = Span::new("[foo]: /url (title)\n");
        let result = link_reference_definition(input);
        
        assert!(result.is_ok(), "Should parse link reference with parenthesized title");
        let (_remaining, (label, url, title)) = result.unwrap();
        assert_eq!(label, "foo");
        assert_eq!(url, "/url");
        assert_eq!(title, Some("title".to_string()));
    }
}

