# Basic Markdown Features Test

This document tests all the standard CommonMark features supported by Marco.

## Headers and Sections

# Heading Level 1 (ATX)
## Heading Level 2 (ATX)
### Heading Level 3 (ATX)
#### Heading Level 4 (ATX)
##### Heading Level 5 (ATX)
###### Heading Level 6 (ATX)

Alternative Setext H1
=====================

Alternative Setext H2
---------------------

### Headers with **formatting** and `code`
### Headers with [links](https://example.com)

## Text Formatting

### Emphasis

This is *italic text* with asterisks.
This is _italic text_ with underscores.

This is **bold text** with asterisks.
This is __bold text__ with underscores.

This is ***bold and italic*** with triple asterisks.
This is ___bold and italic___ with triple underscores.

Mixed formatting: **_bold italic_** and __*italic bold*__.

### Strikethrough

~~This text is deleted/strikethrough with tildes~~
--This text is also deleted with dashes--

### Marco Extensions: Highlight, Superscript, Subscript

==This text is highlighted==

This is text with ^superscript^ formatting.
This is text with ~subscript~ formatting.
This is text with ˅subscript˅ formatting (alternative arrow style).

### Inline Code

Here is `inline code` in backticks.
Here is `code with **formatting** inside` (formatting should be literal).
Here is `code with symbols: *_~=^$`.

## Line Breaks

This line has two spaces at the end for hard break.  
This should be on a new line.

This line has a backslash at the end for hard break.\
This should also be on a new line.

This line has a normal newline
but should continue on the same paragraph (soft break).

## Horizontal Rules

Three dashes:
---

Three asterisks:
***

Three underscores:
___

Spaced dashes:
- - -

Spaced asterisks:
* * *

Spaced underscores:
_ _ _

## Lists

### Unordered Lists

- First item
- Second item
  - Nested item 1
  - Nested item 2
    - Deep nested item
- Third item

Alternative markers:
* Item with asterisk
+ Item with plus
- Item with dash

Mixed markers (should still work):
* First
+ Second  
- Third

### Ordered Lists

1. First item
2. Second item
   1. Nested item 1
   2. Nested item 2
      1. Deep nested item
3. Third item

Different numbering:
1. First
1. Second (same number)
5. Fifth (skip numbers)
2. Second again (wrong order)

Negative numbers:
-1. Negative item
0. Zero item

### Lists with Formatting

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

[Basic link](https://example.com)
[Link with title](https://example.com "Example Website")
[Link to local file](./test.md)
[Link to absolute path](/home/user/file.txt)
[Windows path](C:\Windows\System32\file.txt)
[Windows forward slash](C:/Windows/System32/file.txt)
[Relative path with spaces](./My Documents/file with spaces.txt)

### Autolinks

<https://example.com>
<http://example.com>
<mailto:user@example.com>
<user@domain.com>

### Reference Links

[Reference link][ref1]
[Another reference][ref2]
[Case insensitive][REF1]

[ref1]: https://example.com
[ref2]: https://example.com "Reference with title"

## Images

### Inline Images

![Alt text](https://example.com/image.png)
![Alt text with title](https://example.com/image.jpg "Image Title")
![Local image](./images/local.png)
![JPG image](./test.jpg)
![JPEG image](./test.jpeg)
![GIF image](./animated.gif)
![WebP image](./modern.webp)
![SVG image](./vector.svg)

### Reference Images

![Reference image][img1]
![Another reference image][img2]

[img1]: https://example.com/image1.png
[img2]: ./local/image2.jpg "Local image with title"

## Code Blocks

### Fenced Code Blocks

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

\*Not italic\* because of escapes
\**Not bold\** because of escapes
\# Not a heading
\- Not a list item
\> Not a blockquote

Regular symbols that don't need escaping: . , ; : ! ?

Unicode characters: café, naïve, Москва, 北京, العربية

Emoji characters: 🌟 ⭐ 💫 ✨

:emoji_style: :another: :custom_emoji:

## Mixed Content

Here's a paragraph with **bold**, *italic*, `code`, [links](https://example.com), and math $x = y$.

> A blockquote with **bold text**, `inline code`, and a [link](https://example.com)
> 
> Multiple paragraphs in blockquote with math: $e = mc^2$

- List item with **bold** and `code`
- List item with [link](https://example.com) and math $\pi \approx 3.14$
- List item with ~~strikethrough~~ and ==highlight==

## Edge Cases - Basic

### Empty Elements

**

*

``

[]()

[](https://example.com)

### Unmatched Formatting

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
Text with ==equals but not highlight==

## Whitespace Handling

Text    with    multiple    spaces

Text	with	tabs	between	words

Line with trailing spaces    
Next line after hard break

Line with trailing tabs		
Next line after tab hard break

Text at start of line
  Text with leading spaces
    Text with more leading spaces

## URLs and Email

Automatic URL detection:
https://example.com should be plain text (not in angle brackets)
http://example.com/path?query=value&other=test
www.example.com should be plain text

Email addresses:
user@domain.com should be plain text
test.email+tag@sub.example.org should be plain text

FTP and other protocols:
ftp://files.example.com
file:///local/path

## End of Basic Markdown Test

This concludes the basic CommonMark features test. All standard markdown elements should be properly parsed and rendered.