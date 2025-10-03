# Basic Markdown Features Test

This document tests all the standard CommonMark features supported by Marco.

## Headers and Sections (ok)

# Heading Level 1 (ATX) (ok)
## Heading Level 2 (ATX) (ok)
### Heading Level 3 (ATX) (ok)
#### Heading Level 4 (ATX) (ok)
##### Heading Level 5 (ATX) (ok)
###### Heading Level 6 (ATX) (ok)

Alternative Setext H1 (ok)
=====================

Alternative Setext H2 (ok)
---------------------

### Headers with `code` (ok)
## Header with **bold text** (ok)
## Header with *italic text* (ok)
## Header with ***bold italic***  (ok) 
## Header with ==highlight== (ok)
## Header with :smile: (ok)
## Header with ~~deleted~~ (ok)

## Text Formatting

### Emphasis

This is *italic text* with asterisks. (ok)
This is _italic text_ with underscores. (ok)

This is **bold text** with asterisks. (ok)
This is __bold text__ with underscores. (ok)

This is ***bold and italic*** with triple asterisks. (ok)
This is ___bold and italic___ with triple underscores. (ok)

Mixed formatting: **_bold italic_** and __*italic bold*__.  (ok)

### Strikethrough

~~This text is deleted/strikethrough with tildes~~  (ok)
--This text is also deleted with dashes--  (ok)

### Marco Extensions: Highlight, Superscript, Subscript

==This text is highlighted== (ok)

This is text with ^superscript^ formatting. (ok)
This is text with ~subscript~ formatting. (ok)
This is text with ˅subscript˅ formatting (alternative arrow style). (ok)

### Inline Code

Here is `inline code` in backticks.  (ok)
Here is `code with **formatting** inside` (formatting should be literal).
Here is `code with symbols: *_~=^$`.

## Line Breaks (ok)

This line has two spaces at the end for hard break.  
This should be on a new line.

This line has a backslash at the end for hard break.\
This should also be on a new line.

This line has a normal newline
but should continue on the same paragraph (soft break).

## Horizontal Rules (ok)

Three dashes: (ok)

---

Three asterisks: (ok)

***

Three underscores: (ok)

___

Spaced dashes: (ok)

- - -

Spaced asterisks: (ok)

* * *

Spaced underscores: (ok)

_ _ _

## Lists

### Unordered Lists  (ok)

- First item
- Second item
  - Nested item 1
  - Nested item 2
    - Deep nested item
- Third item

Alternative markers:  (ok)

* Item with asterisk
+ Item with plus
- Item with dash

Mixed markers (should still work):  (ok)

* First
+ Second  
- Third

### Ordered Lists  (ok)

1. First item
2. Second item
   1. Nested item 1
   2. Nested item 2
      1. Deep nested item
3. Third item

Different numbering:  (ok)
1. First
1. Second (same number)
5. Fifth (skip numbers)
2. Second again (wrong order)

Negative numbers:  (ok)
-1. Negative item
0. Zero item

### Lists with Formatting  (ok)

- Item with **bold text**
- Item with *italic text*
- Item with `inline code`
- Item with [link](https://example.com)
- Item with ~~strikethrough~~

1. **Bold** numbered item
2. *Italic* numbered item
3. `Code` in numbered item

## Links

### Inline Links

[Basic link](https://example.com) (ok)
[Link with title](https://example.com "Example Website") (ok)
[Link to local file](./test.md)
[Link to absolute path](/home/user/file.txt)
[Windows path](C:\Windows\System32\file.txt)
[Windows forward slash](C:/Windows/System32/file.txt)
[Relative path with spaces](./My Documents/file with spaces.txt)

### Autolinks

<https://example.com> (ok)
<http://example.com> (ok)
<mailto:user@example.com> (ok)
<user@domain.com> (ok)

### Reference Links

[Reference link][ref1]
[Another reference][ref2]
[Case insensitive][REF1]

[ref1]: https://example.com
[ref2]: https://example.com "Reference with title"

## Images

### Inline Images (ok)

![PNG Test Image](./files/img/test.png)
![JPG Test Image](./files/img/test.jpg "JPG Image Title")
![JPEG Test Image](./files/img/test.jpeg)
![Animated GIF](./files/img/animated.gif)
![WebP Test Image](./files/img/test.webp)
![SVG Test Image](./files/img/test.svg)

### Reference Images

![Reference image][img1]
![Another reference image][img2]

[img1]: ./files/img/test.png
[img2]: ./files/img/test.jpg "Local JPG image with title"

## Code Blocks

### Fenced Code Blocks (ok)

```
Plain code block without language
Multiple lines
With various content
```

```javascript
// JavaScript code
function hello() {
    console.log("Hello, World!");
    return true;
}
```

```python
# Python code
def hello():
    print("Hello, World!")
    return True
```

```rust
// Rust code
fn main() {
    println!("Hello, World!");
}
```

### Nested Code Blocks

````markdown
# Markdown example with nested code

Here's some JavaScript:

```javascript
console.log("Nested code block");
```

End of markdown example.
````

`````html
<!DOCTYPE html>
<html>
<body>
    <pre><code>
````markdown
# Even deeper nesting
```
Code inside markdown inside HTML
```
````
    </code></pre>
</body>
</html>
`````

### Indented Code Blocks

    This is an indented code block
    using four spaces
    
    function indentedCode() {
        return "Four spaces indent";
    }

	This is indented with tabs
	multiple lines
	with tab characters

## Mathematical Content

### Inline Math

Here is inline math: $x^2 + y^2 = z^2$

Complex inline: $\sum_{i=1}^{n} x_i = \int_0^1 f(x) dx$

Math with symbols: $\alpha + \beta = \gamma$

### Block Math

$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$

$$
\begin{matrix}
a & b \\
c & d
\end{matrix}
$$

## Blockquotes

> Simple blockquote
> Multiple lines in quote

> Blockquote with **formatting**
> And `inline code`

> Nested blockquote level 1
>> Nested blockquote level 2
>>> Nested blockquote level 3

> Blockquote with lists:
> 1. First item
> 2. Second item
>    - Nested bullet

## Special Characters and Escaping

\*Not italic\* because of escapes (ok)
\**Not bold\** because of escapes (error)
\# Not a heading (ok)
\- Not a list item (ok)
\> Not a blockquote (ok)

Regular symbols that don't need escaping: . , ; : ! ? (think this has to be in one line)

Unicode characters: café, naïve, Москва, 北京, العربية (ok)

Emoji characters: 🌟 ⭐ 💫 ✨ (ok)

:emoji_style: :another: :custom_emoji:

## Mixed Content

Here's a paragraph with **bold**, *italic*, `code`, [links](https://example.com), and math $x = y$.

> A blockquote with **bold text**, `inline code`, and a [link](https://example.com)
> 
> Multiple paragraphs in blockquote with math: $e = mc^2$

- List item with **bold** and `code` (ok)
- List item with [link](https://example.com) and math $\pi \approx 3.14$
- List item with ~~strikethrough~~ and ==highlight== (ok)

## Edge Cases - Basic

### Empty Elements (ok)

**

*

``

[]()

[](https://example.com) (has to write excatly the line `[](https://example.com)`)

### Unmatched Formatting (dont know if this is correct)

*This italic is not closed

**This bold is not closed

`This code is not closed

~~This strikethrough is not closed

==This highlight is not closed

### Adjacent Formatting

**bold***italic*
*italic***bold**
`code`**bold**
**bold**`code`

### Special Text

Text with * asterisk but not formatting
Text with _ underscore but not formatting
Text with `backtick but not code
Text with ==equals but not highlight

## Whitespace Handling

Text    with    multiple    spaces

Text	with	tabs	between	words (ok)

Line with trailing spaces    (ok)
Next line after hard break (ok)

Line with trailing tabs		(ok)
Next line after tab hard break (ok)

Text at start of line (ok)
  Text with leading spaces
    Text with more leading spaces (ok)

## URLs and Email

Automatic URL detection:
https://example.com should be plain text (ok)
http://example.com/path?query=value&other=test
www.example.com should be plain text (ok)

Email addresses:
user@domain.com should be plain text (ok)
test.email+tag@sub.example.org should be plain text (ok)

FTP and other protocols:
ftp://files.example.com
file:///local/path

## End of Basic Markdown Test

This concludes the basic CommonMark features test. All standard markdown elements should be properly parsed and rendered.