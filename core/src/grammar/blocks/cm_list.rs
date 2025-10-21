// CommonMark List Grammar
// Parses ordered and unordered lists with proper nesting and tight/loose detection
//
// Per CommonMark spec:
// - Unordered list markers: -, +, * (must have space after)
// - Ordered list markers: 1-9 digits followed by . or ) (must have space after)
// - Leading spaces: 0-3 max before marker
// - Content indentation: marker position + spaces (capped at marker + 4)
// - Lazy continuation: non-blank lines continue item without full indentation
// - Tight/loose: lists with blank lines between items are "loose"
// - Nested lists: items can contain indented lists

use crate::grammar::shared::{count_indentation, Span};
use crate::grammar::blocks::{
    cm_fenced_code_block::fenced_code_block,
    cm_heading::heading,
    cm_html_blocks::html_comment,
    cm_thematic_break::thematic_break,
};
use nom::{
    IResult, Slice,
    bytes::complete::take,
    character::complete::{digit1, line_ending, one_of},
    combinator::opt,
};

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
/// Returns: (marker, content_span, has_blank_lines, content_indent)
/// 
/// The content_span includes all content belonging to this item (may span multiple lines).
/// has_blank_lines indicates whether there are blank lines within the item (affects tight/loose).
pub fn list_item(input: Span, expected_marker_type: Option<ListMarker>) -> IResult<Span, (ListMarker, Span, bool, usize)> {
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
            
            // Skip the first line using nom combinators
            let (new_remaining, _) = nom::bytes::complete::take_while(|c| c != '\n' && c != '\r')(remaining)?;
            let (new_remaining, _) = opt(line_ending)(new_remaining)?;
            
            content_end_offset = new_remaining.location_offset();
            remaining = new_remaining;
            continue;
        }
        
        // Now handle subsequent lines (not first line)
        if is_blank {
            // Blank line - could continue the item if followed by indented content
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
                        break;
                    }
                    if html_comment(next_line_span).is_ok() {
                        break;
                    }
                }
            }
            
            // Determine if we should include this blank line
            let should_include_blank = if skip_len < remaining.fragment().len() {
                let mut search_offset = skip_len;
                let mut found_non_blank = false;
                let mut next_non_blank_indent = 0;
                
                while search_offset < remaining.fragment().len() {
                    let search_text = &remaining.fragment()[search_offset..];
                    let line_end = search_text.find('\n').unwrap_or(search_text.len());
                    let line = &search_text[..line_end];
                    
                    if !line.trim().is_empty() {
                        found_non_blank = true;
                        next_non_blank_indent = count_indentation(line);
                        break;
                    }
                    
                    search_offset += line_end + 1;
                    if search_offset > remaining.fragment().len() {
                        break;
                    }
                }
                
                if !found_non_blank {
                    false  // No non-blank line found
                } else {
                    next_non_blank_indent >= content_indent
                }
            } else {
                false  // End of input
            };
            
            if !should_include_blank {
                break;
            }
            
            // Only count as "has blank lines" if NOT inside a fenced code block
            if !in_fenced_code {
                has_blank_lines = true;
            }
            last_was_blank = true;
            
            let (new_remaining, _) = take(skip_len)(remaining)?;
            content_end_offset = new_remaining.location_offset();
            remaining = new_remaining;
            continue;
        }
        
        // Non-blank line - check indentation
        let line_indent = count_indentation(current_line);
        
        // Check for fenced code block markers
        let trimmed_line = current_line.trim_start();
        if !in_fenced_code {
            if (trimmed_line.starts_with("```") || trimmed_line.starts_with("~~~")) && trimmed_line.len() >= 3 {
                let ch = trimmed_line.chars().next().unwrap();
                let fence_len = trimmed_line.chars().take_while(|&c| c == ch).count();
                if fence_len >= 3 {
                    log::debug!("list_item: entering fenced code block");
                    in_fenced_code = true;
                    fence_char = Some(ch);
                    fence_indent = line_indent;
                }
            }
        } else if let Some(fc) = fence_char {
            if trimmed_line.starts_with(fc) {
                let close_fence_len = trimmed_line.chars().take_while(|&c| c == fc).count();
                if close_fence_len >= 3 && line_indent <= fence_indent + content_indent {
                    log::debug!("list_item: exiting fenced code block");
                    in_fenced_code = false;
                    fence_char = None;
                }
            }
        }
        
        // Check if this starts a new list item
        if line_indent < 4 && detect_list_marker(remaining).is_ok() && line_indent <= marker_indent {
            // This is a sibling list item, stop here
            break;
        }
        
        // Check if line is indented enough to continue
        let min_indent = content_indent;
        
        if line_indent >= min_indent {
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
        
        // Line is not indented enough - check for lazy continuation
        if !last_was_blank {
            if line_indent < 4 && detect_list_marker(remaining).is_ok() {
                break;
            }
            
            if thematic_break(remaining).is_ok() {
                break;
            }
            
            if heading(remaining).is_ok() {
                break;
            }
            
            if fenced_code_block(remaining).is_ok() {
                break;
            }
            
            // Lazy continuation
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
        
        // Line doesn't continue the item
        break;
    }
    
    // Extract the content span
    let content_length = content_end_offset - content_start_offset;
    let content = content_start.slice(..content_length);
    let after_content = content_start.slice(content_length..);
    
    Ok((after_content, (marker, content, has_blank_lines, content_indent)))
}

/// Type alias for list item data: (marker, content_span, has_blank_lines_in_item, has_blank_before_next, content_indent)
pub type ListItemData<'a> = (ListMarker, Span<'a>, bool, bool, usize);

/// Parse a complete list (ordered or unordered)
/// Returns: Vec of (marker, content_span, has_blank_lines_in_item, has_blank_before_next, content_indent)
/// The 4th boolean indicates if there's a blank line BETWEEN this item and the next
pub fn list(input: Span) -> IResult<Span, Vec<ListItemData>> {
    // 1. Parse first item to determine list type
    let (mut remaining, (first_marker, first_content, first_has_blank, first_indent)) = list_item(input, None)?;
    
    let mut items = vec![(first_marker, first_content, first_has_blank, false, first_indent)];
    
    // Safety: prevent infinite loops
    const MAX_ITEMS: usize = 1000;
    let mut item_count = 1;
    let mut last_offset = 0;
    
    // 2. Continue parsing items with matching marker type
    loop {
        item_count += 1;
        if item_count > MAX_ITEMS {
            log::warn!("List exceeded MAX_ITEMS");
            break;
        }
        
        if remaining.fragment().is_empty() {
            break;
        }
        
        // Check for blank lines before next item
        let mut has_blank_before_next = false;
        let mut temp_remaining = remaining;
        
        loop {
            if temp_remaining.fragment().is_empty() {
                remaining = temp_remaining;
                break;
            }
            
            let first_line_end = temp_remaining.fragment().find('\n').unwrap_or(temp_remaining.fragment().len());
            let first_line = &temp_remaining.fragment()[..first_line_end];
            
            if first_line.trim().is_empty() {
                has_blank_before_next = true;
                
                let skip_len = if first_line_end < temp_remaining.fragment().len() {
                    first_line_end + 1
                } else {
                    first_line_end
                };
                
                let (new_remaining, _) = take(skip_len)(temp_remaining)?;
                temp_remaining = new_remaining;
            } else {
                remaining = temp_remaining;
                break;
            }
        }
        
        // Safety check: ensure progress
        let current_offset = remaining.location_offset();
        if current_offset == last_offset {
            log::error!("List parser stuck at offset {}", current_offset);
            break;
        }
        last_offset = current_offset;
        
        if remaining.fragment().is_empty() {
            break;
        }
        
        // Try to parse next item
        match list_item(remaining, Some(first_marker)) {
            Ok((new_remaining, (marker, content, has_blank, item_content_indent))) => {
                log::debug!("Parsed list item");
                
                if has_blank_before_next {
                    let last_idx = items.len() - 1;
                    items[last_idx].3 = true;
                }
                
                items.push((marker, content, has_blank, false, item_content_indent));
                remaining = new_remaining;
            }
            Err(_) => {
                log::debug!("Failed to parse next list item");
                break;
            }
        }
    }
    
    log::debug!("List parsing complete, {} items", items.len());
    Ok((remaining, items))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_detect_bullet_marker() {
        let input = Span::new("- Item");
        let result = detect_list_marker(input);
        assert!(result.is_ok());
        let (_, (marker, _)) = result.unwrap();
        assert!(matches!(marker, ListMarker::Bullet('-')));
    }

    #[test]
    fn smoke_test_detect_ordered_marker() {
        let input = Span::new("1. Item");
        let result = detect_list_marker(input);
        assert!(result.is_ok());
        let (_, (marker, _)) = result.unwrap();
        assert!(matches!(marker, ListMarker::Ordered { number: 1, delimiter: '.' }));
    }

    #[test]
    fn smoke_test_list_item_single_line() {
        let input = Span::new("- Item content");
        let result = list_item(input, None);
        assert!(result.is_ok());
        let (_, (marker, content, _, _)) = result.unwrap();
        assert!(matches!(marker, ListMarker::Bullet('-')));
        assert!(content.fragment().contains("Item content"));
    }

    #[test]
    fn smoke_test_list_item_multiline() {
        let input = Span::new("- Line 1\n  Line 2");
        let result = list_item(input, None);
        assert!(result.is_ok());
        let (_, (_, content, _, _)) = result.unwrap();
        assert!(content.fragment().contains("Line 1"));
        assert!(content.fragment().contains("Line 2"));
    }

    #[test]
    fn smoke_test_list_single_item() {
        let input = Span::new("- Item");
        let result = list(input);
        assert!(result.is_ok());
        let (_, items) = result.unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn smoke_test_list_multiple_items() {
        let input = Span::new("- Item 1\n- Item 2\n- Item 3");
        let result = list(input);
        assert!(result.is_ok());
        let (_, items) = result.unwrap();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn smoke_test_ordered_list() {
        let input = Span::new("1. First\n2. Second\n3. Third");
        let result = list(input);
        assert!(result.is_ok());
        let (_, items) = result.unwrap();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn smoke_test_list_with_blank_lines() {
        let input = Span::new("- Item 1\n\n- Item 2");
        let result = list(input);
        assert!(result.is_ok());
        let (_, items) = result.unwrap();
        assert_eq!(items.len(), 2);
        // Should be loose (has_blank_before_next should be true for first item)
        assert!(items[0].3); // has_blank_before_next
    }

    #[test]
    fn smoke_test_list_lazy_continuation() {
        let input = Span::new("- Item 1\nLazy line\n- Item 2");
        let result = list(input);
        assert!(result.is_ok());
        let (_, items) = result.unwrap();
        assert_eq!(items.len(), 2);
        assert!(items[0].1.fragment().contains("Lazy line"));
    }

    #[test]
    fn smoke_test_detect_marker_fails_without_space() {
        let input = Span::new("-Item");
        let result = detect_list_marker(input);
        assert!(result.is_err());
    }
}
