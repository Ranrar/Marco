# Supported Markdown Features

This document lists all Markdown features currently implemented in the project, with a description, test input, and expected output for each.

---

## Headings
**Description:** Lines starting with `#` (1-6) are parsed as headings.
**Test:**
```
# Heading 1
## Heading 2
```
**Expected Output:**
```
<h1>Heading 1</h1>
<h2>Heading 2</h2>
```

---

## Paragraphs
**Description:** Plain text lines are parsed as paragraphs.
**Test:**
```
This is a paragraph.
```
**Expected Output:**
```
<p>This is a paragraph.</p>
```

---

## Emphasis
**Description:** Text wrapped in `*` or `_` is parsed as emphasis.
**Test:**
```
*italic* _italic_
```
**Expected Output:**
```
<em>italic</em> <em>italic</em>
```

---

## Strong Emphasis
**Description:** Text wrapped in `**` or `__` is parsed as strong emphasis.
**Test:**
```
**bold** __bold__
```
**Expected Output:**
```
<strong>bold</strong> <strong>bold</strong>
```

---

## Links
**Description:** `[text](url)` creates a hyperlink.
**Test:**
```
[example](http://example.com)
```
**Expected Output:**
```
<a href="http://example.com">example</a>
```

---

## Images
**Description:** `![alt](src)` creates an image.
**Test:**
```
![alt text](image.png)
```
**Expected Output:**
```
<img src="image.png" alt="alt text" />
```

---

## Code Spans
**Description:** Text wrapped in backticks is parsed as inline code.
**Test:**
```
`code`
```
**Expected Output:**
```
<code>code</code>
```

---

## Math Spans
**Description:** Text wrapped in `$` is parsed as inline math.
**Test:**
```
$E=mc^2$
```
**Expected Output:**
```
<span class="math">E=mc^2</span>
```

---

## Autolinks
**Description:** URLs in angle brackets are parsed as links.
**Test:**
```
<http://example.com>
```
**Expected Output:**
```
<a href="http://example.com">http://example.com</a>
```

---

## Raw HTML
**Description:** Inline HTML is passed through.
**Test:**
```
<b>bold</b>
```
**Expected Output:**
```
<b>bold</b>
```

---

## Hard Breaks
**Description:** Two spaces at end of line create a hard break.
**Test:**
```
foo  
bar
```
**Expected Output:**
```
foo<br />
bar
```

---

## Soft Breaks
**Description:** Newlines without two spaces create soft breaks.
**Test:**
```
foo
bar
```
**Expected Output:**
```
foo bar
```

---

## Backslash Escapes
**Description:** Backslash escapes Markdown characters.
**Test:**
```
foo\*bar
```
**Expected Output:**
```
foo*bar
```

---

## Entity/Numeric References
**Description:** HTML entities and numeric references are parsed as text.
**Test:**
```
&copy; &#x27;
```
**Expected Output:**
```
&copy; &#x27;
```

---

## Attribute Blocks
**Description:** `{#id .class}` after a block or inline is parsed as attributes.
**Test:**
```
Paragraph {#id .class}
```
**Expected Output:**
```
<p id="id" class="class">Paragraph</p>
```

---

## Emoji
**Description:** `:smile:` is rendered as unicode emoji.
**Test:**
```
:smile:
```
**Expected Output:**
```
<span class="emoji" title=":smile:">😄</span>
```

---

## Mentions
**Description:** `@username` is rendered as a mention link.
**Test:**
```
@octocat
```
**Expected Output:**
```
<a class="mention" href="https://github.com/octocat">@octocat</a>
```

---

## Task List Meta
**Description:** `[ ]` or `[x]` in lists is parsed as task list meta.
**Test:**
```
- [x] Task done
- [ ] Task pending
```
**Expected Output:**
```
<ul>
  <li><input type="checkbox" checked /> Task done</li>
  <li><input type="checkbox" /> Task pending</li>
</ul>
```

---

## Table Captions
**Description:** Table captions are supported.
**Test:**
```
| Header |
|--------|
| Cell   |
: Table Caption
```
**Expected Output:**
```
<table>
  <caption>Table Caption</caption>
  ...
</table>
```

---

## Custom Tags
**Description:** Custom tags for extension points are supported.
**Test:**
```
:::custom data="foo"
Custom block
:::
```
**Expected Output:**
```
<div class="custom-tag" data-name="custom" data-data="foo">Custom block</div>
```

---

## Fuzzing and Regression Tests
**Description:** Randomized and regression tests ensure robustness for all delimiter logic and edge cases.

---

## Live Preview
**Description:** All features are available in the live editor preview, updating instantly as you type.
