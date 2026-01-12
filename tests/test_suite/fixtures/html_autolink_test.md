# HTML and Autolink Test Document

This document tests that HTML tags are properly distinguished from autolinks.

## HTML Tags (should be highlighted as InlineHtml)

Simple img tag: <img src="test.png" alt="test image" />

Span with text: <span class="highlight">Important text</span>

Div tag: <div id="container">Content</div>

Self-closing tags: <br /> and <hr />

Closing tag only: Text before </div> text after

## Valid Autolinks (should be highlighted as Link)

Basic HTTP URL: <http://example.com>

HTTPS URL: <https://www.example.com/path?query=value#fragment>

FTP URL: <ftp://files.example.com>

Custom scheme: <git+ssh://github.com/user/repo>

Email address: <user@example.com>

Complex email: <john.doe+tag@example.co.uk>

## Edge Cases

URL in text: Visit <https://example.com> for more info.

HTML and URL together: <span>Link: <https://example.com></span> end

Mixed content: Text with <img src="icon.png" /> and link <mailto:test@example.com>

## Invalid Cases (should NOT match as autolinks)

No colon: <notaurl>

Single char scheme: <x:something>

Scheme starting with digit: <1http://example.com>

Space in brackets: <not an email>

Plain HTML: <table><tr><td>Cell</td></tr></table>

## CommonMark Compliance

According to CommonMark spec section 6.5:
- Autolinks MUST have scheme: 2-32 ASCII letters/digits/+/./-, starting with letter, followed by ":"
- Emails MUST have "@" with text before and after

Our implementation now correctly validates these requirements.
