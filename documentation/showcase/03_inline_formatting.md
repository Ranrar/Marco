# Inline Formatting Extensions

Marco supports all CommonMark inline formatting plus several extensions for richer typography.

---

## Standard Emphasis

*Italic* text using asterisks or _underscores_.

**Bold** text using double asterisks or __double underscores__.

***Bold and italic*** combined.

Nested: **bold with *italic inside* still bold** or *italic with **bold inside** still italic*.

---

## GFM Strikethrough

~~Strike through~~ text with double tildes.

Combined: ~~**important note**~~ (outdated).

---

## Dash Strikethrough (Marco Extension)

--Strike through-- text with double dashes as an alternative.

Both syntaxes produce identical output:

- ~~GFM style~~ — double tildes
- --Marco style-- — double dashes

---

## Highlight / Mark (Marco Extension)

==Highlighted text== renders as `<mark>` — useful for calling out key terms.

Use it in notes: The ==primary key== must be unique.

Multiple highlights in a sentence: The formula uses ==x² + y²== to calculate the ==hypotenuse==.

---

## Superscript (Marco Extension)

Use carets for superscript: E = mc^2^

Chemical formulas: H^+^ ions in solution.

Ordinals: 1^st^, 2^nd^, 3^rd^, 4^th^.

Mathematical notation: x^n^ + y^n^ = z^n^

---

## Subscript (Marco Extension)

Use single tildes for subscript: H~2~O is water.

Chemical formulas: CO~2~ and NO~x~.

Mathematical notation: a~1~, a~2~, … a~n~.

Array indexing: array~i,j~ refers to element at row i, column j.

### Arrow-Style Subscript (Alternative Syntax)

The `˅` character (U+02C5, modifier letter down arrowhead) is an alternative delimiter:

H˅2˅O — water.

CO˅2˅ — carbon dioxide.

Both `~text~` and `˅text˅` render identically.

---

## Code Spans

Inline `code` uses backticks. Common uses:

- Variable names: `count`, `user_id`, `is_valid`
- Commands: `cargo build`, `git commit -m "msg"`
- Paths: `/home/user/.config/marco/settings.ron`
- Type names: `Option<String>`, `Result<T, E>`

Double backticks allow a backtick inside: `` this `code` has backticks ``

---

## Combining Extensions

Extensions compose cleanly with each other and with standard formatting:

| Syntax | Example | Notes |
|--------|---------|-------|
| `==text==` | ==highlighted== | Mark/highlight |
| `^text^` | x^2^ | Superscript |
| `~text~` | H~2~O | Subscript |
| `--text--` | --outdated-- | Dash strikethrough |
| `~~text~~` | ~~outdated~~ | GFM strikethrough |

Bold highlight: **==important term==**

Italic superscript: *x^n^*

Bold subscript: **H~2~O**

Strikethrough mark: ~~==this was highlighted but is now struck==~~

---

## Backslash Escapes

Escape any punctuation character to display it literally:

\* not italic \*

\*\* not bold \*\*

\= \= not highlight \= \=

\^ not superscript \^

\~ not subscript \~

\- \- not strikethrough \- \-

---

## Strong Emphasis (Triple)

***Bold and italic together*** using triple asterisks.

***Three levels***: ***bold italic with ~~strikethrough~~ inside***.
