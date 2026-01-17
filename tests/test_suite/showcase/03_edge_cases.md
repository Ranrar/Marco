# Edge Cases and Parsing Challenges

This document tests complex edge cases, boundary conditions, and potential parsing issues in Marco.

## Whitespace Edge Cases

### Leading and Trailing Whitespace

   Text with leading spaces should still be paragraph
Text with trailing spaces   
should handle hard breaks properly.

		Text with leading tabs
Text with trailing tabs		
should also handle breaks.

### Mixed Whitespace

Text    with    multiple    spaces    between    words
Text	with	tabs	between	words  
Text   with	mixed   spaces	and   tabs

### Whitespace in Formatting

Before `inline code with spaces` after
Before `	inline code with tab	` after
Before `inline code   multiple   spaces` after

Before **bold with spaces** after
Before **	bold with tabs	** after
Before ** bold with space after asterisk** (should not work)
Before **bold with space before asterisk ** (should not work)

### Whitespace in Lists

-  Item with space after dash
-	Item with tab after dash
  - Item with leading spaces
	- Item with leading tab
    - Item with 4 leading spaces

1.  Ordered item with space
1.	Ordered item with tab
  1. Ordered with leading spaces
	1. Ordered with leading tab

## Nesting Edge Cases

### Deep List Nesting

- Level 1
  - Level 2
    - Level 3
      - Level 4
        - Level 5
          - Level 6
            - Level 7
              - Level 8 (very deep)

1. Ordered Level 1
   1. Ordered Level 2
      1. Ordered Level 3
         1. Ordered Level 4
            1. Ordered Level 5

### Mixed List Nesting

- Unordered Level 1
  1. Ordered Level 2
     - Unordered Level 3
       1. Ordered Level 4
          - Unordered Level 5

### Blockquote Nesting

> Level 1 quote
>> Level 2 quote
>>> Level 3 quote
>>>> Level 4 quote
>>>>> Level 5 quote
>>>>>> Level 6 quote

> Complex nesting with formatting
>> **Bold in nested quote**
>>> *Italic in deeper quote*
>>>> `Code in deepest quote`

### Nested Formatting

***Bold and italic together***
**Bold with *nested italic* inside**
*Italic with **nested bold** inside*
~~Strikethrough with **bold inside**~~
==Highlight with **bold inside**==
^Superscript with **bold inside**^
~Subscript with **bold inside**~

### Code in Nested Contexts

> Blockquote with `inline code`
> And block code:
> ```
> code in blockquote
> ```

- List item with `inline code`
- List item with block code:
  ```
  indented code in list
  ```

**Bold with `code inside`**
*Italic with `code inside`*

### Admonition Nesting

:::note
Outer note
:::tip
Inner tip (should this work?)
:::
Back to outer note
:::

:::warning
Outer warning

> Blockquote inside admonition
> Multiple lines

- List inside admonition
- Another item

```
Code block inside admonition
```

:::note
Nested admonition inside warning
:::

End of warning
:::

### Tab Blocks (Marco_Extended)

#### Indentation rules (0–3 spaces allowed)

 :::tab
 @tab One
 This opener and header are indented by 1 space.
 :::

   :::tab
   @tab Two
   This opener and header are indented by 3 spaces.
   :::

    :::tab
    @tab FourSpaces
    This should NOT be recognized as a tab block (4 leading spaces).
    :::

#### Markers inside fenced code (must not terminate tabs)

:::tab
@tab Fence
```txt
@tab NotAHeader
:::
```

This content should still belong to the "Fence" tab.

@tab AfterFence
After fence content.
:::

#### Nested tab blocks are forbidden (inner :::tab should be treated as literal)

:::tab
@tab Outer

:::tab
@tab Inner
Inner content
:::

Outer continues.
:::

#### Invalid header (empty title) should not parse as a tab block

:::tab
@tab
This should render as literal text / normal markdown, not a tab UI.
:::

#### Missing closing marker (should not crash; should not become tabs)

:::tab
@tab Unclosed
This is unclosed.

## Boundary Cases

### Empty Elements

**  ** (bold with only spaces)
*  * (italic with only spaces)
`  ` (code with only spaces)
==  == (highlight with only spaces)
~~  ~~ (strikethrough with only spaces)

[] (empty link text)
![] (empty image alt)
[](empty-url)
![](empty-image-url)

### Single Character Elements

**a** (single char bold)
*b* (single char italic)
`c` (single char code)
==d== (single char highlight)
~~e~~ (single char strikethrough)
^f^ (single char superscript)
~g~ (single char subscript)

### Adjacent Formatting

**bold***italic* (adjacent different formatting)
*italic***bold** (reverse adjacent)
`code`**bold** (code then bold)
**bold**`code` (bold then code)
==highlight==**bold** (highlight then bold)
~~strike~~*italic* (strikethrough then italic)

### Overlapping Markers

This is *italic **and bold* together** (malformed)
This is **bold *and italic** together* (malformed)
This is `code **with bold` inside** (malformed)

### Unmatched Markers

*This italic never closes
**This bold never closes
`This code never closes
==This highlight never closes
~~This strikethrough never closes
^This superscript never closes
~This subscript never closes

This bold starts midway **but never ends
This italic starts *midway but never ends

### Escaped Markers

\*Not italic\* should be literal
\**Not bold\** should be literal
\`Not code\` should be literal
\==Not highlight\== should be literal
\~~Not strikethrough\~~ should be literal
\^Not superscript\^ should be literal
\~Not subscript\~ should be literal

### False Positives

This * is not italic because no closing
This ** is not bold because no closing  
This ` is not code because no closing
This == is not highlight because no closing
This ~~ is not strikethrough because no closing

URL with * in it: https://example.com/path*with*asterisks
Email with _ in it: user_name@domain.com
File path with __ in it: /home/user__name/file

## Link Edge Cases

### Complex URLs

[Link](https://example.com/path?query=value&other=test#anchor)
[Link with spaces](https://example.com/path with spaces.html)
[Link with unicode](https://example.com/ünïcödé/path)
[Local path](./path with spaces/file name.txt)
[Windows path](C:\Program Files\Application\file.exe)
[Windows forward](C:/Program Files/Application/file.exe)

### Malformed Links

[Link with no URL]()
[](URL with no text)
[Link with spaces in URL]( https://example.com )
[Link with newline
in text](https://example.com)

### Nested Link Content

[Link with **bold** text](https://example.com)
[Link with *italic* text](https://example.com)
[Link with `code` text](https://example.com)
[Link with [nested brackets]](https://example.com)
[Link with \[escaped brackets\]](https://example.com)

### Link Titles

[Link]( https://example.com "Title with spaces" )
[Link](https://example.com "Title with \"quotes\"")
[Link](https://example.com 'Title with single quotes')
[Link](https://example.com "Title with 'mixed' quotes")

### Image Edge Cases

![Image with **bold** alt text](./image.png)
![Image with `code` alt text](./image.jpg)
![](./image-with-no-alt.png)
![Image with no URL]()

## Code Block Edge Cases

### Fenced Code Variations

``` 
Code block with space after backticks
```

```   
Code block with multiple spaces after backticks
```

```javascript
// Normal code block
function test() {}
```

```
Code block with backticks inside:
```
This should break the code block
```

### Nested Fencing

````markdown
This is markdown with nested code:

```javascript
console.log("Nested code");
```

End of nested content.
````

`````html
HTML example with even deeper nesting:

````markdown
# Markdown inside HTML example

```javascript
// JavaScript inside markdown inside HTML
function deep() {
    return "very nested";
}
```

````

`````

### Indented Code Edge Cases

    Code with exactly 4 spaces
     Code with 5 spaces
      Code with 6 spaces

	Code with exactly 1 tab
		Code with 2 tabs

    Mixed indentation:
    	This line has spaces then tab
	    This line has tab then spaces

### Code with Special Content

```
Code block with various special chars:
*asterisks* **double asterisks**
_underscores_ __double underscores__
`backticks` inside code
#hashtags #multiple
[brackets] and (parentheses)
<angle brackets>
```

## Table Edge Cases

### Malformed Tables

| Header 1 | Header 2
Missing closing pipe

| Header 1 | Header 2 |
|----------|----------|
| Cell 1 | Cell 2
Missing pipe in data row

| Header 1 Header 2 |
|-------------------|
No separator between headers

### Tables with Special Content

| **Bold** | *Italic* | `Code` |
|----------|----------|--------|
| ==Highlight== | ~~Strike~~ | ^Super^ |
| [Link](url) | ![Image](img) | $Math$ |

### Empty Cells

| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Content  |          | Content  |
|          | Content  |          |
|          |          |          |

### Alignment Edge Cases

|Left|Center|Right|
|:--|:-:|--:|
|L|C|R|

| Loose | Spacing | Table |
| :--- | :---: | ---: |
| L | C | R |

## Math Edge Cases

### Inline Math Variations

$simple$
$ spaced $
$complex^2 + \sum_{i=1}^n x_i$
$\text{with text} \alpha + \beta$

### Escaped Dollar Signs

This \$5 should not be math
This \$variable should not be math
This costs \$100 total

### Math with Special Characters

$x = y \$dollar$
$\text{Cost: \$50}$

### Block Math Edge Cases

$$
Simple block math
$$

$$ 
Block with spaces
$$

$$
\begin{matrix}
a & b \\
c & d
\end{matrix}
$$

## List Edge Cases

### Mixed Markers

- First item
* Second item (different marker)
+ Third item (another marker)
- Fourth item (back to original)

### List Interruption

- List item 1
- List item 2

Not a list item (paragraph break)

- New list starts
- Another item

### Tight vs Loose Lists

- Tight item 1
- Tight item 2
- Tight item 3

- Loose item 1

- Loose item 2

- Loose item 3

### Lists with Paragraphs

- Item with multiple paragraphs

  This is the second paragraph of the first item.

- Second item

  Another second paragraph.
  
  Even a third paragraph.

## Task List Edge Cases

### Mixed Task and Regular Items

- [ ] Task item 1
- Regular item
- [x] Task item 2
- Another regular item

### Task Metadata Variations

- [ ] Task (user: 2024-01-15)
- [x] Task (alice: completed task)
- [ ] Task (team_lead: high priority task)
- [x] Task (developer: implemented feature X on 2024-01-15)

### Malformed Tasks

- [ ] Task with missing closing paren (user: no closing
- [x] Task with extra parens (user: extra) )
- [ ] Task (user without colon)
- [x] Task (:missing user)

## HTML Edge Cases

### Block vs Inline HTML

<div>Block HTML element</div>
<span>Inline HTML element</span>
<p>Another block element</p>

### Self-Closing Tags

<br/>
<hr />
<img src="image.png" alt="alt text" />
<input type="text" value="test" />

### HTML with Markdown

<div class="container">

**Bold text inside HTML block**

- List inside HTML block
- Another item

</div>

### Malformed HTML

<div>Unclosed div
<p>Nested unclosed paragraph
<span>Multiple unclosed tags

## Special Characters and Unicode

### Unicode Text

Café naïve résumé
Москва Санкт-Петербург
北京 上海 広州
العربية الفارسية
हिन्दी ગુજરાતી

### Mathematical Symbols

α β γ δ ε ζ η θ ι κ λ μ ν ξ ο π ρ σ τ υ φ χ ψ ω
∀ ∃ ∈ ∉ ⊂ ⊃ ⊆ ⊇ ∪ ∩
≠ ≤ ≥ ≈ ≡ ± × ÷ √ ∞ ∑ ∏ ∫

### Emoji and Symbols

🌟 ⭐ 💫 ✨ 🔥 💡 ⚡ 🚀 
😀 😃 😄 😁 😆 😅 😂 🤣
❤️ 💙 💚 💛 🧡 💜 🖤 🤍

### Control Characters and Whitespace

Text with zero-width space: ​ (between words)
Text with non-breaking space:   (between words)
Text with various spaces:       (em space, en space, etc.)

## File Path Edge Cases

### Various Path Types

[Local relative](./file.txt)
[Local absolute](/home/user/file.txt)
[Windows C:](C:\Windows\System32\file.exe)
[Windows forward](C:/Windows/System32/file.exe)
[UNC path](\\server\share\file.txt)
[Network drive](Z:\network\file.txt)

### Paths with Special Characters

[Path with spaces](./My Documents/file name.txt)
[Path with unicode](./café/naïve/résumé.txt)
[Path with symbols](./symbols-_$@#/file.txt)
[Path with dots](../../../deep/path/file.txt)

## Footnote Edge Cases

### Footnote Labels

Normal footnote[^1]
Number footnote[^123]
Unicode footnote[^参考]
Mixed footnote[^ref_123]
Long footnote[^very_long_footnote_name_here]

[^1]: Simple footnote
[^123]: Number-only footnote
[^参考]: Unicode footnote definition
[^ref_123]: Mixed character footnote
[^very_long_footnote_name_here]: Very long footnote name with definition

### Inline Footnotes

Text with inline footnote^[Simple inline footnote].
Text with complex inline footnote^[Inline footnote with **bold**, *italic*, and `code`].
Text with long inline footnote^[This is a very long inline footnote that contains multiple sentences and goes on for quite a while to test how the parser handles longer content].

## Marco Extension Edge Cases

### Admonition Edge Cases

:::note
Admonition with nested formatting **bold** *italic* `code`
:::

:::tip[Title with **bold** formatting]
Custom title with formatting
:::

:::warning
Unclosed admonition that goes to end of document...

### Run Block Edge Cases

```run@bash
# Script with special characters
echo "Special chars: $@#%^&*()"
echo 'Single quotes with $variables'
echo "Double quotes with \$escaped"
```

```run@python
# Python with various quotes and escapes
print("Double quotes")
print('Single quotes')
print("""Triple quotes""")
print('''Triple single quotes''')
print("Escaped \"quotes\" inside")
```

### Tab Block Edge Cases

:::tab
Default content before tabs
No @tab defined yet

@tab First
First tab content

@tab Second
Second tab content

More default content after tabs
:::

:::tab Titled Tabs
@tab
Empty tab name

@tab 
Tab with just space name

@tab	
Tab with tab character name

@tab Very Long Tab Name With Spaces
Long tab name content
:::

### User Mention Edge Cases

@user[platform]
@user[](empty platform)
@user[platform](display name)
@user[platform](name with spaces and symbols: @#$%)
@user[very-long-platform-name-here]
@user[platform]()

## End of Edge Cases

This document covers numerous edge cases and boundary conditions that can challenge markdown parsers. The Marco parser should handle these gracefully, either by parsing them correctly or failing in predictable ways.