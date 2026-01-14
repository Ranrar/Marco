# GFM autolink literals showcase

Planned feature: recognize bare URLs/emails (without `<...>`).

## Canonical examples

https://example.com
http://example.com/path
www.example.com
user@example.com

## Edge cases

### Trailing punctuation (should not be part of the link)

https://example.com.
https://example.com,
https://example.com)
https://example.com]

### Parentheses and query strings

(https://example.com)
https://example.com/path_(with_parens)
https://example.com/path?query=value&other=test#anchor

### Emails with plus tags and subdomains

test.email+tag@sub.example.org

### Not links

<div>html</div>
`www.example.com`
www.example.com inside code: `www.example.com`
