/// Delimiter stack entry for emphasis/strong parsing (CommonMark spec)
/// See: https://spec.commonmark.org/0.31.2/#emphasis-and-strong-emphasis
/// and pulldown-cmark/comrak implementations
struct Delim {
    /// Delimiter character: '*' or '_'
    ch: char,
    /// Number of consecutive delimiters in this run
    count: usize,
    /// Source position (line, column) for error reporting and AST
    pos: SourcePos,
    /// Can this delimiter open an emphasis/strong span? (left-flanking)
    can_open: bool,
    /// Can this delimiter close an emphasis/strong span? (right-flanking)
    can_close: bool,
    /// Index in the input string (for stack processing)
    idx: usize,
    /// Is this delimiter active? (for future link/image nesting)
    active: bool,
}

// Inline parser architecture:
// - Delimiter stack for emphasis/strong (see struct Delim)
// - Bracket stack for links/images (see struct Bracket)
// - Attribute parsing handled in AST construction and event emission
// - Tokenizer: raw Markdown → Inline AST nodes (with SourcePos for tracking)
// - parse_phrases: main entry point for inline parsing

/// Bracket stack entry for link/image parsing (CommonMark spec)
/// See: https://spec.commonmark.org/0.31.2/#links
#[derive(Debug, Clone)]
struct Bracket {
    // Removed unused 'opener' field; only openers are pushed to the stack.
    /// Is this an image opener? ('![')
    image: bool,
    /// Source position (line, column) for error reporting and AST
    pos: SourcePos,
    /// Index in the input string (for stack processing)
    idx: usize,
}
use crate::logic::ast::inlines::{Inline, CodeSpan};
use crate::logic::ast::math::MathInline;

use crate::logic::core::event_types::SourcePos;

/// Parse a string into a sequence of Inline AST nodes (phrases) with source positions.
pub fn parse_phrases(input: &str) -> (Vec<(Inline, SourcePos)>, Vec<crate::logic::core::event_types::Event>) {

    let mut chars = input.chars().peekable();
    let mut buffer = String::new();
    let mut result = Vec::new();
    let mut delim_stack: Vec<Delim> = Vec::new();
    let mut bracket_stack: Vec<Bracket> = Vec::new(); // For link/image parsing
    let events = Vec::new();
    let line = 1;
    let mut column = 1;

    // Main parser loop
    while let Some(c) = chars.next() {
        dbg!(c);
        // 8. Attribute blocks ({...})
        if c == '{' {
            dbg!("Attribute block start", &buffer);
            // Flush buffer before attribute block
            if !buffer.is_empty() {
                result.push((Inline::Text(std::mem::take(&mut buffer)), SourcePos { line, column }));
            }
            let start_column = column;
            let mut attr_buf = String::new();
            attr_buf.push('{');
            let mut depth = 1;
            while let Some(next) = chars.next() {
                attr_buf.push(next);
                if next == '{' {
                    depth += 1;
                } else if next == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
            // For now, treat as literal text; can be parsed into attributes if desired
            result.push((Inline::Text(attr_buf), SourcePos { line, column: start_column }));
            continue;
        }
        // 7. Entity and numeric references
        if c == '&' {
            dbg!("Entity/numeric reference start", &buffer);
            // Flush buffer before entity
            if !buffer.is_empty() {
                result.push((Inline::Text(std::mem::take(&mut buffer)), SourcePos { line, column }));
            }
            let start_column = column;
            let mut entity_buf = String::new();
            entity_buf.push('&');
            while let Some(next) = chars.next() {
                entity_buf.push(next);
                if next == ';' {
                    break;
                }
            }
            // For now, treat as literal text; replace with decoded entity if desired
            result.push((Inline::Text(entity_buf), SourcePos { line, column: start_column }));
            continue;
        }
        // 6. Backslash escapes
        if c == '\\' {
            dbg!("Backslash escape", &buffer);
            if let Some(next) = chars.next() {
                // Only escape punctuation per CommonMark spec
                const ESCAPABLE: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
                if ESCAPABLE.contains(next) {
                    buffer.push(next);
                } else {
                    buffer.push('\\');
                    buffer.push(next);
                }
                continue;
            } else {
                buffer.push('\\');
                continue;
            }
        }
        // 5. Hard and soft line breaks
        if c == '\n' {
            dbg!("Line break", &buffer);
            // Check for hard break: two spaces or backslash before newline
            let is_hard = buffer.ends_with("  ") || buffer.ends_with("\\");
            if !buffer.is_empty() {
                // Remove trailing spaces or backslash for hard break
                if is_hard {
                    let len = buffer.len();
                    if buffer.ends_with("  ") {
                        buffer.truncate(len - 2);
                    } else if buffer.ends_with("\\") {
                        buffer.truncate(len - 1);
                    }
                }
                result.push((Inline::Text(std::mem::take(&mut buffer)), SourcePos { line, column }));
            }
            if is_hard {
                result.push((Inline::HardBreak, SourcePos { line, column }));
            } else {
                result.push((Inline::SoftBreak, SourcePos { line, column }));
            }
            continue;
        }
        // 4. Raw HTML (<tag>, comments, CDATA, etc.)
        // Only parse if not already handled as autolink
        if c == '<' {
            dbg!("Raw HTML or Autolink start", &buffer);
            dbg!("Autolink start", &buffer);
            // Look ahead to distinguish autolink vs raw HTML
            let mut lookahead = String::new();
            let mut temp_chars = chars.clone();
            for _ in 0..10 {
                if let Some(next) = temp_chars.next() {
                    lookahead.push(next);
                } else {
                    break;
                }
            }
            let is_autolink = lookahead.contains('@') || lookahead.contains(":");
            if !is_autolink {
                // Flush buffer before raw HTML
                if !buffer.is_empty() {
                    result.push((Inline::Text(std::mem::take(&mut buffer)), SourcePos { line, column }));
                }
                let start_column = column;
                let mut html_buf = String::new();
                html_buf.push('<');
                while let Some(next) = chars.next() {
                    html_buf.push(next);
                    if next == '>' {
                        break;
                    }
                }
                result.push((Inline::RawHtml(html_buf), SourcePos { line, column: start_column }));
                continue;
            }
            // If autolink, let autolink branch handle it
        }
        // 3. Autolinks (<scheme:...> and <email@...>)
        if c == '<' {
            // Flush buffer before autolink
            if !buffer.is_empty() {
                result.push((Inline::Text(std::mem::take(&mut buffer)), SourcePos { line, column }));
            }
            let start_column = column;
            let mut autolink_buf = String::new();
            let mut found_closing = false;
            while let Some(next) = chars.next() {
                if next == '>' {
                    found_closing = true;
                    break;
                } else {
                    autolink_buf.push(next);
                }
            }
            if found_closing {
                // Check if autolink_buf matches URI or email pattern
                let is_email = autolink_buf.contains('@') && !autolink_buf.contains(':');
                let is_uri = autolink_buf.contains(':');
                use crate::logic::ast::inlines::Autolink;
                if is_email {
                    result.push((Inline::Autolink(Autolink::Email(autolink_buf)), SourcePos { line, column: start_column }));
                } else if is_uri {
                    result.push((Inline::Autolink(Autolink::Uri(autolink_buf)), SourcePos { line, column: start_column }));
                } else {
                    // Not a valid autolink, treat as RawHtml
                    result.push((Inline::RawHtml(format!("<{}>", autolink_buf)), SourcePos { line, column: start_column }));
                }
            } else {
                // No closing '>', treat as literal text
                buffer.push('<');
                buffer.push_str(&autolink_buf);
            }
            continue;
        }
        // 2. Math spans ($...$)
        if c == '$' {
            dbg!("Math span start", &buffer);
            // Flush buffer before math span
            if !buffer.is_empty() {
                result.push((Inline::Text(std::mem::take(&mut buffer)), SourcePos { line, column }));
            }
            // Count opening dollar run
            let mut dollar_count = 1;
            let start_column = column;
            while let Some(&next) = chars.peek() {
                if next == '$' {
                    chars.next();
                    dollar_count += 1;
                    column += 1;
                } else {
                    break;
                }
            }
            // Collect math span content until matching closing run
            let mut math_buf = String::new();
            let mut temp = String::new();
            while let Some(next) = chars.next() {
                if next == '$' {
                    temp.clear();
                    temp.push(next);
                    let mut match_count = 1;
                    while let Some(&peek) = chars.peek() {
                        if peek == '$' {
                            chars.next();
                            temp.push('$');
                            match_count += 1;
                        } else {
                            break;
                        }
                    }
                    if match_count == dollar_count {
                        break;
                    } else {
                        math_buf.push_str(&temp);
                    }
                } else {
                    math_buf.push(next);
                }
            }
            // Normalize spaces and newlines
            let normalized = math_buf
                .replace("\r\n", "\n")
                .replace("\r", "\n")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            // Construct MathInline node
            result.push((Inline::Math(MathInline {
                content: normalized,
                math_type: crate::logic::ast::math::MathType::LaTeX, // Default to LaTeX
                position: Some(SourcePos { line, column: start_column }),
                attributes: None,
            }), SourcePos { line, column: start_column }));
            continue;
        }
        // --- Inline Parsing Branches ---
        // 1. Code spans (multiple backticks, normalization, no escapes)
        if c == '`' {
            dbg!("Code span start", &buffer);
            // Flush buffer before code span
            if !buffer.is_empty() {
                result.push((Inline::Text(std::mem::take(&mut buffer)), SourcePos { line, column }));
            }
            // Count opening backtick run
            let mut tick_count = 1;
            let start_column = column;
            while let Some(&next) = chars.peek() {
                if next == '`' {
                    chars.next();
                    tick_count += 1;
                    column += 1;
                } else {
                    break;
                }
            }
            // Collect code span content until matching closing run
            let mut code_buf = String::new();
            let mut temp = String::new();
            while let Some(next) = chars.next() {
                if next == '`' {
                    temp.clear();
                    temp.push(next);
                    let mut match_count = 1;
                    while let Some(&peek) = chars.peek() {
                        if peek == '`' {
                            chars.next();
                            temp.push('`');
                            match_count += 1;
                        } else {
                            break;
                        }
                    }
                    if match_count == tick_count {
                        break;
                    } else {
                        code_buf.push_str(&temp);
                    }
                } else {
                    code_buf.push(next);
                }
            }
            // Normalize spaces and newlines per CommonMark
            let normalized = code_buf
                .replace("\r\n", "\n")
                .replace("\r", "\n")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            result.push((Inline::CodeSpan(CodeSpan { content: normalized, attributes: None }), SourcePos { line, column: start_column }));
            continue;
        }

        // 2. Link/image openers: '[', '!['
        if c == '[' || (c == '!' && chars.peek() == Some(&'[')) {
            dbg!("Link/image opener", &buffer);
            // Flush buffer before bracket
            if !buffer.is_empty() {
                result.push((Inline::Text(std::mem::take(&mut buffer)), SourcePos { line, column }));
            }
            let is_image = c == '!';
            if is_image {
                chars.next(); // consume '[' after '!'
            }
            // Track the index in result vector for opener
            bracket_stack.push(Bracket {
                image: is_image,
                pos: SourcePos { line, column },
                idx: result.len(), // index in result vector
            });
            continue;
        }

        // 3. Link/image closer: ']'
        if c == ']' {
            dbg!("Link/image closer", &buffer);
            // Flush buffer before bracket
            if !buffer.is_empty() {
                result.push((Inline::Text(std::mem::take(&mut buffer)), SourcePos { line, column }));
            }
            // Pop last opener from bracket stack if present
            if let Some(bracket) = bracket_stack.pop() {
                // Extract content between opener and closer
                let start = bracket.idx;
                let end = result.len();
                let mut inner = Vec::new();
                if end > start {
                    inner = result.drain(start..end).collect();
                }
                // Deactivate delimiters before this link opener (CommonMark spec)
                for delim in delim_stack.iter_mut() {
                    if delim.idx < bracket.idx {
                        delim.active = false;
                    }
                }
                // Look ahead for link/image destination and title
                let mut destination = None;
                let mut title = None;
                let mut is_inline = false;
                let mut temp_chars = chars.clone();
                // Inline link/image: [text](dest "title") or ![alt](src "title")
                if temp_chars.peek() == Some(&'(') {
                    chars.next(); // consume '('
                    // Parse destination (URL or src)
                    let mut dest_buf = String::new();
                    let mut title_buf = String::new();
                    let mut in_title = false;
                    let mut paren_depth = 1;
                    while let Some(&ch) = chars.peek() {
                        if ch == ')' && paren_depth == 1 {
                            chars.next();
                            break;
                        }
                        if ch == '(' {
                            paren_depth += 1;
                        }
                        if ch == ')' {
                            paren_depth -= 1;
                        }
                        if !in_title && (ch == '"' || ch == '\'' || ch == '(') {
                            in_title = true;
                            chars.next();
                            continue;
                        }
                        if in_title {
                            title_buf.push(ch);
                        } else {
                            dest_buf.push(ch);
                        }
                        chars.next();
                    }
                    destination = Some(dest_buf.trim().to_string());
                    if !title_buf.is_empty() {
                        title = Some(title_buf.trim().to_string());
                    }
                    is_inline = true;
                }
                // Reference/collapsed/shortcut: [text][label], [text][], [label]
                let mut label = None;
                if !is_inline && temp_chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    let mut label_buf = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == ']' {
                            chars.next();
                            break;
                        }
                        label_buf.push(ch);
                        chars.next();
                    }
                    label = Some(label_buf.trim().to_string());
                }
                // Collapsed: [text][]
                if !is_inline && label == Some(String::new()) {
                    label = Some(inner.iter().map(|(i,_)| match i { Inline::Text(s) => s.clone(), _ => String::new() }).collect::<Vec<_>>().join(" "));
                }
                // Shortcut: [label]
                if !is_inline && label.is_none() {
                    label = Some(inner.iter().map(|(i,_)| match i { Inline::Text(s) => s.clone(), _ => String::new() }).collect::<Vec<_>>().join(" "));
                }
                // Build AST node
                let node = if bracket.image {
                    Inline::Image(crate::logic::ast::inlines::Image {
                        alt: inner,
                        destination: if is_inline {
                            crate::logic::ast::inlines::LinkDestination::Inline(destination.unwrap_or_default())
                        } else {
                            crate::logic::ast::inlines::LinkDestination::Reference(label.unwrap_or_default())
                        },
                        title,
                        attributes: None,
                    })
                } else {
                    Inline::Link(crate::logic::ast::inlines::Link {
                        label: inner,
                        destination: if is_inline {
                            crate::logic::ast::inlines::LinkDestination::Inline(destination.unwrap_or_default())
                        } else {
                            crate::logic::ast::inlines::LinkDestination::Reference(label.unwrap_or_default())
                        },
                        title,
                        attributes: None,
                    })
                };
                result.push((node, bracket.pos.clone()));
            } else {
                // Unmatched closing bracket, treat as text
                result.push((Inline::Text(String::from("]")), SourcePos { line, column }));
            }
            continue;
        }

        // 4. Emphasis/strong delimiter runs (*, _)
        if c == '*' || c == '_' {
            dbg!("Emphasis/strong delimiter run", &buffer);
            // Flush buffer before delimiter run
            if !buffer.is_empty() {
                result.push((Inline::Text(std::mem::take(&mut buffer)), SourcePos { line, column }));
            }
            // Count delimiter run length
            let mut delim_count = 1;
            let start_column = column;
            while let Some(&next) = chars.peek() {
                if next == c {
                    chars.next();
                    delim_count += 1;
                    column += 1;
                } else {
                    break;
                }
            }
            // Determine left/right-flanking per CommonMark spec
            // See: https://spec.commonmark.org/0.31.2/#emphasis-and-strong-emphasis
            // Get previous and next character for flanking checks
            let prev_char = if column > delim_count {
                input.chars().nth(column - delim_count - 1)
            } else {
                None
            };
            let next_char = chars.peek().copied();
            let is_whitespace = |ch: Option<char>| ch.map_or(true, |c| c.is_whitespace());
            let is_punct = |ch: Option<char>| ch.map_or(false, |c| c.is_ascii_punctuation());
            // Left-flanking: not followed by whitespace, and either not followed by punctuation or followed by punctuation and preceded by whitespace/punctuation
            let left_flanking = !is_whitespace(next_char) && (!is_punct(next_char) || is_whitespace(prev_char) || is_punct(prev_char));
            // Right-flanking: not preceded by whitespace, and either not preceded by punctuation or preceded by punctuation and followed by whitespace/punctuation
            let right_flanking = !is_whitespace(prev_char) && (!is_punct(prev_char) || is_whitespace(next_char) || is_punct(next_char));
            // Intraword: '*' allows, '_' does not
            let can_open = if c == '*' {
                left_flanking
            } else {
                left_flanking && (!right_flanking || is_punct(prev_char))
            };
            let can_close = if c == '*' {
                right_flanking
            } else {
                right_flanking && (!left_flanking || is_punct(next_char))
            };
            // Push delimiter run to stack
            delim_stack.push(Delim {
                ch: c,
                count: delim_count,
                pos: SourcePos { line, column: start_column },
                can_open,
                can_close,
                idx: column - delim_count, // index in input
                active: true,
            });
            continue;
        }

        // ...other parsing branches...
        buffer.push(c);
        column += 1;
    }

    // Flush buffer as plain text
    if !buffer.is_empty() {
        result.push((Inline::Text(buffer), SourcePos { line, column }));
    }

    // Delimiter stack post-processing for emphasis/strong
    // See CommonMark spec: https://spec.commonmark.org/0.31.2/#emphasis-and-strong-emphasis
    // and pulldown-cmark/comrak implementations
    let mut used = vec![false; delim_stack.len()];
    let mut i = 0;
    while i < delim_stack.len() {
        if !delim_stack[i].can_close {
            i += 1;
            continue;
        }
        let closer = &delim_stack[i];
        // Look back for matching opener
        let mut j = i;
        while j > 0 {
            j -= 1;
            let opener = &delim_stack[j];
            if used[j] || !opener.can_open || opener.ch != closer.ch {
                continue;
            }
            // Multiples-of-3 rule
            let sum = opener.count + closer.count;
            let both_mult_3 = opener.count % 3 == 0 && closer.count % 3 == 0;
            if (opener.can_close || closer.can_open) && sum % 3 == 0 && !both_mult_3 {
                continue;
            }
            // Determine emphasis/strong
            let min_count = opener.count.min(closer.count);
            let emph_type = if min_count >= 2 { "strong" } else { "emph" };
            // Extract actual inline nodes between opener and closer
            // Find indices in result vector corresponding to opener/closer
            let opener_idx = opener.idx;
            let closer_idx = closer.idx;
            // Defensive: ensure indices are valid and opener < closer
            let (start, end) = if opener_idx < closer_idx {
                (opener_idx + 1, closer_idx)
            } else {
                (closer_idx + 1, opener_idx)
            };
            // Extract nodes between delimiters
            let mut inner_nodes_with_pos = Vec::new();
            if end > start && end <= result.len() {
                inner_nodes_with_pos = result.drain(start..end).collect();
            }
            // Build AST node
            let ast = if emph_type == "strong" {
                Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Strong(
                    inner_nodes_with_pos,
                    None
                ))
            } else {
                Inline::Emphasis(crate::logic::ast::inlines::Emphasis::Emph(
                    inner_nodes_with_pos,
                    None
                ))
            };
            // Use opener's position for AST node
            if start >= result.len() {
                result.push((ast, opener.pos.clone()));
            } else {
                result.insert(start, (ast, opener.pos.clone()));
            }
            // Mark delimiters as used
            used[j] = true;
            used[i] = true;
            break;
        }
        i += 1;
    }
    // Unmatched delimiters become plain text
    for (idx, delim) in delim_stack.iter().enumerate() {
        if !used[idx] {
            let txt = delim.ch.to_string().repeat(delim.count);
            result.push((Inline::Text(txt), delim.pos.clone()));
        }
    }

    // Return AST and events
    return (result, events);
}
