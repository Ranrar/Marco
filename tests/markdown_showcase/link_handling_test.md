# Marco Link Handling Test Document

This document tests all link types defined in `marco_grammar.pest` and their expected behavior when clicked in the WebKit6 viewer.

---

## 1. Inline Links - External (Should Open in System Browser)

### HTTP Links
[Google Search](http://google.com) - Should open in default browser (ok)
[Google with Path](http://google.com/search?q=rust) - Should open in default browser (ok)

### HTTPS Links
[GitHub](https://github.com) - Should open in default browser (ok)
[Rust Lang](https://www.rust-lang.org) - Should open in default browser (ok)
[Docs with Port](https://example.com:8080/docs) - Should open in default browser (ok)

### WWW Links
[Example Site](www.example.com) - Should open in default browser (ok)
[WWW with Path](www.example.com/page) - Should open in default browser (ok)

### Links with Titles (External)
[GitHub with Title](https://github.com "Visit GitHub Homepage") - Hover to see title, click opens browser (ok)

### Autolinks (External)
<https://example.com> - Should open in default browser (ok)
<http://autolink.test.com> - Should open in default browser (ok)

---

## 2. Inline Links - Internal (Should Work in WebView)

### Anchor Links
[Jump to Section 3](#3-reference-style-links) - Should scroll to section 3
[Jump to Mixed Content](#5-mixed-content-testing) - Should scroll to section 5
[Back to Top](#marco-link-handling-test-document) - Should scroll to top

### File Protocol Links
[Local File](file:///tmp/test.txt) - Should attempt to load in WebView (may fail if file doesn't exist)
[Local HTML](file:///home/user/document.html) - Should load in WebView

### Relative Path Links
[Relative Document](./another_document.md) - Should navigate in WebView
[Subdirectory](../docs/readme.md) - Should navigate in WebView
[Current Directory](index.md) - Should navigate in WebView

### Absolute Path Links
[Absolute Path](/usr/share/doc/readme) - Should navigate in WebView

### Data URIs
[Data URI](data:text/html,<h1>Hello</h1>) - Should render in WebView

### About Pages
[About Blank](about:blank) - Should render in WebView

---

## 3. Reference-Style Links

Define references at the bottom and use them here:

### External References
[GitHub Ref Style][github-ref] - Should open in browser
[Rust Documentation][rust-docs] - Should open in browser

### Internal References
[Section 4 Reference][section4] - Should scroll in WebView

---

## 4. Links with Special Characters

### URLs with Query Parameters
[Search Query](https://example.com/search?q=marco+editor&lang=en) - Should open in browser

### URLs with Fragments
[External with Fragment](https://example.com/page#section) - Should open in browser
[Internal Fragment](#4-links-with-special-characters) - Should scroll in WebView

### URLs with Encoded Characters
[Encoded Spaces](https://example.com/path%20with%20spaces) - Should open in browser

### Email Autolinks
<user@example.com> - Should open default email client
<admin@localhost> - Should open default email client

---

## 5. Mixed Content Testing

Here's a paragraph with multiple link types:

Visit [GitHub](https://github.com) for source code, read the [local documentation](./docs/guide.md), or jump to [Section 1](#1-inline-links---external-should-open-in-system-browser). You can also check <https://rust-lang.org> or send email to <support@example.com>.

**Bold text with [external link](https://example.com) inside** should work.

*Italic text with [internal link](#top) inside* should work.

`Code with [link](https://example.com)` should not render as link (should be plain text in code).

---

## 6. Links in Different Block Types

### In Blockquotes
> This is a quote with [external link](https://example.com) and [internal link](#top).
> 
> > Nested quote with [another link](https://github.com).

### In Lists

#### Unordered Lists
- [External Link 1](https://example.com)
- [Internal Link](#section-6-links-in-different-block-types)
- [Another External](www.example.com)
  - Nested: [GitHub](https://github.com)
  - Nested: [Anchor](#top)

#### Ordered Lists
1. [First Link](https://first.com) - External
2. [Second Link](#section-2) - Internal
3. [Third Link](file:///tmp/test.txt) - File protocol

### In Tables

| Link Type | Example | Behavior |
|-----------|---------|----------|
| External | [GitHub](https://github.com) | Opens in browser |
| Internal | [Top](#top) | Scrolls in view |
| Autolink | <https://example.com> | Opens in browser |

---

## 7. Edge Cases and Special Scenarios

### Empty or Minimal URLs
[Hash Only](#) - Should do nothing or scroll to top
[Single Slash](/) - Root path, internal

### Case Sensitivity
[HTTPS UPPERCASE](HTTPS://EXAMPLE.COM) - Should open in browser (case-insensitive)
[WWW UPPERCASE](WWW.EXAMPLE.COM) - Should open in browser (case-insensitive)
[http lowercase](http://example.com) - Should open in browser

### Protocol Variants
[FTP Link](ftp://files.example.com) - Should attempt default handler
[SSH Link](ssh://server.example.com) - Should attempt default handler

### Very Long URLs
[Long URL](https://example.com/very/long/path/with/many/segments/and/parameters?param1=value1&param2=value2&param3=value3#section) - Should handle correctly

---

## 8. Links with Nested Formatting

### Link Text with Formatting
[**Bold Link Text**](https://example.com) - Bold text, external
[*Italic Link Text*](https://example.com) - Italic text, external
[`Code Link Text`](#top) - Code-styled text, internal

### Links within Emphasis
**Bold paragraph with [link inside](https://example.com) continuing** - Link should work
*Italic paragraph with [link inside](#top) continuing* - Link should work
~~Strikethrough with [link](https://example.com)~~ - Link should work

---

## 9. YouTube Embeds (Special Marco Feature)

[Marco Introduction Video](https://youtu.be/dQw4w9WgXcQ) - Should handle as YouTube embed
[YouTube Full URL](https://www.youtube.com/watch?v=dQw4w9WgXcQ) - Should handle as YouTube embed

---

## 10. Bookmark Links (Special Marco Feature)

[Important Code Section](src/main.rs) - Marco bookmark feature
[Function Definition](src/lib.rs:42) - Bookmark with line number

---

## Test Summary & Expected Behavior

### External Links (Open in System Browser):
- ✅ `http://` URLs
- ✅ `https://` URLs  
- ✅ `www.` URLs (with or without protocol)
- ✅ Autolinks: `<https://...>`
- ✅ Email autolinks: `<user@domain>`
- ✅ Case-insensitive protocol detection

### Internal Links (Navigate in WebView):
- ✅ Anchor links: `#section-id`
- ✅ File protocol: `file://...`
- ✅ Relative paths: `./file.md`, `../dir/file.md`
- ✅ Absolute paths: `/usr/...`
- ✅ Data URIs: `data:text/html,...`
- ✅ About pages: `about:blank`
- ✅ Reference-style links to internal targets

### HTML Attributes Generated:
- External links should have: `target="_blank" rel="noopener noreferrer"`
- Internal links should NOT have `target="_blank"`

### Security Considerations:
- ✅ `rel="noopener noreferrer"` prevents window.opener access
- ✅ External links cannot access Marco's internal state
- ✅ XSS prevention through proper HTML escaping

---

## Reference Definitions

[github-ref]: https://github.com "GitHub Homepage"
[rust-docs]: https://doc.rust-lang.org "Rust Documentation"
[section4]: #4-links-with-special-characters "Jump to Section 4"

---

## Testing Instructions

1. **Build Marco**: `cargo build --release`
2. **Open this file**: `./target/release/marco tests/markdown_showcase/link_handling_test.md`
3. **Test External Links**:
   - Click any `http://`, `https://`, or `www.` link
   - Verify it opens in your system's default web browser
   - Verify Marco remains open and focused
4. **Test Internal Links**:
   - Click any `#anchor` link
   - Verify the preview scrolls to the target section
   - Verify navigation stays within Marco
5. **Test Edge Cases**:
   - Try email autolinks (should open email client)
   - Try file:// links (may fail if file doesn't exist, but should stay in WebView)
   - Try reference-style links
6. **Inspect Generated HTML**:
   - Check browser dev tools or HTML output
   - External links should have `target="_blank" rel="noopener noreferrer"`
   - Internal links should NOT have these attributes
7. **Check Logs**:
   - Run with `RUST_LOG=marco::components::viewer::webkit6=debug`
   - Verify policy decision logs appear for link clicks
   - Check for "External link detected" or "Internal/local link" messages

---

## Implementation Notes

This test covers all link types from `marco_grammar.pest`:
- `inline_link` (bracket_link_with_title, bracket_link_without_title)
- `autolink` (autolink_url, autolink_email)
- `reference_link` and `reference_definition`
- `block_youtube` (special Marco YouTube embeds)
- Special URL patterns: `link_url`, `youtube_url`, `image_url`

The two-layer implementation ensures:
1. **HTML Layer**: Adds `target="_blank"` to external links during rendering
2. **WebView Layer**: Intercepts NEW_WINDOW_ACTION and opens in system browser
