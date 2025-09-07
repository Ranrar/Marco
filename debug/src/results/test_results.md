# Marco Grammar Test Results

Generated automatically from test_cases.toml

## commonmark_images

✅ **cm_example_571**: `text`
   Input: `!\[foo\](/url "title")
`
   Parse Tree:
   ```
  └── text: "![foo](/url "title")
"
   ```

✅ **cm_example_572**: `text`
   Input: `!\[foo \*bar\*\]

\[foo \*bar\*\]: train.jpg "train & tracks"
`
   Parse Tree:
   ```
  └── text: "![foo *bar*]

[foo *bar*]: train.jpg "train & tracks"
"
   ```

✅ **cm_example_573**: `text`
   Input: `!\[foo !\[bar\](/url)\](/url2)
`
   Parse Tree:
   ```
  └── text: "![foo ![bar](/url)](/url2)
"
   ```

✅ **cm_example_574**: `text`
   Input: `!\[foo \[bar\](/url)\](/url2)
`
   Parse Tree:
   ```
  └── text: "![foo [bar](/url)](/url2)
"
   ```

✅ **cm_example_575**: `text`
   Input: `!\[foo \*bar\*\]\[\]

\[foo \*bar\*\]: train.jpg "train & tracks"
`
   Parse Tree:
   ```
  └── text: "![foo *bar*][]

[foo *bar*]: train.jpg "train & tracks"
"
   ```

✅ **cm_example_576**: `text`
   Input: `!\[foo \*bar\*\]\[foobar\]

\[FOOBAR\]: train.jpg "train & tracks"
`
   Parse Tree:
   ```
  └── text: "![foo *bar*][foobar]

[FOOBAR]: train.jpg "train & tracks"
"
   ```

✅ **cm_example_577**: `text`
   Input: `!\[foo\](train.jpg)
`
   Parse Tree:
   ```
  └── text: "![foo](train.jpg)
"
   ```

✅ **cm_example_578**: `text`
   Input: `My !\[foo bar\](/path/to/train.jpg  "title"   )
`
   Parse Tree:
   ```
  └── text: "My ![foo bar](/path/to/train.jpg  "title"   )
"
   ```

✅ **cm_example_579**: `text`
   Input: `!\[foo\](<url>)
`
   Parse Tree:
   ```
  └── text: "![foo](<url>)
"
   ```

✅ **cm_example_580**: `text`
   Input: `!\[\](/url)
`
   Parse Tree:
   ```
  └── text: "![](/url)
"
   ```

✅ **cm_example_581**: `text`
   Input: `!\[foo\]\[bar\]

\[bar\]: /url
`
   Parse Tree:
   ```
  └── text: "![foo][bar]

[bar]: /url
"
   ```

✅ **cm_example_582**: `text`
   Input: `!\[foo\]\[bar\]

\[BAR\]: /url
`
   Parse Tree:
   ```
  └── text: "![foo][bar]

[BAR]: /url
"
   ```

✅ **cm_example_583**: `text`
   Input: `!\[foo\]\[\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "![foo][]

[foo]: /url "title"
"
   ```

✅ **cm_example_584**: `text`
   Input: `!\[\*foo\* bar\]\[\]

\[\*foo\* bar\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "![*foo* bar][]

[*foo* bar]: /url "title"
"
   ```

✅ **cm_example_585**: `text`
   Input: `!\[Foo\]\[\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "![Foo][]

[foo]: /url "title"
"
   ```

✅ **cm_example_586**: `text`
   Input: `!\[foo\] 
\[\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "![foo] 
[]

[foo]: /url "title"
"
   ```

✅ **cm_example_587**: `text`
   Input: `!\[foo\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "![foo]

[foo]: /url "title"
"
   ```

✅ **cm_example_588**: `text`
   Input: `!\[\*foo\* bar\]

\[\*foo\* bar\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "![*foo* bar]

[*foo* bar]: /url "title"
"
   ```

✅ **cm_example_589**: `text`
   Input: `!\[\[foo\]\]

\[\[foo\]\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "![[foo]]

[[foo]]: /url "title"
"
   ```

✅ **cm_example_590**: `text`
   Input: `!\[Foo\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "![Foo]

[foo]: /url "title"
"
   ```

✅ **cm_example_591**: `text`
   Input: `!\\\[foo\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

❌ **cm_example_592**: `text` (Unexpected failure)
   Input: `\\!\[foo\]

\[foo\]: /url "title"
`
   Error: ` --> 1:1
  |
1 | \\![foo]
  | ^---
  |
  = expected text`

## inline_links

✅ **link_http**: `inline_link`
   Input: `\[link\](https://example.com)`
   Parse Tree:
   ```
  ├── inline_link > "[link](https://example.com)"
    └── bracket_link_without_title: "[link](https://example.com)"
   ```

✅ **link_https**: `inline_link`
   Input: `\[secure link\](https://secure.example.com)`
   Parse Tree:
   ```
  ├── inline_link > "[secure link](https://secure.example.com)"
    └── bracket_link_without_title: "[secure link](https://secure.example.com)"
   ```

✅ **link_local**: `inline_link`
   Input: `\[local file\](./path/to/file.md)`
   Parse Tree:
   ```
  ├── inline_link > "[local file](./path/to/file.md)"
    └── bracket_link_without_title: "[local file](./path/to/file.md)"
   ```

✅ **link_empty_text**: `inline_link`
   Input: `\[\](https://example.com)`
   Parse Tree:
   ```
  ├── inline_link > "[](https://example.com)"
    └── bracket_link_without_title: "[](https://example.com)"
   ```

✅ **link_with_title**: `inline_link`
   Input: `\[link\](https://example.com "Title")`
   Parse Tree:
   ```
  ├── inline_link > "[link](https://example.com "Title")"
    └── bracket_link_with_title: "[link](https://example.com "Title")"
   ```

❌ **link_nested_brackets**: `inline_link` (Unexpected failure)
   Input: `\[link \[with\] brackets\](https://example.com)`
   Error: ` --> 1:1
  |
1 | [link [with] brackets](https://example.com)
  | ^---
  |
  = expected inline_link`

✅ **link_with_formatting**: `inline_link`
   Input: `\[\*\*bold link\*\*\](https://example.com)`
   Parse Tree:
   ```
  ├── inline_link > "[**bold link**](https://example.com)"
    └── bracket_link_without_title: "[**bold link**](https://example.com)"
   ```

✅ **link_unicode**: `inline_link`
   Input: `\[café link\](https://example.com)`
   Parse Tree:
   ```
  ├── inline_link > "[café link](https://example.com)"
    └── bracket_link_without_title: "[café link](https://example.com)"
   ```

✅ **link_empty_url**: `inline_link` (Expected failure)
   Input: `\[text\]()`
   Error: ` --> 1:1
  |
1 | [text]()
  | ^---
  |
  = expected inline_link`

❌ **link_unclosed_bracket**: `inline_link` (Unexpected failure)
   Input: `\[text(missing closing bracket`
   Error: ` --> 1:1
  |
1 | [text(missing closing bracket
  | ^---
  |
  = expected inline_link`

❌ **link_unclosed_paren**: `inline_link` (Unexpected failure)
   Input: `\[text\](missing closing paren`
   Error: ` --> 1:1
  |
1 | [text](missing closing paren
  | ^---
  |
  = expected inline_link`

## reference_links

✅ **ref_link_basic**: `reference_link`
   Input: `\[link text\]\[ref\]`
   Parse Tree:
   ```
  ├── reference_link > "[link text][ref]"
    └── block_caption: "link text"
    └── block_caption: "ref"
   ```

✅ **ref_link_empty**: `reference_link` (Expected failure)
   Input: `\[\]\[ref\]`
   Error: ` --> 1:2
  |
1 | [][ref]
  |  ^---
  |
  = expected block_caption`

❌ **ref_link_self**: `reference_link` (Unexpected failure)
   Input: `\[link text\]\[\]`
   Error: ` --> 1:13
  |
1 | [link text][]
  |             ^---
  |
  = expected block_caption`

❌ **ref_def_basic**: `reference_definition` (Unexpected failure)
   Input: `\[ref\]: https://example.com`
   Error: ` --> 1:8
  |
1 | [ref]: https://example.com
  |        ^---
  |
  = expected inline_link`

❌ **ref_def_with_title**: `reference_definition` (Unexpected failure)
   Input: `\[ref\]: https://example.com "Title"`
   Error: ` --> 1:8
  |
1 | [ref]: https://example.com "Title"
  |        ^---
  |
  = expected inline_link`

❌ **ref_def_with_spaces**: `reference_definition` (Unexpected failure)
   Input: `\[ref\]:   https://example.com   "Title"   `
   Error: ` --> 1:10
  |
1 | [ref]:   https://example.com   "Title"   
  |          ^---
  |
  = expected inline_link`

✅ **ref_image_basic**: `reference_image`
   Input: `!\[alt text\]\[ref\]`
   Parse Tree:
   ```
  ├── reference_image > "![alt text][ref]"
    └── block_caption: "alt text"
    └── block_caption: "ref"
   ```

✅ **ref_image_empty**: `reference_image` (Expected failure)
   Input: `!\[\]\[ref\]`
   Error: ` --> 1:3
  |
1 | ![][ref]
  |   ^---
  |
  = expected block_caption`

## html_elements

✅ **html_span**: `inline_html`
   Input: `<span>text</span>`
   Parse Tree:
   ```
  └── inline_html: "<span>"
   ```

✅ **html_strong**: `inline_html`
   Input: `<strong>bold</strong>`
   Parse Tree:
   ```
  └── inline_html: "<strong>"
   ```

✅ **html_em**: `inline_html`
   Input: `<em>italic</em>`
   Parse Tree:
   ```
  └── inline_html: "<em>"
   ```

✅ **html_self_closing**: `inline_html`
   Input: `<br/>`
   Parse Tree:
   ```
  └── inline_html: "<br/>"
   ```

✅ **html_with_attrs**: `inline_html`
   Input: `<a href="url">link</a>`
   Parse Tree:
   ```
  └── inline_html: "<a href="url">"
   ```

✅ **html_div**: `inline_html`
   Input: `<div>
content
</div>`
   Parse Tree:
   ```
  └── inline_html: "<div>"
   ```

✅ **html_complex**: `inline_html`
   Input: `<div class="container">
<p>Paragraph</p>
</div>`
   Parse Tree:
   ```
  └── inline_html: "<div class="container">"
   ```

✅ **html_empty**: `inline_html` (Expected failure)
   Input: `<>`
   Error: ` --> 1:1
  |
1 | <>
  | ^---
  |
  = expected inline_html`

✅ **html_unclosed**: `inline_html`
   Input: `<div>unclosed`
   Parse Tree:
   ```
  └── inline_html: "<div>"
   ```

✅ **comment_inline**: `inline_comment`
   Input: `<!-- inline comment -->`
   Parse Tree:
   ```
   ```

✅ **comment_block**: `inline_comment`
   Input: `<!--
block comment
with multiple lines
-->`
   Parse Tree:
   ```
   ```

✅ **comment_nested**: `inline_comment`
   Input: `<!-- outer <!-- inner --> comment -->`
   Parse Tree:
   ```
   ```

## commonmark_paragraphs

✅ **cm_example_219**: `text`
   Input: `aaa

bbb
`
   Parse Tree:
   ```
  └── text: "aaa

bbb
"
   ```

✅ **cm_example_220**: `text`
   Input: `aaa
bbb

ccc
ddd
`
   Parse Tree:
   ```
  └── text: "aaa
bbb

ccc
ddd
"
   ```

✅ **cm_example_221**: `text`
   Input: `aaa


bbb
`
   Parse Tree:
   ```
  └── text: "aaa


bbb
"
   ```

✅ **cm_example_222**: `text`
   Input: `  aaa
 bbb
`
   Parse Tree:
   ```
  └── text: "  aaa
 bbb
"
   ```

✅ **cm_example_223**: `text`
   Input: `aaa
             bbb
                                       ccc
`
   Parse Tree:
   ```
  └── text: "aaa
             bbb
                                       ccc
"
   ```

✅ **cm_example_224**: `text`
   Input: `   aaa
bbb
`
   Parse Tree:
   ```
  └── text: "   aaa
bbb
"
   ```

✅ **cm_example_225**: `text`
   Input: `    aaa
bbb
`
   Parse Tree:
   ```
  └── text: "    aaa
bbb
"
   ```

✅ **cm_example_226**: `text`
   Input: `aaa     
bbb     
`
   Parse Tree:
   ```
  └── text: "aaa     
bbb     
"
   ```

## tables

✅ **table_simple**: `table`
   Input: `| Col1 | Col2 |
|------|------|
| A    | B    |`
   Parse Tree:
   ```
  ├── table > "| Col1 | Col2 |
|------|------|
| A    | B    |"
    ├── table_header > "| Col1 | Col2 |"
      ├── table_cell > "Col1 "
        ├── table_cell_content > "Col1 "
          └── table_safe_text: "Col1 "
      ├── table_cell > "Col2 "
        ├── table_cell_content > "Col2 "
          └── table_safe_text: "Col2 "
      └── table_cell: ""
    ├── table_sep > "|------|------|"
      └── table_sep_cell: "------"
      └── table_sep_cell: "------"
    ├── table_row > "| A    | B    |"
      ├── table_cell > "A    "
        ├── table_cell_content > "A    "
          └── table_safe_text: "A    "
      ├── table_cell > "B    "
        ├── table_cell_content > "B    "
          └── table_safe_text: "B    "
      └── table_cell: ""
   ```

✅ **table_with_alignment**: `table`
   Input: `| Left | Center | Right |
|:-----|:------:|------:|
| A    | B      | C     |`
   Parse Tree:
   ```
  ├── table > "| Left | Center | Right |
|:-----|:------:|------:|
| A    | B      | C     |"
    ├── table_header > "| Left | Center | Right |"
      ├── table_cell > "Left "
        ├── table_cell_content > "Left "
          └── table_safe_text: "Left "
      ├── table_cell > "Center "
        ├── table_cell_content > "Center "
          └── table_safe_text: "Center "
      ├── table_cell > "Right "
        ├── table_cell_content > "Right "
          └── table_safe_text: "Right "
      └── table_cell: ""
    ├── table_sep > "|:-----|:------:|------:|"
      └── table_sep_cell: ":-----"
      └── table_sep_cell: ":------:"
      └── table_sep_cell: "------:"
    ├── table_row > "| A    | B      | C     |"
      ├── table_cell > "A    "
        ├── table_cell_content > "A    "
          └── table_safe_text: "A    "
      ├── table_cell > "B      "
        ├── table_cell_content > "B      "
          └── table_safe_text: "B      "
      ├── table_cell > "C     "
        ├── table_cell_content > "C     "
          └── table_safe_text: "C     "
      └── table_cell: ""
   ```

✅ **table_minimal**: `table`
   Input: `|A|B|
|-|-|
|1|2|`
   Parse Tree:
   ```
  ├── table > "|A|B|
|-|-|
|1|2|"
    ├── table_header > "|A|B|"
      ├── table_cell > "A"
        ├── table_cell_content > "A"
          └── table_safe_text: "A"
      ├── table_cell > "B"
        ├── table_cell_content > "B"
          └── table_safe_text: "B"
      └── table_cell: ""
    ├── table_sep > "|-|-|"
      └── table_sep_cell: "-"
      └── table_sep_cell: "-"
    ├── table_row > "|1|2|"
      ├── table_cell > "1"
        ├── table_cell_content > "1"
          └── table_safe_text: "1"
      ├── table_cell > "2"
        ├── table_cell_content > "2"
          └── table_safe_text: "2"
      └── table_cell: ""
   ```

✅ **table_with_formatting**: `table`
   Input: `| \*\*Bold\*\* | \*Italic\* |
|----------|----------|
| \`code\`   | \[link\](url) |`
   Parse Tree:
   ```
  ├── table > "| **Bold** | *Italic* |
|----------|----------|
| `code`   | [link](url) |"
    ├── table_header > "| **Bold** | *Italic* |"
      ├── table_cell > "**Bold** "
        ├── table_cell_content > "**Bold**"
          ├── emphasis > "**Bold**"
            ├── bold > "**Bold**"
              └── bold_asterisk: "**Bold**"
      ├── table_cell > "*Italic* "
        ├── table_cell_content > "*Italic*"
          ├── emphasis > "*Italic*"
            ├── italic > "*Italic*"
              └── italic_asterisk: "*Italic*"
      └── table_cell: ""
    ├── table_sep > "|----------|----------|"
      └── table_sep_cell: "----------"
      └── table_sep_cell: "----------"
    ├── table_row > "| `code`   | [link](url) |"
      ├── table_cell > "`code`   "
        ├── table_cell_content > "`code`"
          └── code_inline: "`code`"
      ├── table_cell > "[link](url) "
        ├── table_cell_content > "[link](url)"
          ├── inline_link > "[link](url)"
            └── bracket_link_without_title: "[link](url)"
      └── table_cell: ""
   ```

✅ **table_with_pipes**: `table`
   Input: `| Text | With \\| Pipe |
|------|------------|
| A    | B          |`
   Parse Tree:
   ```
  ├── table > "| Text | With \\| Pipe |
|------|------------|
| A    | B          |"
    ├── table_header > "| Text | With \\| Pipe |"
      ├── table_cell > "Text "
        ├── table_cell_content > "Text "
          └── table_safe_text: "Text "
      ├── table_cell > "With \\"
        ├── table_cell_content > "With \\"
          └── table_safe_text: "With \\"
      ├── table_cell > "Pipe "
        ├── table_cell_content > "Pipe "
          └── table_safe_text: "Pipe "
      └── table_cell: ""
    ├── table_sep > "|------|------------|"
      └── table_sep_cell: "------"
      └── table_sep_cell: "------------"
    ├── table_row > "| A    | B          |"
      ├── table_cell > "A    "
        ├── table_cell_content > "A    "
          └── table_safe_text: "A    "
      ├── table_cell > "B          "
        ├── table_cell_content > "B          "
          └── table_safe_text: "B          "
      └── table_cell: ""
   ```

✅ **table_empty_cells**: `table`
   Input: `| | |
|-|-|
| | |`
   Parse Tree:
   ```
  ├── table > "| | |
|-|-|
| | |"
    ├── table_header > "| | |"
      └── table_cell: ""
      └── table_cell: ""
      └── table_cell: ""
    ├── table_sep > "|-|-|"
      └── table_sep_cell: "-"
      └── table_sep_cell: "-"
    ├── table_row > "| | |"
      └── table_cell: ""
      └── table_cell: ""
      └── table_cell: ""
   ```

✅ **table_uneven_columns**: `table`
   Input: `| A | B | C |
|---|---|
| 1 | 2 |`
   Parse Tree:
   ```
  ├── table > "| A | B | C |
|---|---|
| 1 | 2 |"
    ├── table_header > "| A | B | C |"
      ├── table_cell > "A "
        ├── table_cell_content > "A "
          └── table_safe_text: "A "
      ├── table_cell > "B "
        ├── table_cell_content > "B "
          └── table_safe_text: "B "
      ├── table_cell > "C "
        ├── table_cell_content > "C "
          └── table_safe_text: "C "
      └── table_cell: ""
    ├── table_sep > "|---|---|"
      └── table_sep_cell: "---"
      └── table_sep_cell: "---"
    ├── table_row > "| 1 | 2 |"
      ├── table_cell > "1 "
        ├── table_cell_content > "1 "
          └── table_safe_text: "1 "
      ├── table_cell > "2 "
        ├── table_cell_content > "2 "
          └── table_safe_text: "2 "
      └── table_cell: ""
   ```

✅ **table_no_separator**: `table` (Expected failure)
   Input: `| A | B |
| 1 | 2 |`
   Error: ` --> 2:3
  |
2 | | 1 | 2 |
  |   ^---
  |
  = expected table_sep_cell`

✅ **table_malformed**: `table`
   Input: `| A | B
|---|
| 1 | 2 |`
   Parse Tree:
   ```
  ├── table > "| A | B
|---|
| 1 | 2 |"
    ├── table_header > "| A | B"
      ├── table_cell > "A "
        ├── table_cell_content > "A "
          └── table_safe_text: "A "
      ├── table_cell > "B"
        ├── table_cell_content > "B"
          └── table_safe_text: "B"
    ├── table_sep > "|---|"
      └── table_sep_cell: "---"
    ├── table_row > "| 1 | 2 |"
      ├── table_cell > "1 "
        ├── table_cell_content > "1 "
          └── table_safe_text: "1 "
      ├── table_cell > "2 "
        ├── table_cell_content > "2 "
          └── table_safe_text: "2 "
      └── table_cell: ""
   ```

## commonmark_tabs

✅ **cm_example_1**: `text`
   Input: `	foo	baz		bim
`
   Parse Tree:
   ```
  └── text: "	foo	baz		bim
"
   ```

✅ **cm_example_2**: `text`
   Input: `  	foo	baz		bim
`
   Parse Tree:
   ```
  └── text: "  	foo	baz		bim
"
   ```

✅ **cm_example_3**: `text`
   Input: `    a	a
    ὐ	a
`
   Parse Tree:
   ```
  └── text: "    a	a
    ὐ	a
"
   ```

✅ **cm_example_4**: `text`
   Input: `  - foo

	bar
`
   Parse Tree:
   ```
  └── text: "  - foo

	bar
"
   ```

✅ **cm_example_5**: `text`
   Input: `- foo

		bar
`
   Parse Tree:
   ```
  └── text: "- foo

		bar
"
   ```

✅ **cm_example_6**: `text`
   Input: `>		foo
`
   Parse Tree:
   ```
  └── text: ">		foo
"
   ```

✅ **cm_example_7**: `text`
   Input: `-		foo
`
   Parse Tree:
   ```
  └── text: "-		foo
"
   ```

✅ **cm_example_8**: `text`
   Input: `    foo
	bar
`
   Parse Tree:
   ```
  └── text: "    foo
	bar
"
   ```

✅ **cm_example_9**: `text`
   Input: ` - foo
   - bar
	 - baz
`
   Parse Tree:
   ```
  └── text: " - foo
   - bar
	 - baz
"
   ```

✅ **cm_example_10**: `text`
   Input: `#	Foo
`
   Parse Tree:
   ```
  └── text: "#	Foo
"
   ```

✅ **cm_example_11**: `text`
   Input: `\*	\*	\*	
`
   Parse Tree:
   ```
  └── text: "*	*	*	
"
   ```

## commonmark_precedence

✅ **cm_example_42**: `text`
   Input: `- \`one
- two\`
`
   Parse Tree:
   ```
  └── text: "- `one
- two`
"
   ```

## boundary_conditions

✅ **max_list_nesting**: `list`
   Input: `- 1
  - 2
    - 3
      - 4
        - 5
          - 6
            - 7
              - 8
                - 9
                  - 10
                    - 11
                      - 12
                        - 13
                          - 14
                            - 15
                              - 16
                                - 17
                                  - 18
                                    - 19
                                      - 20
                                        - 21
                                          - 22
                                            - 23
                                              - 24
                                                - 25
                                                  - 26
                                                    - 27
                                                      - 28
                                                        - 29
                                                          - 30
                                                            - 31
                                                              - 32`
   Parse Tree:
   ```
  ├── list > "- 1
  - 2
    - 3
      - 4
        - 5
          - 6
            - 7
              - 8
                - 9
                  - 10
                    - 11
                      - 12
                        - 13
                          - 14
                            - 15
                              - 16
                                - 17
                                  - 18
                                    - 19
                                      - 20
                                        - 21
                                          - 22
                                            - 23
                                              - 24
                                                - 25
                                                  - 26
                                                    - 27
                                                      - 28
                                                        - 29
                                                          - 30
                                                            - 31
                                                              - 32"
    ├── list_item > "- 1"
      ├── regular_list_item > "- 1"
        └── list_marker: "-"
        └── list_item_content: "1"
    ├── list_item > "- 2"
      ├── regular_list_item > "- 2"
        └── list_marker: "-"
        └── list_item_content: "2"
    ├── list_item > "- 3"
      ├── regular_list_item > "- 3"
        └── list_marker: "-"
        └── list_item_content: "3"
    ├── list_item > "- 4"
      ├── regular_list_item > "- 4"
        └── list_marker: "-"
        └── list_item_content: "4"
    ├── list_item > "- 5"
      ├── regular_list_item > "- 5"
        └── list_marker: "-"
        └── list_item_content: "5"
    ├── list_item > "- 6"
      ├── regular_list_item > "- 6"
        └── list_marker: "-"
        └── list_item_content: "6"
    ├── list_item > "- 7"
      ├── regular_list_item > "- 7"
        └── list_marker: "-"
        └── list_item_content: "7"
    ├── list_item > "- 8"
      ├── regular_list_item > "- 8"
        └── list_marker: "-"
        └── list_item_content: "8"
    ├── list_item > "- 9"
      ├── regular_list_item > "- 9"
        └── list_marker: "-"
        └── list_item_content: "9"
    ├── list_item > "- 10"
      ├── regular_list_item > "- 10"
        └── list_marker: "-"
        └── list_item_content: "10"
    ├── list_item > "- 11"
      ├── regular_list_item > "- 11"
        └── list_marker: "-"
        └── list_item_content: "11"
    ├── list_item > "- 12"
      ├── regular_list_item > "- 12"
        └── list_marker: "-"
        └── list_item_content: "12"
    ├── list_item > "- 13"
      ├── regular_list_item > "- 13"
        └── list_marker: "-"
        └── list_item_content: "13"
    ├── list_item > "- 14"
      ├── regular_list_item > "- 14"
        └── list_marker: "-"
        └── list_item_content: "14"
    ├── list_item > "- 15"
      ├── regular_list_item > "- 15"
        └── list_marker: "-"
        └── list_item_content: "15"
    ├── list_item > "- 16"
      ├── regular_list_item > "- 16"
        └── list_marker: "-"
        └── list_item_content: "16"
    ├── list_item > "- 17"
      ├── regular_list_item > "- 17"
        └── list_marker: "-"
        └── list_item_content: "17"
    ├── list_item > "- 18"
      ├── regular_list_item > "- 18"
        └── list_marker: "-"
        └── list_item_content: "18"
    ├── list_item > "- 19"
      ├── regular_list_item > "- 19"
        └── list_marker: "-"
        └── list_item_content: "19"
    ├── list_item > "- 20"
      ├── regular_list_item > "- 20"
        └── list_marker: "-"
        └── list_item_content: "20"
    ├── list_item > "- 21"
      ├── regular_list_item > "- 21"
        └── list_marker: "-"
        └── list_item_content: "21"
    ├── list_item > "- 22"
      ├── regular_list_item > "- 22"
        └── list_marker: "-"
        └── list_item_content: "22"
    ├── list_item > "- 23"
      ├── regular_list_item > "- 23"
        └── list_marker: "-"
        └── list_item_content: "23"
    ├── list_item > "- 24"
      ├── regular_list_item > "- 24"
        └── list_marker: "-"
        └── list_item_content: "24"
    ├── list_item > "- 25"
      ├── regular_list_item > "- 25"
        └── list_marker: "-"
        └── list_item_content: "25"
    ├── list_item > "- 26"
      ├── regular_list_item > "- 26"
        └── list_marker: "-"
        └── list_item_content: "26"
    ├── list_item > "- 27"
      ├── regular_list_item > "- 27"
        └── list_marker: "-"
        └── list_item_content: "27"
    ├── list_item > "- 28"
      ├── regular_list_item > "- 28"
        └── list_marker: "-"
        └── list_item_content: "28"
    ├── list_item > "- 29"
      ├── regular_list_item > "- 29"
        └── list_marker: "-"
        └── list_item_content: "29"
    ├── list_item > "- 30"
      ├── regular_list_item > "- 30"
        └── list_marker: "-"
        └── list_item_content: "30"
    ├── list_item > "- 31"
      ├── regular_list_item > "- 31"
        └── list_marker: "-"
        └── list_item_content: "31"
    ├── list_item > "- 32"
      ├── regular_list_item > "- 32"
        └── list_marker: "-"
        └── list_item_content: "32"
   ```

✅ **almost_empty**: `text`
   Input: ` `
   Parse Tree:
   ```
  └── text: " "
   ```

✅ **just_newlines**: `text`
   Input: `




`
   Parse Tree:
   ```
  └── text: "




"
   ```

✅ **only_markdown_chars**: `text`
   Input: `\*\_\`#\[\]~>|$@^=-`
   Parse Tree:
   ```
  └── text: "*_`#[]"
   ```

✅ **largest_heading_number**: `ordered_marker`
   Input: `999999999999999999999. Heading`
   Parse Tree:
   ```
  └── ordered_marker: "999999999999999999999."
   ```

✅ **zero_heading**: `ordered_marker`
   Input: `0. Zero heading`
   Parse Tree:
   ```
  └── ordered_marker: "0."
   ```

❌ **negative_heading**: `ordered_marker` (Unexpected failure)
   Input: `-1. Negative heading`
   Error: ` --> 1:1
  |
1 | -1. Negative heading
  | ^---
  |
  = expected ordered_marker`

✅ **extremely_long_url**: `inline_url`
   Input: `https://example.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
   Parse Tree:
   ```
  └── link_url: "https://example.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
   ```

✅ **url_with_many_params**: `inline_url`
   Input: `https://example.com?param0=value0&param1=value1&param2=value2&param3=value3&param4=value4&param5=value5&param6=value6&param7=value7&param8=value8&param9=value9&param10=value10&param11=value11&param12=value12&param13=value13&param14=value14&param15=value15&param16=value16&param17=value17&param18=value18&param19=value19`
   Parse Tree:
   ```
  └── link_url: "https://example.com?param0=value0&param1=value1&param2=value2&param3=value3&param4=value4&param5=value5&param6=value6&param7=value7&param8=value8&param9=value9&param10=value10&param11=value11&param12=value12&param13=value13&param14=value14&param15=value15&param16=value16&param17=value17&param18=value18&param19=value19"
   ```

✅ **ipv6_url**: `inline_url`
   Input: `http://\[2001:db8::1\]:8080/path`
   Parse Tree:
   ```
  └── link_url: "http://"
   ```

✅ **localhost_variants**: `text`
   Input: `http://127.0.0.1:8080/path`
   Parse Tree:
   ```
  └── text: "http://127.0.0.1:8080/path"
   ```

## user_mentions

✅ **user_simple**: `user_mention`
   Input: `@username`
   Parse Tree:
   ```
  ├── user_mention > "@username"
    └── username: "username"
   ```

✅ **user_underscore**: `user_mention`
   Input: `@user\_name`
   Parse Tree:
   ```
  ├── user_mention > "@user_name"
    └── username: "user_name"
   ```

✅ **user_hyphen**: `user_mention`
   Input: `@user-name`
   Parse Tree:
   ```
  ├── user_mention > "@user-name"
    └── username: "user-name"
   ```

✅ **user_unicode**: `user_mention`
   Input: `@café\_user`
   Parse Tree:
   ```
  ├── user_mention > "@café_user"
    └── username: "café_user"
   ```

✅ **user_with_platform**: `user_mention`
   Input: `@user \[github\]`
   Parse Tree:
   ```
  ├── user_mention > "@user "
    └── username: "user"
   ```

✅ **user_platform_complex**: `user_mention`
   Input: `@user \[platform.name\]`
   Parse Tree:
   ```
  ├── user_mention > "@user "
    └── username: "user"
   ```

✅ **user_with_display**: `user_mention`
   Input: `@user \[platform\](Display Name)`
   Parse Tree:
   ```
  ├── user_mention > "@user "
    └── username: "user"
   ```

✅ **user_full**: `user_mention`
   Input: `@user \[github\](John Doe)`
   Parse Tree:
   ```
  ├── user_mention > "@user "
    └── username: "user"
   ```

✅ **user_empty**: `user_mention` (Expected failure)
   Input: `@`
   Error: ` --> 1:2
  |
1 | @
  |  ^---
  |
  = expected username`

✅ **user_with_space**: `user_mention`
   Input: `@user name`
   Parse Tree:
   ```
  ├── user_mention > "@user "
    └── username: "user"
   ```

## horizontal_rules

✅ **hr_dashes**: `hr`
   Input: `---`
   Parse Tree:
   ```
  └── hr: "---"
   ```

✅ **hr_asterisks**: `hr`
   Input: `\*\*\*`
   Parse Tree:
   ```
  └── hr: "***"
   ```

✅ **hr_underscores**: `hr`
   Input: `\_\_\_`
   Parse Tree:
   ```
  └── hr: "___"
   ```

✅ **hr_spaced_dashes**: `hr`
   Input: ` --- `
   Parse Tree:
   ```
  └── hr: " --- "
   ```

✅ **hr_spaced_asterisks**: `hr`
   Input: ` \*\*\* `
   Parse Tree:
   ```
  └── hr: " *** "
   ```

✅ **hr_long_dashes**: `hr`
   Input: `----------`
   Parse Tree:
   ```
  └── hr: "---"
   ```

✅ **hr_long_asterisks**: `hr`
   Input: `\*\*\*\*\*\*\*\*\*\*`
   Parse Tree:
   ```
  └── hr: "***"
   ```

❌ **hr_too_short**: `hr` (Unexpected failure)
   Input: `--`
   Error: ` --> 1:1
  |
1 | --
  | ^---
  |
  = expected hr`

❌ **hr_mixed**: `hr` (Unexpected failure)
   Input: `-\*-`
   Error: ` --> 1:1
  |
1 | -*-
  | ^---
  |
  = expected hr`

## integration_tests

✅ **real_world_blog_post**: `document`
   Input: `# How to Use Marco

\*\*Marco\*\* is a powerful \*markdown\* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

\`\`\`rust
fn main() {
    println!("Hello, world!");
}
\`\`\`

> Marco makes markdown easy!

Visit \[our website\](https://example.com) for more info.`
   Parse Tree:
   ```
  ├── document > "# How to Use Marco

**Marco** is a powerful *markdown* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

```rust
fn main() {
    println!("Hello, world!");
}
```

> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
    ├── block > "# How to Use Marco"
      ├── heading > "# How to Use Marco"
        ├── H1 > "# How to Use Marco"
          ├── heading_content > "How to Use Marco"
            ├── heading_inline > "How"
              └── word: "How"
            ├── heading_inline > "to"
              └── word: "to"
            ├── heading_inline > "Use"
              └── word: "Use"
            ├── heading_inline > "Marco"
              └── word: "Marco"
    ├── block > "**Marco** is a powerful *markdown* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

```rust
fn main() {
    println!("Hello, world!");
}
```

> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
      ├── paragraph > "**Marco** is a powerful *markdown* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

```rust
fn main() {
    println!("Hello, world!");
}
```

> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
        ├── paragraph_line > "**Marco** is a powerful *markdown* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

```rust
fn main() {
    println!("Hello, world!");
}
```

> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
          ├── inline > "**Marco**"
            ├── inline_core > "**Marco**"
              ├── emphasis > "**Marco**"
                ├── bold > "**Marco**"
                  └── bold_asterisk: "**Marco**"
          ├── inline > "is a powerful *markdown* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

```rust
fn main() {
    println!("Hello, world!");
}
```

> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
            ├── inline_core > "is a powerful *markdown* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

```rust
fn main() {
    println!("Hello, world!");
}
```

> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
              └── text: "is a powerful *markdown* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

```rust
fn main() {
    println!("Hello, world!");
}
```

> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
   ```

✅ **real_world_technical_doc**: `document`
   Input: `# API Reference

## Authentication

Use JWT tokens:

\`\`\`http
GET /api/users
Authorization: Bearer <token>
\`\`\`

### Response

| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::`
   Parse Tree:
   ```
  ├── document > "# API Reference

## Authentication

Use JWT tokens:

```http
GET /api/users
Authorization: Bearer <token>
```

### Response

| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
    ├── block > "# API Reference"
      ├── heading > "# API Reference"
        ├── H1 > "# API Reference"
          ├── heading_content > "API Reference"
            ├── heading_inline > "API"
              └── word: "API"
            ├── heading_inline > "Reference"
              └── word: "Reference"
    ├── block > "## Authentication"
      ├── heading > "## Authentication"
        ├── H2 > "## Authentication"
          ├── heading_content > "Authentication"
            ├── heading_inline > "Authentication"
              └── word: "Authentication"
    ├── block > "Use JWT tokens:

```http
GET /api/users
Authorization: Bearer <token>
```

### Response

| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
      ├── paragraph > "Use JWT tokens:

```http
GET /api/users
Authorization: Bearer <token>
```

### Response

| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
        ├── paragraph_line > "Use JWT tokens:

```http
GET /api/users
Authorization: Bearer <token>
```

### Response

| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
          ├── inline > "Use JWT tokens:

```http
GET /api/users
Authorization: Bearer <token>
```

### Response

| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
            ├── inline_core > "Use JWT tokens:

```http
GET /api/users
Authorization: Bearer <token>
```

### Response

| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
              └── text: "Use JWT tokens:

```http
GET /api/users
Authorization: Bearer <token>
```

### Response

| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
   ```

✅ **real_world_readme**: `document`
   Input: `# My Project

\[!\[CI\](https://img.shields.io/badge/CI-passing-green)\](https://example.com)

## Quick Start

1. Install dependencies: \`npm install\`
2. Run tests: \`npm test\`
3. Build: \`npm run build\`

### Configuration

Create a \`.env\` file:

\`\`\`bash
API\_KEY=your\_key\_here
DEBUG=true
\`\`\`

## Contributing

- \[x\] Write tests
- \[ \] Update docs
- \[ \] Add examples

\*\*Note\*\*: Please follow our \[style guide\](STYLE.md).`
   Parse Tree:
   ```
  ├── document > "# My Project

[![CI](https://img.shields.io/badge/CI-passing-green)](https://example.com)

## Quick Start

1. Install dependencies: `npm install`
2. Run tests: `npm test`
3. Build: `npm run build`

### Configuration

Create a `.env` file:

```bash
API_KEY=your_key_here
DEBUG=true
```

## Contributing

- [x] Write tests
- [ ] Update docs
- [ ] Add examples

**Note**: Please follow our [style guide](STYLE.md)."
    ├── block > "# My Project"
      ├── heading > "# My Project"
        ├── H1 > "# My Project"
          ├── heading_content > "My Project"
            ├── heading_inline > "My"
              └── word: "My"
            ├── heading_inline > "Project"
              └── word: "Project"
    ├── block > "[![CI](https://img.shields.io/badge/CI-passing-green)](https://example.com)

## Quick Start

1. Install dependencies: `npm install`
2. Run tests: `npm test`
3. Build: `npm run build`

### Configuration

Create a `.env` file:

```bash
API_KEY=your_key_here
DEBUG=true
```

## Contributing

- [x] Write tests
- [ ] Update docs
- [ ] Add examples

**Note**: Please follow our [style guide](STYLE.md)."
      ├── paragraph > "[![CI](https://img.shields.io/badge/CI-passing-green)](https://example.com)

## Quick Start

1. Install dependencies: `npm install`
2. Run tests: `npm test`
3. Build: `npm run build`

### Configuration

Create a `.env` file:

```bash
API_KEY=your_key_here
DEBUG=true
```

## Contributing

- [x] Write tests
- [ ] Update docs
- [ ] Add examples

**Note**: Please follow our [style guide](STYLE.md)."
        ├── paragraph_line > "[![CI](https://img.shields.io/badge/CI-passing-green)](https://example.com)

## Quick Start

1. Install dependencies: `npm install`
2. Run tests: `npm test`
3. Build: `npm run build`

### Configuration

Create a `.env` file:

```bash
API_KEY=your_key_here
DEBUG=true
```

## Contributing

- [x] Write tests
- [ ] Update docs
- [ ] Add examples

**Note**: Please follow our [style guide](STYLE.md)."
          ├── inline > "[![CI](https://img.shields.io/badge/CI-passing-green)"
            ├── inline_core > "[![CI](https://img.shields.io/badge/CI-passing-green)"
              ├── inline_link > "[![CI](https://img.shields.io/badge/CI-passing-green)"
                └── bracket_link_without_title: "[![CI](https://img.shields.io/badge/CI-passing-green)"
          ├── inline > "](https://example.com)

## Quick Start

1. Install dependencies: `npm install`
2. Run tests: `npm test`
3. Build: `npm run build`

### Configuration

Create a `.env` file:

```bash
API_KEY=your_key_here
DEBUG=true
```

## Contributing

- [x] Write tests
- [ ] Update docs
- [ ] Add examples

**Note**: Please follow our [style guide](STYLE.md)."
            ├── inline_core > "](https://example.com)

## Quick Start

1. Install dependencies: `npm install`
2. Run tests: `npm test`
3. Build: `npm run build`

### Configuration

Create a `.env` file:

```bash
API_KEY=your_key_here
DEBUG=true
```

## Contributing

- [x] Write tests
- [ ] Update docs
- [ ] Add examples

**Note**: Please follow our [style guide](STYLE.md)."
              └── text: "](https://example.com)

## Quick Start

1. Install dependencies: `npm install`
2. Run tests: `npm test`
3. Build: `npm run build`

### Configuration

Create a `.env` file:

```bash
API_KEY=your_key_here
DEBUG=true
```

## Contributing

- [x] Write tests
- [ ] Update docs
- [ ] Add examples

**Note**: Please follow our [style guide](STYLE.md)."
   ```

## blockquotes

✅ **quote_simple**: `blockquote`
   Input: `> This is a quote`
   Parse Tree:
   ```
  ├── blockquote > "> This is a quote"
    ├── blockquote_line > "> This is a quote"
      ├── inline > "This is a quote"
        ├── inline_core > "This is a quote"
          └── text: "This is a quote"
   ```

✅ **quote_multiline**: `blockquote`
   Input: `> First line
> Second line`
   Parse Tree:
   ```
  ├── blockquote > "> First line
> Second line"
    ├── blockquote_line > "> First line
> Second line"
      ├── inline > "First line
> Second line"
        ├── inline_core > "First line
> Second line"
          └── text: "First line
> Second line"
   ```

✅ **quote_with_space**: `blockquote`
   Input: `>   Spaced quote`
   Parse Tree:
   ```
  ├── blockquote > ">   Spaced quote"
    ├── blockquote_line > ">   Spaced quote"
      ├── inline > "Spaced quote"
        ├── inline_core > "Spaced quote"
          └── text: "Spaced quote"
   ```

✅ **quote_no_space**: `blockquote`
   Input: `>No space quote`
   Parse Tree:
   ```
  ├── blockquote > ">No space quote"
    ├── blockquote_line > ">No space quote"
      ├── inline > "No space quote"
        ├── inline_core > "No space quote"
          └── text: "No space quote"
   ```

✅ **quote_empty**: `blockquote`
   Input: `>`
   Parse Tree:
   ```
  ├── blockquote > ">"
    └── blockquote_line: ">"
   ```

✅ **quote_nested**: `blockquote`
   Input: `> Level 1
>> Level 2
>>> Level 3`
   Parse Tree:
   ```
  ├── blockquote > "> Level 1
>> Level 2
>>> Level 3"
    ├── blockquote_line > "> Level 1
>> Level 2
>>> Level 3"
      ├── inline > "Level 1
>> Level 2
>>> Level 3"
        ├── inline_core > "Level 1
>> Level 2
>>> Level 3"
          └── text: "Level 1
>> Level 2
>>> Level 3"
   ```

✅ **quote_with_bold**: `blockquote`
   Input: `> \*\*Bold quote\*\*`
   Parse Tree:
   ```
  ├── blockquote > "> **Bold quote**"
    ├── blockquote_line > "> **Bold quote**"
      ├── inline > "**Bold quote**"
        ├── inline_core > "**Bold quote**"
          ├── emphasis > "**Bold quote**"
            ├── bold > "**Bold quote**"
              └── bold_asterisk: "**Bold quote**"
   ```

✅ **quote_with_code**: `blockquote`
   Input: `> Quote with \`code\``
   Parse Tree:
   ```
  ├── blockquote > "> Quote with `code`"
    ├── blockquote_line > "> Quote with `code`"
      ├── inline > "Quote with `code`"
        ├── inline_core > "Quote with `code`"
          └── text: "Quote with `code`"
   ```

✅ **quote_with_link**: `blockquote`
   Input: `> Quote with \[link\](url)`
   Parse Tree:
   ```
  ├── blockquote > "> Quote with [link](url)"
    ├── blockquote_line > "> Quote with [link](url)"
      ├── inline > "Quote with [link](url)"
        ├── inline_core > "Quote with [link](url)"
          └── text: "Quote with [link](url)"
   ```

## commonmark_inlines

✅ **cm_example_327**: `text`
   Input: `\`hi\`lo\`
`
   Parse Tree:
   ```
  └── text: "`hi`lo`
"
   ```

## commonmark_links

✅ **cm_example_481**: `text`
   Input: `\[link\](/uri "title")
`
   Parse Tree:
   ```
  └── text: "[link](/uri "title")
"
   ```

✅ **cm_example_482**: `text`
   Input: `\[link\](/uri)
`
   Parse Tree:
   ```
  └── text: "[link](/uri)
"
   ```

✅ **cm_example_483**: `text`
   Input: `\[\](./target.md)
`
   Parse Tree:
   ```
  └── text: "[](./target.md)
"
   ```

✅ **cm_example_484**: `text`
   Input: `\[link\]()
`
   Parse Tree:
   ```
  └── text: "[link]()
"
   ```

✅ **cm_example_485**: `text`
   Input: `\[link\](<>)
`
   Parse Tree:
   ```
  └── text: "[link](<>)
"
   ```

✅ **cm_example_486**: `text`
   Input: `\[\]()
`
   Parse Tree:
   ```
  └── text: "[]()
"
   ```

✅ **cm_example_487**: `text`
   Input: `\[link\](/my uri)
`
   Parse Tree:
   ```
  └── text: "[link](/my uri)
"
   ```

✅ **cm_example_488**: `text`
   Input: `\[link\](</my uri>)
`
   Parse Tree:
   ```
  └── text: "[link](</my uri>)
"
   ```

✅ **cm_example_489**: `text`
   Input: `\[link\](foo
bar)
`
   Parse Tree:
   ```
  └── text: "[link](foo
bar)
"
   ```

✅ **cm_example_490**: `text`
   Input: `\[link\](<foo
bar>)
`
   Parse Tree:
   ```
  └── text: "[link](<foo
bar>)
"
   ```

✅ **cm_example_491**: `text`
   Input: `\[a\](<b)c>)
`
   Parse Tree:
   ```
  └── text: "[a](<b)c>)
"
   ```

✅ **cm_example_492**: `text`
   Input: `\[link\](<foo\\>)
`
   Parse Tree:
   ```
  └── text: "[link](<foo"
   ```

✅ **cm_example_493**: `text`
   Input: `\[a\](<b)c
\[a\](<b)c>
\[a\](<b>c)
`
   Parse Tree:
   ```
  └── text: "[a](<b)c
[a](<b)c>
[a](<b>c)
"
   ```

✅ **cm_example_494**: `text`
   Input: `\[link\](\\(foo\\))
`
   Parse Tree:
   ```
  └── text: "[link]("
   ```

✅ **cm_example_495**: `text`
   Input: `\[link\](foo(and(bar)))
`
   Parse Tree:
   ```
  └── text: "[link](foo(and(bar)))
"
   ```

✅ **cm_example_496**: `text`
   Input: `\[link\](foo(and(bar))
`
   Parse Tree:
   ```
  └── text: "[link](foo(and(bar))
"
   ```

✅ **cm_example_497**: `text`
   Input: `\[link\](foo\\(and\\(bar\\))
`
   Parse Tree:
   ```
  └── text: "[link](foo"
   ```

✅ **cm_example_498**: `text`
   Input: `\[link\](<foo(and(bar)>)
`
   Parse Tree:
   ```
  └── text: "[link](<foo(and(bar)>)
"
   ```

✅ **cm_example_499**: `text`
   Input: `\[link\](foo\\)\\:)
`
   Parse Tree:
   ```
  └── text: "[link](foo"
   ```

✅ **cm_example_500**: `text`
   Input: `\[link\](#fragment)

\[link\](http://example.com#fragment)

\[link\](http://example.com?foo=3#frag)
`
   Parse Tree:
   ```
  └── text: "[link](#fragment)

[link](http://example.com#fragment)

[link](http://example.com?foo=3#frag)
"
   ```

✅ **cm_example_501**: `text`
   Input: `\[link\](foo\\bar)
`
   Parse Tree:
   ```
  └── text: "[link](foo"
   ```

✅ **cm_example_502**: `text`
   Input: `\[link\](foo%20b&auml;)
`
   Parse Tree:
   ```
  └── text: "[link](foo%20b&auml;)
"
   ```

✅ **cm_example_503**: `text`
   Input: `\[link\]("title")
`
   Parse Tree:
   ```
  └── text: "[link]("title")
"
   ```

✅ **cm_example_504**: `text`
   Input: `\[link\](/url "title")
\[link\](/url 'title')
\[link\](/url (title))
`
   Parse Tree:
   ```
  └── text: "[link](/url "title")
[link](/url 'title')
[link](/url (title))
"
   ```

✅ **cm_example_505**: `text`
   Input: `\[link\](/url "title \\"&quot;")
`
   Parse Tree:
   ```
  └── text: "[link](/url "title "
   ```

✅ **cm_example_506**: `text`
   Input: `\[link\](/url"title")
`
   Parse Tree:
   ```
  └── text: "[link](/url"title")
"
   ```

✅ **cm_example_507**: `text`
   Input: `\[link\](/url "title "and" title")
`
   Parse Tree:
   ```
  └── text: "[link](/url "title "and" title")
"
   ```

✅ **cm_example_508**: `text`
   Input: `\[link\](/url 'title "and" title')
`
   Parse Tree:
   ```
  └── text: "[link](/url 'title "and" title')
"
   ```

✅ **cm_example_509**: `text`
   Input: `\[link\](   /uri
  "title"  )
`
   Parse Tree:
   ```
  └── text: "[link](   /uri
  "title"  )
"
   ```

✅ **cm_example_510**: `text`
   Input: `\[link\] (/uri)
`
   Parse Tree:
   ```
  └── text: "[link] (/uri)
"
   ```

✅ **cm_example_511**: `text`
   Input: `\[link \[foo \[bar\]\]\](/uri)
`
   Parse Tree:
   ```
  └── text: "[link [foo [bar]]](/uri)
"
   ```

✅ **cm_example_512**: `text`
   Input: `\[link\] bar\](/uri)
`
   Parse Tree:
   ```
  └── text: "[link] bar](/uri)
"
   ```

✅ **cm_example_513**: `text`
   Input: `\[link \[bar\](/uri)
`
   Parse Tree:
   ```
  └── text: "[link [bar](/uri)
"
   ```

✅ **cm_example_514**: `text`
   Input: `\[link \\\[bar\](/uri)
`
   Parse Tree:
   ```
  └── text: "[link "
   ```

✅ **cm_example_515**: `text`
   Input: `\[link \*foo \*\*bar\*\* \`#\`\*\](/uri)
`
   Parse Tree:
   ```
  └── text: "[link *foo **bar** `#`*](/uri)
"
   ```

✅ **cm_example_516**: `text`
   Input: `\[!\[moon\](moon.jpg)\](/uri)
`
   Parse Tree:
   ```
  └── text: "[![moon](moon.jpg)](/uri)
"
   ```

✅ **cm_example_517**: `text`
   Input: `\[foo \[bar\](/uri)\](/uri)
`
   Parse Tree:
   ```
  └── text: "[foo [bar](/uri)](/uri)
"
   ```

✅ **cm_example_518**: `text`
   Input: `\[foo \*\[bar \[baz\](/uri)\](/uri)\*\](/uri)
`
   Parse Tree:
   ```
  └── text: "[foo *[bar [baz](/uri)](/uri)*](/uri)
"
   ```

✅ **cm_example_519**: `text`
   Input: `!\[\[\[foo\](uri1)\](uri2)\](uri3)
`
   Parse Tree:
   ```
  └── text: "![[[foo](uri1)](uri2)](uri3)
"
   ```

✅ **cm_example_520**: `text`
   Input: `\*\[foo\*\](/uri)
`
   Parse Tree:
   ```
  └── text: "*[foo*](/uri)
"
   ```

✅ **cm_example_521**: `text`
   Input: `\[foo \*bar\](baz\*)
`
   Parse Tree:
   ```
  └── text: "[foo *bar](baz*)
"
   ```

✅ **cm_example_522**: `text`
   Input: `\*foo \[bar\* baz\]
`
   Parse Tree:
   ```
  └── text: "*foo [bar* baz]
"
   ```

✅ **cm_example_523**: `text`
   Input: `\[foo <bar attr="\](baz)">
`
   Parse Tree:
   ```
  └── text: "[foo <bar attr="](baz)">
"
   ```

✅ **cm_example_524**: `text`
   Input: `\[foo\`\](/uri)\`
`
   Parse Tree:
   ```
  └── text: "[foo`](/uri)`
"
   ```

✅ **cm_example_525**: `text`
   Input: `\[foo<http://example.com/?search=\](uri)>
`
   Parse Tree:
   ```
  └── text: "[foo<http://example.com/?search=](uri)>
"
   ```

✅ **cm_example_526**: `text`
   Input: `\[foo\]\[bar\]

\[bar\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[foo][bar]

[bar]: /url "title"
"
   ```

✅ **cm_example_527**: `text`
   Input: `\[link \[foo \[bar\]\]\]\[ref\]

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "[link [foo [bar]]][ref]

[ref]: /uri
"
   ```

✅ **cm_example_528**: `text`
   Input: `\[link \\\[bar\]\[ref\]

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "[link "
   ```

✅ **cm_example_529**: `text`
   Input: `\[link \*foo \*\*bar\*\* \`#\`\*\]\[ref\]

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "[link *foo **bar** `#`*][ref]

[ref]: /uri
"
   ```

✅ **cm_example_530**: `text`
   Input: `\[!\[moon\](moon.jpg)\]\[ref\]

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "[![moon](moon.jpg)][ref]

[ref]: /uri
"
   ```

✅ **cm_example_531**: `text`
   Input: `\[foo \[bar\](/uri)\]\[ref\]

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "[foo [bar](/uri)][ref]

[ref]: /uri
"
   ```

✅ **cm_example_532**: `text`
   Input: `\[foo \*bar \[baz\]\[ref\]\*\]\[ref\]

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "[foo *bar [baz][ref]*][ref]

[ref]: /uri
"
   ```

✅ **cm_example_533**: `text`
   Input: `\*\[foo\*\]\[ref\]

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "*[foo*][ref]

[ref]: /uri
"
   ```

✅ **cm_example_534**: `text`
   Input: `\[foo \*bar\]\[ref\]\*

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "[foo *bar][ref]*

[ref]: /uri
"
   ```

✅ **cm_example_535**: `text`
   Input: `\[foo <bar attr="\]\[ref\]">

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "[foo <bar attr="][ref]">

[ref]: /uri
"
   ```

✅ **cm_example_536**: `text`
   Input: `\[foo\`\]\[ref\]\`

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "[foo`][ref]`

[ref]: /uri
"
   ```

✅ **cm_example_537**: `text`
   Input: `\[foo<http://example.com/?search=\]\[ref\]>

\[ref\]: /uri
`
   Parse Tree:
   ```
  └── text: "[foo<http://example.com/?search=][ref]>

[ref]: /uri
"
   ```

✅ **cm_example_538**: `text`
   Input: `\[foo\]\[BaR\]

\[bar\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[foo][BaR]

[bar]: /url "title"
"
   ```

✅ **cm_example_539**: `text`
   Input: `\[ẞ\]

\[SS\]: /url
`
   Parse Tree:
   ```
  └── text: "[ẞ]

[SS]: /url
"
   ```

✅ **cm_example_540**: `text`
   Input: `\[Foo
  bar\]: /url

\[Baz\]\[Foo bar\]
`
   Parse Tree:
   ```
  └── text: "[Foo
  bar]: /url

[Baz][Foo bar]
"
   ```

✅ **cm_example_541**: `text`
   Input: `\[foo\] \[bar\]

\[bar\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[foo] [bar]

[bar]: /url "title"
"
   ```

✅ **cm_example_542**: `text`
   Input: `\[foo\]
\[bar\]

\[bar\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[foo]
[bar]

[bar]: /url "title"
"
   ```

✅ **cm_example_543**: `text`
   Input: `\[foo\]: /url1

\[foo\]: /url2

\[bar\]\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]: /url1

[foo]: /url2

[bar][foo]
"
   ```

✅ **cm_example_544**: `text`
   Input: `\[bar\]\[foo\\!\]

\[foo!\]: /url
`
   Parse Tree:
   ```
  └── text: "[bar][foo"
   ```

✅ **cm_example_545**: `text`
   Input: `\[foo\]\[ref\[\]

\[ref\[\]: /uri
`
   Parse Tree:
   ```
  └── text: "[foo][ref[]

[ref[]: /uri
"
   ```

✅ **cm_example_546**: `text`
   Input: `\[foo\]\[ref\[bar\]\]

\[ref\[bar\]\]: /uri
`
   Parse Tree:
   ```
  └── text: "[foo][ref[bar]]

[ref[bar]]: /uri
"
   ```

✅ **cm_example_547**: `text`
   Input: `\[\[\[foo\]\]\]

\[\[\[foo\]\]\]: /url
`
   Parse Tree:
   ```
  └── text: "[[[foo]]]

[[[foo]]]: /url
"
   ```

✅ **cm_example_548**: `text`
   Input: `\[foo\]\[ref\\\[\]

\[ref\\\[\]: /uri
`
   Parse Tree:
   ```
  └── text: "[foo][ref"
   ```

✅ **cm_example_549**: `text`
   Input: `\[bar\\\\\]: /uri

\[bar\\\\\]
`
   Parse Tree:
   ```
  └── text: "[bar"
   ```

✅ **cm_example_550**: `text`
   Input: `\[\]

\[\]: /uri
`
   Parse Tree:
   ```
  └── text: "[]

[]: /uri
"
   ```

✅ **cm_example_551**: `text`
   Input: `\[
 \]

\[
 \]: /uri
`
   Parse Tree:
   ```
  └── text: "[
 ]

[
 ]: /uri
"
   ```

✅ **cm_example_552**: `text`
   Input: `\[foo\]\[\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[foo][]

[foo]: /url "title"
"
   ```

✅ **cm_example_553**: `text`
   Input: `\[\*foo\* bar\]\[\]

\[\*foo\* bar\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[*foo* bar][]

[*foo* bar]: /url "title"
"
   ```

✅ **cm_example_554**: `text`
   Input: `\[Foo\]\[\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[Foo][]

[foo]: /url "title"
"
   ```

✅ **cm_example_555**: `text`
   Input: `\[foo\] 
\[\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[foo] 
[]

[foo]: /url "title"
"
   ```

✅ **cm_example_556**: `text`
   Input: `\[foo\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[foo]

[foo]: /url "title"
"
   ```

✅ **cm_example_557**: `text`
   Input: `\[\*foo\* bar\]

\[\*foo\* bar\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[*foo* bar]

[*foo* bar]: /url "title"
"
   ```

✅ **cm_example_558**: `text`
   Input: `\[\[\*foo\* bar\]\]

\[\*foo\* bar\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[[*foo* bar]]

[*foo* bar]: /url "title"
"
   ```

✅ **cm_example_559**: `text`
   Input: `\[\[bar \[foo\]

\[foo\]: /url
`
   Parse Tree:
   ```
  └── text: "[[bar [foo]

[foo]: /url
"
   ```

✅ **cm_example_560**: `text`
   Input: `\[Foo\]

\[foo\]: /url "title"
`
   Parse Tree:
   ```
  └── text: "[Foo]

[foo]: /url "title"
"
   ```

✅ **cm_example_561**: `text`
   Input: `\[foo\] bar

\[foo\]: /url
`
   Parse Tree:
   ```
  └── text: "[foo] bar

[foo]: /url
"
   ```

❌ **cm_example_562**: `text` (Unexpected failure)
   Input: `\\\[foo\]

\[foo\]: /url "title"
`
   Error: ` --> 1:1
  |
1 | \\[foo]
  | ^---
  |
  = expected text`

✅ **cm_example_563**: `text`
   Input: `\[foo\*\]: /url

\*\[foo\*\]
`
   Parse Tree:
   ```
  └── text: "[foo*]: /url

*[foo*]
"
   ```

✅ **cm_example_564**: `text`
   Input: `\[foo\]\[bar\]

\[foo\]: /url1
\[bar\]: /url2
`
   Parse Tree:
   ```
  └── text: "[foo][bar]

[foo]: /url1
[bar]: /url2
"
   ```

✅ **cm_example_565**: `text`
   Input: `\[foo\]\[\]

\[foo\]: /url1
`
   Parse Tree:
   ```
  └── text: "[foo][]

[foo]: /url1
"
   ```

✅ **cm_example_566**: `text`
   Input: `\[foo\]()

\[foo\]: /url1
`
   Parse Tree:
   ```
  └── text: "[foo]()

[foo]: /url1
"
   ```

✅ **cm_example_567**: `text`
   Input: `\[foo\](not a link)

\[foo\]: /url1
`
   Parse Tree:
   ```
  └── text: "[foo](not a link)

[foo]: /url1
"
   ```

✅ **cm_example_568**: `text`
   Input: `\[foo\]\[bar\]\[baz\]

\[baz\]: /url
`
   Parse Tree:
   ```
  └── text: "[foo][bar][baz]

[baz]: /url
"
   ```

✅ **cm_example_569**: `text`
   Input: `\[foo\]\[bar\]\[baz\]

\[baz\]: /url1
\[bar\]: /url2
`
   Parse Tree:
   ```
  └── text: "[foo][bar][baz]

[baz]: /url1
[bar]: /url2
"
   ```

✅ **cm_example_570**: `text`
   Input: `\[foo\]\[bar\]\[baz\]

\[baz\]: /url1
\[foo\]: /url2
`
   Parse Tree:
   ```
  └── text: "[foo][bar][baz]

[baz]: /url1
[foo]: /url2
"
   ```

## definition_lists

❌ **def_list_simple**: `def_list` (Unexpected failure)
   Input: `Term
: Definition`
   Error: ` --> 2:1
  |
2 | : Definition
  | ^---
  |
  = expected def_line`

❌ **def_list_multiple**: `def_list` (Unexpected failure)
   Input: `Term
: First definition
: Second definition`
   Error: ` --> 2:1
  |
2 | : First definition
  | ^---
  |
  = expected def_line`

❌ **def_list_complex**: `def_list` (Unexpected failure)
   Input: `Complex Term
: A very detailed definition that explains the term`
   Error: ` --> 2:1
  |
2 | : A very detailed definition that explains the term
  | ^---
  |
  = expected def_line`

## code_inline

✅ **code_simple**: `code_inline`
   Input: `\`code\``
   Parse Tree:
   ```
  └── code_inline: "`code`"
   ```

✅ **code_with_spaces**: `code_inline`
   Input: `\`some code here\``
   Parse Tree:
   ```
  └── code_inline: "`some code here`"
   ```

✅ **code_empty**: `code_inline` (Expected failure)
   Input: `\`\``
   Error: ` --> 1:1
  |
1 | ``
  | ^---
  |
  = expected code_inline`

✅ **code_with_punctuation**: `code_inline`
   Input: `\`hello, world!\``
   Parse Tree:
   ```
  └── code_inline: "`hello, world!`"
   ```

✅ **code_with_backticks**: `code_inline`
   Input: `\`code with \\\` backtick\``
   Parse Tree:
   ```
  └── code_inline: "`code with \\`"
   ```

✅ **code_multiline**: `code_inline`
   Input: `\`code
with newline\``
   Parse Tree:
   ```
  └── code_inline: "`code
with newline`"
   ```

❌ **code_in_sentence**: `code_inline` (Unexpected failure)
   Input: `Use \`print()\` function`
   Error: ` --> 1:1
  |
1 | Use `print()` function
  | ^---
  |
  = expected code_inline`

✅ **code_multiple**: `code_inline`
   Input: `\`first\` and \`second\` code`
   Parse Tree:
   ```
  └── code_inline: "`first`"
   ```

❌ **code_unclosed**: `code_inline` (Unexpected failure)
   Input: `\`missing closing`
   Error: ` --> 1:1
  |
1 | `missing closing
  | ^---
  |
  = expected code_inline`

❌ **code_triple_backtick**: `code_inline` (Unexpected failure)
   Input: `\`\`\`not inline\`\`\``
   Error: ` --> 1:1
  |
1 | ```not inline```
  | ^---
  |
  = expected code_inline`

## page_and_doc

✅ **page_a4**: `page_tag`
   Input: `\[page=A4\]`
   Parse Tree:
   ```
  ├── page_tag > "[page=A4]"
    └── KW_PAGE: "page"
    └── page_format: "A4"
   ```

✅ **page_us**: `page_tag`
   Input: `\[page=US\]`
   Parse Tree:
   ```
  ├── page_tag > "[page=US]"
    └── KW_PAGE: "page"
    └── page_format: "US"
   ```

✅ **page_custom_size**: `page_tag`
   Input: `\[page=210\]`
   Parse Tree:
   ```
  ├── page_tag > "[page=210]"
    └── KW_PAGE: "page"
    └── page_format: "210"
   ```

✅ **page_empty**: `page_tag`
   Input: `\[page=\]`
   Parse Tree:
   ```
  ├── page_tag > "[page=]"
    └── KW_PAGE: "page"
   ```

✅ **doc_ref_simple**: `doc_ref`
   Input: `\[@doc\](./document.md)`
   Parse Tree:
   ```
  ├── doc_ref > "[@doc](./document.md)"
    └── KW_DOC: "doc"
    └── local_path: "./document.md"
   ```

✅ **doc_ref_complex**: `doc_ref`
   Input: `\[@doc\](../docs/guide/installation.md)`
   Parse Tree:
   ```
  ├── doc_ref > "[@doc](../docs/guide/installation.md)"
    └── KW_DOC: "doc"
    └── local_path: "../docs/guide/installation.md"
   ```

✅ **toc_simple**: `toc`
   Input: `\[toc\]`
   Parse Tree:
   ```
  ├── toc > "[toc]"
    └── KW_TOC: "toc"
   ```

✅ **toc_with_depth**: `toc`
   Input: `\[toc=2\]`
   Parse Tree:
   ```
  ├── toc > "[toc=2]"
    └── KW_TOC: "toc"
    └── toc_depth: "=2"
   ```

✅ **toc_max_depth**: `toc`
   Input: `\[toc=4\]`
   Parse Tree:
   ```
  ├── toc > "[toc=4]"
    └── KW_TOC: "toc"
    └── toc_depth: "=4"
   ```

✅ **toc_with_doc**: `toc`
   Input: `\[toc\](@doc)`
   Parse Tree:
   ```
  ├── toc > "[toc](@doc)"
    └── KW_TOC: "toc"
    ├── toc_doc > "(@doc)"
      └── KW_DOC: "doc"
   ```

✅ **page_invalid_format**: `page_tag` (Expected failure)
   Input: `\[page=invalid\]`
   Error: ` --> 1:7
  |
1 | [page=invalid]
  |       ^---
  |
  = expected page_format`

✅ **toc_invalid_depth**: `toc` (Expected failure)
   Input: `\[toc=5\]`
   Error: ` --> 1:5
  |
1 | [toc=5]
  |     ^---
  |
  = expected toc_depth`

## math_blocks

✅ **math_block_simple**: `math_block`
   Input: `$$x = 1$$`
   Parse Tree:
   ```
  └── math_block: "$$x = 1$$"
   ```

✅ **math_block_complex**: `math_block`
   Input: `$$\\frac{\\partial f}{\\partial x} = \\lim\_{h \	o 0} \\frac{f(x+h) - f(x)}{h}$$`
   Parse Tree:
   ```
  └── math_block: "$$\\frac{\\partial f}{\\partial x} = \\lim_{h \	o 0} \\frac{f(x+h) - f(x)}{h}$$"
   ```

✅ **math_block_empty**: `math_block`
   Input: `$$$$`
   Parse Tree:
   ```
  └── math_block: "$$$$"
   ```

✅ **math_block_multiline**: `math_block`
   Input: `$$
x = 1
y = 2
$$`
   Parse Tree:
   ```
  └── math_block: "$$
x = 1
y = 2
$$"
   ```

❌ **math_block_unclosed**: `math_block` (Unexpected failure)
   Input: `$$missing closing`
   Error: ` --> 1:1
  |
1 | $$missing closing
  | ^---
  |
  = expected math_block`

❌ **math_block_single**: `math_block` (Unexpected failure)
   Input: `$not block$`
   Error: ` --> 1:1
  |
1 | $not block$
  | ^---
  |
  = expected math_block`

## fuzzing_tests

✅ **random_unicode_basic**: `text`
   Input: `🜴🝺🞩🟊🠂🡑🢈🣘🤇🥞🦋🧚🨻🩲🪱🫰`
   Parse Tree:
   ```
  └── text: "🜴🝺🞩🟊🠂🡑🢈🣘🤇🥞🦋🧚🨻🩲🪱🫰"
   ```

✅ **random_unicode_astral**: `text`
   Input: `𝄞𝄢𝅘𝅥𝆺𝇇𝇈𝇉𝇊𝇋𝇌𝇍𝇎𝇏𝇐𝇑`
   Parse Tree:
   ```
  └── text: "𝄞𝄢𝅘𝅥𝆺𝇇𝇈𝇉𝇊𝇋𝇌𝇍𝇎𝇏𝇐𝇑"
   ```

✅ **random_unicode_cjk**: `text`
   Input: `丂丄丅丆丏丒丗丟丠両丣並丩丮丯丱丳乃乄乚乜`
   Parse Tree:
   ```
  └── text: "丂丄丅丆丏丒丗丟丠両丣並丩丮丯丱丳乃乄乚乜"
   ```

✅ **random_unicode_arabic**: `text`
   Input: `؀؁؂؃؄؅؆؇؈؉؊؋،؍؎؏ؘؙؚ؛؜؝؞؟`
   Parse Tree:
   ```
  └── text: "؀؁؂؃؄؅؆؇؈؉؊؋،؍؎؏ؘؙؚ؛؜؝؞؟"
   ```

✅ **malformed_utf8_high_surrogate**: `text` (Expected failure)
   Input: `\\uD800`
   Error: ` --> 1:1
  |
1 | \\uD800
  | ^---
  |
  = expected text`

✅ **malformed_utf8_low_surrogate**: `text` (Expected failure)
   Input: `\\uDFFF`
   Error: ` --> 1:1
  |
1 | \\uDFFF
  | ^---
  |
  = expected text`

✅ **malformed_utf8_overlong**: `text` (Expected failure)
   Input: `\\u0000`
   Error: ` --> 1:1
  |
1 | \\u0000
  | ^---
  |
  = expected text`

❌ **random_ascii_control**: `text` (Unexpected failure)
   Input: `\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\u0008`
   Error: ` --> 1:1
  |
1 | \\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\u0008
  | ^---
  |
  = expected text`

✅ **random_ascii_printable**: `text`
   Input: `!@#$%^&\*()\_+{}|:<>?\[\];',./`
   Parse Tree:
   ```
  └── text: "!@#$%^&*()_+{}|:<>?[];',./"
   ```

✅ **random_ascii_extended**: `text`
   Input: `¡¢£¤¥¦§¨©ª«¬®¯°±²³´µ¶·¸¹º»¼½¾¿`
   Parse Tree:
   ```
  └── text: "¡¢£¤¥¦§¨©ª«¬®¯°±²³´µ¶·¸¹º»¼½¾¿"
   ```

✅ **chaos_markdown_soup**: `text`
   Input: `\*\_\`#\[\]()~>|$@^=-\\\*\*\_\`#\[\]()~>|$@^=-\\\*`
   Parse Tree:
   ```
  └── text: "*_`#[]()"
   ```

✅ **chaos_nested_delimiters**: `text`
   Input: `(\[{<>}\])((\[{<>}\]))(((\[{<>}\])))`
   Parse Tree:
   ```
  └── text: "([{<>}])(([{<>}]))((([{<>}])))"
   ```

✅ **chaos_unicode_soup**: `text`
   Input: `🏳️‍🌈👨‍👩‍👧‍👦🤷🏽‍♀️🧑🏻‍💻🇺🇸🇬🇧🇩🇪🇫🇷🇮🇹`
   Parse Tree:
   ```
  └── text: "🏳️‍🌈👨‍👩‍👧‍👦🤷🏽‍♀️🧑🏻‍💻🇺🇸🇬🇧🇩🇪🇫🇷🇮🇹"
   ```

✅ **exactly_64_chars**: `text`
   Input: `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
   Parse Tree:
   ```
  └── text: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
   ```

✅ **exactly_256_chars**: `text`
   Input: `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
   Parse Tree:
   ```
  └── text: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
   ```

✅ **exactly_1024_chars**: `text`
   Input: `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
   Parse Tree:
   ```
  └── text: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
   ```

## error_recovery

✅ **partial_bold_recovery**: `bold`
   Input: `\*\*bold but not closed and more text`
   Parse Tree:
   ```
  ├── bold > "**bold but not closed and more text"
    └── bold_asterisk: "**bold but not closed and more text"
   ```

❌ **partial_link_recovery**: `inline_link` (Unexpected failure)
   Input: `\[link text but no closing and more text`
   Error: ` --> 1:1
  |
1 | [link text but no closing and more text
  | ^---
  |
  = expected inline_link`

✅ **mixed_delimiters_recovery**: `text`
   Input: `\*\*bold \_italic\* underscore\_\_`
   Parse Tree:
   ```
  └── text: "**bold _italic* underscore__"
   ```

✅ **malformed_table_recovery**: `table`
   Input: `| A | B |
|---|
| 1 | 2 | 3 |`
   Parse Tree:
   ```
  ├── table > "| A | B |
|---|
| 1 | 2 | 3 |"
    ├── table_header > "| A | B |"
      ├── table_cell > "A "
        ├── table_cell_content > "A "
          └── table_safe_text: "A "
      ├── table_cell > "B "
        ├── table_cell_content > "B "
          └── table_safe_text: "B "
      └── table_cell: ""
    ├── table_sep > "|---|"
      └── table_sep_cell: "---"
    ├── table_row > "| 1 | 2 | 3 |"
      ├── table_cell > "1 "
        ├── table_cell_content > "1 "
          └── table_safe_text: "1 "
      ├── table_cell > "2 "
        ├── table_cell_content > "2 "
          └── table_safe_text: "2 "
      ├── table_cell > "3 "
        ├── table_cell_content > "3 "
          └── table_safe_text: "3 "
      └── table_cell: ""
   ```

✅ **unknown_language_code**: `fenced_code`
   Input: `\`\`\`unknown\_lang
code content
\`\`\``
   Parse Tree:
   ```
  ├── fenced_code > "```unknown_lang
code content
```"
    └── language_id: "unknown_lang"
   ```

❌ **unknown_admonition_type**: `admonition_block` (Unexpected failure)
   Input: `:::
custom\_type
content
:::`
   Error: ` --> 1:4
  |
1 | :::␊
  |    ^---
  |
  = expected admonition_type`

✅ **invalid_macro_syntax**: `text`
   Input: `\[invalid:syntax\](no closing`
   Parse Tree:
   ```
  └── text: "[invalid:syntax](no closing"
   ```

✅ **empty_inline_code**: `fenced_code` (Expected failure)
   Input: `\`\``
   Error: ` --> 1:1
  |
1 | ``
  | ^---
  |
  = expected fenced_code`

✅ **empty_emphasis**: `text`
   Input: `\*\*\*\*`
   Parse Tree:
   ```
  └── text: "****"
   ```

✅ **empty_link_text**: `inline_link`
   Input: `\[\](url)`
   Parse Tree:
   ```
  ├── inline_link > "[](url)"
    └── bracket_link_without_title: "[](url)"
   ```

✅ **empty_image_alt**: `text`
   Input: `!\[\](image.png)`
   Parse Tree:
   ```
  └── text: "![](image.png)"
   ```

## commonmark_atx_headings

✅ **cm_example_62**: `text`
   Input: `# foo
## foo
### foo
#### foo
##### foo
###### foo
`
   Parse Tree:
   ```
  └── text: "# foo
## foo
### foo
#### foo
##### foo
###### foo
"
   ```

✅ **cm_example_63**: `text`
   Input: `####### foo
`
   Parse Tree:
   ```
  └── text: "####### foo
"
   ```

✅ **cm_example_64**: `text`
   Input: `#5 bolt

#hashtag
`
   Parse Tree:
   ```
  └── text: "#5 bolt

#hashtag
"
   ```

❌ **cm_example_65**: `text` (Unexpected failure)
   Input: `\\## foo
`
   Error: ` --> 1:1
  |
1 | \\## foo
  | ^---
  |
  = expected text`

✅ **cm_example_66**: `text`
   Input: `# foo \*bar\* \\\*baz\\\*
`
   Parse Tree:
   ```
  └── text: "# foo *bar* "
   ```

✅ **cm_example_67**: `text`
   Input: `#                  foo                     
`
   Parse Tree:
   ```
  └── text: "#                  foo                     
"
   ```

✅ **cm_example_68**: `text`
   Input: ` ### foo
  ## foo
   # foo
`
   Parse Tree:
   ```
  └── text: " ### foo
  ## foo
   # foo
"
   ```

✅ **cm_example_69**: `text`
   Input: `    # foo
`
   Parse Tree:
   ```
  └── text: "    # foo
"
   ```

✅ **cm_example_70**: `text`
   Input: `foo
    # bar
`
   Parse Tree:
   ```
  └── text: "foo
    # bar
"
   ```

✅ **cm_example_71**: `text`
   Input: `## foo ##
  ###   bar    ###
`
   Parse Tree:
   ```
  └── text: "## foo ##
  ###   bar    ###
"
   ```

✅ **cm_example_72**: `text`
   Input: `# foo ##################################
##### foo ##
`
   Parse Tree:
   ```
  └── text: "# foo ##################################
##### foo ##
"
   ```

✅ **cm_example_73**: `text`
   Input: `### foo ###     
`
   Parse Tree:
   ```
  └── text: "### foo ###     
"
   ```

✅ **cm_example_74**: `text`
   Input: `### foo ### b
`
   Parse Tree:
   ```
  └── text: "### foo ### b
"
   ```

✅ **cm_example_75**: `text`
   Input: `# foo#
`
   Parse Tree:
   ```
  └── text: "# foo#
"
   ```

✅ **cm_example_76**: `text`
   Input: `### foo \\###
## foo #\\##
# foo \\#
`
   Parse Tree:
   ```
  └── text: "### foo "
   ```

✅ **cm_example_77**: `text`
   Input: `\*\*\*\*
## foo
\*\*\*\*
`
   Parse Tree:
   ```
  └── text: "****
## foo
****
"
   ```

✅ **cm_example_78**: `text`
   Input: `Foo bar
# baz
Bar foo
`
   Parse Tree:
   ```
  └── text: "Foo bar
# baz
Bar foo
"
   ```

✅ **cm_example_79**: `text`
   Input: `## 
#
### ###
`
   Parse Tree:
   ```
  └── text: "## 
#
### ###
"
   ```

## commonmark_html_blocks

✅ **cm_example_148**: `text`
   Input: `<table><tr><td>
<pre>
\*\*Hello\*\*,

\_world\_.
</pre>
</td></tr></table>
`
   Parse Tree:
   ```
  └── text: "<table><tr><td>
<pre>
**Hello**,

_world_.
</pre>
</td></tr></table>
"
   ```

✅ **cm_example_149**: `text`
   Input: `<table>
  <tr>
    <td>
           hi
    </td>
  </tr>
</table>

okay.
`
   Parse Tree:
   ```
  └── text: "<table>
  <tr>
    <td>
           hi
    </td>
  </tr>
</table>

okay.
"
   ```

✅ **cm_example_150**: `text`
   Input: ` <div>
  \*hello\*
         <foo><a>
`
   Parse Tree:
   ```
  └── text: " <div>
  *hello*
         <foo><a>
"
   ```

✅ **cm_example_151**: `text`
   Input: `</div>
\*foo\*
`
   Parse Tree:
   ```
  └── text: "</div>
*foo*
"
   ```

✅ **cm_example_152**: `text`
   Input: `<DIV CLASS="foo">

\*Markdown\*

</DIV>
`
   Parse Tree:
   ```
  └── text: "<DIV CLASS="foo">

*Markdown*

</DIV>
"
   ```

✅ **cm_example_153**: `text`
   Input: `<div id="foo"
  class="bar">
</div>
`
   Parse Tree:
   ```
  └── text: "<div id="foo"
  class="bar">
</div>
"
   ```

✅ **cm_example_154**: `text`
   Input: `<div id="foo" class="bar
  baz">
</div>
`
   Parse Tree:
   ```
  └── text: "<div id="foo" class="bar
  baz">
</div>
"
   ```

✅ **cm_example_155**: `text`
   Input: `<div>
\*foo\*

\*bar\*
`
   Parse Tree:
   ```
  └── text: "<div>
*foo*

*bar*
"
   ```

✅ **cm_example_156**: `text`
   Input: `<div id="foo"
\*hi\*
`
   Parse Tree:
   ```
  └── text: "<div id="foo"
*hi*
"
   ```

✅ **cm_example_157**: `text`
   Input: `<div class
foo
`
   Parse Tree:
   ```
  └── text: "<div class
foo
"
   ```

✅ **cm_example_158**: `text`
   Input: `<div \*???-&&&-<---
\*foo\*
`
   Parse Tree:
   ```
  └── text: "<div *???-&&&-<---
*foo*
"
   ```

✅ **cm_example_159**: `text`
   Input: `<div><a href="bar">\*foo\*</a></div>
`
   Parse Tree:
   ```
  └── text: "<div><a href="bar">*foo*</a></div>
"
   ```

✅ **cm_example_160**: `text`
   Input: `<table><tr><td>
foo
</td></tr></table>
`
   Parse Tree:
   ```
  └── text: "<table><tr><td>
foo
</td></tr></table>
"
   ```

✅ **cm_example_161**: `text`
   Input: `<div></div>
\`\`\` c
int x = 33;
\`\`\`
`
   Parse Tree:
   ```
  └── text: "<div></div>
``` c
int x = 33;
```
"
   ```

✅ **cm_example_162**: `text`
   Input: `<a href="foo">
\*bar\*
</a>
`
   Parse Tree:
   ```
  └── text: "<a href="foo">
*bar*
</a>
"
   ```

✅ **cm_example_163**: `text`
   Input: `<Warning>
\*bar\*
</Warning>
`
   Parse Tree:
   ```
  └── text: "<Warning>
*bar*
</Warning>
"
   ```

✅ **cm_example_164**: `text`
   Input: `<i class="foo">
\*bar\*
</i>
`
   Parse Tree:
   ```
  └── text: "<i class="foo">
*bar*
</i>
"
   ```

✅ **cm_example_165**: `text`
   Input: `</ins>
\*bar\*
`
   Parse Tree:
   ```
  └── text: "</ins>
*bar*
"
   ```

✅ **cm_example_166**: `text`
   Input: `<del>
\*foo\*
</del>
`
   Parse Tree:
   ```
  └── text: "<del>
*foo*
</del>
"
   ```

✅ **cm_example_167**: `text`
   Input: `<del>

\*foo\*

</del>
`
   Parse Tree:
   ```
  └── text: "<del>

*foo*

</del>
"
   ```

✅ **cm_example_168**: `text`
   Input: `<del>\*foo\*</del>
`
   Parse Tree:
   ```
  └── text: "<del>*foo*</del>
"
   ```

✅ **cm_example_169**: `text`
   Input: `<pre language="haskell"><code>
import Text.HTML.TagSoup

main :: IO ()
main = print $ parseTags tags
</code></pre>
okay
`
   Parse Tree:
   ```
  └── text: "<pre language="haskell"><code>
import Text.HTML.TagSoup

main :: IO ()
main = print $ parseTags tags
</code></pre>
okay
"
   ```

✅ **cm_example_170**: `text`
   Input: `<script type="text/javascript">
// JavaScript example

document.getElementById("demo").innerHTML = "Hello JavaScript!";
</script>
okay
`
   Parse Tree:
   ```
  └── text: "<script type="text/javascript">
// JavaScript example

document.getElementById("demo").innerHTML = "Hello JavaScript!";
</script>
okay
"
   ```

✅ **cm_example_171**: `text`
   Input: `<textarea>

\*foo\*

\_bar\_

</textarea>
`
   Parse Tree:
   ```
  └── text: "<textarea>

*foo*

_bar_

</textarea>
"
   ```

✅ **cm_example_172**: `text`
   Input: `<style
  type="text/css">
h1 {color:red;}

p {color:blue;}
</style>
okay
`
   Parse Tree:
   ```
  └── text: "<style
  type="text/css">
h1 {color:red;}

p {color:blue;}
</style>
okay
"
   ```

✅ **cm_example_173**: `text`
   Input: `<style
  type="text/css">

foo
`
   Parse Tree:
   ```
  └── text: "<style
  type="text/css">

foo
"
   ```

✅ **cm_example_174**: `text`
   Input: `> <div>
> foo

bar
`
   Parse Tree:
   ```
  └── text: "> <div>
> foo

bar
"
   ```

✅ **cm_example_175**: `text`
   Input: `- <div>
- foo
`
   Parse Tree:
   ```
  └── text: "- <div>
- foo
"
   ```

✅ **cm_example_176**: `text`
   Input: `<style>p{color:red;}</style>
\*foo\*
`
   Parse Tree:
   ```
  └── text: "<style>p{color:red;}</style>
*foo*
"
   ```

✅ **cm_example_177**: `text`
   Input: `<!-- foo -->\*bar\*
\*baz\*
`
   Parse Tree:
   ```
  └── text: "<!-- foo -->*bar*
*baz*
"
   ```

✅ **cm_example_178**: `text`
   Input: `<script>
foo
</script>1. \*bar\*
`
   Parse Tree:
   ```
  └── text: "<script>
foo
</script>1. *bar*
"
   ```

✅ **cm_example_179**: `text`
   Input: `<!-- Foo

bar
   baz -->
okay
`
   Parse Tree:
   ```
  └── text: "<!-- Foo

bar
   baz -->
okay
"
   ```

✅ **cm_example_180**: `text`
   Input: `<?php

  echo '>';

?>
okay
`
   Parse Tree:
   ```
  └── text: "<?php

  echo '>';

?>
okay
"
   ```

✅ **cm_example_181**: `text`
   Input: `<!DOCTYPE html>
`
   Parse Tree:
   ```
  └── text: "<!DOCTYPE html>
"
   ```

✅ **cm_example_182**: `text`
   Input: `<!\[CDATA\[
function matchwo(a,b)
{
  if (a < b && a < 0) then {
    return 1;

  } else {

    return 0;
  }
}
\]\]>
okay
`
   Parse Tree:
   ```
  └── text: "<![CDATA[
function matchwo(a,b)
{
  if (a < b && a < 0) then {
    return 1;

  } else {

    return 0;
  }
}
]]>
okay
"
   ```

✅ **cm_example_183**: `text`
   Input: `  <!-- foo -->

    <!-- foo -->
`
   Parse Tree:
   ```
  └── text: "  <!-- foo -->

    <!-- foo -->
"
   ```

✅ **cm_example_184**: `text`
   Input: `  <div>

    <div>
`
   Parse Tree:
   ```
  └── text: "  <div>

    <div>
"
   ```

✅ **cm_example_185**: `text`
   Input: `Foo
<div>
bar
</div>
`
   Parse Tree:
   ```
  └── text: "Foo
<div>
bar
</div>
"
   ```

✅ **cm_example_186**: `text`
   Input: `<div>
bar
</div>
\*foo\*
`
   Parse Tree:
   ```
  └── text: "<div>
bar
</div>
*foo*
"
   ```

✅ **cm_example_187**: `text`
   Input: `Foo
<a href="bar">
baz
`
   Parse Tree:
   ```
  └── text: "Foo
<a href="bar">
baz
"
   ```

✅ **cm_example_188**: `text`
   Input: `<div>

\*Emphasized\* text.

</div>
`
   Parse Tree:
   ```
  └── text: "<div>

*Emphasized* text.

</div>
"
   ```

✅ **cm_example_189**: `text`
   Input: `<div>
\*Emphasized\* text.
</div>
`
   Parse Tree:
   ```
  └── text: "<div>
*Emphasized* text.
</div>
"
   ```

✅ **cm_example_190**: `text`
   Input: `<table>

<tr>

<td>
Hi
</td>

</tr>

</table>
`
   Parse Tree:
   ```
  └── text: "<table>

<tr>

<td>
Hi
</td>

</tr>

</table>
"
   ```

✅ **cm_example_191**: `text`
   Input: `<table>

  <tr>

    <td>
      Hi
    </td>

  </tr>

</table>
`
   Parse Tree:
   ```
  └── text: "<table>

  <tr>

    <td>
      Hi
    </td>

  </tr>

</table>
"
   ```

## italic_formatting

✅ **italic_asterisk**: `italic`
   Input: `\*italic text\*`
   Parse Tree:
   ```
  ├── italic > "*italic text*"
    └── italic_asterisk: "*italic text*"
   ```

✅ **italic_asterisk_empty**: `italic` (Expected failure)
   Input: `\*\*`
   Error: ` --> 1:1
  |
1 | **
  | ^---
  |
  = expected italic`

✅ **italic_asterisk_nested**: `italic`
   Input: `\*italic with \*inner\* italic\*`
   Parse Tree:
   ```
  ├── italic > "*italic with *"
    └── italic_asterisk: "*italic with *"
   ```

✅ **italic_underscore**: `italic`
   Input: `\_italic text\_`
   Parse Tree:
   ```
  ├── italic > "_italic text_"
    └── italic_underscore: "_italic text_"
   ```

✅ **italic_underscore_empty**: `italic` (Expected failure)
   Input: `\_\_`
   Error: ` --> 1:1
  |
1 | __
  | ^---
  |
  = expected italic`

❌ **italic_in_word**: `italic` (Unexpected failure)
   Input: `un\*believable\*ly`
   Error: ` --> 1:1
  |
1 | un*believable*ly
  | ^---
  |
  = expected italic`

✅ **italic_with_punctuation**: `italic`
   Input: `\*hello, world!\*`
   Parse Tree:
   ```
  ├── italic > "*hello, world!*"
    └── italic_asterisk: "*hello, world!*"
   ```

❌ **italic_double_asterisk**: `italic` (Unexpected failure)
   Input: `\*\*not italic\*\*`
   Error: ` --> 1:1
  |
1 | **not italic**
  | ^---
  |
  = expected italic`

✅ **italic_unclosed**: `italic`
   Input: `\*missing closing`
   Parse Tree:
   ```
  ├── italic > "*missing closing"
    └── italic_asterisk: "*missing closing"
   ```

## commonmark_raw_html

✅ **cm_example_612**: `text`
   Input: `<a><bab><c2c>
`
   Parse Tree:
   ```
  └── text: "<a><bab><c2c>
"
   ```

✅ **cm_example_613**: `text`
   Input: `<a/><b2/>
`
   Parse Tree:
   ```
  └── text: "<a/><b2/>
"
   ```

✅ **cm_example_614**: `text`
   Input: `<a  /><b2
data="foo" >
`
   Parse Tree:
   ```
  └── text: "<a  /><b2
data="foo" >
"
   ```

✅ **cm_example_615**: `text`
   Input: `<a foo="bar" bam = 'baz <em>"</em>'
\_boolean zoop:33=zoop:33 />
`
   Parse Tree:
   ```
  └── text: "<a foo="bar" bam = 'baz <em>"</em>'
_boolean zoop:33=zoop:33 />
"
   ```

✅ **cm_example_616**: `text`
   Input: `Foo <responsive-image src="foo.jpg" />
`
   Parse Tree:
   ```
  └── text: "Foo <responsive-image src="foo.jpg" />
"
   ```

✅ **cm_example_617**: `text`
   Input: `<33> <\_\_>
`
   Parse Tree:
   ```
  └── text: "<33> <__>
"
   ```

✅ **cm_example_618**: `text`
   Input: `<a h\*#ref="hi">
`
   Parse Tree:
   ```
  └── text: "<a h*#ref="hi">
"
   ```

✅ **cm_example_619**: `text`
   Input: `<a href="hi'> <a href=hi'>
`
   Parse Tree:
   ```
  └── text: "<a href="hi'> <a href=hi'>
"
   ```

✅ **cm_example_620**: `text`
   Input: `< a><
foo><bar/ >
<foo bar=baz
bim!bop />
`
   Parse Tree:
   ```
  └── text: "< a><
foo><bar/ >
<foo bar=baz
bim!bop />
"
   ```

✅ **cm_example_621**: `text`
   Input: `<a href='bar'title=title>
`
   Parse Tree:
   ```
  └── text: "<a href='bar'title=title>
"
   ```

✅ **cm_example_622**: `text`
   Input: `</a></foo >
`
   Parse Tree:
   ```
  └── text: "</a></foo >
"
   ```

✅ **cm_example_623**: `text`
   Input: `</a href="foo">
`
   Parse Tree:
   ```
  └── text: "</a href="foo">
"
   ```

✅ **cm_example_624**: `text`
   Input: `foo <!-- this is a
comment - with hyphen -->
`
   Parse Tree:
   ```
  └── text: "foo <!-- this is a
comment - with hyphen -->
"
   ```

✅ **cm_example_625**: `text`
   Input: `foo <!-- not a comment -- two hyphens -->
`
   Parse Tree:
   ```
  └── text: "foo <!-- not a comment -- two hyphens -->
"
   ```

✅ **cm_example_626**: `text`
   Input: `foo <!--> foo -->

foo <!-- foo--->
`
   Parse Tree:
   ```
  └── text: "foo <!--> foo -->

foo <!-- foo--->
"
   ```

✅ **cm_example_627**: `text`
   Input: `foo <?php echo $a; ?>
`
   Parse Tree:
   ```
  └── text: "foo <?php echo $a; ?>
"
   ```

✅ **cm_example_628**: `text`
   Input: `foo <!ELEMENT br EMPTY>
`
   Parse Tree:
   ```
  └── text: "foo <!ELEMENT br EMPTY>
"
   ```

✅ **cm_example_629**: `text`
   Input: `foo <!\[CDATA\[>&<\]\]>
`
   Parse Tree:
   ```
  └── text: "foo <![CDATA[>&<]]>
"
   ```

✅ **cm_example_630**: `text`
   Input: `foo <a href="&ouml;">
`
   Parse Tree:
   ```
  └── text: "foo <a href="&ouml;">
"
   ```

✅ **cm_example_631**: `text`
   Input: `foo <a href="\\\*">
`
   Parse Tree:
   ```
  └── text: "foo <a href=""
   ```

✅ **cm_example_632**: `text`
   Input: `<a href="\\"">
`
   Parse Tree:
   ```
  └── text: "<a href=""
   ```

## edge_cases

✅ **only_whitespace**: `text`
   Input: `   	   `
   Parse Tree:
   ```
  └── text: "   	   "
   ```

✅ **mixed_line_endings**: `text`
   Input: `text\r
more text
final text`
   Parse Tree:
   ```
  └── text: "text"
   ```

✅ **trailing_spaces**: `text`
   Input: `text   `
   Parse Tree:
   ```
  └── text: "text   "
   ```

✅ **leading_spaces**: `text`
   Input: `   text`
   Parse Tree:
   ```
  └── text: "   text"
   ```

✅ **emoji_unicode**: `text`
   Input: `😀 😃 😄 😁 😆`
   Parse Tree:
   ```
  └── text: "😀 😃 😄 😁 😆"
   ```

✅ **zero_width_chars**: `text`
   Input: `text\u200Bwith\u200Cinvisible\u200Dchars`
   Parse Tree:
   ```
  └── text: "text"
   ```

✅ **rtl_text**: `text`
   Input: `العربية من اليمين`
   Parse Tree:
   ```
  └── text: "العربية من اليمين"
   ```

✅ **combining_chars**: `text`
   Input: `café (é = e + ́)`
   Parse Tree:
   ```
  └── text: "café (é = e + ́)"
   ```

✅ **very_long_line**: `text`
   Input: `Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.`
   Parse Tree:
   ```
  └── text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
   ```

✅ **deeply_nested**: `text`
   Input: `\*\*bold with \*italic and \`code\` inside\* text\*\*`
   Parse Tree:
   ```
  └── text: "**bold with *italic and `code` inside* text**"
   ```

✅ **mixed_formatting**: `text`
   Input: `\*\*bold\*\* and \*italic\* and \`code\` and ~~strike~~`
   Parse Tree:
   ```
  └── text: "**bold** and *italic* and `code` and "
   ```

✅ **single_char**: `text`
   Input: `a`
   Parse Tree:
   ```
  └── text: "a"
   ```

✅ **two_chars**: `text`
   Input: `ab`
   Parse Tree:
   ```
  └── text: "ab"
   ```

✅ **all_digits**: `text`
   Input: `123456789`
   Parse Tree:
   ```
  └── text: "123456789"
   ```

✅ **all_punctuation**: `text`
   Input: `.,;:!?()\[\]{}"'`
   Parse Tree:
   ```
  └── text: ".,;:!?()[]{}"'"
   ```

## bold_italic_combinations

✅ **bold_italic_triple_ast**: `bold_italic`
   Input: `\*\*\*bold and italic\*\*\*`
   Parse Tree:
   ```
  ├── bold_italic > "***bold and italic***"
    └── bold_italic_triple_asterisk: "***bold and italic***"
   ```

✅ **bold_italic_triple_under**: `bold_italic`
   Input: `\_\_\_bold and italic\_\_\_`
   Parse Tree:
   ```
  ├── bold_italic > "___bold and italic___"
    └── bold_italic_triple_underscore: "___bold and italic___"
   ```

✅ **bold_italic_mixed_1**: `bold_italic`
   Input: `\*\*\_bold and italic\_\*\*`
   Parse Tree:
   ```
  ├── bold_italic > "**_bold and italic_**"
    └── bold_italic_mixed_ast_under: "**_bold and italic_**"
   ```

✅ **bold_italic_mixed_2**: `bold_italic`
   Input: `\_\_\*bold and italic\*\_\_`
   Parse Tree:
   ```
  ├── bold_italic > "__*bold and italic*__"
    └── bold_italic_mixed_under_ast: "__*bold and italic*__"
   ```

❌ **bold_italic_mismatch**: `bold_italic` (Unexpected failure)
   Input: `\*\*\*bold italic with underscore\_\_\_`
   Error: ` --> 1:1
  |
1 | ***bold italic with underscore___
  | ^---
  |
  = expected bold_italic`

## commonmark_setext_headings

✅ **cm_example_80**: `text`
   Input: `Foo \*bar\*
=========

Foo \*bar\*
---------
`
   Parse Tree:
   ```
  └── text: "Foo *bar*
=========

Foo *bar*
---------
"
   ```

✅ **cm_example_81**: `text`
   Input: `Foo \*bar
baz\*
====
`
   Parse Tree:
   ```
  └── text: "Foo *bar
baz*
====
"
   ```

✅ **cm_example_82**: `text`
   Input: `  Foo \*bar
baz\*	
====
`
   Parse Tree:
   ```
  └── text: "  Foo *bar
baz*	
====
"
   ```

✅ **cm_example_83**: `text`
   Input: `Foo
-------------------------

Foo
=
`
   Parse Tree:
   ```
  └── text: "Foo
-------------------------

Foo
=
"
   ```

✅ **cm_example_84**: `text`
   Input: `   Foo
---

  Foo
-----

  Foo
  ===
`
   Parse Tree:
   ```
  └── text: "   Foo
---

  Foo
-----

  Foo
  ===
"
   ```

✅ **cm_example_85**: `text`
   Input: `    Foo
    ---

    Foo
---
`
   Parse Tree:
   ```
  └── text: "    Foo
    ---

    Foo
---
"
   ```

✅ **cm_example_86**: `text`
   Input: `Foo
   ----      
`
   Parse Tree:
   ```
  └── text: "Foo
   ----      
"
   ```

✅ **cm_example_87**: `text`
   Input: `Foo
    ---
`
   Parse Tree:
   ```
  └── text: "Foo
    ---
"
   ```

✅ **cm_example_88**: `text`
   Input: `Foo
= =

Foo
--- -
`
   Parse Tree:
   ```
  └── text: "Foo
= =

Foo
--- -
"
   ```

✅ **cm_example_89**: `text`
   Input: `Foo  
-----
`
   Parse Tree:
   ```
  └── text: "Foo  
-----
"
   ```

✅ **cm_example_90**: `text`
   Input: `Foo\\
----
`
   Parse Tree:
   ```
  └── text: "Foo"
   ```

✅ **cm_example_91**: `text`
   Input: `\`Foo
----
\`

<a title="a lot
---
of dashes"/>
`
   Parse Tree:
   ```
  └── text: "`Foo
----
`

<a title="a lot
---
of dashes"/>
"
   ```

✅ **cm_example_92**: `text`
   Input: `> Foo
---
`
   Parse Tree:
   ```
  └── text: "> Foo
---
"
   ```

✅ **cm_example_93**: `text`
   Input: `> foo
bar
===
`
   Parse Tree:
   ```
  └── text: "> foo
bar
===
"
   ```

✅ **cm_example_94**: `text`
   Input: `- Foo
---
`
   Parse Tree:
   ```
  └── text: "- Foo
---
"
   ```

✅ **cm_example_95**: `text`
   Input: `Foo
Bar
---
`
   Parse Tree:
   ```
  └── text: "Foo
Bar
---
"
   ```

✅ **cm_example_96**: `text`
   Input: `---
Foo
---
Bar
---
Baz
`
   Parse Tree:
   ```
  └── text: "---
Foo
---
Bar
---
Baz
"
   ```

✅ **cm_example_97**: `text`
   Input: `
====
`
   Parse Tree:
   ```
  └── text: "
====
"
   ```

✅ **cm_example_98**: `text`
   Input: `---
---
`
   Parse Tree:
   ```
  └── text: "---
---
"
   ```

✅ **cm_example_99**: `text`
   Input: `- foo
-----
`
   Parse Tree:
   ```
  └── text: "- foo
-----
"
   ```

✅ **cm_example_100**: `text`
   Input: `    foo
---
`
   Parse Tree:
   ```
  └── text: "    foo
---
"
   ```

✅ **cm_example_101**: `text`
   Input: `> foo
-----
`
   Parse Tree:
   ```
  └── text: "> foo
-----
"
   ```

❌ **cm_example_102**: `text` (Unexpected failure)
   Input: `\\> foo
------
`
   Error: ` --> 1:1
  |
1 | \\> foo
  | ^---
  |
  = expected text`

✅ **cm_example_103**: `text`
   Input: `Foo

bar
---
baz
`
   Parse Tree:
   ```
  └── text: "Foo

bar
---
baz
"
   ```

✅ **cm_example_104**: `text`
   Input: `Foo
bar

---

baz
`
   Parse Tree:
   ```
  └── text: "Foo
bar

---

baz
"
   ```

✅ **cm_example_105**: `text`
   Input: `Foo
bar
\* \* \*
baz
`
   Parse Tree:
   ```
  └── text: "Foo
bar
* * *
baz
"
   ```

✅ **cm_example_106**: `text`
   Input: `Foo
bar
\\---
baz
`
   Parse Tree:
   ```
  └── text: "Foo
bar
"
   ```

## commonmark_blank_lines

✅ **cm_example_227**: `text`
   Input: `  

aaa
  

# aaa

  
`
   Parse Tree:
   ```
  └── text: "  "
   ```

## run_commands

✅ **run_bash**: `run_inline`
   Input: `run@bash(ls -la)`
   Parse Tree:
   ```
  ├── run_inline > "run@bash(ls -la)"
    └── KW_RUN: "run@"
    ├── script_type > "bash"
      └── KW_BASH: "bash"
   ```

✅ **run_python**: `run_inline`
   Input: `run@python(print('hello'))`
   Parse Tree:
   ```
  ├── run_inline > "run@python(print('hello')"
    └── KW_RUN: "run@"
    ├── script_type > "python"
      └── KW_PYTHON: "python"
   ```

✅ **run_zsh**: `run_inline`
   Input: `run@zsh(echo $HOME)`
   Parse Tree:
   ```
  ├── run_inline > "run@zsh(echo $HOME)"
    └── KW_RUN: "run@"
    ├── script_type > "zsh"
      └── KW_ZSH: "zsh"
   ```

✅ **run_powershell**: `run_inline`
   Input: `run@powershell(Get-Location)`
   Parse Tree:
   ```
  ├── run_inline > "run@powershell(Get-Location)"
    └── KW_RUN: "run@"
    ├── script_type > "powershell"
      └── KW_POWERSHELL: "powershell"
   ```

✅ **run_bat**: `run_inline`
   Input: `run@bat(dir)`
   Parse Tree:
   ```
  ├── run_inline > "run@bat(dir)"
    └── KW_RUN: "run@"
    ├── script_type > "bat"
      └── KW_BAT: "bat"
   ```

✅ **run_escaped**: `run_inline`
   Input: `run@bash(echo "hello world")`
   Parse Tree:
   ```
  ├── run_inline > "run@bash(echo "hello world")"
    └── KW_RUN: "run@"
    ├── script_type > "bash"
      └── KW_BASH: "bash"
   ```

✅ **run_complex**: `run_inline`
   Input: `run@python(import os; print(os.getcwd()))`
   Parse Tree:
   ```
  ├── run_inline > "run@python(import os; print(os.getcwd()"
    └── KW_RUN: "run@"
    ├── script_type > "python"
      └── KW_PYTHON: "python"
   ```

✅ **run_block_bash**: `run_block_fenced`
   Input: `\`\`\`run@bash
ls -la
echo "done"
\`\`\``
   Parse Tree:
   ```
  ├── run_block_fenced > "```run@bash
ls -la
echo "done"
```"
    └── KW_RUN: "run@"
    ├── script_type > "bash"
      └── KW_BASH: "bash"
   ```

✅ **run_block_python**: `run_block_fenced`
   Input: `\`\`\`run@python
print('hello')
for i in range(3):
    print(i)
\`\`\``
   Parse Tree:
   ```
  ├── run_block_fenced > "```run@python
print('hello')
for i in range(3):
    print(i)
```"
    └── KW_RUN: "run@"
    ├── script_type > "python"
      └── KW_PYTHON: "python"
   ```

✅ **run_upper_bash**: `run_inline`
   Input: `run@BASH(echo test)`
   Parse Tree:
   ```
  ├── run_inline > "run@BASH(echo test)"
    └── KW_RUN: "run@"
    ├── script_type > "BASH"
      └── KW_BASH: "BASH"
   ```

✅ **run_mixed_python**: `run_inline`
   Input: `run@Python(print('test'))`
   Parse Tree:
   ```
  ├── run_inline > "run@Python(print('test')"
    └── KW_RUN: "run@"
    ├── script_type > "Python"
      └── KW_PYTHON: "Python"
   ```

## tab

✅ **tab_simple**: `tab_block`
   Input: `:::tab
General content
@tab Tab 1
Content 1
@tab Tab 2
Content 2
:::`
   Parse Tree:
   ```
  ├── tab_block > ":::tab
General content
@tab Tab 1
Content 1
@tab Tab 2
Content 2
:::"
    ├── tab_header > ":::tab"
      └── KW_TAB: "tab"
    ├── tabs_content_I > "General content
@tab Tab 1
Content 1
@tab Tab 2
Content 2
"
      └── tab_content_line: "General content
"
      └── tab_content_line: "@tab Tab 1
"
      └── tab_content_line: "Content 1
"
      └── tab_content_line: "@tab Tab 2
"
      └── tab_content_line: "Content 2
"
    └── tab_end: ":::"
   ```

❌ **tab_with_title**: `tab_block` (Unexpected failure)
   Input: `:::tab Main Tab
@tab First
First content
@tab Second
Second content
:::`
   Error: ` --> 1:8
  |
1 | :::tab Main Tab
  |        ^---
  |
  = expected tab_title`

✅ **tab_formatted**: `tab_block`
   Input: `:::tab
@tab \*\*Bold Tab\*\*
Content with \*\*formatting\*\*
@tab \*Italic Tab\*
More content
:::`
   Parse Tree:
   ```
  ├── tab_block > ":::tab
@tab **Bold Tab**
Content with **formatting**
@tab *Italic Tab*
More content
:::"
    ├── tab_header > ":::tab"
      └── KW_TAB: "tab"
    ├── tabs_content_I > "@tab **Bold Tab**
Content with **formatting**
@tab *Italic Tab*
More content
"
      └── tab_content_line: "@tab **Bold Tab**
"
      └── tab_content_line: "Content with **formatting**
"
      └── tab_content_line: "@tab *Italic Tab*
"
      └── tab_content_line: "More content
"
    └── tab_end: ":::"
   ```

✅ **tab_empty_content**: `tab_block`
   Input: `:::tab
@tab Empty
@tab Also Empty
:::`
   Parse Tree:
   ```
  ├── tab_block > ":::tab
@tab Empty
@tab Also Empty
:::"
    ├── tab_header > ":::tab"
      └── KW_TAB: "tab"
    ├── tabs_content_I > "@tab Empty
@tab Also Empty
"
      └── tab_content_line: "@tab Empty
"
      └── tab_content_line: "@tab Also Empty
"
    └── tab_end: ":::"
   ```

✅ **tab_no_general**: `tab_block`
   Input: `:::tab
@tab Only Tab
Only content
:::`
   Parse Tree:
   ```
  ├── tab_block > ":::tab
@tab Only Tab
Only content
:::"
    ├── tab_header > ":::tab"
      └── KW_TAB: "tab"
    ├── tabs_content_I > "@tab Only Tab
Only content
"
      └── tab_content_line: "@tab Only Tab
"
      └── tab_content_line: "Only content
"
    └── tab_end: ":::"
   ```

## regression_tests

✅ **bug_emphasis_underscore**: `emphasis`
   Input: `\_emphasis\_with\_underscores\_inside\_`
   Parse Tree:
   ```
  ├── emphasis > "_emphasis_"
    ├── italic > "_emphasis_"
      └── italic_underscore: "_emphasis_"
   ```

✅ **bug_link_in_emphasis**: `emphasis`
   Input: `\*\[link\](url) in emphasis\*`
   Parse Tree:
   ```
  ├── emphasis > "*[link](url) in emphasis*"
    ├── italic > "*[link](url) in emphasis*"
      └── italic_asterisk: "*[link](url) in emphasis*"
   ```

✅ **bug_code_in_link**: `inline_link`
   Input: `\[\`code\` in link\](url)`
   Parse Tree:
   ```
  ├── inline_link > "[`code` in link](url)"
    └── bracket_link_without_title: "[`code` in link](url)"
   ```

✅ **bug_nested_quotes**: `blockquote`
   Input: `> > > Quote with \`code\` and \*emphasis\*`
   Parse Tree:
   ```
  ├── blockquote > "> > > Quote with `code` and *emphasis*"
    ├── blockquote_line > "> > > Quote with `code` and *emphasis*"
      ├── inline > "> > Quote with `code` and *emphasis*"
        ├── inline_core > "> > Quote with `code` and *emphasis*"
          └── text: "> > Quote with `code` and *emphasis*"
   ```

✅ **bug_table_alignment**: `table`
   Input: `| Left | Center | Right |
|:-----|:------:|------:|
| A | B | C |`
   Parse Tree:
   ```
  ├── table > "| Left | Center | Right |
|:-----|:------:|------:|
| A | B | C |"
    ├── table_header > "| Left | Center | Right |"
      ├── table_cell > "Left "
        ├── table_cell_content > "Left "
          └── table_safe_text: "Left "
      ├── table_cell > "Center "
        ├── table_cell_content > "Center "
          └── table_safe_text: "Center "
      ├── table_cell > "Right "
        ├── table_cell_content > "Right "
          └── table_safe_text: "Right "
      └── table_cell: ""
    ├── table_sep > "|:-----|:------:|------:|"
      └── table_sep_cell: ":-----"
      └── table_sep_cell: ":------:"
      └── table_sep_cell: "------:"
    ├── table_row > "| A | B | C |"
      ├── table_cell > "A "
        ├── table_cell_content > "A "
          └── table_safe_text: "A "
      ├── table_cell > "B "
        ├── table_cell_content > "B "
          └── table_safe_text: "B "
      ├── table_cell > "C "
        ├── table_cell_content > "C "
          └── table_safe_text: "C "
      └── table_cell: ""
   ```

✅ **bug_list_continuation_indent**: `list`
   Input: `1. First item

   Continued paragraph

2. Second item`
   Parse Tree:
   ```
  ├── list > "1. First item
"
    ├── list_item > "1. First item"
      ├── regular_list_item > "1. First item"
        └── list_marker: "1."
        └── list_item_content: "First item"
   ```

❌ **bug_setext_with_markup**: `setext_h2` (Unexpected failure)
   Input: `\*Emphasized\* heading
===================`
   Error: ` --> 1:21
  |
1 | *Emphasized* heading␊
  |                     ^---
  |
  = expected heading_inline`

✅ **bug_html_comment_multiline**: `inline_html`
   Input: `<!-- This is a
multiline comment
with \*\*markdown\*\* inside -->`
   Parse Tree:
   ```
  └── inline_html: "<!-- This is a
multiline comment
with **markdown** inside -->"
   ```

✅ **cm_link_title_quotes**: `inline_link`
   Input: `\[link\](url "title with 'quotes'")`
   Parse Tree:
   ```
  ├── inline_link > "[link](url "title with 'quotes'")"
    └── bracket_link_with_title: "[link](url "title with 'quotes'")"
   ```

✅ **cm_reference_case_insensitive**: `text`
   Input: `\[FOO\]\[bar\]
\[bar\]: /url`
   Parse Tree:
   ```
  └── text: "[FOO][bar]
[bar]: /url"
   ```

✅ **cm_autolink_scheme_case**: `inline_link`
   Input: `<HTTP://EXAMPLE.COM>`
   Parse Tree:
   ```
  ├── inline_link > "<HTTP://EXAMPLE.COM>"
    ├── autolink > "<HTTP://EXAMPLE.COM>"
      ├── autolink_url > "<HTTP://EXAMPLE.COM>"
        └── link_url: "HTTP://EXAMPLE.COM"
   ```

❌ **cm_entity_in_link**: `inline_link` (Unexpected failure)
   Input: `\[link\](url?param=value&amp;other=2)`
   Error: ` --> 1:1
  |
1 | [link](url?param=value&amp;other=2)
  | ^---
  |
  = expected inline_link`

## escaped_characters

✅ **escaped_asterisk**: `escaped_char`
   Input: `\\\*not bold\\\*`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_underscore**: `escaped_char`
   Input: `\\\_not italic\\\_`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_backtick**: `escaped_char`
   Input: `\\\`not code\\\``
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_hash**: `escaped_char`
   Input: `\\# not heading`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_bracket**: `escaped_char`
   Input: `\\\[not link\\\]`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_tilde**: `escaped_char`
   Input: `\\~not strikethrough\\~`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_greater**: `escaped_char`
   Input: `\\> not blockquote`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_pipe**: `escaped_char`
   Input: `\\| not table`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_dollar**: `escaped_char`
   Input: `\\$ not math`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_at**: `escaped_char`
   Input: `\\@ not mention`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_caret**: `escaped_char`
   Input: `\\^ not superscript`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_equals**: `escaped_char`
   Input: `\\= not highlight`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **escaped_dash**: `escaped_char`
   Input: `\\- not list`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

✅ **multiple_escapes**: `escaped_char`
   Input: `\\\*\\\*not bold\\\*\\\*`
   Parse Tree:
   ```
  └── escaped_char: "\\"
   ```

❌ **escaped_in_text**: `escaped_char` (Unexpected failure)
   Input: `This is \\\*not\\\* bold text`
   Error: ` --> 1:1
  |
1 | This is \\*not\\* bold text
  | ^---
  |
  = expected escaped_char`

## unicode_advanced

✅ **rtl_arabic**: `text`
   Input: `مرحبا بالعالم \*\*نص عريض\*\* \*نص مائل\*`
   Parse Tree:
   ```
  └── text: "مرحبا بالعالم **نص عريض** *نص مائل*"
   ```

✅ **rtl_hebrew**: `text`
   Input: `שלום עולם \*\*טקסט מודגש\*\* \*טקסט נטוי\*`
   Parse Tree:
   ```
  └── text: "שלום עולם **טקסט מודגש** *טקסט נטוי*"
   ```

✅ **mixed_direction**: `text`
   Input: `Hello מרحبا world بالعالم!`
   Parse Tree:
   ```
  └── text: "Hello מרحبا world بالعالم!"
   ```

✅ **emoji_sequences**: `text`
   Input: `👨‍👩‍👧‍👦 👍🏽 🇺🇸 🏳️‍🌈`
   Parse Tree:
   ```
  └── text: "👨‍👩‍👧‍👦 👍🏽 🇺🇸 🏳️‍🌈"
   ```

✅ **emoji_in_formatting**: `text`
   Input: `\*\*👍 bold emoji\*\* \*🎉 italic emoji\*`
   Parse Tree:
   ```
  └── text: "**👍 bold emoji** *🎉 italic emoji*"
   ```

✅ **emoji_in_links**: `text`
   Input: `\[🔗 emoji link\](https://example.com)`
   Parse Tree:
   ```
  └── text: "[🔗 emoji link](https://example.com)"
   ```

✅ **zero_width_joiner**: `text`
   Input: `text\u200Dwith\u200Dzwj`
   Parse Tree:
   ```
  └── text: "text"
   ```

✅ **zero_width_non_joiner**: `text`
   Input: `text\u200Cwith\u200Cznj`
   Parse Tree:
   ```
  └── text: "text"
   ```

✅ **zero_width_space**: `text`
   Input: `text\u200Bwith\u200Bzws`
   Parse Tree:
   ```
  └── text: "text"
   ```

✅ **combining_diacritics**: `text`
   Input: `e\u0301\u0302\u0303\u0304`
   Parse Tree:
   ```
  └── text: "e"
   ```

✅ **normalization_test**: `text`
   Input: `café vs cafe\u0301`
   Parse Tree:
   ```
  └── text: "café vs cafe"
   ```

✅ **astral_symbols**: `text`
   Input: `𝕳𝖊𝖑𝖑𝖔 𝖜𝖔𝖗𝖑𝖉`
   Parse Tree:
   ```
  └── text: "𝕳𝖊𝖑𝖑𝖔 𝖜𝖔𝖗𝖑𝖉"
   ```

✅ **musical_symbols**: `text`
   Input: `𝄞 𝄢 𝅘𝅥 𝅘𝅥𝅮`
   Parse Tree:
   ```
  └── text: "𝄞 𝄢 𝅘𝅥 𝅘𝅥𝅮"
   ```

## real_world_cases

✅ **github_issue**: `text`
   Input: `#123 @user fixes issue`
   Parse Tree:
   ```
  └── text: "#123 @user fixes issue"
   ```

✅ **github_mention**: `text`
   Input: `@octocat please review`
   Parse Tree:
   ```
  └── text: "@octocat please review"
   ```

✅ **github_commit**: `text`
   Input: `Fixed in commit abc123def456`
   Parse Tree:
   ```
  └── text: "Fixed in commit abc123def456"
   ```

✅ **citation_style**: `text`
   Input: `According to Smith et al. (2023)\[^smith2023\], this is correct.`
   Parse Tree:
   ```
  └── text: "According to Smith et al. (2023)[^smith2023], this is correct."
   ```

✅ **doi_link**: `text`
   Input: `https://doi.org/10.1000/182`
   Parse Tree:
   ```
  └── text: "https://doi.org/10.1000/182"
   ```

✅ **arxiv_link**: `text`
   Input: `https://arxiv.org/abs/2301.00001`
   Parse Tree:
   ```
  └── text: "https://arxiv.org/abs/2301.00001"
   ```

✅ **api_doc**: `text`
   Input: `\`GET /api/v1/users/{id}\` returns user data`
   Parse Tree:
   ```
  └── text: "`GET /api/v1/users/{id}` returns user data"
   ```

✅ **code_with_backticks**: `text`
   Input: `Use \`\\\`\` to escape backticks in code`
   Parse Tree:
   ```
  └── text: "Use `"
   ```

✅ **regex_example**: `text`
   Input: `Pattern: \`/^\[a-zA-Z0-9\]+$/g\``
   Parse Tree:
   ```
  └── text: "Pattern: `/^[a-zA-Z0-9]+$/g`"
   ```

✅ **code_switching**: `text`
   Input: `In Python, use \`print()\`, but in Rust use \`println!()\``
   Parse Tree:
   ```
  └── text: "In Python, use `print()`, but in Rust use `println!()`"
   ```

✅ **mixed_scripts_complex**: `text`
   Input: `English 中文 العربية русский 日本語 한국어 हिन्दी`
   Parse Tree:
   ```
  └── text: "English 中文 العربية русский 日本語 한국어 हिन्दी"
   ```

✅ **hashtag_like**: `text`
   Input: `This is #not-a-heading but markdown might confuse it`
   Parse Tree:
   ```
  └── text: "This is #not-a-heading but markdown might confuse it"
   ```

✅ **at_symbol_usage**: `text`
   Input: `Email: user@domain.com vs mention @user`
   Parse Tree:
   ```
  └── text: "Email: user@domain.com vs mention @user"
   ```

✅ **url_in_parentheses**: `text`
   Input: `See (https://example.com) for details`
   Parse Tree:
   ```
  └── text: "See (https://example.com) for details"
   ```

## headings_atx

✅ **h1_simple**: `H1`
   Input: `# Hello`
   Parse Tree:
   ```
  ├── H1 > "# Hello"
    ├── heading_content > "Hello"
      ├── heading_inline > "Hello"
        └── word: "Hello"
   ```

✅ **h1_no_space**: `H1`
   Input: `#NoSpace`
   Parse Tree:
   ```
  ├── H1 > "#NoSpace"
    ├── heading_content > "NoSpace"
      ├── heading_inline > "NoSpace"
        └── word: "NoSpace"
   ```

✅ **h1_multiple_spaces**: `H1`
   Input: `#   Multiple   Spaces   `
   Parse Tree:
   ```
  ├── H1 > "#   Multiple   Spaces"
    ├── heading_content > "Multiple   Spaces"
      ├── heading_inline > "Multiple"
        └── word: "Multiple"
      ├── heading_inline > "Spaces"
        └── word: "Spaces"
   ```

✅ **h1_with_formatting**: `H1`
   Input: `# \*\*Bold\*\* and \*italic\* heading`
   Parse Tree:
   ```
  ├── H1 > "# **Bold** and *italic* heading"
    ├── heading_content > "**Bold** and *italic* heading"
      ├── heading_inline > "**Bold**"
        ├── emphasis > "**Bold**"
          ├── bold > "**Bold**"
            └── bold_asterisk: "**Bold**"
      ├── heading_inline > "and"
        └── word: "and"
      ├── heading_inline > "*italic*"
        ├── emphasis > "*italic*"
          ├── italic > "*italic*"
            └── italic_asterisk: "*italic*"
      ├── heading_inline > "heading"
        └── word: "heading"
   ```

✅ **h1_unicode**: `H1`
   Input: `# Café & Résumé`
   Parse Tree:
   ```
  ├── H1 > "# Café & Résumé"
    ├── heading_content > "Café & Résumé"
      ├── heading_inline > "Café"
        └── word: "Café"
      ├── heading_inline > "&"
        └── safe_punct: "&"
      ├── heading_inline > "Résumé"
        └── word: "Résumé"
   ```

✅ **h1_numbers**: `H1`
   Input: `# Chapter 1: Introduction`
   Parse Tree:
   ```
  ├── H1 > "# Chapter 1: Introduction"
    ├── heading_content > "Chapter 1: Introduction"
      ├── heading_inline > "Chapter"
        └── word: "Chapter"
      ├── heading_inline > "1"
        └── word: "1"
      ├── heading_inline > ":"
        └── safe_punct: ":"
      ├── heading_inline > "Introduction"
        └── word: "Introduction"
   ```

✅ **h2_simple**: `H2`
   Input: `## Section`
   Parse Tree:
   ```
  ├── H2 > "## Section"
    ├── heading_content > "Section"
      ├── heading_inline > "Section"
        └── word: "Section"
   ```

✅ **h2_empty**: `H2` (Expected failure)
   Input: `##`
   Error: ` --> 1:3
  |
1 | ##
  |   ^---
  |
  = expected heading_inline`

❌ **h2_only_spaces**: `H2` (Unexpected failure)
   Input: `##   `
   Error: ` --> 1:6
  |
1 | ##   
  |      ^---
  |
  = expected heading_inline`

✅ **h2_long**: `H2`
   Input: `## This is a very long heading that should still parse correctly`
   Parse Tree:
   ```
  ├── H2 > "## This is a very long heading that should still parse correctly"
    ├── heading_content > "This is a very long heading that should still parse correctly"
      ├── heading_inline > "This"
        └── word: "This"
      ├── heading_inline > "is"
        └── word: "is"
      ├── heading_inline > "a"
        └── word: "a"
      ├── heading_inline > "very"
        └── word: "very"
      ├── heading_inline > "long"
        └── word: "long"
      ├── heading_inline > "heading"
        └── word: "heading"
      ├── heading_inline > "that"
        └── word: "that"
      ├── heading_inline > "should"
        └── word: "should"
      ├── heading_inline > "still"
        └── word: "still"
      ├── heading_inline > "parse"
        └── word: "parse"
      ├── heading_inline > "correctly"
        └── word: "correctly"
   ```

✅ **h3_simple**: `H3`
   Input: `### Subsection`
   Parse Tree:
   ```
  ├── H3 > "### Subsection"
    ├── heading_content > "Subsection"
      ├── heading_inline > "Subsection"
        └── word: "Subsection"
   ```

✅ **h4_simple**: `H4`
   Input: `#### Sub-subsection`
   Parse Tree:
   ```
  ├── H4 > "#### Sub-subsection"
    ├── heading_content > "Sub-subsection"
      ├── heading_inline > "Sub-subsection"
        └── word: "Sub-subsection"
   ```

✅ **h5_simple**: `H5`
   Input: `##### Deep Section`
   Parse Tree:
   ```
  ├── H5 > "##### Deep Section"
    ├── heading_content > "Deep Section"
      ├── heading_inline > "Deep"
        └── word: "Deep"
      ├── heading_inline > "Section"
        └── word: "Section"
   ```

✅ **h6_simple**: `H6`
   Input: `###### Deepest Section`
   Parse Tree:
   ```
  ├── H6 > "###### Deepest Section"
    ├── heading_content > "Deepest Section"
      ├── heading_inline > "Deepest"
        └── word: "Deepest"
      ├── heading_inline > "Section"
        └── word: "Section"
   ```

✅ **h7_invalid**: `heading` (Expected failure)
   Input: `####### Too Many Hashes`
   Error: ` --> 1:7
  |
1 | ####### Too Many Hashes
  |       ^---
  |
  = expected heading_inline`

✅ **h8_invalid**: `heading` (Expected failure)
   Input: `######## Even More Hashes`
   Error: ` --> 1:7
  |
1 | ######## Even More Hashes
  |       ^---
  |
  = expected heading_inline`

✅ **no_hash**: `heading` (Expected failure)
   Input: `Not a heading`
   Error: ` --> 1:14
  |
1 | Not a heading
  |              ^---
  |
  = expected heading_inline`

## other_formatting

✅ **strike_tilde**: `strikethrough`
   Input: `~~strikethrough~~`
   Parse Tree:
   ```
  ├── strikethrough > "~~strikethrough~~"
    └── strikethrough_tilde: "~~strikethrough~~"
   ```

✅ **strike_dash**: `strikethrough`
   Input: `--strikethrough--`
   Parse Tree:
   ```
  ├── strikethrough > "--strikethrough--"
    └── strikethrough_dash: "--strikethrough--"
   ```

✅ **strike_empty_tilde**: `strikethrough` (Expected failure)
   Input: `~~~~`
   Error: ` --> 1:1
  |
1 | ~~~~
  | ^---
  |
  = expected strikethrough`

✅ **strike_empty_dash**: `strikethrough` (Expected failure)
   Input: `----`
   Error: ` --> 1:1
  |
1 | ----
  | ^---
  |
  = expected strikethrough`

✅ **strike_nested**: `strikethrough`
   Input: `~~strike with ~~inner~~ strike~~`
   Parse Tree:
   ```
  ├── strikethrough > "~~strike with ~~"
    └── strikethrough_tilde: "~~strike with ~~"
   ```

✅ **highlight_simple**: `highlight`
   Input: `==highlighted text==`
   Parse Tree:
   ```
  └── highlight: "==highlighted text=="
   ```

✅ **highlight_empty**: `highlight` (Expected failure)
   Input: `====`
   Error: ` --> 1:1
  |
1 | ====
  | ^---
  |
  = expected highlight`

✅ **highlight_nested**: `highlight`
   Input: `==highlight with ==inner== highlight==`
   Parse Tree:
   ```
  └── highlight: "==highlight with =="
   ```

✅ **superscript_simple**: `superscript`
   Input: `^super^`
   Parse Tree:
   ```
  └── superscript: "^super^"
   ```

✅ **superscript_empty**: `superscript` (Expected failure)
   Input: `^^`
   Error: ` --> 1:1
  |
1 | ^^
  | ^---
  |
  = expected superscript`

❌ **superscript_math**: `superscript` (Unexpected failure)
   Input: `x^2^`
   Error: ` --> 1:1
  |
1 | x^2^
  | ^---
  |
  = expected superscript`

✅ **subscript_simple**: `subscript`
   Input: `˅sub˅`
   Parse Tree:
   ```
  ├── subscript > "˅sub˅"
    └── subscript_arrow: "˅sub˅"
   ```

✅ **subscript_empty**: `subscript` (Expected failure)
   Input: `˅˅`
   Error: ` --> 1:1
  |
1 | ˅˅
  | ^---
  |
  = expected subscript`

❌ **subscript_chemical**: `subscript` (Unexpected failure)
   Input: `H˅2˅O`
   Error: ` --> 1:1
  |
1 | H˅2˅O
  | ^---
  |
  = expected subscript`

## commonmark_entity_and_numeric_character_references

✅ **cm_example_25**: `text`
   Input: `&nbsp; &amp; &copy; &AElig; &Dcaron;
&frac34; &HilbertSpace; &DifferentialD;
&ClockwiseContourIntegral; &ngE;
`
   Parse Tree:
   ```
  └── text: "&nbsp; &amp; &copy; &AElig; &Dcaron;
&frac34; &HilbertSpace; &DifferentialD;
&ClockwiseContourIntegral; &ngE;
"
   ```

✅ **cm_example_26**: `text`
   Input: `&#35; &#1234; &#992; &#0;
`
   Parse Tree:
   ```
  └── text: "&#35; &#1234; &#992; &#0;
"
   ```

✅ **cm_example_27**: `text`
   Input: `&#X22; &#XD06; &#xcab;
`
   Parse Tree:
   ```
  └── text: "&#X22; &#XD06; &#xcab;
"
   ```

✅ **cm_example_28**: `text`
   Input: `&nbsp &x; &#; &#x;
&#87654321;
&#abcdef0;
&ThisIsNotDefined; &hi?;
`
   Parse Tree:
   ```
  └── text: "&nbsp &x; &#; &#x;
&#87654321;
&#abcdef0;
&ThisIsNotDefined; &hi?;
"
   ```

✅ **cm_example_29**: `text`
   Input: `&copy
`
   Parse Tree:
   ```
  └── text: "&copy
"
   ```

✅ **cm_example_30**: `text`
   Input: `&MadeUpEntity;
`
   Parse Tree:
   ```
  └── text: "&MadeUpEntity;
"
   ```

✅ **cm_example_31**: `text`
   Input: `<a href="&ouml;&ouml;.html">
`
   Parse Tree:
   ```
  └── text: "<a href="&ouml;&ouml;.html">
"
   ```

✅ **cm_example_32**: `text`
   Input: `\[foo\](/f&ouml;&ouml; "f&ouml;&ouml;")
`
   Parse Tree:
   ```
  └── text: "[foo](/f&ouml;&ouml; "f&ouml;&ouml;")
"
   ```

✅ **cm_example_33**: `text`
   Input: `\[foo\]

\[foo\]: /f&ouml;&ouml; "f&ouml;&ouml;"
`
   Parse Tree:
   ```
  └── text: "[foo]

[foo]: /f&ouml;&ouml; "f&ouml;&ouml;"
"
   ```

✅ **cm_example_34**: `text`
   Input: `\`\`\` f&ouml;&ouml;
foo
\`\`\`
`
   Parse Tree:
   ```
  └── text: "``` f&ouml;&ouml;
foo
```
"
   ```

✅ **cm_example_35**: `text`
   Input: `\`f&ouml;&ouml;\`
`
   Parse Tree:
   ```
  └── text: "`f&ouml;&ouml;`
"
   ```

✅ **cm_example_36**: `text`
   Input: `    f&ouml;f&ouml;
`
   Parse Tree:
   ```
  └── text: "    f&ouml;f&ouml;
"
   ```

✅ **cm_example_37**: `text`
   Input: `&#42;foo&#42;
\*foo\*
`
   Parse Tree:
   ```
  └── text: "&#42;foo&#42;
*foo*
"
   ```

✅ **cm_example_38**: `text`
   Input: `&#42; foo

\* foo
`
   Parse Tree:
   ```
  └── text: "&#42; foo

* foo
"
   ```

✅ **cm_example_39**: `text`
   Input: `foo&#10;&#10;bar
`
   Parse Tree:
   ```
  └── text: "foo&#10;&#10;bar
"
   ```

✅ **cm_example_40**: `text`
   Input: `&#9;foo
`
   Parse Tree:
   ```
  └── text: "&#9;foo
"
   ```

✅ **cm_example_41**: `text`
   Input: `\[a\](url &quot;tit&quot;)
`
   Parse Tree:
   ```
  └── text: "[a](url &quot;tit&quot;)
"
   ```

## commonmark_hard_line_breaks

✅ **cm_example_633**: `text`
   Input: `foo  
baz
`
   Parse Tree:
   ```
  └── text: "foo  
baz
"
   ```

✅ **cm_example_634**: `text`
   Input: `foo\\
baz
`
   Parse Tree:
   ```
  └── text: "foo"
   ```

✅ **cm_example_635**: `text`
   Input: `foo       
baz
`
   Parse Tree:
   ```
  └── text: "foo       
baz
"
   ```

✅ **cm_example_636**: `text`
   Input: `foo  
     bar
`
   Parse Tree:
   ```
  └── text: "foo  
     bar
"
   ```

✅ **cm_example_637**: `text`
   Input: `foo\\
     bar
`
   Parse Tree:
   ```
  └── text: "foo"
   ```

✅ **cm_example_638**: `text`
   Input: `\*foo  
bar\*
`
   Parse Tree:
   ```
  └── text: "*foo  
bar*
"
   ```

✅ **cm_example_639**: `text`
   Input: `\*foo\\
bar\*
`
   Parse Tree:
   ```
  └── text: "*foo"
   ```

✅ **cm_example_640**: `text`
   Input: `\`code  
span\`
`
   Parse Tree:
   ```
  └── text: "`code  
span`
"
   ```

✅ **cm_example_641**: `text`
   Input: `\`code\\
span\`
`
   Parse Tree:
   ```
  └── text: "`code"
   ```

✅ **cm_example_642**: `text`
   Input: `<a href="foo  
bar">
`
   Parse Tree:
   ```
  └── text: "<a href="foo  
bar">
"
   ```

✅ **cm_example_643**: `text`
   Input: `<a href="foo\\
bar">
`
   Parse Tree:
   ```
  └── text: "<a href="foo"
   ```

✅ **cm_example_644**: `text`
   Input: `foo\\
`
   Parse Tree:
   ```
  └── text: "foo"
   ```

✅ **cm_example_645**: `text`
   Input: `foo  
`
   Parse Tree:
   ```
  └── text: "foo  
"
   ```

✅ **cm_example_646**: `text`
   Input: `### foo\\
`
   Parse Tree:
   ```
  └── text: "### foo"
   ```

✅ **cm_example_647**: `text`
   Input: `### foo  
`
   Parse Tree:
   ```
  └── text: "### foo  
"
   ```

## memory_stress

✅ **huge_document_headings**: `text`
   Input: `# Heading 1
## Subheading 1
### Sub-sub 1
Content

# Heading 2
## Subheading 2
### Sub-sub 2
More content

# Heading 3
## Subheading 3
### Sub-sub 3
Even more content

# Heading 4
## Subheading 4
### Sub-sub 4
Final content`
   Parse Tree:
   ```
  └── text: "# Heading 1
## Subheading 1
### Sub-sub 1
Content

# Heading 2
## Subheading 2
### Sub-sub 2
More content

# Heading 3
## Subheading 3
### Sub-sub 3
Even more content

# Heading 4
## Subheading 4
### Sub-sub 4
Final content"
   ```

✅ **many_bold_words**: `text`
   Input: `\*\*word1\*\* \*\*word2\*\* \*\*word3\*\* \*\*word4\*\* \*\*word5\*\* \*\*word6\*\* \*\*word7\*\* \*\*word8\*\* \*\*word9\*\* \*\*word10\*\* \*\*word11\*\* \*\*word12\*\* \*\*word13\*\* \*\*word14\*\* \*\*word15\*\* \*\*word16\*\* \*\*word17\*\* \*\*word18\*\* \*\*word19\*\* \*\*word20\*\*`
   Parse Tree:
   ```
  └── text: "**word1** **word2** **word3** **word4** **word5** **word6** **word7** **word8** **word9** **word10** **word11** **word12** **word13** **word14** **word15** **word16** **word17** **word18** **word19** **word20**"
   ```

✅ **many_links**: `text`
   Input: `\[link1\](url1) \[link2\](url2) \[link3\](url3) \[link4\](url4) \[link5\](url5) \[link6\](url6) \[link7\](url7) \[link8\](url8) \[link9\](url9) \[link10\](url10)`
   Parse Tree:
   ```
  └── text: "[link1](url1) [link2](url2) [link3](url3) [link4](url4) [link5](url5) [link6](url6) [link7](url7) [link8](url8) [link9](url9) [link10](url10)"
   ```

❌ **many_footnotes_refs**: `footnote_ref` (Unexpected failure)
   Input: `Text\[^1\] more\[^2\] text\[^3\] here\[^4\] and\[^5\] there\[^6\] everywhere\[^7\] with\[^8\] many\[^9\] footnotes\[^10\] to\[^11\] test\[^12\] memory\[^13\] usage\[^14\] patterns\[^15\]`
   Error: ` --> 1:1
  |
1 | Text[^1] more[^2] text[^3] here[^4] and[^5] there[^6] everywhere[^7] with[^8] many[^9] footnotes[^10] to[^11] test[^12] memory[^13] usage[^14] patterns[^15]
  | ^---
  |
  = expected footnote_ref`

✅ **huge_nested_list**: `list`
   Input: `- Level 1 Item 1
  - Level 2 Item 1
    - Level 3 Item 1
      - Level 4 Item 1
        - Level 5 Item 1
          - Level 6 Item 1
            - Level 7 Item 1
              - Level 8 Item 1
                - Level 9 Item 1
                  - Level 10 Item 1
- Level 1 Item 2
  - Level 2 Item 2
    - Level 3 Item 2
      - Level 4 Item 2
        - Level 5 Item 2
- Level 1 Item 3
  - Level 2 Item 3
    - Level 3 Item 3`
   Parse Tree:
   ```
  ├── list > "- Level 1 Item 1
  - Level 2 Item 1
    - Level 3 Item 1
      - Level 4 Item 1
        - Level 5 Item 1
          - Level 6 Item 1
            - Level 7 Item 1
              - Level 8 Item 1
                - Level 9 Item 1
                  - Level 10 Item 1
- Level 1 Item 2
  - Level 2 Item 2
    - Level 3 Item 2
      - Level 4 Item 2
        - Level 5 Item 2
- Level 1 Item 3
  - Level 2 Item 3
    - Level 3 Item 3"
    ├── list_item > "- Level 1 Item 1"
      ├── regular_list_item > "- Level 1 Item 1"
        └── list_marker: "-"
        └── list_item_content: "Level 1 Item 1"
    ├── list_item > "- Level 2 Item 1"
      ├── regular_list_item > "- Level 2 Item 1"
        └── list_marker: "-"
        └── list_item_content: "Level 2 Item 1"
    ├── list_item > "- Level 3 Item 1"
      ├── regular_list_item > "- Level 3 Item 1"
        └── list_marker: "-"
        └── list_item_content: "Level 3 Item 1"
    ├── list_item > "- Level 4 Item 1"
      ├── regular_list_item > "- Level 4 Item 1"
        └── list_marker: "-"
        └── list_item_content: "Level 4 Item 1"
    ├── list_item > "- Level 5 Item 1"
      ├── regular_list_item > "- Level 5 Item 1"
        └── list_marker: "-"
        └── list_item_content: "Level 5 Item 1"
    ├── list_item > "- Level 6 Item 1"
      ├── regular_list_item > "- Level 6 Item 1"
        └── list_marker: "-"
        └── list_item_content: "Level 6 Item 1"
    ├── list_item > "- Level 7 Item 1"
      ├── regular_list_item > "- Level 7 Item 1"
        └── list_marker: "-"
        └── list_item_content: "Level 7 Item 1"
    ├── list_item > "- Level 8 Item 1"
      ├── regular_list_item > "- Level 8 Item 1"
        └── list_marker: "-"
        └── list_item_content: "Level 8 Item 1"
    ├── list_item > "- Level 9 Item 1"
      ├── regular_list_item > "- Level 9 Item 1"
        └── list_marker: "-"
        └── list_item_content: "Level 9 Item 1"
    ├── list_item > "- Level 10 Item 1"
      ├── regular_list_item > "- Level 10 Item 1"
        └── list_marker: "-"
        └── list_item_content: "Level 10 Item 1"
    ├── list_item > "- Level 1 Item 2"
      ├── regular_list_item > "- Level 1 Item 2"
        └── list_marker: "-"
        └── list_item_content: "Level 1 Item 2"
    ├── list_item > "- Level 2 Item 2"
      ├── regular_list_item > "- Level 2 Item 2"
        └── list_marker: "-"
        └── list_item_content: "Level 2 Item 2"
    ├── list_item > "- Level 3 Item 2"
      ├── regular_list_item > "- Level 3 Item 2"
        └── list_marker: "-"
        └── list_item_content: "Level 3 Item 2"
    ├── list_item > "- Level 4 Item 2"
      ├── regular_list_item > "- Level 4 Item 2"
        └── list_marker: "-"
        └── list_item_content: "Level 4 Item 2"
    ├── list_item > "- Level 5 Item 2"
      ├── regular_list_item > "- Level 5 Item 2"
        └── list_marker: "-"
        └── list_item_content: "Level 5 Item 2"
    ├── list_item > "- Level 1 Item 3"
      ├── regular_list_item > "- Level 1 Item 3"
        └── list_marker: "-"
        └── list_item_content: "Level 1 Item 3"
    ├── list_item > "- Level 2 Item 3"
      ├── regular_list_item > "- Level 2 Item 3"
        └── list_marker: "-"
        └── list_item_content: "Level 2 Item 3"
    ├── list_item > "- Level 3 Item 3"
      ├── regular_list_item > "- Level 3 Item 3"
        └── list_marker: "-"
        └── list_item_content: "Level 3 Item 3"
   ```

✅ **exponential_nesting**: `text`
   Input: `\*\*bold \*italic \`code \*\*bold \*italic \`code \*\*bold \*italic \`code\` italic\* bold\*\* code\` italic\* bold\*\* \`code\` italic\* bold\*\*`
   Parse Tree:
   ```
  └── text: "**bold *italic `code **bold *italic `code **bold *italic `code` italic* bold** code` italic* bold** `code` italic* bold**"
   ```

✅ **parse_tree_explosion**: `text`
   Input: `((((((((((nested parentheses))))))))))`
   Parse Tree:
   ```
  └── text: "((((((((((nested parentheses))))))))))"
   ```

✅ **large_table_data**: `table`
   Input: `| Col1 | Col2 | Col3 | Col4 | Col5 | Col6 | Col7 | Col8 |
|------|------|------|------|------|------|------|------|
| Data1| Data2| Data3| Data4| Data5| Data6| Data7| Data8|
| Data9| Data10| Data11| Data12| Data13| Data14| Data15| Data16|
| Data17| Data18| Data19| Data20| Data21| Data22| Data23| Data24|
| Data25| Data26| Data27| Data28| Data29| Data30| Data31| Data32|`
   Parse Tree:
   ```
  ├── table > "| Col1 | Col2 | Col3 | Col4 | Col5 | Col6 | Col7 | Col8 |
|------|------|------|------|------|------|------|------|
| Data1| Data2| Data3| Data4| Data5| Data6| Data7| Data8|
| Data9| Data10| Data11| Data12| Data13| Data14| Data15| Data16|
| Data17| Data18| Data19| Data20| Data21| Data22| Data23| Data24|
| Data25| Data26| Data27| Data28| Data29| Data30| Data31| Data32|"
    ├── table_header > "| Col1 | Col2 | Col3 | Col4 | Col5 | Col6 | Col7 | Col8 |"
      ├── table_cell > "Col1 "
        ├── table_cell_content > "Col1 "
          └── table_safe_text: "Col1 "
      ├── table_cell > "Col2 "
        ├── table_cell_content > "Col2 "
          └── table_safe_text: "Col2 "
      ├── table_cell > "Col3 "
        ├── table_cell_content > "Col3 "
          └── table_safe_text: "Col3 "
      ├── table_cell > "Col4 "
        ├── table_cell_content > "Col4 "
          └── table_safe_text: "Col4 "
      ├── table_cell > "Col5 "
        ├── table_cell_content > "Col5 "
          └── table_safe_text: "Col5 "
      ├── table_cell > "Col6 "
        ├── table_cell_content > "Col6 "
          └── table_safe_text: "Col6 "
      ├── table_cell > "Col7 "
        ├── table_cell_content > "Col7 "
          └── table_safe_text: "Col7 "
      ├── table_cell > "Col8 "
        ├── table_cell_content > "Col8 "
          └── table_safe_text: "Col8 "
      └── table_cell: ""
    ├── table_sep > "|------|------|------|------|------|------|------|------|"
      └── table_sep_cell: "------"
      └── table_sep_cell: "------"
      └── table_sep_cell: "------"
      └── table_sep_cell: "------"
      └── table_sep_cell: "------"
      └── table_sep_cell: "------"
      └── table_sep_cell: "------"
      └── table_sep_cell: "------"
    ├── table_row > "| Data1| Data2| Data3| Data4| Data5| Data6| Data7| Data8|"
      ├── table_cell > "Data1"
        ├── table_cell_content > "Data1"
          └── table_safe_text: "Data1"
      ├── table_cell > "Data2"
        ├── table_cell_content > "Data2"
          └── table_safe_text: "Data2"
      ├── table_cell > "Data3"
        ├── table_cell_content > "Data3"
          └── table_safe_text: "Data3"
      ├── table_cell > "Data4"
        ├── table_cell_content > "Data4"
          └── table_safe_text: "Data4"
      ├── table_cell > "Data5"
        ├── table_cell_content > "Data5"
          └── table_safe_text: "Data5"
      ├── table_cell > "Data6"
        ├── table_cell_content > "Data6"
          └── table_safe_text: "Data6"
      ├── table_cell > "Data7"
        ├── table_cell_content > "Data7"
          └── table_safe_text: "Data7"
      ├── table_cell > "Data8"
        ├── table_cell_content > "Data8"
          └── table_safe_text: "Data8"
      └── table_cell: ""
    ├── table_row > "| Data9| Data10| Data11| Data12| Data13| Data14| Data15| Data16|"
      ├── table_cell > "Data9"
        ├── table_cell_content > "Data9"
          └── table_safe_text: "Data9"
      ├── table_cell > "Data10"
        ├── table_cell_content > "Data10"
          └── table_safe_text: "Data10"
      ├── table_cell > "Data11"
        ├── table_cell_content > "Data11"
          └── table_safe_text: "Data11"
      ├── table_cell > "Data12"
        ├── table_cell_content > "Data12"
          └── table_safe_text: "Data12"
      ├── table_cell > "Data13"
        ├── table_cell_content > "Data13"
          └── table_safe_text: "Data13"
      ├── table_cell > "Data14"
        ├── table_cell_content > "Data14"
          └── table_safe_text: "Data14"
      ├── table_cell > "Data15"
        ├── table_cell_content > "Data15"
          └── table_safe_text: "Data15"
      ├── table_cell > "Data16"
        ├── table_cell_content > "Data16"
          └── table_safe_text: "Data16"
      └── table_cell: ""
    ├── table_row > "| Data17| Data18| Data19| Data20| Data21| Data22| Data23| Data24|"
      ├── table_cell > "Data17"
        ├── table_cell_content > "Data17"
          └── table_safe_text: "Data17"
      ├── table_cell > "Data18"
        ├── table_cell_content > "Data18"
          └── table_safe_text: "Data18"
      ├── table_cell > "Data19"
        ├── table_cell_content > "Data19"
          └── table_safe_text: "Data19"
      ├── table_cell > "Data20"
        ├── table_cell_content > "Data20"
          └── table_safe_text: "Data20"
      ├── table_cell > "Data21"
        ├── table_cell_content > "Data21"
          └── table_safe_text: "Data21"
      ├── table_cell > "Data22"
        ├── table_cell_content > "Data22"
          └── table_safe_text: "Data22"
      ├── table_cell > "Data23"
        ├── table_cell_content > "Data23"
          └── table_safe_text: "Data23"
      ├── table_cell > "Data24"
        ├── table_cell_content > "Data24"
          └── table_safe_text: "Data24"
      └── table_cell: ""
    ├── table_row > "| Data25| Data26| Data27| Data28| Data29| Data30| Data31| Data32|"
      ├── table_cell > "Data25"
        ├── table_cell_content > "Data25"
          └── table_safe_text: "Data25"
      ├── table_cell > "Data26"
        ├── table_cell_content > "Data26"
          └── table_safe_text: "Data26"
      ├── table_cell > "Data27"
        ├── table_cell_content > "Data27"
          └── table_safe_text: "Data27"
      ├── table_cell > "Data28"
        ├── table_cell_content > "Data28"
          └── table_safe_text: "Data28"
      ├── table_cell > "Data29"
        ├── table_cell_content > "Data29"
          └── table_safe_text: "Data29"
      ├── table_cell > "Data30"
        ├── table_cell_content > "Data30"
          └── table_safe_text: "Data30"
      ├── table_cell > "Data31"
        ├── table_cell_content > "Data31"
          └── table_safe_text: "Data31"
      ├── table_cell > "Data32"
        ├── table_cell_content > "Data32"
          └── table_safe_text: "Data32"
      └── table_cell: ""
   ```

## urls

✅ **http_simple**: `http_url`
   Input: `http://example.com`
   Parse Tree:
   ```
  └── http_url: "http://example.com"
   ```

✅ **https_simple**: `http_url`
   Input: `https://example.com`
   Parse Tree:
   ```
  └── http_url: "https://example.com"
   ```

✅ **url_with_path**: `inline_url`
   Input: `https://example.com/path/to/page`
   Parse Tree:
   ```
  └── link_url: "https://example.com/path/to/page"
   ```

✅ **url_with_query**: `inline_url`
   Input: `https://example.com/search?q=test&lang=en`
   Parse Tree:
   ```
  └── link_url: "https://example.com/search?q=test&lang=en"
   ```

✅ **url_with_fragment**: `inline_url`
   Input: `https://example.com/page#section`
   Parse Tree:
   ```
  └── link_url: "https://example.com/page#section"
   ```

✅ **url_complex**: `inline_url`
   Input: `https://subdomain.example.com:8080/path/to/page?param1=value1&param2=value2#section`
   Parse Tree:
   ```
  └── link_url: "https://subdomain.example.com:8080/path/to/page?param1=value1&param2=value2#section"
   ```

✅ **www_simple**: `www_url`
   Input: `www.example.com`
   Parse Tree:
   ```
  └── www_url: "www.example.com"
   ```

✅ **www_with_path**: `www_url`
   Input: `www.example.com/path`
   Parse Tree:
   ```
  └── www_url: "www.example.com/path"
   ```

✅ **mailto_simple**: `mailto`
   Input: `mailto:user@example.com`
   Parse Tree:
   ```
  └── mailto: "mailto:user@example.com"
   ```

✅ **mailto_complex**: `mailto`
   Input: `mailto:user.name+tag@sub.example.com`
   Parse Tree:
   ```
  └── mailto: "mailto:user.name+tag@sub.example.com"
   ```

✅ **local_relative**: `local_path`
   Input: `./path/to/file`
   Parse Tree:
   ```
  └── local_path: "./path/to/file"
   ```

✅ **local_parent**: `local_path`
   Input: `../path/to/file`
   Parse Tree:
   ```
  └── local_path: "../path/to/file"
   ```

✅ **local_absolute**: `local_path`
   Input: `/absolute/path/to/file`
   Parse Tree:
   ```
  └── local_path: "/absolute/path/to/file"
   ```

✅ **local_windows**: `local_path`
   Input: `C:\\path\	o\\file`
   Parse Tree:
   ```
  └── local_path: "C:\\path\	o\\file"
   ```

✅ **local_complex**: `local_path`
   Input: `docs/guide/installation.md`
   Parse Tree:
   ```
  └── local_path: "docs/guide/installation.md"
   ```

✅ **youtube_watch**: `youtube_url`
   Input: `https://www.youtube.com/watch?v=dQw4w9WgXcQ`
   Parse Tree:
   ```
  └── youtube_url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
   ```

✅ **youtube_short**: `youtube_url`
   Input: `https://youtu.be/dQw4w9WgXcQ`
   Parse Tree:
   ```
  └── youtube_url: "https://youtu.be/dQw4w9WgXcQ"
   ```

✅ **youtube_with_params**: `youtube_url`
   Input: `https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42`
   Parse Tree:
   ```
  └── youtube_url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42"
   ```

❌ **image_jpg**: `image_url` (Unexpected failure)
   Input: `https://example.com/image.jpg`
   Error: ` --> 1:1
  |
1 | https://example.com/image.jpg
  | ^---
  |
  = expected image_url`

❌ **image_png**: `image_url` (Unexpected failure)
   Input: `https://example.com/image.png`
   Error: ` --> 1:1
  |
1 | https://example.com/image.png
  | ^---
  |
  = expected image_url`

❌ **image_gif**: `image_url` (Unexpected failure)
   Input: `https://example.com/image.gif`
   Error: ` --> 1:1
  |
1 | https://example.com/image.gif
  | ^---
  |
  = expected image_url`

❌ **image_webp**: `image_url` (Unexpected failure)
   Input: `https://example.com/image.webp`
   Error: ` --> 1:1
  |
1 | https://example.com/image.webp
  | ^---
  |
  = expected image_url`

❌ **image_svg**: `image_url` (Unexpected failure)
   Input: `https://example.com/image.svg`
   Error: ` --> 1:1
  |
1 | https://example.com/image.svg
  | ^---
  |
  = expected image_url`

❌ **image_local**: `image_url` (Unexpected failure)
   Input: `./images/photo.jpg`
   Error: ` --> 1:1
  |
1 | ./images/photo.jpg
  | ^---
  |
  = expected image_url`

## commonmark_fenced_code_blocks

✅ **cm_example_119**: `text`
   Input: `\`\`\`
<
 >
\`\`\`
`
   Parse Tree:
   ```
  └── text: "```
<
 >
```
"
   ```

✅ **cm_example_120**: `text`
   Input: `~~~
<
 >
~~~
`
   Parse Tree:
   ```
  └── text: "~"
   ```

✅ **cm_example_121**: `text`
   Input: `\`\`
foo
\`\`
`
   Parse Tree:
   ```
  └── text: "``
foo
``
"
   ```

✅ **cm_example_122**: `text`
   Input: `\`\`\`
aaa
~~~
\`\`\`
`
   Parse Tree:
   ```
  └── text: "```
aaa
"
   ```

✅ **cm_example_123**: `text`
   Input: `~~~
aaa
\`\`\`
~~~
`
   Parse Tree:
   ```
  └── text: "~"
   ```

✅ **cm_example_124**: `text`
   Input: `\`\`\`\`
aaa
\`\`\`
\`\`\`\`\`\`
`
   Parse Tree:
   ```
  └── text: "````
aaa
```
``````
"
   ```

✅ **cm_example_125**: `text`
   Input: `~~~~
aaa
~~~
~~~~
`
   Parse Tree:
   ```
  └── text: "~"
   ```

✅ **cm_example_126**: `text`
   Input: `\`\`\`
`
   Parse Tree:
   ```
  └── text: "```
"
   ```

✅ **cm_example_127**: `text`
   Input: `\`\`\`\`\`

\`\`\`
aaa
`
   Parse Tree:
   ```
  └── text: "`````

```
aaa
"
   ```

✅ **cm_example_128**: `text`
   Input: `> \`\`\`
> aaa

bbb
`
   Parse Tree:
   ```
  └── text: "> ```
> aaa

bbb
"
   ```

✅ **cm_example_129**: `text`
   Input: `\`\`\`

  
\`\`\`
`
   Parse Tree:
   ```
  └── text: "```

  
```
"
   ```

✅ **cm_example_130**: `text`
   Input: `\`\`\`
\`\`\`
`
   Parse Tree:
   ```
  └── text: "```
```
"
   ```

✅ **cm_example_131**: `text`
   Input: ` \`\`\`
 aaa
aaa
\`\`\`
`
   Parse Tree:
   ```
  └── text: " ```
 aaa
aaa
```
"
   ```

✅ **cm_example_132**: `text`
   Input: `  \`\`\`
aaa
  aaa
aaa
  \`\`\`
`
   Parse Tree:
   ```
  └── text: "  ```
aaa
  aaa
aaa
  ```
"
   ```

✅ **cm_example_133**: `text`
   Input: `   \`\`\`
   aaa
    aaa
  aaa
   \`\`\`
`
   Parse Tree:
   ```
  └── text: "   ```
   aaa
    aaa
  aaa
   ```
"
   ```

✅ **cm_example_134**: `text`
   Input: `    \`\`\`
    aaa
    \`\`\`
`
   Parse Tree:
   ```
  └── text: "    ```
    aaa
    ```
"
   ```

✅ **cm_example_135**: `text`
   Input: `\`\`\`
aaa
  \`\`\`
`
   Parse Tree:
   ```
  └── text: "```
aaa
  ```
"
   ```

✅ **cm_example_136**: `text`
   Input: `   \`\`\`
aaa
  \`\`\`
`
   Parse Tree:
   ```
  └── text: "   ```
aaa
  ```
"
   ```

✅ **cm_example_137**: `text`
   Input: `\`\`\`
aaa
    \`\`\`
`
   Parse Tree:
   ```
  └── text: "```
aaa
    ```
"
   ```

✅ **cm_example_138**: `text`
   Input: `\`\`\` \`\`\`
aaa
`
   Parse Tree:
   ```
  └── text: "``` ```
aaa
"
   ```

✅ **cm_example_139**: `text`
   Input: `~~~~~~
aaa
~~~ ~~
`
   Parse Tree:
   ```
  └── text: "~"
   ```

✅ **cm_example_140**: `text`
   Input: `foo
\`\`\`
bar
\`\`\`
baz
`
   Parse Tree:
   ```
  └── text: "foo
```
bar
```
baz
"
   ```

✅ **cm_example_141**: `text`
   Input: `foo
---
~~~
bar
~~~
# baz
`
   Parse Tree:
   ```
  └── text: "foo
---
"
   ```

✅ **cm_example_142**: `text`
   Input: `\`\`\`ruby
def foo(x)
  return 3
end
\`\`\`
`
   Parse Tree:
   ```
  └── text: "```ruby
def foo(x)
  return 3
end
```
"
   ```

✅ **cm_example_143**: `text`
   Input: `~~~~    ruby startline=3 $%@#$
def foo(x)
  return 3
end
~~~~~~~
`
   Parse Tree:
   ```
  └── text: "~"
   ```

✅ **cm_example_144**: `text`
   Input: `\`\`\`\`;
\`\`\`\`
`
   Parse Tree:
   ```
  └── text: "````;
````
"
   ```

✅ **cm_example_145**: `text`
   Input: `\`\`\` aa \`\`\`
foo
`
   Parse Tree:
   ```
  └── text: "``` aa ```
foo
"
   ```

✅ **cm_example_146**: `text`
   Input: `~~~ aa \`\`\` ~~~
foo
~~~
`
   Parse Tree:
   ```
  └── text: "~"
   ```

✅ **cm_example_147**: `text`
   Input: `\`\`\`
\`\`\` aaa
\`\`\`
`
   Parse Tree:
   ```
  └── text: "```
``` aaa
```
"
   ```

## math_inline

✅ **math_simple**: `math_inline`
   Input: `$x = 1$`
   Parse Tree:
   ```
  └── math_inline: "$x = 1$"
   ```

✅ **math_complex**: `math_inline`
   Input: `$\\frac{a}{b} = c$`
   Parse Tree:
   ```
  └── math_inline: "$\\frac{a}{b} = c$"
   ```

✅ **math_empty**: `math_inline`
   Input: `$$`
   Parse Tree:
   ```
  └── math_inline: "$$"
   ```

✅ **math_with_spaces**: `math_inline`
   Input: `$ x = 1 $`
   Parse Tree:
   ```
  └── math_inline: "$ x = 1 $"
   ```

✅ **math_escaped_dollar**: `math_inline`
   Input: `$price is \\$5$`
   Parse Tree:
   ```
  └── math_inline: "$price is \\$"
   ```

✅ **math_formula**: `math_inline`
   Input: `$E = mc^2$`
   Parse Tree:
   ```
  └── math_inline: "$E = mc^2$"
   ```

❌ **math_unclosed**: `math_inline` (Unexpected failure)
   Input: `$missing closing`
   Error: ` --> 1:1
  |
1 | $missing closing
  | ^---
  |
  = expected math_inline`

✅ **math_nested**: `math_inline`
   Input: `$$not inline$$`
   Parse Tree:
   ```
  └── math_inline: "$$"
   ```

## footnotes

✅ **footnote_ref_simple**: `footnote_ref`
   Input: `\[^1\]`
   Parse Tree:
   ```
  ├── footnote_ref > "[^1]"
    └── footnote_label: "1"
   ```

✅ **footnote_ref_alpha**: `footnote_ref`
   Input: `\[^note\]`
   Parse Tree:
   ```
  ├── footnote_ref > "[^note]"
    └── footnote_label: "note"
   ```

✅ **footnote_ref_mixed**: `footnote_ref`
   Input: `\[^note1\]`
   Parse Tree:
   ```
  ├── footnote_ref > "[^note1]"
    └── footnote_label: "note1"
   ```

✅ **footnote_ref_unicode**: `footnote_ref`
   Input: `\[^café\]`
   Parse Tree:
   ```
  ├── footnote_ref > "[^café]"
    └── footnote_label: "café"
   ```

❌ **footnote_def_simple**: `footnote_def` (Unexpected failure)
   Input: `\[^1\]: This is a footnote`
   Error: ` --> 1:1
  |
1 | [^1]: This is a footnote
  | ^---
  |
  = expected footnote_def`

❌ **footnote_def_multiline**: `footnote_def` (Unexpected failure)
   Input: `\[^note\]: This is a longer footnote
    with multiple lines`
   Error: ` --> 1:1
  |
1 | [^note]: This is a longer footnote
  | ^---
  |
  = expected footnote_def`

❌ **footnote_def_complex**: `footnote_def` (Unexpected failure)
   Input: `\[^complex\]: A footnote with \*\*formatting\*\* and \[links\](url)`
   Error: ` --> 1:1
  |
1 | [^complex]: A footnote with **formatting** and [links](url)
  | ^---
  |
  = expected footnote_def`

✅ **inline_footnote_simple**: `inline_footnote_ref`
   Input: `^\[This is an inline footnote\]`
   Parse Tree:
   ```
  └── inline_footnote_ref: "^[This is an inline footnote]"
   ```

✅ **inline_footnote_complex**: `inline_footnote_ref`
   Input: `^\[Inline footnote with \*\*formatting\*\*\]`
   Parse Tree:
   ```
  └── inline_footnote_ref: "^[Inline footnote with **formatting**]"
   ```

✅ **footnote_empty_label**: `footnote_ref` (Expected failure)
   Input: `\[^\]`
   Error: ` --> 1:3
  |
1 | [^]
  |   ^---
  |
  = expected footnote_label`

❌ **footnote_unclosed**: `footnote_ref` (Unexpected failure)
   Input: `\[^note`
   Error: ` --> 1:1
  |
1 | [^note
  | ^---
  |
  = expected footnote_ref`

## admonitions

✅ **note_simple**: `admonition_block`
   Input: `:::note
This is a note
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::note
This is a note
:::"
    ├── admonition_open > ":::note"
      ├── admonition_type > "note"
        └── KW_NOTE: "note"
    └── admonition_close: ":::"
   ```

✅ **tip_simple**: `admonition_block`
   Input: `:::tip
This is a tip
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::tip
This is a tip
:::"
    ├── admonition_open > ":::tip"
      ├── admonition_type > "tip"
        └── KW_TIP: "tip"
    └── admonition_close: ":::"
   ```

✅ **warning_simple**: `admonition_block`
   Input: `:::warning
This is a warning
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::warning
This is a warning
:::"
    ├── admonition_open > ":::warning"
      ├── admonition_type > "warning"
        └── KW_WARNING: "warning"
    └── admonition_close: ":::"
   ```

✅ **danger_simple**: `admonition_block`
   Input: `:::danger
This is dangerous
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::danger
This is dangerous
:::"
    ├── admonition_open > ":::danger"
      ├── admonition_type > "danger"
        └── KW_DANGER: "danger"
    └── admonition_close: ":::"
   ```

✅ **info_simple**: `admonition_block`
   Input: `:::info
This is info
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::info
This is info
:::"
    ├── admonition_open > ":::info"
      ├── admonition_type > "info"
        └── KW_INFO: "info"
    └── admonition_close: ":::"
   ```

✅ **note_with_title**: `admonition_block`
   Input: `:::note\[Custom Title\]
Note content
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::note[Custom Title]
Note content
:::"
    ├── admonition_open > ":::note[Custom Title]"
      ├── admonition_type > "note"
        └── KW_NOTE: "note"
    └── admonition_close: ":::"
   ```

✅ **warning_titled**: `admonition_block`
   Input: `:::warning\[Important Warning\]
Warning content
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::warning[Important Warning]
Warning content
:::"
    ├── admonition_open > ":::warning[Important Warning]"
      ├── admonition_type > "warning"
        └── KW_WARNING: "warning"
    └── admonition_close: ":::"
   ```

✅ **emoji_admonition**: `admonition_block`
   Input: `:::\[💡\] Custom Emoji
Content here
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::[💡] Custom Emoji
Content here
:::"
    └── admonition_emoji: ":::[💡] Custom Emoji"
    └── admonition_close: ":::"
   ```

✅ **note_uppercase**: `admonition_block`
   Input: `:::NOTE
Uppercase note
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::NOTE
Uppercase note
:::"
    ├── admonition_open > ":::NOTE"
      ├── admonition_type > "NOTE"
        └── KW_NOTE: "NOTE"
    └── admonition_close: ":::"
   ```

✅ **tip_mixed_case**: `admonition_block`
   Input: `:::TiP
Mixed case tip
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::TiP
Mixed case tip
:::"
    ├── admonition_open > ":::TiP"
      ├── admonition_type > "TiP"
        └── KW_TIP: "TiP"
    └── admonition_close: ":::"
   ```

✅ **admonition_unclosed**: `admonition_block`
   Input: `:::note
Unclosed admonition`
   Parse Tree:
   ```
  ├── admonition_block > ":::note
Unclosed admonition"
    ├── admonition_open > ":::note"
      ├── admonition_type > "note"
        └── KW_NOTE: "note"
   ```

❌ **admonition_unknown**: `admonition_block` (Unexpected failure)
   Input: `:::custom
Unknown type
:::`
   Error: ` --> 1:4
  |
1 | :::custom
  |    ^---
  |
  = expected admonition_type`

## commonmark_conformance

✅ **cm_atx_basic**: `heading`
   Input: `# foo`
   Parse Tree:
   ```
  ├── heading > "# foo"
    ├── H1 > "# foo"
      ├── heading_content > "foo"
        ├── heading_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_multiple**: `heading`
   Input: `## foo`
   Parse Tree:
   ```
  ├── heading > "## foo"
    ├── H2 > "## foo"
      ├── heading_content > "foo"
        ├── heading_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_max_level**: `heading`
   Input: `###### foo`
   Parse Tree:
   ```
  ├── heading > "###### foo"
    ├── H6 > "###### foo"
      ├── heading_content > "foo"
        ├── heading_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_no_space**: `heading`
   Input: `#5 bolt`
   Parse Tree:
   ```
  ├── heading > "#5 bolt"
    ├── H1 > "#5 bolt"
      ├── heading_content > "5 bolt"
        ├── heading_inline > "5"
          └── word: "5"
        ├── heading_inline > "bolt"
          └── word: "bolt"
   ```

❌ **cm_atx_escaped**: `heading` (Unexpected failure)
   Input: `\\## foo`
   Error: ` --> 1:3
  |
1 | \\## foo
  |   ^---
  |
  = expected heading_inline`

✅ **cm_atx_content_formatting**: `heading`
   Input: `# foo \*bar\* \\\*baz\\\*`
   Parse Tree:
   ```
  ├── heading > "# foo *bar* \\*baz\\*"
    ├── H1 > "# foo *bar* \\*baz\\*"
      ├── heading_content > "foo *bar* \\*baz\\*"
        ├── heading_inline > "foo"
          └── word: "foo"
        ├── heading_inline > "*bar*"
          ├── emphasis > "*bar*"
            ├── italic > "*bar*"
              └── italic_asterisk: "*bar*"
        ├── heading_inline > "\\"
          └── escaped_char: "\\"
        ├── heading_inline > "*baz\\*"
          ├── emphasis > "*baz\\*"
            ├── italic > "*baz\\*"
              └── italic_asterisk: "*baz\\*"
   ```

✅ **cm_atx_spaces_after**: `heading`
   Input: `#                  foo                     `
   Parse Tree:
   ```
  ├── heading > "#                  foo                     "
    ├── H1 > "#                  foo                     "
      ├── heading_content > "foo                     "
        ├── heading_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_trailing_spaces**: `heading`
   Input: `### foo ### `
   Parse Tree:
   ```
  ├── heading > "### foo "
    ├── H3 > "### foo "
      ├── heading_content > "foo "
        ├── heading_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_trailing_hash_count**: `heading`
   Input: `### foo #### `
   Parse Tree:
   ```
  ├── heading > "### foo "
    ├── H3 > "### foo "
      ├── heading_content > "foo "
        ├── heading_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_setext_h1_basic**: `setext_h1`
   Input: `Foo
===`
   Parse Tree:
   ```
  ├── setext_h1 > "Foo
==="
    ├── heading_content > "Foo"
      ├── heading_inline > "Foo"
        └── word: "Foo"
   ```

✅ **cm_setext_h2_basic**: `setext_h2`
   Input: `Foo
---`
   Parse Tree:
   ```
  ├── setext_h2 > "Foo
---"
    ├── heading_content > "Foo"
      ├── heading_inline > "Foo"
        └── word: "Foo"
   ```

❌ **cm_setext_content**: `setext_h2` (Unexpected failure)
   Input: `Foo \*bar\*
=========`
   Error: ` --> 1:10
  |
1 | Foo *bar*␊
  |          ^---
  |
  = expected heading_inline`

❌ **cm_setext_underline_count**: `setext_h2` (Unexpected failure)
   Input: `Foo
=========================`
   Error: ` --> 1:4
  |
1 | Foo␊
  |    ^---
  |
  = expected heading_inline`

✅ **cm_setext_spaces**: `setext_h2`
   Input: `   Foo
---`
   Parse Tree:
   ```
  ├── setext_h2 > "   Foo
---"
    ├── heading_content > "Foo"
      ├── heading_inline > "Foo"
        └── word: "Foo"
   ```

❌ **cm_setext_indent_content**: `setext_h2` (Unexpected failure)
   Input: ` Foo
  ===`
   Error: ` --> 1:5
  |
1 |  Foo␊
  |     ^---
  |
  = expected heading_inline`

❌ **cm_setext_lazy**: `setext_h2` (Unexpected failure)
   Input: `Foo
Bar
---`
   Error: ` --> 1:4
  |
1 | Foo␊
  |    ^---
  |
  = expected heading_inline`

✅ **cm_emphasis_basic**: `emphasis`
   Input: `\*foo bar\*`
   Parse Tree:
   ```
  ├── emphasis > "*foo bar*"
    ├── italic > "*foo bar*"
      └── italic_asterisk: "*foo bar*"
   ```

✅ **cm_emphasis_underscore**: `emphasis`
   Input: `\_foo bar\_`
   Parse Tree:
   ```
  ├── emphasis > "_foo bar_"
    ├── italic > "_foo bar_"
      └── italic_underscore: "_foo bar_"
   ```

✅ **cm_strong_basic**: `emphasis`
   Input: `\*\*foo bar\*\*`
   Parse Tree:
   ```
  ├── emphasis > "**foo bar**"
    ├── bold > "**foo bar**"
      └── bold_asterisk: "**foo bar**"
   ```

✅ **cm_strong_underscore**: `emphasis`
   Input: `\_\_foo bar\_\_`
   Parse Tree:
   ```
  ├── emphasis > "__foo bar__"
    ├── bold > "__foo bar__"
      └── bold_underscore: "__foo bar__"
   ```

✅ **cm_emphasis_nested**: `emphasis`
   Input: `\*foo \*\*bar\*\* baz\*`
   Parse Tree:
   ```
  ├── emphasis > "*foo *"
    ├── italic > "*foo *"
      └── italic_asterisk: "*foo *"
   ```

❌ **cm_emphasis_intraword**: `emphasis` (Unexpected failure)
   Input: `foo\*bar\*baz`
   Error: ` --> 1:1
  |
1 | foo*bar*baz
  | ^---
  |
  = expected emphasis`

✅ **cm_emphasis_punctuation**: `emphasis`
   Input: `\*foo.\*`
   Parse Tree:
   ```
  ├── emphasis > "*foo.*"
    ├── italic > "*foo.*"
      └── italic_asterisk: "*foo.*"
   ```

✅ **cm_emphasis_newline_fail**: `emphasis`
   Input: `\*foo
bar\*`
   Parse Tree:
   ```
  ├── emphasis > "*foo"
    ├── italic > "*foo"
      └── italic_asterisk: "*foo"
   ```

✅ **cm_link_basic**: `inline_link`
   Input: `\[link\](/uri)`
   Parse Tree:
   ```
  ├── inline_link > "[link](/uri)"
    └── bracket_link_without_title: "[link](/uri)"
   ```

✅ **cm_link_title**: `inline_link`
   Input: `\[link\](/uri "title")`
   Parse Tree:
   ```
  ├── inline_link > "[link](/uri "title")"
    └── bracket_link_without_title: "[link](/uri "title")"
   ```

✅ **cm_link_empty**: `inline_link` (Expected failure)
   Input: `\[\]()`
   Error: ` --> 1:1
  |
1 | []()
  | ^---
  |
  = expected inline_link`

✅ **cm_link_with_parens**: `inline_link`
   Input: `\[link\](/uri(and(nested)))`
   Parse Tree:
   ```
  ├── inline_link > "[link](/uri(and(nested)"
    └── bracket_link_without_title: "[link](/uri(and(nested)"
   ```

✅ **cm_link_escaped_parens**: `inline_link`
   Input: `\[link\](/uri\\(paren\\))`
   Parse Tree:
   ```
  ├── inline_link > "[link](/uri\\(paren\\)"
    └── bracket_link_without_title: "[link](/uri\\(paren\\)"
   ```

✅ **cm_autolink_uri**: `inline_link`
   Input: `<http://foo.bar.baz>`
   Parse Tree:
   ```
  ├── inline_link > "<http://foo.bar.baz>"
    ├── autolink > "<http://foo.bar.baz>"
      ├── autolink_url > "<http://foo.bar.baz>"
        └── link_url: "http://foo.bar.baz"
   ```

✅ **cm_autolink_email**: `inline_link`
   Input: `<foo@bar.example.com>`
   Parse Tree:
   ```
  ├── inline_link > "<foo@bar.example.com>"
    ├── autolink > "<foo@bar.example.com>"
      ├── autolink_email > "<foo@bar.example.com>"
        └── EMAIL_LOCAL: "foo"
        └── EMAIL_FULL_DOMAIN: "bar.example.com"
   ```

✅ **cm_code_basic**: `code_inline`
   Input: `\`foo\``
   Parse Tree:
   ```
  └── code_inline: "`foo`"
   ```

❌ **cm_code_with_backticks**: `code_inline` (Unexpected failure)
   Input: `\`\` foo \` bar \`\``
   Error: ` --> 1:1
  |
1 | `` foo ` bar ``
  | ^---
  |
  = expected code_inline`

✅ **cm_code_strip_spaces**: `code_inline`
   Input: `\` \`\` \``
   Parse Tree:
   ```
  └── code_inline: "` `"
   ```

✅ **cm_code_preserve_spaces**: `code_inline`
   Input: `\`  \``
   Parse Tree:
   ```
  └── code_inline: "`  `"
   ```

✅ **cm_code_line_endings**: `code_inline`
   Input: `\`foo   bar 
baz\``
   Parse Tree:
   ```
  └── code_inline: "`foo   bar 
baz`"
   ```

## commonmark_block_quotes

✅ **cm_example_228**: `text`
   Input: `> # Foo
> bar
> baz
`
   Parse Tree:
   ```
  └── text: "> # Foo
> bar
> baz
"
   ```

✅ **cm_example_229**: `text`
   Input: `># Foo
>bar
> baz
`
   Parse Tree:
   ```
  └── text: "># Foo
>bar
> baz
"
   ```

✅ **cm_example_230**: `text`
   Input: `   > # Foo
   > bar
 > baz
`
   Parse Tree:
   ```
  └── text: "   > # Foo
   > bar
 > baz
"
   ```

✅ **cm_example_231**: `text`
   Input: `    > # Foo
    > bar
    > baz
`
   Parse Tree:
   ```
  └── text: "    > # Foo
    > bar
    > baz
"
   ```

✅ **cm_example_232**: `text`
   Input: `> # Foo
> bar
baz
`
   Parse Tree:
   ```
  └── text: "> # Foo
> bar
baz
"
   ```

✅ **cm_example_233**: `text`
   Input: `> bar
baz
> foo
`
   Parse Tree:
   ```
  └── text: "> bar
baz
> foo
"
   ```

✅ **cm_example_234**: `text`
   Input: `> foo
---
`
   Parse Tree:
   ```
  └── text: "> foo
---
"
   ```

✅ **cm_example_235**: `text`
   Input: `> - foo
- bar
`
   Parse Tree:
   ```
  └── text: "> - foo
- bar
"
   ```

✅ **cm_example_236**: `text`
   Input: `>     foo
    bar
`
   Parse Tree:
   ```
  └── text: ">     foo
    bar
"
   ```

✅ **cm_example_237**: `text`
   Input: `> \`\`\`
foo
\`\`\`
`
   Parse Tree:
   ```
  └── text: "> ```
foo
```
"
   ```

✅ **cm_example_238**: `text`
   Input: `> foo
    - bar
`
   Parse Tree:
   ```
  └── text: "> foo
    - bar
"
   ```

✅ **cm_example_239**: `text`
   Input: `>
`
   Parse Tree:
   ```
  └── text: ">
"
   ```

✅ **cm_example_240**: `text`
   Input: `>
>  
> 
`
   Parse Tree:
   ```
  └── text: ">
>  
> 
"
   ```

✅ **cm_example_241**: `text`
   Input: `>
> foo
>  
`
   Parse Tree:
   ```
  └── text: ">
> foo
>  
"
   ```

✅ **cm_example_242**: `text`
   Input: `> foo

> bar
`
   Parse Tree:
   ```
  └── text: "> foo

> bar
"
   ```

✅ **cm_example_243**: `text`
   Input: `> foo
> bar
`
   Parse Tree:
   ```
  └── text: "> foo
> bar
"
   ```

✅ **cm_example_244**: `text`
   Input: `> foo
>
> bar
`
   Parse Tree:
   ```
  └── text: "> foo
>
> bar
"
   ```

✅ **cm_example_245**: `text`
   Input: `foo
> bar
`
   Parse Tree:
   ```
  └── text: "foo
> bar
"
   ```

✅ **cm_example_246**: `text`
   Input: `> aaa
\*\*\*
> bbb
`
   Parse Tree:
   ```
  └── text: "> aaa
***
> bbb
"
   ```

✅ **cm_example_247**: `text`
   Input: `> bar
baz
`
   Parse Tree:
   ```
  └── text: "> bar
baz
"
   ```

✅ **cm_example_248**: `text`
   Input: `> bar

baz
`
   Parse Tree:
   ```
  └── text: "> bar

baz
"
   ```

✅ **cm_example_249**: `text`
   Input: `> bar
>
baz
`
   Parse Tree:
   ```
  └── text: "> bar
>
baz
"
   ```

✅ **cm_example_250**: `text`
   Input: `> > > foo
bar
`
   Parse Tree:
   ```
  └── text: "> > > foo
bar
"
   ```

✅ **cm_example_251**: `text`
   Input: `>>> foo
> bar
>>baz
`
   Parse Tree:
   ```
  └── text: ">>> foo
> bar
>>baz
"
   ```

✅ **cm_example_252**: `text`
   Input: `>     code

>    not code
`
   Parse Tree:
   ```
  └── text: ">     code

>    not code
"
   ```

## commonmark_code_spans

✅ **cm_example_328**: `text`
   Input: `\`foo\`
`
   Parse Tree:
   ```
  └── text: "`foo`
"
   ```

✅ **cm_example_329**: `text`
   Input: `\`\` foo \` bar \`\`
`
   Parse Tree:
   ```
  └── text: "`` foo ` bar ``
"
   ```

✅ **cm_example_330**: `text`
   Input: `\` \`\` \`
`
   Parse Tree:
   ```
  └── text: "` `` `
"
   ```

✅ **cm_example_331**: `text`
   Input: `\`  \`\`  \`
`
   Parse Tree:
   ```
  └── text: "`  ``  `
"
   ```

✅ **cm_example_332**: `text`
   Input: `\` a\`
`
   Parse Tree:
   ```
  └── text: "` a`
"
   ```

✅ **cm_example_333**: `text`
   Input: `\`b\`
`
   Parse Tree:
   ```
  └── text: "`b`
"
   ```

✅ **cm_example_334**: `text`
   Input: `\`\`
\`  \`
`
   Parse Tree:
   ```
  └── text: "``
`  `
"
   ```

✅ **cm_example_335**: `text`
   Input: `\`\`
foo
bar  
baz
\`\`
`
   Parse Tree:
   ```
  └── text: "``
foo
bar  
baz
``
"
   ```

✅ **cm_example_336**: `text`
   Input: `\`\`
foo 
\`\`
`
   Parse Tree:
   ```
  └── text: "``
foo 
``
"
   ```

✅ **cm_example_337**: `text`
   Input: `\`foo   bar 
baz\`
`
   Parse Tree:
   ```
  └── text: "`foo   bar 
baz`
"
   ```

✅ **cm_example_338**: `text`
   Input: `\`foo\\\`bar\`
`
   Parse Tree:
   ```
  └── text: "`foo"
   ```

✅ **cm_example_339**: `text`
   Input: `\`\`foo\`bar\`\`
`
   Parse Tree:
   ```
  └── text: "``foo`bar``
"
   ```

✅ **cm_example_340**: `text`
   Input: `\` foo \`\` bar \`
`
   Parse Tree:
   ```
  └── text: "` foo `` bar `
"
   ```

✅ **cm_example_341**: `text`
   Input: `\*foo\`\*\`
`
   Parse Tree:
   ```
  └── text: "*foo`*`
"
   ```

✅ **cm_example_342**: `text`
   Input: `\[not a \`link\](/foo\`)
`
   Parse Tree:
   ```
  └── text: "[not a `link](/foo`)
"
   ```

✅ **cm_example_343**: `text`
   Input: `\`<a href="\`">\`
`
   Parse Tree:
   ```
  └── text: "`<a href="`">`
"
   ```

✅ **cm_example_344**: `text`
   Input: `<a href="\`">\`
`
   Parse Tree:
   ```
  └── text: "<a href="`">`
"
   ```

✅ **cm_example_345**: `text`
   Input: `\`<http://foo.bar.\`baz>\`
`
   Parse Tree:
   ```
  └── text: "`<http://foo.bar.`baz>`
"
   ```

✅ **cm_example_346**: `text`
   Input: `<http://foo.bar.\`baz>\`
`
   Parse Tree:
   ```
  └── text: "<http://foo.bar.`baz>`
"
   ```

✅ **cm_example_347**: `text`
   Input: `\`\`\`foo\`\`
`
   Parse Tree:
   ```
  └── text: "```foo``
"
   ```

✅ **cm_example_348**: `text`
   Input: `\`foo
`
   Parse Tree:
   ```
  └── text: "`foo
"
   ```

✅ **cm_example_349**: `text`
   Input: `\`foo\`\`bar\`\`
`
   Parse Tree:
   ```
  └── text: "`foo``bar``
"
   ```

## commonmark_lists

✅ **cm_example_301**: `text`
   Input: `- foo
- bar
+ baz
`
   Parse Tree:
   ```
  └── text: "- foo
- bar
+ baz
"
   ```

✅ **cm_example_302**: `text`
   Input: `1. foo
2. bar
3) baz
`
   Parse Tree:
   ```
  └── text: "1. foo
2. bar
3) baz
"
   ```

✅ **cm_example_303**: `text`
   Input: `Foo
- bar
- baz
`
   Parse Tree:
   ```
  └── text: "Foo
- bar
- baz
"
   ```

✅ **cm_example_304**: `text`
   Input: `The number of windows in my house is
14.  The number of doors is 6.
`
   Parse Tree:
   ```
  └── text: "The number of windows in my house is
14.  The number of doors is 6.
"
   ```

✅ **cm_example_305**: `text`
   Input: `The number of windows in my house is
1.  The number of doors is 6.
`
   Parse Tree:
   ```
  └── text: "The number of windows in my house is
1.  The number of doors is 6.
"
   ```

✅ **cm_example_306**: `text`
   Input: `- foo

- bar


- baz
`
   Parse Tree:
   ```
  └── text: "- foo

- bar


- baz
"
   ```

✅ **cm_example_307**: `text`
   Input: `- foo
  - bar
    - baz


      bim
`
   Parse Tree:
   ```
  └── text: "- foo
  - bar
    - baz


      bim
"
   ```

✅ **cm_example_308**: `text`
   Input: `- foo
- bar

<!-- -->

- baz
- bim
`
   Parse Tree:
   ```
  └── text: "- foo
- bar

<!-- -->

- baz
- bim
"
   ```

✅ **cm_example_309**: `text`
   Input: `-   foo

    notcode

-   foo

<!-- -->

    code
`
   Parse Tree:
   ```
  └── text: "-   foo

    notcode

-   foo

<!-- -->

    code
"
   ```

✅ **cm_example_310**: `text`
   Input: `- a
 - b
  - c
   - d
  - e
 - f
- g
`
   Parse Tree:
   ```
  └── text: "- a
 - b
  - c
   - d
  - e
 - f
- g
"
   ```

✅ **cm_example_311**: `text`
   Input: `1. a

  2. b

   3. c
`
   Parse Tree:
   ```
  └── text: "1. a

  2. b

   3. c
"
   ```

✅ **cm_example_312**: `text`
   Input: `- a
 - b
  - c
   - d
    - e
`
   Parse Tree:
   ```
  └── text: "- a
 - b
  - c
   - d
    - e
"
   ```

✅ **cm_example_313**: `text`
   Input: `1. a

  2. b

    3. c
`
   Parse Tree:
   ```
  └── text: "1. a

  2. b

    3. c
"
   ```

✅ **cm_example_314**: `text`
   Input: `- a
- b

- c
`
   Parse Tree:
   ```
  └── text: "- a
- b

- c
"
   ```

✅ **cm_example_315**: `text`
   Input: `\* a
\*

\* c
`
   Parse Tree:
   ```
  └── text: "* a
*

* c
"
   ```

✅ **cm_example_316**: `text`
   Input: `- a
- b

  c
- d
`
   Parse Tree:
   ```
  └── text: "- a
- b

  c
- d
"
   ```

✅ **cm_example_317**: `text`
   Input: `- a
- b

  \[ref\]: /url
- d
`
   Parse Tree:
   ```
  └── text: "- a
- b

  [ref]: /url
- d
"
   ```

✅ **cm_example_318**: `text`
   Input: `- a
- \`\`\`
  b


  \`\`\`
- c
`
   Parse Tree:
   ```
  └── text: "- a
- ```
  b


  ```
- c
"
   ```

✅ **cm_example_319**: `text`
   Input: `- a
  - b

    c
- d
`
   Parse Tree:
   ```
  └── text: "- a
  - b

    c
- d
"
   ```

✅ **cm_example_320**: `text`
   Input: `\* a
  > b
  >
\* c
`
   Parse Tree:
   ```
  └── text: "* a
  > b
  >
* c
"
   ```

✅ **cm_example_321**: `text`
   Input: `- a
  > b
  \`\`\`
  c
  \`\`\`
- d
`
   Parse Tree:
   ```
  └── text: "- a
  > b
  ```
  c
  ```
- d
"
   ```

✅ **cm_example_322**: `text`
   Input: `- a
`
   Parse Tree:
   ```
  └── text: "- a
"
   ```

✅ **cm_example_323**: `text`
   Input: `- a
  - b
`
   Parse Tree:
   ```
  └── text: "- a
  - b
"
   ```

✅ **cm_example_324**: `text`
   Input: `1. \`\`\`
   foo
   \`\`\`

   bar
`
   Parse Tree:
   ```
  └── text: "1. ```
   foo
   ```

   bar
"
   ```

✅ **cm_example_325**: `text`
   Input: `\* foo
  \* bar

  baz
`
   Parse Tree:
   ```
  └── text: "* foo
  * bar

  baz
"
   ```

✅ **cm_example_326**: `text`
   Input: `- a
  - b
  - c

- d
  - e
  - f
`
   Parse Tree:
   ```
  └── text: "- a
  - b
  - c

- d
  - e
  - f
"
   ```

## link_title

✅ **title_double_quotes**: `link_title`
   Input: `"This is a tooltip"`
   Parse Tree:
   ```
  └── link_title: ""This is a tooltip""
   ```

✅ **title_single_quotes**: `link_title`
   Input: `'This is a tooltip'`
   Parse Tree:
   ```
  └── link_title: "'This is a tooltip'"
   ```

✅ **title_with_spaces**: `link_title`
   Input: `"Title with multiple spaces"`
   Parse Tree:
   ```
  └── link_title: ""Title with multiple spaces""
   ```

✅ **title_empty_double**: `link_title`
   Input: `""`
   Parse Tree:
   ```
  └── link_title: """"
   ```

✅ **title_empty_single**: `link_title`
   Input: `''`
   Parse Tree:
   ```
  └── link_title: "''"
   ```

✅ **title_with_quotes**: `link_title`
   Input: `"Title with 'inner quotes'"`
   Parse Tree:
   ```
  └── link_title: ""Title with 'inner quotes'""
   ```

✅ **title_with_apostrophe**: `link_title`
   Input: `'Title with "inner quotes"'`
   Parse Tree:
   ```
  └── link_title: "'Title with "inner quotes"'"
   ```

✅ **title_unicode**: `link_title`
   Input: `"Café ñoño"`
   Parse Tree:
   ```
  └── link_title: ""Café ñoño""
   ```

✅ **title_multiword**: `link_title`
   Input: `"Multiple words in title"`
   Parse Tree:
   ```
  └── link_title: ""Multiple words in title""
   ```

❌ **title_unclosed_double**: `link_title` (Unexpected failure)
   Input: `"unclosed title`
   Error: ` --> 1:1
  |
1 | "unclosed title
  | ^---
  |
  = expected link_title`

❌ **title_unclosed_single**: `link_title` (Unexpected failure)
   Input: `'unclosed title`
   Error: ` --> 1:1
  |
1 | 'unclosed title
  | ^---
  |
  = expected link_title`

❌ **title_mixed_quotes**: `link_title` (Unexpected failure)
   Input: `"mixed quotes'`
   Error: ` --> 1:1
  |
1 | "mixed quotes'
  | ^---
  |
  = expected link_title`

## task_lists

❌ **task_incomplete**: `task_list_item` (Unexpected failure)
   Input: `- \[ \] Todo item`
   Error: ` --> 1:3
  |
1 | - [ ] Todo item
  |   ^---
  |
  = expected task_marker`

✅ **task_complete**: `task_list_item`
   Input: `- \[x\] Done item`
   Parse Tree:
   ```
  ├── task_list_item > "- [x] Done item"
    └── list_marker: "-"
    └── task_marker: "[x]"
    └── list_item_content: "Done item"
   ```

✅ **task_uppercase**: `task_list_item`
   Input: `- \[X\] Also done`
   Parse Tree:
   ```
  ├── task_list_item > "- [X] Also done"
    └── list_marker: "-"
    └── task_marker: "[X]"
    └── list_item_content: "Also done"
   ```

❌ **task_with_meta**: `task_list_item` (Unexpected failure)
   Input: `- \[ \] Task (priority: high)`
   Error: ` --> 1:3
  |
1 | - [ ] Task (priority: high)
  |   ^---
  |
  = expected task_marker`

✅ **task_complete_meta**: `task_list_item`
   Input: `- \[x\] Completed (assignee: john)`
   Parse Tree:
   ```
  ├── task_list_item > "- [x] Completed (assignee: john)"
    └── list_marker: "-"
    └── task_marker: "[x]"
    └── list_item_content: "Completed (assignee: john)"
   ```

✅ **task_no_space**: `task_list_item` (Expected failure)
   Input: `-\[ \] No space`
   Error: ` --> 1:2
  |
1 | -[ ] No space
  |  ^---
  |
  = expected task_marker`

✅ **task_multiple_spaces**: `task_list_item`
   Input: `-   \[x\]   Multiple spaces`
   Parse Tree:
   ```
  ├── task_list_item > "-   [x]   Multiple spaces"
    └── list_marker: "-"
    └── task_marker: "[x]"
    └── list_item_content: "Multiple spaces"
   ```

✅ **task_invalid_marker**: `task_list_item` (Expected failure)
   Input: `- \[?\] Invalid marker`
   Error: ` --> 1:3
  |
1 | - [?] Invalid marker
  |   ^---
  |
  = expected task_marker`

❌ **inline_task_simple**: `inline_task_item` (Unexpected failure)
   Input: `\[ \] Inline task`
   Error: ` --> 1:1
  |
1 | [ ] Inline task
  | ^---
  |
  = expected task_marker`

❌ **inline_task_complete**: `inline_task_item` (Unexpected failure)
   Input: `\[x\] Completed inline`
   Error: ` --> 1:1
  |
1 | [x] Completed inline
  | ^---
  |
  = expected inline_task_item`

❌ **inline_task_with_meta**: `inline_task_item` (Unexpected failure)
   Input: `\[ \] Inline task (due: tomorrow)`
   Error: ` --> 1:1
  |
1 | [ ] Inline task (due: tomorrow)
  | ^---
  |
  = expected task_marker`

## commonmark_textual_content

✅ **cm_example_650**: `text`
   Input: `hello $.;'there
`
   Parse Tree:
   ```
  └── text: "hello $.;'there
"
   ```

✅ **cm_example_651**: `text`
   Input: `Foo χρῆν
`
   Parse Tree:
   ```
  └── text: "Foo χρῆν
"
   ```

✅ **cm_example_652**: `text`
   Input: `Multiple     spaces
`
   Parse Tree:
   ```
  └── text: "Multiple     spaces
"
   ```

## commonmark_list_items

✅ **cm_example_253**: `text`
   Input: `A paragraph
with two lines.

    indented code

> A block quote.
`
   Parse Tree:
   ```
  └── text: "A paragraph
with two lines.

    indented code

> A block quote.
"
   ```

✅ **cm_example_254**: `text`
   Input: `1.  A paragraph
    with two lines.

        indented code

    > A block quote.
`
   Parse Tree:
   ```
  └── text: "1.  A paragraph
    with two lines.

        indented code

    > A block quote.
"
   ```

✅ **cm_example_255**: `text`
   Input: `- one

 two
`
   Parse Tree:
   ```
  └── text: "- one

 two
"
   ```

✅ **cm_example_256**: `text`
   Input: `- one

  two
`
   Parse Tree:
   ```
  └── text: "- one

  two
"
   ```

✅ **cm_example_257**: `text`
   Input: ` -    one

     two
`
   Parse Tree:
   ```
  └── text: " -    one

     two
"
   ```

✅ **cm_example_258**: `text`
   Input: ` -    one

      two
`
   Parse Tree:
   ```
  └── text: " -    one

      two
"
   ```

✅ **cm_example_259**: `text`
   Input: `   > > 1.  one
>>
>>     two
`
   Parse Tree:
   ```
  └── text: "   > > 1.  one
>>
>>     two
"
   ```

✅ **cm_example_260**: `text`
   Input: `>>- one
>>
  >  > two
`
   Parse Tree:
   ```
  └── text: ">>- one
>>
  >  > two
"
   ```

✅ **cm_example_261**: `text`
   Input: `-one

2.two
`
   Parse Tree:
   ```
  └── text: "-one

2.two
"
   ```

✅ **cm_example_262**: `text`
   Input: `- foo


  bar
`
   Parse Tree:
   ```
  └── text: "- foo


  bar
"
   ```

✅ **cm_example_263**: `text`
   Input: `1.  foo

    \`\`\`
    bar
    \`\`\`

    baz

    > bam
`
   Parse Tree:
   ```
  └── text: "1.  foo

    ```
    bar
    ```

    baz

    > bam
"
   ```

✅ **cm_example_264**: `text`
   Input: `- Foo

      bar


      baz
`
   Parse Tree:
   ```
  └── text: "- Foo

      bar


      baz
"
   ```

✅ **cm_example_265**: `text`
   Input: `123456789. ok
`
   Parse Tree:
   ```
  └── text: "123456789. ok
"
   ```

✅ **cm_example_266**: `text`
   Input: `1234567890. not ok
`
   Parse Tree:
   ```
  └── text: "1234567890. not ok
"
   ```

✅ **cm_example_267**: `text`
   Input: `0. ok
`
   Parse Tree:
   ```
  └── text: "0. ok
"
   ```

✅ **cm_example_268**: `text`
   Input: `003. ok
`
   Parse Tree:
   ```
  └── text: "003. ok
"
   ```

✅ **cm_example_269**: `text`
   Input: `-1. not ok
`
   Parse Tree:
   ```
  └── text: "-1. not ok
"
   ```

✅ **cm_example_270**: `text`
   Input: `- foo

      bar
`
   Parse Tree:
   ```
  └── text: "- foo

      bar
"
   ```

✅ **cm_example_271**: `text`
   Input: `  10.  foo

           bar
`
   Parse Tree:
   ```
  └── text: "  10.  foo

           bar
"
   ```

✅ **cm_example_272**: `text`
   Input: `    indented code

paragraph

    more code
`
   Parse Tree:
   ```
  └── text: "    indented code

paragraph

    more code
"
   ```

✅ **cm_example_273**: `text`
   Input: `1.     indented code

   paragraph

       more code
`
   Parse Tree:
   ```
  └── text: "1.     indented code

   paragraph

       more code
"
   ```

✅ **cm_example_274**: `text`
   Input: `1.      indented code

   paragraph

       more code
`
   Parse Tree:
   ```
  └── text: "1.      indented code

   paragraph

       more code
"
   ```

✅ **cm_example_275**: `text`
   Input: `   foo

bar
`
   Parse Tree:
   ```
  └── text: "   foo

bar
"
   ```

✅ **cm_example_276**: `text`
   Input: `-    foo

  bar
`
   Parse Tree:
   ```
  └── text: "-    foo

  bar
"
   ```

✅ **cm_example_277**: `text`
   Input: `-  foo

   bar
`
   Parse Tree:
   ```
  └── text: "-  foo

   bar
"
   ```

✅ **cm_example_278**: `text`
   Input: `-
  foo
-
  \`\`\`
  bar
  \`\`\`
-
      baz
`
   Parse Tree:
   ```
  └── text: "-
  foo
-
  ```
  bar
  ```
-
      baz
"
   ```

✅ **cm_example_279**: `text`
   Input: `-   
  foo
`
   Parse Tree:
   ```
  └── text: "-   
  foo
"
   ```

✅ **cm_example_280**: `text`
   Input: `-

  foo
`
   Parse Tree:
   ```
  └── text: "-

  foo
"
   ```

✅ **cm_example_281**: `text`
   Input: `- foo
-
- bar
`
   Parse Tree:
   ```
  └── text: "- foo
-
- bar
"
   ```

✅ **cm_example_282**: `text`
   Input: `- foo
-   
- bar
`
   Parse Tree:
   ```
  └── text: "- foo
-   
- bar
"
   ```

✅ **cm_example_283**: `text`
   Input: `1. foo
2.
3. bar
`
   Parse Tree:
   ```
  └── text: "1. foo
2.
3. bar
"
   ```

✅ **cm_example_284**: `text`
   Input: `\*
`
   Parse Tree:
   ```
  └── text: "*
"
   ```

✅ **cm_example_285**: `text`
   Input: `foo
\*

foo
1.
`
   Parse Tree:
   ```
  └── text: "foo
*

foo
1.
"
   ```

✅ **cm_example_286**: `text`
   Input: ` 1.  A paragraph
     with two lines.

         indented code

     > A block quote.
`
   Parse Tree:
   ```
  └── text: " 1.  A paragraph
     with two lines.

         indented code

     > A block quote.
"
   ```

✅ **cm_example_287**: `text`
   Input: `  1.  A paragraph
      with two lines.

          indented code

      > A block quote.
`
   Parse Tree:
   ```
  └── text: "  1.  A paragraph
      with two lines.

          indented code

      > A block quote.
"
   ```

✅ **cm_example_288**: `text`
   Input: `   1.  A paragraph
       with two lines.

           indented code

       > A block quote.
`
   Parse Tree:
   ```
  └── text: "   1.  A paragraph
       with two lines.

           indented code

       > A block quote.
"
   ```

✅ **cm_example_289**: `text`
   Input: `    1.  A paragraph
        with two lines.

            indented code

        > A block quote.
`
   Parse Tree:
   ```
  └── text: "    1.  A paragraph
        with two lines.

            indented code

        > A block quote.
"
   ```

✅ **cm_example_290**: `text`
   Input: `  1.  A paragraph
with two lines.

          indented code

      > A block quote.
`
   Parse Tree:
   ```
  └── text: "  1.  A paragraph
with two lines.

          indented code

      > A block quote.
"
   ```

✅ **cm_example_291**: `text`
   Input: `  1.  A paragraph
    with two lines.
`
   Parse Tree:
   ```
  └── text: "  1.  A paragraph
    with two lines.
"
   ```

✅ **cm_example_292**: `text`
   Input: `> 1. > Blockquote
continued here.
`
   Parse Tree:
   ```
  └── text: "> 1. > Blockquote
continued here.
"
   ```

✅ **cm_example_293**: `text`
   Input: `> 1. > Blockquote
> continued here.
`
   Parse Tree:
   ```
  └── text: "> 1. > Blockquote
> continued here.
"
   ```

✅ **cm_example_294**: `text`
   Input: `- foo
  - bar
    - baz
      - boo
`
   Parse Tree:
   ```
  └── text: "- foo
  - bar
    - baz
      - boo
"
   ```

✅ **cm_example_295**: `text`
   Input: `- foo
 - bar
  - baz
   - boo
`
   Parse Tree:
   ```
  └── text: "- foo
 - bar
  - baz
   - boo
"
   ```

✅ **cm_example_296**: `text`
   Input: `10) foo
    - bar
`
   Parse Tree:
   ```
  └── text: "10) foo
    - bar
"
   ```

✅ **cm_example_297**: `text`
   Input: `10) foo
   - bar
`
   Parse Tree:
   ```
  └── text: "10) foo
   - bar
"
   ```

✅ **cm_example_298**: `text`
   Input: `- - foo
`
   Parse Tree:
   ```
  └── text: "- - foo
"
   ```

✅ **cm_example_299**: `text`
   Input: `1. - 2. foo
`
   Parse Tree:
   ```
  └── text: "1. - 2. foo
"
   ```

✅ **cm_example_300**: `text`
   Input: `- # Foo
- Bar
  ---
  baz
`
   Parse Tree:
   ```
  └── text: "- # Foo
- Bar
  ---
  baz
"
   ```

## benchmark_tests

✅ **perf_simple_parse**: `text`
   Input: `Simple text with no formatting`
   Parse Tree:
   ```
  └── text: "Simple text with no formatting"
   ```

✅ **perf_complex_formatting**: `emphasis`
   Input: `\*\*Bold\*\* \*italic\* \`code\` ~~strike~~ ==highlight== ^super^ ˅sub˅`
   Parse Tree:
   ```
  ├── emphasis > "**Bold**"
    ├── bold > "**Bold**"
      └── bold_asterisk: "**Bold**"
   ```

✅ **perf_nested_structures**: `text`
   Input: `> Quote with \*\*bold\*\* and \*italic\*
> 
> Another line`
   Parse Tree:
   ```
  └── text: "> Quote with **bold** and *italic*
> 
> Another line"
   ```

✅ **perf_large_paragraph**: `text`
   Input: `This is a very long paragraph that tests parsing performance with lots of text content that should be parsed efficiently without creating excessive memory allocations or taking too long to process even when the content is quite extensive and contains various types of formatting.`
   Parse Tree:
   ```
  └── text: "This is a very long paragraph that tests parsing performance with lots of text content that should be parsed efficiently without creating excessive memory allocations or taking too long to process even when the content is quite extensive and contains various types of formatting."
   ```

✅ **perf_github_readme**: `document`
   Input: `# Project Title

\[!\[Build Status\](badge.svg)\](link)

## Description

This project does amazing things.

### Installation

\`\`\`bash
npm install
\`\`\`

### Usage

\`\`\`javascript
const lib = require('lib');
\`\`\``
   Parse Tree:
   ```
  ├── document > "# Project Title

[![Build Status](badge.svg)](link)

## Description

This project does amazing things.

### Installation

```bash
npm install
```

### Usage

```javascript
const lib = require('lib');
```"
    ├── block > "# Project Title"
      ├── heading > "# Project Title"
        ├── H1 > "# Project Title"
          ├── heading_content > "Project Title"
            ├── heading_inline > "Project"
              └── word: "Project"
            ├── heading_inline > "Title"
              └── word: "Title"
    ├── block > "[![Build Status](badge.svg)](link)

## Description

This project does amazing things.

### Installation

```bash
npm install
```

### Usage

```javascript
const lib = require('lib');
```"
      ├── paragraph > "[![Build Status](badge.svg)](link)

## Description

This project does amazing things.

### Installation

```bash
npm install
```

### Usage

```javascript
const lib = require('lib');
```"
        ├── paragraph_line > "[![Build Status](badge.svg)](link)

## Description

This project does amazing things.

### Installation

```bash
npm install
```

### Usage

```javascript
const lib = require('lib');
```"
          ├── inline > "[![Build Status](badge.svg)"
            ├── inline_core > "[![Build Status](badge.svg)"
              ├── inline_link > "[![Build Status](badge.svg)"
                └── bracket_link_without_title: "[![Build Status](badge.svg)"
          ├── inline > "](link)

## Description

This project does amazing things.

### Installation

```bash
npm install
```

### Usage

```javascript
const lib = require('lib');
```"
            ├── inline_core > "](link)

## Description

This project does amazing things.

### Installation

```bash
npm install
```

### Usage

```javascript
const lib = require('lib');
```"
              └── text: "](link)

## Description

This project does amazing things.

### Installation

```bash
npm install
```

### Usage

```javascript
const lib = require('lib');
```"
   ```

✅ **perf_academic_paper**: `document`
   Input: `# Abstract

This paper presents novel findings\[^1\].

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

\[^1\]: Important reference here`
   Parse Tree:
   ```
  ├── document > "# Abstract

This paper presents novel findings[^1].

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

[^1]: Important reference here"
    ├── block > "# Abstract"
      ├── heading > "# Abstract"
        ├── H1 > "# Abstract"
          ├── heading_content > "Abstract"
            ├── heading_inline > "Abstract"
              └── word: "Abstract"
    ├── block > "This paper presents novel findings[^1].

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

[^1]: Important reference here"
      ├── paragraph > "This paper presents novel findings[^1].

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

[^1]: Important reference here"
        ├── paragraph_line > "This paper presents novel findings[^1].

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

[^1]: Important reference here"
          ├── inline > "This paper presents novel findings[^1].

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

[^1]: Important reference here"
            ├── inline_core > "This paper presents novel findings[^1].

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

[^1]: Important reference here"
              └── text: "This paper presents novel findings[^1].

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

[^1]: Important reference here"
   ```

✅ **perf_many_small_elements**: `text`
   Input: `\`code1\` \`code2\` \`code3\` \`code4\` \`code5\` \`code6\` \`code7\` \`code8\` \`code9\` \`code10\``
   Parse Tree:
   ```
  └── text: "`code1` `code2` `code3` `code4` `code5` `code6` `code7` `code8` `code9` `code10`"
   ```

✅ **perf_few_large_elements**: `text`
   Input: `\`\`\`
very long code block with lots of content
that spans multiple lines and contains
various programming constructs and
other text that needs to be parsed
efficiently by the parser
\`\`\``
   Parse Tree:
   ```
  └── text: "```
very long code block with lots of content
that spans multiple lines and contains
various programming constructs and
other text that needs to be parsed
efficiently by the parser
```"
   ```

✅ **perf_shallow_wide**: `text`
   Input: `\*\*bold1\*\* \*\*bold2\*\* \*\*bold3\*\* \*\*bold4\*\* \*\*bold5\*\* \*\*bold6\*\* \*\*bold7\*\* \*\*bold8\*\*`
   Parse Tree:
   ```
  └── text: "**bold1** **bold2** **bold3** **bold4** **bold5** **bold6** **bold7** **bold8**"
   ```

✅ **perf_deep_narrow**: `text`
   Input: `\*\*bold \*italic \`code\` italic\* bold\*\*`
   Parse Tree:
   ```
  └── text: "**bold *italic `code` italic* bold**"
   ```

## specification_compliance

✅ **gfm_table_basic**: `table`
   Input: `| foo | bar |
| --- | --- |
| baz | bim |`
   Parse Tree:
   ```
  ├── table > "| foo | bar |
| --- | --- |
| baz | bim |"
    ├── table_header > "| foo | bar |"
      ├── table_cell > "foo "
        ├── table_cell_content > "foo "
          └── table_safe_text: "foo "
      ├── table_cell > "bar "
        ├── table_cell_content > "bar "
          └── table_safe_text: "bar "
      └── table_cell: ""
    ├── table_sep > "| --- | --- |"
      └── table_sep_cell: "--- "
      └── table_sep_cell: "--- "
    ├── table_row > "| baz | bim |"
      ├── table_cell > "baz "
        ├── table_cell_content > "baz "
          └── table_safe_text: "baz "
      ├── table_cell > "bim "
        ├── table_cell_content > "bim "
          └── table_safe_text: "bim "
      └── table_cell: ""
   ```

✅ **gfm_table_alignment**: `table`
   Input: `| left | center | right |
|:-----|:------:|------:|
| L    | C      | R     |`
   Parse Tree:
   ```
  ├── table > "| left | center | right |
|:-----|:------:|------:|
| L    | C      | R     |"
    ├── table_header > "| left | center | right |"
      ├── table_cell > "left "
        ├── table_cell_content > "left "
          └── table_safe_text: "left "
      ├── table_cell > "center "
        ├── table_cell_content > "center "
          └── table_safe_text: "center "
      ├── table_cell > "right "
        ├── table_cell_content > "right "
          └── table_safe_text: "right "
      └── table_cell: ""
    ├── table_sep > "|:-----|:------:|------:|"
      └── table_sep_cell: ":-----"
      └── table_sep_cell: ":------:"
      └── table_sep_cell: "------:"
    ├── table_row > "| L    | C      | R     |"
      ├── table_cell > "L    "
        ├── table_cell_content > "L    "
          └── table_safe_text: "L    "
      ├── table_cell > "C      "
        ├── table_cell_content > "C      "
          └── table_safe_text: "C      "
      ├── table_cell > "R     "
        ├── table_cell_content > "R     "
          └── table_safe_text: "R     "
      └── table_cell: ""
   ```

✅ **gfm_strikethrough**: `strikethrough`
   Input: `~~Hi~~ Hello, world!`
   Parse Tree:
   ```
  ├── strikethrough > "~~Hi~~"
    └── strikethrough_tilde: "~~Hi~~"
   ```

❌ **gfm_autolink_www**: `http_url` (Unexpected failure)
   Input: `www.commonmark.org`
   Error: ` --> 1:1
  |
1 | www.commonmark.org
  | ^---
  |
  = expected http_url`

❌ **gfm_autolink_url**: `http_url` (Unexpected failure)
   Input: `Visit https://github.com`
   Error: ` --> 1:1
  |
1 | Visit https://github.com
  | ^---
  |
  = expected http_url`

✅ **gfm_task_list**: `task_list_item`
   Input: `- \[x\] foo
  - \[ \] bar
  - \[x\] baz
- \[ \] bim`
   Parse Tree:
   ```
  ├── task_list_item > "- [x] foo"
    └── list_marker: "-"
    └── task_marker: "[x]"
    └── list_item_content: "foo"
   ```

❌ **pandoc_subscript**: `subscript` (Unexpected failure)
   Input: `H~2~O`
   Error: ` --> 1:1
  |
1 | H~2~O
  | ^---
  |
  = expected subscript`

❌ **pandoc_superscript**: `superscript` (Unexpected failure)
   Input: `x^2^`
   Error: ` --> 1:1
  |
1 | x^2^
  | ^---
  |
  = expected superscript`

❌ **pandoc_definition_list**: `def_list` (Unexpected failure)
   Input: `Term 1
:   Definition 1

Term 2
:   Definition 2a
:   Definition 2b`
   Error: ` --> 2:1
  |
2 | :   Definition 1
  | ^---
  |
  = expected def_line`

✅ **mmd_table_caption**: `table`
   Input: `| foo | bar |
|-----|-----|
| baz | bim |
\[Table caption\]`
   Parse Tree:
   ```
  ├── table > "| foo | bar |
|-----|-----|
| baz | bim |"
    ├── table_header > "| foo | bar |"
      ├── table_cell > "foo "
        ├── table_cell_content > "foo "
          └── table_safe_text: "foo "
      ├── table_cell > "bar "
        ├── table_cell_content > "bar "
          └── table_safe_text: "bar "
      └── table_cell: ""
    ├── table_sep > "|-----|-----|"
      └── table_sep_cell: "-----"
      └── table_sep_cell: "-----"
    ├── table_row > "| baz | bim |"
      ├── table_cell > "baz "
        ├── table_cell_content > "baz "
          └── table_safe_text: "baz "
      ├── table_cell > "bim "
        ├── table_cell_content > "bim "
          └── table_safe_text: "bim "
      └── table_cell: ""
   ```

✅ **mmd_footnote_inline**: `text`
   Input: `Here is some text^\[and a footnote\]`
   Parse Tree:
   ```
  └── text: "Here is some text^[and a footnote]"
   ```

## headings_setext

✅ **setext_h1_simple**: `setext_h1`
   Input: `Heading
=======`
   Parse Tree:
   ```
  ├── setext_h1 > "Heading
======="
    ├── heading_content > "Heading"
      ├── heading_inline > "Heading"
        └── word: "Heading"
   ```

✅ **setext_h1_uneven**: `setext_h1`
   Input: `Heading
============`
   Parse Tree:
   ```
  ├── setext_h1 > "Heading
============"
    ├── heading_content > "Heading"
      ├── heading_inline > "Heading"
        └── word: "Heading"
   ```

✅ **setext_h1_short**: `setext_h1`
   Input: `Long Heading Text
===`
   Parse Tree:
   ```
  ├── setext_h1 > "Long Heading Text
==="
    ├── heading_content > "Long Heading Text"
      ├── heading_inline > "Long"
        └── word: "Long"
      ├── heading_inline > "Heading"
        └── word: "Heading"
      ├── heading_inline > "Text"
        └── word: "Text"
   ```

✅ **setext_h2_simple**: `setext_h2`
   Input: `Subheading
----------`
   Parse Tree:
   ```
  ├── setext_h2 > "Subheading
----------"
    ├── heading_content > "Subheading"
      ├── heading_inline > "Subheading"
        └── word: "Subheading"
   ```

✅ **setext_h2_uneven**: `setext_h2`
   Input: `Subheading
-----------`
   Parse Tree:
   ```
  ├── setext_h2 > "Subheading
-----------"
    ├── heading_content > "Subheading"
      ├── heading_inline > "Subheading"
        └── word: "Subheading"
   ```

✅ **setext_empty_underline**: `heading` (Expected failure)
   Input: `Heading
`
   Error: ` --> 1:8
  |
1 | Heading␊
  |        ^---
  |
  = expected heading_inline`

✅ **setext_no_text**: `heading` (Expected failure)
   Input: `
======`
   Error: ` --> 1:1
  |
1 | ␊
  | ^---
  |
  = expected heading`

## ordered_lists

✅ **ordered_simple**: `list`
   Input: `1. First item`
   Parse Tree:
   ```
  ├── list > "1. First item"
    ├── list_item > "1. First item"
      ├── regular_list_item > "1. First item"
        └── list_marker: "1."
        └── list_item_content: "First item"
   ```

✅ **ordered_double_digit**: `list`
   Input: `10. Tenth item`
   Parse Tree:
   ```
  ├── list > "10. Tenth item"
    ├── list_item > "10. Tenth item"
      ├── regular_list_item > "10. Tenth item"
        └── list_marker: "10."
        └── list_item_content: "Tenth item"
   ```

✅ **ordered_large_number**: `list`
   Input: `999. Large number`
   Parse Tree:
   ```
  ├── list > "999. Large number"
    ├── list_item > "999. Large number"
      ├── regular_list_item > "999. Large number"
        └── list_marker: "999."
        └── list_item_content: "Large number"
   ```

✅ **ordered_start_5**: `list`
   Input: `5. Fifth item`
   Parse Tree:
   ```
  ├── list > "5. Fifth item"
    ├── list_item > "5. Fifth item"
      ├── regular_list_item > "5. Fifth item"
        └── list_marker: "5."
        └── list_item_content: "Fifth item"
   ```

✅ **ordered_zero**: `list`
   Input: `0. Zero item`
   Parse Tree:
   ```
  ├── list > "0. Zero item"
    ├── list_item > "0. Zero item"
      ├── regular_list_item > "0. Zero item"
        └── list_marker: "0."
        └── list_item_content: "Zero item"
   ```

✅ **mixed_list**: `list`
   Input: `1. Ordered
- Unordered
2. Back to ordered`
   Parse Tree:
   ```
  ├── list > "1. Ordered
- Unordered
2. Back to ordered"
    ├── list_item > "1. Ordered"
      ├── regular_list_item > "1. Ordered"
        └── list_marker: "1."
        └── list_item_content: "Ordered"
    ├── list_item > "- Unordered"
      ├── regular_list_item > "- Unordered"
        └── list_marker: "-"
        └── list_item_content: "Unordered"
    ├── list_item > "2. Back to ordered"
      ├── regular_list_item > "2. Back to ordered"
        └── list_marker: "2."
        └── list_item_content: "Back to ordered"
   ```

## commonmark_thematic_breaks

✅ **cm_example_43**: `text`
   Input: `\*\*\*
---
\_\_\_
`
   Parse Tree:
   ```
  └── text: "***
---
___
"
   ```

✅ **cm_example_44**: `text`
   Input: `+++
`
   Parse Tree:
   ```
  └── text: "+++
"
   ```

✅ **cm_example_45**: `text`
   Input: `===
`
   Parse Tree:
   ```
  └── text: "===
"
   ```

✅ **cm_example_46**: `text`
   Input: `--
\*\*
\_\_
`
   Parse Tree:
   ```
  └── text: "--
**
__
"
   ```

✅ **cm_example_47**: `text`
   Input: ` \*\*\*
  \*\*\*
   \*\*\*
`
   Parse Tree:
   ```
  └── text: " ***
  ***
   ***
"
   ```

✅ **cm_example_48**: `text`
   Input: `    \*\*\*
`
   Parse Tree:
   ```
  └── text: "    ***
"
   ```

✅ **cm_example_49**: `text`
   Input: `Foo
    \*\*\*
`
   Parse Tree:
   ```
  └── text: "Foo
    ***
"
   ```

✅ **cm_example_50**: `text`
   Input: `\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
`
   Parse Tree:
   ```
  └── text: "_____________________________________
"
   ```

✅ **cm_example_51**: `text`
   Input: ` - - -
`
   Parse Tree:
   ```
  └── text: " - - -
"
   ```

✅ **cm_example_52**: `text`
   Input: ` \*\*  \* \*\* \* \*\* \* \*\*
`
   Parse Tree:
   ```
  └── text: " **  * ** * ** * **
"
   ```

✅ **cm_example_53**: `text`
   Input: `-     -      -      -
`
   Parse Tree:
   ```
  └── text: "-     -      -      -
"
   ```

✅ **cm_example_54**: `text`
   Input: `- - - -    
`
   Parse Tree:
   ```
  └── text: "- - - -    
"
   ```

✅ **cm_example_55**: `text`
   Input: `\_ \_ \_ \_ a

a------

---a---
`
   Parse Tree:
   ```
  └── text: "_ _ _ _ a

a------

---a---
"
   ```

✅ **cm_example_56**: `text`
   Input: ` \*-\*
`
   Parse Tree:
   ```
  └── text: " *-*
"
   ```

✅ **cm_example_57**: `text`
   Input: `- foo
\*\*\*
- bar
`
   Parse Tree:
   ```
  └── text: "- foo
***
- bar
"
   ```

✅ **cm_example_58**: `text`
   Input: `Foo
\*\*\*
bar
`
   Parse Tree:
   ```
  └── text: "Foo
***
bar
"
   ```

✅ **cm_example_59**: `text`
   Input: `Foo
---
bar
`
   Parse Tree:
   ```
  └── text: "Foo
---
bar
"
   ```

✅ **cm_example_60**: `text`
   Input: `\* Foo
\* \* \*
\* Bar
`
   Parse Tree:
   ```
  └── text: "* Foo
* * *
* Bar
"
   ```

✅ **cm_example_61**: `text`
   Input: `- Foo
- \* \* \*
`
   Parse Tree:
   ```
  └── text: "- Foo
- * * *
"
   ```

## commonmark_indented_code_blocks

✅ **cm_example_107**: `text`
   Input: `    a simple
      indented code block
`
   Parse Tree:
   ```
  └── text: "    a simple
      indented code block
"
   ```

✅ **cm_example_108**: `text`
   Input: `  - foo

    bar
`
   Parse Tree:
   ```
  └── text: "  - foo

    bar
"
   ```

✅ **cm_example_109**: `text`
   Input: `1.  foo

    - bar
`
   Parse Tree:
   ```
  └── text: "1.  foo

    - bar
"
   ```

✅ **cm_example_110**: `text`
   Input: `    <a/>
    \*hi\*

    - one
`
   Parse Tree:
   ```
  └── text: "    <a/>
    *hi*

    - one
"
   ```

✅ **cm_example_111**: `text`
   Input: `    chunk1

    chunk2
  
 
 
    chunk3
`
   Parse Tree:
   ```
  └── text: "    chunk1

    chunk2
  
 
 
    chunk3
"
   ```

✅ **cm_example_112**: `text`
   Input: `    chunk1
      
      chunk2
`
   Parse Tree:
   ```
  └── text: "    chunk1
      
      chunk2
"
   ```

✅ **cm_example_113**: `text`
   Input: `Foo
    bar

`
   Parse Tree:
   ```
  └── text: "Foo
    bar

"
   ```

✅ **cm_example_114**: `text`
   Input: `    foo
bar
`
   Parse Tree:
   ```
  └── text: "    foo
bar
"
   ```

✅ **cm_example_115**: `text`
   Input: `# Heading
    foo
Heading
------
    foo
----
`
   Parse Tree:
   ```
  └── text: "# Heading
    foo
Heading
------
    foo
----
"
   ```

✅ **cm_example_116**: `text`
   Input: `        foo
    bar
`
   Parse Tree:
   ```
  └── text: "        foo
    bar
"
   ```

✅ **cm_example_117**: `text`
   Input: `
    
    foo
    

`
   Parse Tree:
   ```
  └── text: "
    
    foo
    

"
   ```

✅ **cm_example_118**: `text`
   Input: `    foo  
`
   Parse Tree:
   ```
  └── text: "    foo  
"
   ```

## bold_formatting

✅ **bold_asterisk**: `bold`
   Input: `\*\*bold text\*\*`
   Parse Tree:
   ```
  ├── bold > "**bold text**"
    └── bold_asterisk: "**bold text**"
   ```

✅ **bold_asterisk_with_spaces**: `bold`
   Input: `\*\* spaced bold \*\*`
   Parse Tree:
   ```
  ├── bold > "** spaced bold **"
    └── bold_asterisk: "** spaced bold **"
   ```

✅ **bold_asterisk_empty**: `bold` (Expected failure)
   Input: `\*\*\*\*`
   Error: ` --> 1:1
  |
1 | ****
  | ^---
  |
  = expected bold`

✅ **bold_asterisk_nested**: `bold`
   Input: `\*\*bold with \*\*inner\*\* bold\*\*`
   Parse Tree:
   ```
  ├── bold > "**bold with **"
    └── bold_asterisk: "**bold with **"
   ```

✅ **bold_asterisk_multiline_fail**: `bold`
   Input: `\*\*bold
text\*\*`
   Parse Tree:
   ```
  ├── bold > "**bold"
    └── bold_asterisk: "**bold"
   ```

✅ **bold_underscore**: `bold`
   Input: `\_\_bold text\_\_`
   Parse Tree:
   ```
  ├── bold > "__bold text__"
    └── bold_underscore: "__bold text__"
   ```

✅ **bold_underscore_empty**: `bold` (Expected failure)
   Input: `\_\_\_\_`
   Error: ` --> 1:1
  |
1 | ____
  | ^---
  |
  = expected bold`

✅ **bold_underscore_nested**: `bold`
   Input: `\_\_bold with \_\_inner\_\_ bold\_\_`
   Parse Tree:
   ```
  ├── bold > "__bold with __"
    └── bold_underscore: "__bold with __"
   ```

❌ **bold_single_asterisk**: `bold` (Unexpected failure)
   Input: `\*not bold\*`
   Error: ` --> 1:1
  |
1 | *not bold*
  | ^---
  |
  = expected bold`

✅ **bold_mismatched**: `bold`
   Input: `\*\*bold with underscore\_\_`
   Parse Tree:
   ```
  ├── bold > "**bold with underscore__"
    └── bold_asterisk: "**bold with underscore__"
   ```

✅ **bold_unclosed**: `bold`
   Input: `\*\*missing closing`
   Parse Tree:
   ```
  ├── bold > "**missing closing"
    └── bold_asterisk: "**missing closing"
   ```

## text_and_words

✅ **simple_word**: `word`
   Input: `hello`
   Parse Tree:
   ```
  └── word: "hello"
   ```

✅ **multiple_words**: `word`
   Input: `hello world test`
   Parse Tree:
   ```
  └── word: "hello"
   ```

✅ **with_apostrophe**: `text`
   Input: `can't won't it's`
   Parse Tree:
   ```
  └── text: "can't won't it's"
   ```

✅ **with_hyphens**: `text`
   Input: `well-known state-of-the-art`
   Parse Tree:
   ```
  └── text: "well-known state-of-the-art"
   ```

✅ **unicode_basic**: `text`
   Input: `café résumé naïve`
   Parse Tree:
   ```
  └── text: "café résumé naïve"
   ```

✅ **mixed_scripts**: `text`
   Input: `English 中文 العربية русский 日本語`
   Parse Tree:
   ```
  └── text: "English 中文 العربية русский 日本語"
   ```

✅ **numbers_in_text**: `text`
   Input: `Test 123 numbers`
   Parse Tree:
   ```
  └── text: "Test 123 numbers"
   ```

✅ **math_symbols**: `math_symbol`
   Input: `π ≈ 3.14 ± 0.01`
   Parse Tree:
   ```
  └── math_symbol: "π"
   ```

✅ **all_math_symbols**: `math_symbol`
   Input: `±√∞∑≈≠≤≥∆παβγλμσΩ+=×÷`
   Parse Tree:
   ```
  └── math_symbol: "±"
   ```

✅ **safe_punctuation**: `text`
   Input: `Hello, world! How are you?`
   Parse Tree:
   ```
  └── text: "Hello, world! How are you?"
   ```

✅ **all_punctuation**: `text`
   Input: `!@#$%^&\*()\_+-=\[\]{}|;:,.<>?`
   Parse Tree:
   ```
  └── text: "!@#$%^&*()_+-=[]{}|;:,.<>?"
   ```

✅ **markdown_specials**: `text`
   Input: `\*\_\`#\[\]~>|$@^=-`
   Parse Tree:
   ```
  └── text: "*_`#[]"
   ```

✅ **empty_string**: `text` (Expected failure)
   Input: ``
   Error: ` --> 1:1
  |
1 | 
  | ^---
  |
  = expected text`

✅ **only_spaces**: `text`
   Input: `   `
   Parse Tree:
   ```
  └── text: "   "
   ```

✅ **only_tab**: `text`
   Input: `		`
   Parse Tree:
   ```
  └── text: "		"
   ```

✅ **mixed_whitespace**: `text`
   Input: ` 	 	 `
   Parse Tree:
   ```
  └── text: " 	 	 "
   ```

✅ **very_long_text**: `text`
   Input: `This is a very long text string that should test how the parser handles extended content without any special formatting or markdown syntax just plain text that goes on and on and should continue to parse correctly even with this much content`
   Parse Tree:
   ```
  └── text: "This is a very long text string that should test how the parser handles extended content without any special formatting or markdown syntax just plain text that goes on and on and should continue to parse correctly even with this much content"
   ```

## marco_stress_tests

❌ **nested_admonitions**: `admonition_block` (Unexpected failure)
   Input: `:::
note
Outer note
:::
warning
Inner warning
:::
:::`
   Error: ` --> 1:4
  |
1 | :::␊
  |    ^---
  |
  = expected admonition_type`

✅ **run_multiline_complex**: `run_block_fenced`
   Input: `\`\`\`run@bash
for i in {1..10}; do
  echo "Line $i"
  if \[ $i -eq 5 \]; then
    break
  fi
done
\`\`\``
   Parse Tree:
   ```
  ├── run_block_fenced > "```run@bash
for i in {1..10}; do
  echo "Line $i"
  if [ $i -eq 5 ]; then
    break
  fi
done
```"
    └── KW_RUN: "run@"
    ├── script_type > "bash"
      └── KW_BASH: "bash"
   ```

✅ **user_mention_unicode**: `user_mention`
   Input: `@café\_user \[github.com\](Café User Name)`
   Parse Tree:
   ```
  ├── user_mention > "@café_user "
    └── username: "café_user"
   ```

✅ **user_mention_complex**: `user_mention`
   Input: `@user\_name-123 \[platform.sub.domain\](Very Long Display Name With Symbols!)`
   Parse Tree:
   ```
  ├── user_mention > "@user_name-123 "
    └── username: "user_name-123"
   ```

✅ **tab_with_code**: `text`
   Input: `:::
tab Code Examples
@tab Python
\`\`\`python
print('hello')
\`\`\`
@tab Rust
\`\`\`rust
fn main() {}
\`\`\`
:::`
   Parse Tree:
   ```
  └── text: ":::
tab Code Examples
@tab Python
```python
print('hello')
```
@tab Rust
```rust
fn main() {}
```
:::"
   ```

✅ **bookmark_deep_path**: `bookmark`
   Input: `\[bookmark:section\](./very/deep/nested/folder/structure/file.md=999)`
   Parse Tree:
   ```
  ├── bookmark > "[bookmark:section](./very/deep/nested/folder/structure/file.md=999)"
    └── KW_BOOKMARK: "bookmark"
    └── local_path: "./very/deep/nested/folder/structure/file.md=999"
   ```

✅ **toc_with_doc_complex**: `toc`
   Input: `\[toc=3\](@doc ../../../deep/nested/docs/guide.md)`
   Parse Tree:
   ```
  ├── toc > "[toc=3]"
    └── KW_TOC: "toc"
    └── toc_depth: "=3"
   ```

## commonmark_backslash_escapes

❌ **cm_example_12**: `text` (Unexpected failure)
   Input: `\\!\\"\\#\\$\\%\\&\'\\(\\)\\\*\\+\\,\\-\\.\\/\\:\\;\\<\\=\\>\\?\\@\\\[\\\\\\\]\\^\\\_\\\`\\{\\|\\}\\~
`
   Error: ` --> 1:1
  |
1 | \\!\\"\\#\\$\\%\\&\'\\(\\)\\*\\+\\,\\-\\.\\/\\:\\;\\<\\=\\>\\?\\@\\[\\\\\\]\\^\\_\\`\\{\\|\\}\\~
  | ^---
  |
  = expected text`

❌ **cm_example_13**: `text` (Unexpected failure)
   Input: `\\	\\A\\a\\ \\3\\φ\\«
`
   Error: ` --> 1:1
  |
1 | \\	\\A\\a\\ \\3\\φ\\«
  | ^---
  |
  = expected text`

❌ **cm_example_14**: `text` (Unexpected failure)
   Input: `\\\*not emphasized\*
\\<br/> not a tag
\\\[not a link\](/foo)
\\\`not code\`
1\\. not a list
\\\* not a list
\\# not a heading
\\\[foo\]: /url "not a reference"
\\&ouml; not a character entity
`
   Error: ` --> 1:1
  |
1 | \\*not emphasized*
  | ^---
  |
  = expected text`

❌ **cm_example_15**: `text` (Unexpected failure)
   Input: `\\\\\*emphasis\*
`
   Error: ` --> 1:1
  |
1 | \\\\*emphasis*
  | ^---
  |
  = expected text`

✅ **cm_example_16**: `text`
   Input: `foo\\
bar
`
   Parse Tree:
   ```
  └── text: "foo"
   ```

✅ **cm_example_17**: `text`
   Input: `\`\` \\\[\\\` \`\`
`
   Parse Tree:
   ```
  └── text: "`` "
   ```

✅ **cm_example_18**: `text`
   Input: `    \\\[\\\]
`
   Parse Tree:
   ```
  └── text: "    "
   ```

✅ **cm_example_19**: `text`
   Input: `~~~
\\\[\\\]
~~~
`
   Parse Tree:
   ```
  └── text: "~"
   ```

✅ **cm_example_20**: `text`
   Input: `<http://example.com?find=\\\*>
`
   Parse Tree:
   ```
  └── text: "<http://example.com?find="
   ```

✅ **cm_example_21**: `text`
   Input: `<a href="/bar\\/)">
`
   Parse Tree:
   ```
  └── text: "<a href="/bar"
   ```

✅ **cm_example_22**: `text`
   Input: `\[foo\](/bar\\\* "ti\\\*tle")
`
   Parse Tree:
   ```
  └── text: "[foo](/bar"
   ```

✅ **cm_example_23**: `text`
   Input: `\[foo\]

\[foo\]: /bar\\\* "ti\\\*tle"
`
   Parse Tree:
   ```
  └── text: "[foo]

[foo]: /bar"
   ```

✅ **cm_example_24**: `text`
   Input: `\`\`\` foo\\+bar
foo
\`\`\`
`
   Parse Tree:
   ```
  └── text: "``` foo"
   ```

## commonmark_link_reference_definitions

✅ **cm_example_192**: `text`
   Input: `\[foo\]: /url "title"

\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]: /url "title"

[foo]
"
   ```

✅ **cm_example_193**: `text`
   Input: `   \[foo\]: 
      /url  
           'the title'  

\[foo\]
`
   Parse Tree:
   ```
  └── text: "   [foo]: 
      /url  
           'the title'  

[foo]
"
   ```

✅ **cm_example_194**: `text`
   Input: `\[Foo\*bar\\\]\]:my\_(url) 'title (with parens)'

\[Foo\*bar\\\]\]
`
   Parse Tree:
   ```
  └── text: "[Foo*bar"
   ```

✅ **cm_example_195**: `text`
   Input: `\[Foo bar\]:
<my url>
'title'

\[Foo bar\]
`
   Parse Tree:
   ```
  └── text: "[Foo bar]:
<my url>
'title'

[Foo bar]
"
   ```

✅ **cm_example_196**: `text`
   Input: `\[foo\]: /url '
title
line1
line2
'

\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]: /url '
title
line1
line2
'

[foo]
"
   ```

✅ **cm_example_197**: `text`
   Input: `\[foo\]: /url 'title

with blank line'

\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]: /url 'title

with blank line'

[foo]
"
   ```

✅ **cm_example_198**: `text`
   Input: `\[foo\]:
/url

\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]:
/url

[foo]
"
   ```

✅ **cm_example_199**: `text`
   Input: `\[foo\]:

\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]:

[foo]
"
   ```

✅ **cm_example_200**: `text`
   Input: `\[foo\]: <>

\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]: <>

[foo]
"
   ```

✅ **cm_example_201**: `text`
   Input: `\[foo\]: <bar>(baz)

\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]: <bar>(baz)

[foo]
"
   ```

✅ **cm_example_202**: `text`
   Input: `\[foo\]: /url\\bar\\\*baz "foo\\"bar\\baz"

\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]: /url"
   ```

✅ **cm_example_203**: `text`
   Input: `\[foo\]

\[foo\]: url
`
   Parse Tree:
   ```
  └── text: "[foo]

[foo]: url
"
   ```

✅ **cm_example_204**: `text`
   Input: `\[foo\]

\[foo\]: first
\[foo\]: second
`
   Parse Tree:
   ```
  └── text: "[foo]

[foo]: first
[foo]: second
"
   ```

✅ **cm_example_205**: `text`
   Input: `\[FOO\]: /url

\[Foo\]
`
   Parse Tree:
   ```
  └── text: "[FOO]: /url

[Foo]
"
   ```

✅ **cm_example_206**: `text`
   Input: `\[ΑΓΩ\]: /φου

\[αγω\]
`
   Parse Tree:
   ```
  └── text: "[ΑΓΩ]: /φου

[αγω]
"
   ```

✅ **cm_example_207**: `text`
   Input: `\[foo\]: /url
`
   Parse Tree:
   ```
  └── text: "[foo]: /url
"
   ```

✅ **cm_example_208**: `text`
   Input: `\[
foo
\]: /url
bar
`
   Parse Tree:
   ```
  └── text: "[
foo
]: /url
bar
"
   ```

✅ **cm_example_209**: `text`
   Input: `\[foo\]: /url "title" ok
`
   Parse Tree:
   ```
  └── text: "[foo]: /url "title" ok
"
   ```

✅ **cm_example_210**: `text`
   Input: `\[foo\]: /url
"title" ok
`
   Parse Tree:
   ```
  └── text: "[foo]: /url
"title" ok
"
   ```

✅ **cm_example_211**: `text`
   Input: `    \[foo\]: /url "title"

\[foo\]
`
   Parse Tree:
   ```
  └── text: "    [foo]: /url "title"

[foo]
"
   ```

✅ **cm_example_212**: `text`
   Input: `\`\`\`
\[foo\]: /url
\`\`\`

\[foo\]
`
   Parse Tree:
   ```
  └── text: "```
[foo]: /url
```

[foo]
"
   ```

✅ **cm_example_213**: `text`
   Input: `Foo
\[bar\]: /baz

\[bar\]
`
   Parse Tree:
   ```
  └── text: "Foo
[bar]: /baz

[bar]
"
   ```

✅ **cm_example_214**: `text`
   Input: `# \[Foo\]
\[foo\]: /url
> bar
`
   Parse Tree:
   ```
  └── text: "# [Foo]
[foo]: /url
> bar
"
   ```

✅ **cm_example_215**: `text`
   Input: `\[foo\]: /url
bar
===
\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]: /url
bar
===
[foo]
"
   ```

✅ **cm_example_216**: `text`
   Input: `\[foo\]: /url
===
\[foo\]
`
   Parse Tree:
   ```
  └── text: "[foo]: /url
===
[foo]
"
   ```

✅ **cm_example_217**: `text`
   Input: `\[foo\]: /foo-url "foo"
\[bar\]: /bar-url
  "bar"
\[baz\]: /baz-url

\[foo\],
\[bar\],
\[baz\]
`
   Parse Tree:
   ```
  └── text: "[foo]: /foo-url "foo"
[bar]: /bar-url
  "bar"
[baz]: /baz-url

[foo],
[bar],
[baz]
"
   ```

✅ **cm_example_218**: `text`
   Input: `\[foo\]

> \[foo\]: /url
`
   Parse Tree:
   ```
  └── text: "[foo]

> [foo]: /url
"
   ```

## diagrams

✅ **mermaid_simple**: `diagram_fenced`
   Input: `\`\`\`mermaid
graph TD
A --> B
\`\`\``
   Parse Tree:
   ```
  ├── diagram_fenced > "```mermaid
graph TD
A --> B
```"
    ├── diagram_type > "mermaid"
      └── KW_MERMAID: "mermaid"
   ```

✅ **mermaid_complex**: `diagram_fenced`
   Input: `\`\`\`mermaid
sequenceDiagram
Alice->>Bob: Hello
Bob-->>Alice: Hi
\`\`\``
   Parse Tree:
   ```
  ├── diagram_fenced > "```mermaid
sequenceDiagram
Alice->>Bob: Hello
Bob-->>Alice: Hi
```"
    ├── diagram_type > "mermaid"
      └── KW_MERMAID: "mermaid"
   ```

✅ **graphviz_simple**: `diagram_fenced`
   Input: `\`\`\`graphviz
digraph G {
A -> B
}
\`\`\``
   Parse Tree:
   ```
  ├── diagram_fenced > "```graphviz
digraph G {
A -> B
}
```"
    ├── diagram_type > "graphviz"
      └── KW_GRAPHVIZ: "graphviz"
   ```

✅ **mermaid_upper**: `diagram_fenced`
   Input: `\`\`\`MERMAID
graph LR
A --> B
\`\`\``
   Parse Tree:
   ```
  ├── diagram_fenced > "```MERMAID
graph LR
A --> B
```"
    ├── diagram_type > "MERMAID"
      └── KW_MERMAID: "MERMAID"
   ```

## performance_tests

✅ **backtrack_emphasis**: `text`
   Input: `\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*`
   Parse Tree:
   ```
  └── text: "*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*"
   ```

✅ **backtrack_links**: `text`
   Input: `\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[not a link\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]`
   Parse Tree:
   ```
  └── text: "[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[not a link]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]"
   ```

✅ **backtrack_code**: `text`
   Input: `\`\`\`
\`\`\`
\`\`\`
\`\`\`
\`\`\`
\`\`\`
\`\`\`
\`\`\`
\`\`\`
\`\`\`
\`\`\`
\`\`\`
\`\`\``
   Parse Tree:
   ```
  └── text: "```
```
```
```
```
```
```
```
```
```
```
```
```"
   ```

✅ **large_table**: `table`
   Input: `| A | B | C | D | E | F | G | H |
|---|---|---|---|---|---|---|---|
| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
| 9 | 10| 11| 12| 13| 14| 15| 16|
| 17| 18| 19| 20| 21| 22| 23| 24|`
   Parse Tree:
   ```
  ├── table > "| A | B | C | D | E | F | G | H |
|---|---|---|---|---|---|---|---|
| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
| 9 | 10| 11| 12| 13| 14| 15| 16|
| 17| 18| 19| 20| 21| 22| 23| 24|"
    ├── table_header > "| A | B | C | D | E | F | G | H |"
      ├── table_cell > "A "
        ├── table_cell_content > "A "
          └── table_safe_text: "A "
      ├── table_cell > "B "
        ├── table_cell_content > "B "
          └── table_safe_text: "B "
      ├── table_cell > "C "
        ├── table_cell_content > "C "
          └── table_safe_text: "C "
      ├── table_cell > "D "
        ├── table_cell_content > "D "
          └── table_safe_text: "D "
      ├── table_cell > "E "
        ├── table_cell_content > "E "
          └── table_safe_text: "E "
      ├── table_cell > "F "
        ├── table_cell_content > "F "
          └── table_safe_text: "F "
      ├── table_cell > "G "
        ├── table_cell_content > "G "
          └── table_safe_text: "G "
      ├── table_cell > "H "
        ├── table_cell_content > "H "
          └── table_safe_text: "H "
      └── table_cell: ""
    ├── table_sep > "|---|---|---|---|---|---|---|---|"
      └── table_sep_cell: "---"
      └── table_sep_cell: "---"
      └── table_sep_cell: "---"
      └── table_sep_cell: "---"
      └── table_sep_cell: "---"
      └── table_sep_cell: "---"
      └── table_sep_cell: "---"
      └── table_sep_cell: "---"
    ├── table_row > "| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |"
      ├── table_cell > "1 "
        ├── table_cell_content > "1 "
          └── table_safe_text: "1 "
      ├── table_cell > "2 "
        ├── table_cell_content > "2 "
          └── table_safe_text: "2 "
      ├── table_cell > "3 "
        ├── table_cell_content > "3 "
          └── table_safe_text: "3 "
      ├── table_cell > "4 "
        ├── table_cell_content > "4 "
          └── table_safe_text: "4 "
      ├── table_cell > "5 "
        ├── table_cell_content > "5 "
          └── table_safe_text: "5 "
      ├── table_cell > "6 "
        ├── table_cell_content > "6 "
          └── table_safe_text: "6 "
      ├── table_cell > "7 "
        ├── table_cell_content > "7 "
          └── table_safe_text: "7 "
      ├── table_cell > "8 "
        ├── table_cell_content > "8 "
          └── table_safe_text: "8 "
      └── table_cell: ""
    ├── table_row > "| 9 | 10| 11| 12| 13| 14| 15| 16|"
      ├── table_cell > "9 "
        ├── table_cell_content > "9 "
          └── table_safe_text: "9 "
      ├── table_cell > "10"
        ├── table_cell_content > "10"
          └── table_safe_text: "10"
      ├── table_cell > "11"
        ├── table_cell_content > "11"
          └── table_safe_text: "11"
      ├── table_cell > "12"
        ├── table_cell_content > "12"
          └── table_safe_text: "12"
      ├── table_cell > "13"
        ├── table_cell_content > "13"
          └── table_safe_text: "13"
      ├── table_cell > "14"
        ├── table_cell_content > "14"
          └── table_safe_text: "14"
      ├── table_cell > "15"
        ├── table_cell_content > "15"
          └── table_safe_text: "15"
      ├── table_cell > "16"
        ├── table_cell_content > "16"
          └── table_safe_text: "16"
      └── table_cell: ""
    ├── table_row > "| 17| 18| 19| 20| 21| 22| 23| 24|"
      ├── table_cell > "17"
        ├── table_cell_content > "17"
          └── table_safe_text: "17"
      ├── table_cell > "18"
        ├── table_cell_content > "18"
          └── table_safe_text: "18"
      ├── table_cell > "19"
        ├── table_cell_content > "19"
          └── table_safe_text: "19"
      ├── table_cell > "20"
        ├── table_cell_content > "20"
          └── table_safe_text: "20"
      ├── table_cell > "21"
        ├── table_cell_content > "21"
          └── table_safe_text: "21"
      ├── table_cell > "22"
        ├── table_cell_content > "22"
          └── table_safe_text: "22"
      ├── table_cell > "23"
        ├── table_cell_content > "23"
          └── table_safe_text: "23"
      ├── table_cell > "24"
        ├── table_cell_content > "24"
          └── table_safe_text: "24"
      └── table_cell: ""
   ```

❌ **many_footnotes**: `footnote_ref` (Unexpected failure)
   Input: `Text\[^1\] more\[^2\] text\[^3\] here\[^4\] and\[^5\] there\[^6\] everywhere\[^7\]`
   Error: ` --> 1:1
  |
1 | Text[^1] more[^2] text[^3] here[^4] and[^5] there[^6] everywhere[^7]
  | ^---
  |
  = expected footnote_ref`

✅ **recursive_refs**: `reference_link`
   Input: `\[foo\]\[bar\]
\[bar\]\[baz\]
\[baz\]\[foo\]`
   Parse Tree:
   ```
  ├── reference_link > "[foo][bar]"
    └── block_caption: "foo"
    └── block_caption: "bar"
   ```

## bookmarks

✅ **bookmark_simple**: `bookmark`
   Input: `\[bookmark:section\](./file.md)`
   Parse Tree:
   ```
  ├── bookmark > "[bookmark:section](./file.md)"
    └── KW_BOOKMARK: "bookmark"
    └── local_path: "./file.md"
   ```

✅ **bookmark_with_line**: `bookmark`
   Input: `\[bookmark:function\](./code.rs=42)`
   Parse Tree:
   ```
  ├── bookmark > "[bookmark:function](./code.rs=42)"
    └── KW_BOOKMARK: "bookmark"
    └── local_path: "./code.rs=42"
   ```

✅ **bookmark_complex**: `bookmark`
   Input: `\[bookmark:important-section\](../docs/guide.md=123)`
   Parse Tree:
   ```
  ├── bookmark > "[bookmark:important-section](../docs/guide.md=123)"
    └── KW_BOOKMARK: "bookmark"
    └── local_path: "../docs/guide.md=123"
   ```

✅ **bookmark_no_path**: `bookmark` (Expected failure)
   Input: `\[bookmark:section\]`
   Error: ` --> 1:1
  |
1 | [bookmark:section]
  | ^---
  |
  = expected bookmark`

✅ **bookmark_empty**: `bookmark` (Expected failure)
   Input: `\[bookmark:\]`
   Error: ` --> 1:1
  |
1 | [bookmark:]
  | ^---
  |
  = expected bookmark`

## pathological_inputs

✅ **deeply_nested_quotes**: `blockquote`
   Input: `> > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > Deep`
   Parse Tree:
   ```
  ├── blockquote > "> > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > Deep"
    ├── blockquote_line > "> > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > Deep"
      ├── inline > "> > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > Deep"
        ├── inline_core > "> > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > Deep"
          └── text: "> > > > > > > > > > > > > > > > > > > > > > > > > > > > > > > Deep"
   ```

✅ **deeply_nested_lists**: `list`
   Input: `- Level 1
  - Level 2
    - Level 3
      - Level 4
        - Level 5
          - Level 6
            - Level 7
              - Level 8`
   Parse Tree:
   ```
  ├── list > "- Level 1
  - Level 2
    - Level 3
      - Level 4
        - Level 5
          - Level 6
            - Level 7
              - Level 8"
    ├── list_item > "- Level 1"
      ├── regular_list_item > "- Level 1"
        └── list_marker: "-"
        └── list_item_content: "Level 1"
    ├── list_item > "- Level 2"
      ├── regular_list_item > "- Level 2"
        └── list_marker: "-"
        └── list_item_content: "Level 2"
    ├── list_item > "- Level 3"
      ├── regular_list_item > "- Level 3"
        └── list_marker: "-"
        └── list_item_content: "Level 3"
    ├── list_item > "- Level 4"
      ├── regular_list_item > "- Level 4"
        └── list_marker: "-"
        └── list_item_content: "Level 4"
    ├── list_item > "- Level 5"
      ├── regular_list_item > "- Level 5"
        └── list_marker: "-"
        └── list_item_content: "Level 5"
    ├── list_item > "- Level 6"
      ├── regular_list_item > "- Level 6"
        └── list_marker: "-"
        └── list_item_content: "Level 6"
    ├── list_item > "- Level 7"
      ├── regular_list_item > "- Level 7"
        └── list_marker: "-"
        └── list_item_content: "Level 7"
    ├── list_item > "- Level 8"
      ├── regular_list_item > "- Level 8"
        └── list_marker: "-"
        └── list_item_content: "Level 8"
   ```

✅ **deeply_nested_emphasis**: `emphasis`
   Input: `\*\*bold \*italic \*\*bold \_italic\_ bold\*\* italic\* bold\*\*`
   Parse Tree:
   ```
  ├── emphasis > "**bold *italic **"
    ├── bold > "**bold *italic **"
      └── bold_asterisk: "**bold *italic **"
   ```

✅ **extremely_long_line**: `text`
   Input: `Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.`
   Parse Tree:
   ```
  └── text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
   ```

✅ **many_emphasis_markers**: `text`
   Input: `\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*`
   Parse Tree:
   ```
  └── text: "*****************************************************************************"
   ```

✅ **alternating_chars**: `text`
   Input: `\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*`
   Parse Tree:
   ```
  └── text: "*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*"
   ```

✅ **quadratic_blowup**: `text`
   Input: `\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[`
   Parse Tree:
   ```
  └── text: "[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[["
   ```

✅ **mixed_line_endings_complex**: `text`
   Input: `Line 1\r
Line 2
Line 3\r
Line 4
`
   Parse Tree:
   ```
  └── text: "Line 1"
   ```

❌ **binary_like_data**: `text` (Unexpected failure)
   Input: `\\u0000\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\u0008\\u0009\
\\u000B\\u000C\\r\\u000E\\u000F`
   Error: ` --> 1:1
  |
1 | \\u0000\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\u0008\\u0009\
  | ^---
  |
  = expected text`

✅ **massive_nested_brackets**: `text`
   Input: `\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]`
   Parse Tree:
   ```
  └── text: "[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]"
   ```

## unordered_lists

✅ **list_dash**: `list`
   Input: `- Item 1`
   Parse Tree:
   ```
  ├── list > "- Item 1"
    ├── list_item > "- Item 1"
      ├── regular_list_item > "- Item 1"
        └── list_marker: "-"
        └── list_item_content: "Item 1"
   ```

✅ **list_asterisk**: `list`
   Input: `\* Item 2`
   Parse Tree:
   ```
  ├── list > "* Item 2"
    ├── list_item > "* Item 2"
      ├── regular_list_item > "* Item 2"
        └── list_marker: "*"
        └── list_item_content: "Item 2"
   ```

✅ **list_plus**: `list`
   Input: `+ Item 3`
   Parse Tree:
   ```
  ├── list > "+ Item 3"
    ├── list_item > "+ Item 3"
      ├── regular_list_item > "+ Item 3"
        └── list_marker: "+"
        └── list_item_content: "Item 3"
   ```

✅ **list_nested**: `list`
   Input: `- Level 1
  - Level 2
    - Level 3`
   Parse Tree:
   ```
  ├── list > "- Level 1
  - Level 2
    - Level 3"
    ├── list_item > "- Level 1"
      ├── regular_list_item > "- Level 1"
        └── list_marker: "-"
        └── list_item_content: "Level 1"
    ├── list_item > "- Level 2"
      ├── regular_list_item > "- Level 2"
        └── list_marker: "-"
        └── list_item_content: "Level 2"
    ├── list_item > "- Level 3"
      ├── regular_list_item > "- Level 3"
        └── list_marker: "-"
        └── list_item_content: "Level 3"
   ```

✅ **list_mixed_markers**: `list`
   Input: `- Item 1
\* Item 2
+ Item 3`
   Parse Tree:
   ```
  ├── list > "- Item 1
* Item 2
+ Item 3"
    ├── list_item > "- Item 1"
      ├── regular_list_item > "- Item 1"
        └── list_marker: "-"
        └── list_item_content: "Item 1"
    ├── list_item > "* Item 2"
      ├── regular_list_item > "* Item 2"
        └── list_marker: "*"
        └── list_item_content: "Item 2"
    ├── list_item > "+ Item 3"
      ├── regular_list_item > "+ Item 3"
        └── list_marker: "+"
        └── list_item_content: "Item 3"
   ```

✅ **list_with_formatting**: `list`
   Input: `- \*\*Bold item\*\*`
   Parse Tree:
   ```
  ├── list > "- **Bold item**"
    ├── list_item > "- **Bold item**"
      ├── regular_list_item > "- **Bold item**"
        └── list_marker: "-"
        └── list_item_content: "**Bold item**"
   ```

✅ **list_with_links**: `list`
   Input: `- \[Link item\](https://example.com)`
   Parse Tree:
   ```
  ├── list > "- [Link item](https://example.com)"
    ├── list_item > "- [Link item](https://example.com)"
      ├── regular_list_item > "- [Link item](https://example.com)"
        └── list_marker: "-"
        └── list_item_content: "[Link item](https://example.com)"
   ```

✅ **list_with_code**: `list`
   Input: `- Item with \`code\``
   Parse Tree:
   ```
  ├── list > "- Item with `code`"
    ├── list_item > "- Item with `code`"
      ├── regular_list_item > "- Item with `code`"
        └── list_marker: "-"
        └── list_item_content: "Item with `code`"
   ```

✅ **list_empty_item**: `list` (Expected failure)
   Input: `-`
   Error: ` --> 1:2
  |
1 | -
  |  ^---
  |
  = expected task_marker`

✅ **list_only_spaces**: `list`
   Input: `-   `
   Parse Tree:
   ```
  ├── list > "-   "
    ├── list_item > "-   "
      ├── regular_list_item > "-   "
        └── list_marker: "-"
        └── list_item_content: "  "
   ```

✅ **list_multiline_item**: `list`
   Input: `- This is a very long list item that spans multiple lines and should still be parsed correctly`
   Parse Tree:
   ```
  ├── list > "- This is a very long list item that spans multiple lines and should still be parsed correctly"
    ├── list_item > "- This is a very long list item that spans multiple lines and should still be parsed correctly"
      ├── regular_list_item > "- This is a very long list item that spans multiple lines and should still be parsed correctly"
        └── list_marker: "-"
        └── list_item_content: "This is a very long list item that spans multiple lines and should still be parsed correctly"
   ```

## failure_cases

✅ **malformed_link**: `inline_link` (Expected failure)
   Input: `\[text(missing closing bracket`
   Error: ` --> 1:1
  |
1 | [text(missing closing bracket
  | ^---
  |
  = expected inline_link`

✅ **malformed_image**: `inline_image` (Expected failure)
   Input: `!\[alt(missing closing bracket`
   Error: ` --> 1:1
  |
1 | ![alt(missing closing bracket
  | ^---
  |
  = expected inline_image`

✅ **malformed_bold**: `bold`
   Input: `\*\*missing closing`
   Parse Tree:
   ```
  ├── bold > "**missing closing"
    └── bold_asterisk: "**missing closing"
   ```

✅ **malformed_italic**: `italic`
   Input: `\*missing closing`
   Parse Tree:
   ```
  ├── italic > "*missing closing"
    └── italic_asterisk: "*missing closing"
   ```

✅ **malformed_code**: `code_inline` (Expected failure)
   Input: `\`missing closing`
   Error: ` --> 1:1
  |
1 | `missing closing
  | ^---
  |
  = expected code_inline`

✅ **malformed_math**: `math_inline` (Expected failure)
   Input: `$missing closing`
   Error: ` --> 1:1
  |
1 | $missing closing
  | ^---
  |
  = expected math_inline`

✅ **malformed_emoji**: `emoji` (Expected failure)
   Input: `:missing closing`
   Error: ` --> 1:1
  |
1 | :missing closing
  | ^---
  |
  = expected emoji`

✅ **malformed_html**: `inline_html` (Expected failure)
   Input: `<unclosed tag`
   Error: ` --> 1:1
  |
1 | <unclosed tag
  | ^---
  |
  = expected inline_html`

✅ **malformed_comment**: `text`
   Input: `<!-- unclosed comment`
   Parse Tree:
   ```
  └── text: "<!-- unclosed comment"
   ```

✅ **invalid_heading**: `heading` (Expected failure)
   Input: `############ too many hashes`
   Error: ` --> 1:7
  |
1 | ############ too many hashes
  |       ^---
  |
  = expected heading_inline`

✅ **invalid_list_marker**: `text`
   Input: `? Not a list`
   Parse Tree:
   ```
  └── text: "? Not a list"
   ```

✅ **invalid_table**: `table` (Expected failure)
   Input: `| A | B |
| 1 | 2 | 3 |`
   Error: ` --> 2:3
  |
2 | | 1 | 2 | 3 |
  |   ^---
  |
  = expected table_sep_cell`

✅ **invalid_footnote**: `footnote_ref` (Expected failure)
   Input: `\[^invalid label with spaces\]`
   Error: ` --> 1:1
  |
1 | [^invalid label with spaces]
  | ^---
  |
  = expected footnote_ref`

✅ **invalid_reference**: `text`
   Input: `\[ref with spaces\]: url`
   Parse Tree:
   ```
  └── text: "[ref with spaces]: url"
   ```

✅ **nested_conflict_1**: `text`
   Input: `\*\*bold with \`code\*\* inside\``
   Parse Tree:
   ```
  └── text: "**bold with `code** inside`"
   ```

✅ **nested_conflict_2**: `text`
   Input: `\*italic with \*\*bold\* text\*\*`
   Parse Tree:
   ```
  └── text: "*italic with **bold* text**"
   ```

✅ **nested_conflict_3**: `text`
   Input: `~~strike with \*\*bold~~ text\*\*`
   Parse Tree:
   ```
  └── text: "~"
   ```

✅ **invalid_url_scheme**: `text`
   Input: `ftp://not.supported.com`
   Parse Tree:
   ```
  └── text: "ftp://not.supported.com"
   ```

✅ **malformed_url**: `text`
   Input: `https://.`
   Parse Tree:
   ```
  └── text: "https://."
   ```

✅ **empty_url_parts**: `text`
   Input: `https:///empty/authority`
   Parse Tree:
   ```
  └── text: "https:///empty/authority"
   ```

✅ **invalid_admonition_type**: `admonition_block` (Expected failure)
   Input: `:::
custom\_type
content
:::`
   Error: ` --> 1:4
  |
1 | :::␊
  |    ^---
  |
  = expected admonition_type`

✅ **malformed_user_mention**: `user_mention`
   Input: `@user \[platform`
   Parse Tree:
   ```
  ├── user_mention > "@user "
    └── username: "user"
   ```

✅ **invalid_script_type**: `run_inline` (Expected failure)
   Input: `run@invalid\_shell(command)`
   Error: ` --> 1:5
  |
1 | run@invalid_shell(command)
  |     ^---
  |
  = expected script_type`

✅ **malformed_bookmark**: `bookmark` (Expected failure)
   Input: `\[bookmark\](no\_colon)`
   Error: ` --> 1:1
  |
1 | [bookmark](no_colon)
  | ^---
  |
  = expected bookmark`

✅ **invalid_utf8**: `text`
   Input: `text with invalid utf8 bytes`
   Parse Tree:
   ```
  └── text: "text with invalid utf8 bytes"
   ```

✅ **null_bytes**: `text`
   Input: `text with null bytes`
   Parse Tree:
   ```
  └── text: "text with null bytes"
   ```

✅ **control_chars**: `text`
   Input: `text with control chars`
   Parse Tree:
   ```
  └── text: "text with control chars"
   ```

## code_blocks

✅ **fenced_simple**: `fenced_code`
   Input: `\`\`\`
code here
\`\`\``
   Parse Tree:
   ```
  └── fenced_code: "```
code here
```"
   ```

✅ **fenced_with_lang**: `fenced_code`
   Input: `\`\`\`rust
fn main() {}
\`\`\``
   Parse Tree:
   ```
  ├── fenced_code > "```rust
fn main() {}
```"
    └── language_id: "rust"
   ```

✅ **fenced_python**: `fenced_code`
   Input: `\`\`\`python
print('hello')
\`\`\``
   Parse Tree:
   ```
  ├── fenced_code > "```python
print('hello')
```"
    └── language_id: "python"
   ```

✅ **fenced_empty**: `fenced_code`
   Input: `\`\`\`

\`\`\``
   Parse Tree:
   ```
  └── fenced_code: "```

```"
   ```

✅ **fenced_no_lang**: `fenced_code`
   Input: `\`\`\`
some code
more code
\`\`\``
   Parse Tree:
   ```
  └── fenced_code: "```
some code
more code
```"
   ```

✅ **fenced_with_backticks**: `fenced_code`
   Input: `\`\`\`
code with \`\`\` inside
\`\`\``
   Parse Tree:
   ```
  └── fenced_code: "```
code with ```"
   ```

✅ **fenced_multiline**: `fenced_code`
   Input: `\`\`\`rust
fn main() {
    println!("hello");
}
\`\`\``
   Parse Tree:
   ```
  ├── fenced_code > "```rust
fn main() {
    println!("hello");
}
```"
    └── language_id: "rust"
   ```

❌ **fenced_unclosed**: `fenced_code` (Unexpected failure)
   Input: `\`\`\`
code without closing`
   Error: ` --> 1:4
  |
1 | ```␊
  |    ^---
  |
  = expected language_id`

❌ **fenced_wrong_close**: `fenced_code` (Unexpected failure)
   Input: `\`\`\`
code
\`\``
   Error: ` --> 1:4
  |
1 | ```␊
  |    ^---
  |
  = expected language_id`

## commonmark_autolinks

✅ **cm_example_593**: `text`
   Input: `<http://foo.bar.baz>
`
   Parse Tree:
   ```
  └── text: "<http://foo.bar.baz>
"
   ```

✅ **cm_example_594**: `text`
   Input: `<http://foo.bar.baz/test?q=hello&id=22&boolean>
`
   Parse Tree:
   ```
  └── text: "<http://foo.bar.baz/test?q=hello&id=22&boolean>
"
   ```

✅ **cm_example_595**: `text`
   Input: `<irc://foo.bar:2233/baz>
`
   Parse Tree:
   ```
  └── text: "<irc://foo.bar:2233/baz>
"
   ```

✅ **cm_example_596**: `text`
   Input: `<MAILTO:FOO@BAR.BAZ>
`
   Parse Tree:
   ```
  └── text: "<MAILTO:FOO@BAR.BAZ>
"
   ```

✅ **cm_example_597**: `text`
   Input: `<a+b+c:d>
`
   Parse Tree:
   ```
  └── text: "<a+b+c:d>
"
   ```

✅ **cm_example_598**: `text`
   Input: `<made-up-scheme://foo,bar>
`
   Parse Tree:
   ```
  └── text: "<made-up-scheme://foo,bar>
"
   ```

✅ **cm_example_599**: `text`
   Input: `<http://../>
`
   Parse Tree:
   ```
  └── text: "<http://../>
"
   ```

✅ **cm_example_600**: `text`
   Input: `<localhost:5001/foo>
`
   Parse Tree:
   ```
  └── text: "<localhost:5001/foo>
"
   ```

✅ **cm_example_601**: `text`
   Input: `<http://foo.bar/baz bim>
`
   Parse Tree:
   ```
  └── text: "<http://foo.bar/baz bim>
"
   ```

✅ **cm_example_602**: `text`
   Input: `<http://example.com/\\\[\\>
`
   Parse Tree:
   ```
  └── text: "<http://example.com/"
   ```

✅ **cm_example_603**: `text`
   Input: `<foo@bar.example.com>
`
   Parse Tree:
   ```
  └── text: "<foo@bar.example.com>
"
   ```

✅ **cm_example_604**: `text`
   Input: `<foo+special@Bar.baz-bar0.com>
`
   Parse Tree:
   ```
  └── text: "<foo+special@Bar.baz-bar0.com>
"
   ```

✅ **cm_example_605**: `text`
   Input: `<foo\\+@bar.example.com>
`
   Parse Tree:
   ```
  └── text: "<foo"
   ```

✅ **cm_example_606**: `text`
   Input: `<>
`
   Parse Tree:
   ```
  └── text: "<>
"
   ```

✅ **cm_example_607**: `text`
   Input: `< http://foo.bar >
`
   Parse Tree:
   ```
  └── text: "< http://foo.bar >
"
   ```

✅ **cm_example_608**: `text`
   Input: `<m:abc>
`
   Parse Tree:
   ```
  └── text: "<m:abc>
"
   ```

✅ **cm_example_609**: `text`
   Input: `<foo.bar.baz>
`
   Parse Tree:
   ```
  └── text: "<foo.bar.baz>
"
   ```

✅ **cm_example_610**: `text`
   Input: `http://example.com
`
   Parse Tree:
   ```
  └── text: "http://example.com
"
   ```

✅ **cm_example_611**: `text`
   Input: `foo@bar.example.com
`
   Parse Tree:
   ```
  └── text: "foo@bar.example.com
"
   ```

## commonmark_emphasis_and_strong_emphasis

✅ **cm_example_350**: `text`
   Input: `\*foo bar\*
`
   Parse Tree:
   ```
  └── text: "*foo bar*
"
   ```

✅ **cm_example_351**: `text`
   Input: `a \* foo bar\*
`
   Parse Tree:
   ```
  └── text: "a * foo bar*
"
   ```

✅ **cm_example_352**: `text`
   Input: `a\*"foo"\*
`
   Parse Tree:
   ```
  └── text: "a*"foo"*
"
   ```

✅ **cm_example_353**: `text`
   Input: `\*a\*
`
   Parse Tree:
   ```
  └── text: "*a*
"
   ```

✅ **cm_example_354**: `text`
   Input: `foo\*bar\*
`
   Parse Tree:
   ```
  └── text: "foo*bar*
"
   ```

✅ **cm_example_355**: `text`
   Input: `5\*6\*78
`
   Parse Tree:
   ```
  └── text: "5*6*78
"
   ```

✅ **cm_example_356**: `text`
   Input: `\_foo bar\_
`
   Parse Tree:
   ```
  └── text: "_foo bar_
"
   ```

✅ **cm_example_357**: `text`
   Input: `\_ foo bar\_
`
   Parse Tree:
   ```
  └── text: "_ foo bar_
"
   ```

✅ **cm_example_358**: `text`
   Input: `a\_"foo"\_
`
   Parse Tree:
   ```
  └── text: "a_"foo"_
"
   ```

✅ **cm_example_359**: `text`
   Input: `foo\_bar\_
`
   Parse Tree:
   ```
  └── text: "foo_bar_
"
   ```

✅ **cm_example_360**: `text`
   Input: `5\_6\_78
`
   Parse Tree:
   ```
  └── text: "5_6_78
"
   ```

✅ **cm_example_361**: `text`
   Input: `пристаням\_стремятся\_
`
   Parse Tree:
   ```
  └── text: "пристаням_стремятся_
"
   ```

✅ **cm_example_362**: `text`
   Input: `aa\_"bb"\_cc
`
   Parse Tree:
   ```
  └── text: "aa_"bb"_cc
"
   ```

✅ **cm_example_363**: `text`
   Input: `foo-\_(bar)\_
`
   Parse Tree:
   ```
  └── text: "foo-_(bar)_
"
   ```

✅ **cm_example_364**: `text`
   Input: `\_foo\*
`
   Parse Tree:
   ```
  └── text: "_foo*
"
   ```

✅ **cm_example_365**: `text`
   Input: `\*foo bar \*
`
   Parse Tree:
   ```
  └── text: "*foo bar *
"
   ```

✅ **cm_example_366**: `text`
   Input: `\*foo bar
\*
`
   Parse Tree:
   ```
  └── text: "*foo bar
*
"
   ```

✅ **cm_example_367**: `text`
   Input: `\*(\*foo)
`
   Parse Tree:
   ```
  └── text: "*(*foo)
"
   ```

✅ **cm_example_368**: `text`
   Input: `\*(\*foo\*)\*
`
   Parse Tree:
   ```
  └── text: "*(*foo*)*
"
   ```

✅ **cm_example_369**: `text`
   Input: `\*foo\*bar
`
   Parse Tree:
   ```
  └── text: "*foo*bar
"
   ```

✅ **cm_example_370**: `text`
   Input: `\_foo bar \_
`
   Parse Tree:
   ```
  └── text: "_foo bar _
"
   ```

✅ **cm_example_371**: `text`
   Input: `\_(\_foo)
`
   Parse Tree:
   ```
  └── text: "_(_foo)
"
   ```

✅ **cm_example_372**: `text`
   Input: `\_(\_foo\_)\_
`
   Parse Tree:
   ```
  └── text: "_(_foo_)_
"
   ```

✅ **cm_example_373**: `text`
   Input: `\_foo\_bar
`
   Parse Tree:
   ```
  └── text: "_foo_bar
"
   ```

✅ **cm_example_374**: `text`
   Input: `\_пристаням\_стремятся
`
   Parse Tree:
   ```
  └── text: "_пристаням_стремятся
"
   ```

✅ **cm_example_375**: `text`
   Input: `\_foo\_bar\_baz\_
`
   Parse Tree:
   ```
  └── text: "_foo_bar_baz_
"
   ```

✅ **cm_example_376**: `text`
   Input: `\_(bar)\_.
`
   Parse Tree:
   ```
  └── text: "_(bar)_.
"
   ```

✅ **cm_example_377**: `text`
   Input: `\*\*foo bar\*\*
`
   Parse Tree:
   ```
  └── text: "**foo bar**
"
   ```

✅ **cm_example_378**: `text`
   Input: `\*\* foo bar\*\*
`
   Parse Tree:
   ```
  └── text: "** foo bar**
"
   ```

✅ **cm_example_379**: `text`
   Input: `a\*\*"foo"\*\*
`
   Parse Tree:
   ```
  └── text: "a**"foo"**
"
   ```

✅ **cm_example_380**: `text`
   Input: `foo\*\*bar\*\*
`
   Parse Tree:
   ```
  └── text: "foo**bar**
"
   ```

✅ **cm_example_381**: `text`
   Input: `\_\_foo bar\_\_
`
   Parse Tree:
   ```
  └── text: "__foo bar__
"
   ```

✅ **cm_example_382**: `text`
   Input: `\_\_ foo bar\_\_
`
   Parse Tree:
   ```
  └── text: "__ foo bar__
"
   ```

✅ **cm_example_383**: `text`
   Input: `\_\_
foo bar\_\_
`
   Parse Tree:
   ```
  └── text: "__
foo bar__
"
   ```

✅ **cm_example_384**: `text`
   Input: `a\_\_"foo"\_\_
`
   Parse Tree:
   ```
  └── text: "a__"foo"__
"
   ```

✅ **cm_example_385**: `text`
   Input: `foo\_\_bar\_\_
`
   Parse Tree:
   ```
  └── text: "foo__bar__
"
   ```

✅ **cm_example_386**: `text`
   Input: `5\_\_6\_\_78
`
   Parse Tree:
   ```
  └── text: "5__6__78
"
   ```

✅ **cm_example_387**: `text`
   Input: `пристаням\_\_стремятся\_\_
`
   Parse Tree:
   ```
  └── text: "пристаням__стремятся__
"
   ```

✅ **cm_example_388**: `text`
   Input: `\_\_foo, \_\_bar\_\_, baz\_\_
`
   Parse Tree:
   ```
  └── text: "__foo, __bar__, baz__
"
   ```

✅ **cm_example_389**: `text`
   Input: `foo-\_\_(bar)\_\_
`
   Parse Tree:
   ```
  └── text: "foo-__(bar)__
"
   ```

✅ **cm_example_390**: `text`
   Input: `\*\*foo bar \*\*
`
   Parse Tree:
   ```
  └── text: "**foo bar **
"
   ```

✅ **cm_example_391**: `text`
   Input: `\*\*(\*\*foo)
`
   Parse Tree:
   ```
  └── text: "**(**foo)
"
   ```

✅ **cm_example_392**: `text`
   Input: `\*(\*\*foo\*\*)\*
`
   Parse Tree:
   ```
  └── text: "*(**foo**)*
"
   ```

✅ **cm_example_393**: `text`
   Input: `\*\*Gomphocarpus (\*Gomphocarpus physocarpus\*, syn.
\*Asclepias physocarpa\*)\*\*
`
   Parse Tree:
   ```
  └── text: "**Gomphocarpus (*Gomphocarpus physocarpus*, syn.
*Asclepias physocarpa*)**
"
   ```

✅ **cm_example_394**: `text`
   Input: `\*\*foo "\*bar\*" foo\*\*
`
   Parse Tree:
   ```
  └── text: "**foo "*bar*" foo**
"
   ```

✅ **cm_example_395**: `text`
   Input: `\*\*foo\*\*bar
`
   Parse Tree:
   ```
  └── text: "**foo**bar
"
   ```

✅ **cm_example_396**: `text`
   Input: `\_\_foo bar \_\_
`
   Parse Tree:
   ```
  └── text: "__foo bar __
"
   ```

✅ **cm_example_397**: `text`
   Input: `\_\_(\_\_foo)
`
   Parse Tree:
   ```
  └── text: "__(__foo)
"
   ```

✅ **cm_example_398**: `text`
   Input: `\_(\_\_foo\_\_)\_
`
   Parse Tree:
   ```
  └── text: "_(__foo__)_
"
   ```

✅ **cm_example_399**: `text`
   Input: `\_\_foo\_\_bar
`
   Parse Tree:
   ```
  └── text: "__foo__bar
"
   ```

✅ **cm_example_400**: `text`
   Input: `\_\_пристаням\_\_стремятся
`
   Parse Tree:
   ```
  └── text: "__пристаням__стремятся
"
   ```

✅ **cm_example_401**: `text`
   Input: `\_\_foo\_\_bar\_\_baz\_\_
`
   Parse Tree:
   ```
  └── text: "__foo__bar__baz__
"
   ```

✅ **cm_example_402**: `text`
   Input: `\_\_(bar)\_\_.
`
   Parse Tree:
   ```
  └── text: "__(bar)__.
"
   ```

✅ **cm_example_403**: `text`
   Input: `\*foo \[bar\](/url)\*
`
   Parse Tree:
   ```
  └── text: "*foo [bar](/url)*
"
   ```

✅ **cm_example_404**: `text`
   Input: `\*foo
bar\*
`
   Parse Tree:
   ```
  └── text: "*foo
bar*
"
   ```

✅ **cm_example_405**: `text`
   Input: `\_foo \_\_bar\_\_ baz\_
`
   Parse Tree:
   ```
  └── text: "_foo __bar__ baz_
"
   ```

✅ **cm_example_406**: `text`
   Input: `\_foo \_bar\_ baz\_
`
   Parse Tree:
   ```
  └── text: "_foo _bar_ baz_
"
   ```

✅ **cm_example_407**: `text`
   Input: `\_\_foo\_ bar\_
`
   Parse Tree:
   ```
  └── text: "__foo_ bar_
"
   ```

✅ **cm_example_408**: `text`
   Input: `\*foo \*bar\*\*
`
   Parse Tree:
   ```
  └── text: "*foo *bar**
"
   ```

✅ **cm_example_409**: `text`
   Input: `\*foo \*\*bar\*\* baz\*
`
   Parse Tree:
   ```
  └── text: "*foo **bar** baz*
"
   ```

✅ **cm_example_410**: `text`
   Input: `\*foo\*\*bar\*\*baz\*
`
   Parse Tree:
   ```
  └── text: "*foo**bar**baz*
"
   ```

✅ **cm_example_411**: `text`
   Input: `\*foo\*\*bar\*
`
   Parse Tree:
   ```
  └── text: "*foo**bar*
"
   ```

✅ **cm_example_412**: `text`
   Input: `\*\*\*foo\*\* bar\*
`
   Parse Tree:
   ```
  └── text: "***foo** bar*
"
   ```

✅ **cm_example_413**: `text`
   Input: `\*foo \*\*bar\*\*\*
`
   Parse Tree:
   ```
  └── text: "*foo **bar***
"
   ```

✅ **cm_example_414**: `text`
   Input: `\*foo\*\*bar\*\*\*
`
   Parse Tree:
   ```
  └── text: "*foo**bar***
"
   ```

✅ **cm_example_415**: `text`
   Input: `foo\*\*\*bar\*\*\*baz
`
   Parse Tree:
   ```
  └── text: "foo***bar***baz
"
   ```

✅ **cm_example_416**: `text`
   Input: `foo\*\*\*\*\*\*bar\*\*\*\*\*\*\*\*\*baz
`
   Parse Tree:
   ```
  └── text: "foo******bar*********baz
"
   ```

✅ **cm_example_417**: `text`
   Input: `\*foo \*\*bar \*baz\* bim\*\* bop\*
`
   Parse Tree:
   ```
  └── text: "*foo **bar *baz* bim** bop*
"
   ```

✅ **cm_example_418**: `text`
   Input: `\*foo \[\*bar\*\](/url)\*
`
   Parse Tree:
   ```
  └── text: "*foo [*bar*](/url)*
"
   ```

✅ **cm_example_419**: `text`
   Input: `\*\* is not an empty emphasis
`
   Parse Tree:
   ```
  └── text: "** is not an empty emphasis
"
   ```

✅ **cm_example_420**: `text`
   Input: `\*\*\*\* is not an empty strong emphasis
`
   Parse Tree:
   ```
  └── text: "**** is not an empty strong emphasis
"
   ```

✅ **cm_example_421**: `text`
   Input: `\*\*foo \[bar\](/url)\*\*
`
   Parse Tree:
   ```
  └── text: "**foo [bar](/url)**
"
   ```

✅ **cm_example_422**: `text`
   Input: `\*\*foo
bar\*\*
`
   Parse Tree:
   ```
  └── text: "**foo
bar**
"
   ```

✅ **cm_example_423**: `text`
   Input: `\_\_foo \_bar\_ baz\_\_
`
   Parse Tree:
   ```
  └── text: "__foo _bar_ baz__
"
   ```

✅ **cm_example_424**: `text`
   Input: `\_\_foo \_\_bar\_\_ baz\_\_
`
   Parse Tree:
   ```
  └── text: "__foo __bar__ baz__
"
   ```

✅ **cm_example_425**: `text`
   Input: `\_\_\_\_foo\_\_ bar\_\_
`
   Parse Tree:
   ```
  └── text: "____foo__ bar__
"
   ```

✅ **cm_example_426**: `text`
   Input: `\*\*foo \*\*bar\*\*\*\*
`
   Parse Tree:
   ```
  └── text: "**foo **bar****
"
   ```

✅ **cm_example_427**: `text`
   Input: `\*\*foo \*bar\* baz\*\*
`
   Parse Tree:
   ```
  └── text: "**foo *bar* baz**
"
   ```

✅ **cm_example_428**: `text`
   Input: `\*\*foo\*bar\*baz\*\*
`
   Parse Tree:
   ```
  └── text: "**foo*bar*baz**
"
   ```

✅ **cm_example_429**: `text`
   Input: `\*\*\*foo\* bar\*\*
`
   Parse Tree:
   ```
  └── text: "***foo* bar**
"
   ```

✅ **cm_example_430**: `text`
   Input: `\*\*foo \*bar\*\*\*
`
   Parse Tree:
   ```
  └── text: "**foo *bar***
"
   ```

✅ **cm_example_431**: `text`
   Input: `\*\*foo \*bar \*\*baz\*\*
bim\* bop\*\*
`
   Parse Tree:
   ```
  └── text: "**foo *bar **baz**
bim* bop**
"
   ```

✅ **cm_example_432**: `text`
   Input: `\*\*foo \[\*bar\*\](/url)\*\*
`
   Parse Tree:
   ```
  └── text: "**foo [*bar*](/url)**
"
   ```

✅ **cm_example_433**: `text`
   Input: `\_\_ is not an empty emphasis
`
   Parse Tree:
   ```
  └── text: "__ is not an empty emphasis
"
   ```

✅ **cm_example_434**: `text`
   Input: `\_\_\_\_ is not an empty strong emphasis
`
   Parse Tree:
   ```
  └── text: "____ is not an empty strong emphasis
"
   ```

✅ **cm_example_435**: `text`
   Input: `foo \*\*\*
`
   Parse Tree:
   ```
  └── text: "foo ***
"
   ```

✅ **cm_example_436**: `text`
   Input: `foo \*\\\*\*
`
   Parse Tree:
   ```
  └── text: "foo *"
   ```

✅ **cm_example_437**: `text`
   Input: `foo \*\_\*
`
   Parse Tree:
   ```
  └── text: "foo *_*
"
   ```

✅ **cm_example_438**: `text`
   Input: `foo \*\*\*\*\*
`
   Parse Tree:
   ```
  └── text: "foo *****
"
   ```

✅ **cm_example_439**: `text`
   Input: `foo \*\*\\\*\*\*
`
   Parse Tree:
   ```
  └── text: "foo **"
   ```

✅ **cm_example_440**: `text`
   Input: `foo \*\*\_\*\*
`
   Parse Tree:
   ```
  └── text: "foo **_**
"
   ```

✅ **cm_example_441**: `text`
   Input: `\*\*foo\*
`
   Parse Tree:
   ```
  └── text: "**foo*
"
   ```

✅ **cm_example_442**: `text`
   Input: `\*foo\*\*
`
   Parse Tree:
   ```
  └── text: "*foo**
"
   ```

✅ **cm_example_443**: `text`
   Input: `\*\*\*foo\*\*
`
   Parse Tree:
   ```
  └── text: "***foo**
"
   ```

✅ **cm_example_444**: `text`
   Input: `\*\*\*\*foo\*
`
   Parse Tree:
   ```
  └── text: "****foo*
"
   ```

✅ **cm_example_445**: `text`
   Input: `\*\*foo\*\*\*
`
   Parse Tree:
   ```
  └── text: "**foo***
"
   ```

✅ **cm_example_446**: `text`
   Input: `\*foo\*\*\*\*
`
   Parse Tree:
   ```
  └── text: "*foo****
"
   ```

✅ **cm_example_447**: `text`
   Input: `foo \_\_\_
`
   Parse Tree:
   ```
  └── text: "foo ___
"
   ```

✅ **cm_example_448**: `text`
   Input: `foo \_\\\_\_
`
   Parse Tree:
   ```
  └── text: "foo _"
   ```

✅ **cm_example_449**: `text`
   Input: `foo \_\*\_
`
   Parse Tree:
   ```
  └── text: "foo _*_
"
   ```

✅ **cm_example_450**: `text`
   Input: `foo \_\_\_\_\_
`
   Parse Tree:
   ```
  └── text: "foo _____
"
   ```

✅ **cm_example_451**: `text`
   Input: `foo \_\_\\\_\_\_
`
   Parse Tree:
   ```
  └── text: "foo __"
   ```

✅ **cm_example_452**: `text`
   Input: `foo \_\_\*\_\_
`
   Parse Tree:
   ```
  └── text: "foo __*__
"
   ```

✅ **cm_example_453**: `text`
   Input: `\_\_foo\_
`
   Parse Tree:
   ```
  └── text: "__foo_
"
   ```

✅ **cm_example_454**: `text`
   Input: `\_foo\_\_
`
   Parse Tree:
   ```
  └── text: "_foo__
"
   ```

✅ **cm_example_455**: `text`
   Input: `\_\_\_foo\_\_
`
   Parse Tree:
   ```
  └── text: "___foo__
"
   ```

✅ **cm_example_456**: `text`
   Input: `\_\_\_\_foo\_
`
   Parse Tree:
   ```
  └── text: "____foo_
"
   ```

✅ **cm_example_457**: `text`
   Input: `\_\_foo\_\_\_
`
   Parse Tree:
   ```
  └── text: "__foo___
"
   ```

✅ **cm_example_458**: `text`
   Input: `\_foo\_\_\_\_
`
   Parse Tree:
   ```
  └── text: "_foo____
"
   ```

✅ **cm_example_459**: `text`
   Input: `\*\*foo\*\*
`
   Parse Tree:
   ```
  └── text: "**foo**
"
   ```

✅ **cm_example_460**: `text`
   Input: `\*\_foo\_\*
`
   Parse Tree:
   ```
  └── text: "*_foo_*
"
   ```

✅ **cm_example_461**: `text`
   Input: `\_\_foo\_\_
`
   Parse Tree:
   ```
  └── text: "__foo__
"
   ```

✅ **cm_example_462**: `text`
   Input: `\_\*foo\*\_
`
   Parse Tree:
   ```
  └── text: "_*foo*_
"
   ```

✅ **cm_example_463**: `text`
   Input: `\*\*\*\*foo\*\*\*\*
`
   Parse Tree:
   ```
  └── text: "****foo****
"
   ```

✅ **cm_example_464**: `text`
   Input: `\_\_\_\_foo\_\_\_\_
`
   Parse Tree:
   ```
  └── text: "____foo____
"
   ```

✅ **cm_example_465**: `text`
   Input: `\*\*\*\*\*\*foo\*\*\*\*\*\*
`
   Parse Tree:
   ```
  └── text: "******foo******
"
   ```

✅ **cm_example_466**: `text`
   Input: `\*\*\*foo\*\*\*
`
   Parse Tree:
   ```
  └── text: "***foo***
"
   ```

✅ **cm_example_467**: `text`
   Input: `\_\_\_\_\_foo\_\_\_\_\_
`
   Parse Tree:
   ```
  └── text: "_____foo_____
"
   ```

✅ **cm_example_468**: `text`
   Input: `\*foo \_bar\* baz\_
`
   Parse Tree:
   ```
  └── text: "*foo _bar* baz_
"
   ```

✅ **cm_example_469**: `text`
   Input: `\*foo \_\_bar \*baz bim\_\_ bam\*
`
   Parse Tree:
   ```
  └── text: "*foo __bar *baz bim__ bam*
"
   ```

✅ **cm_example_470**: `text`
   Input: `\*\*foo \*\*bar baz\*\*
`
   Parse Tree:
   ```
  └── text: "**foo **bar baz**
"
   ```

✅ **cm_example_471**: `text`
   Input: `\*foo \*bar baz\*
`
   Parse Tree:
   ```
  └── text: "*foo *bar baz*
"
   ```

✅ **cm_example_472**: `text`
   Input: `\*\[bar\*\](/url)
`
   Parse Tree:
   ```
  └── text: "*[bar*](/url)
"
   ```

✅ **cm_example_473**: `text`
   Input: `\_foo \[bar\_\](/url)
`
   Parse Tree:
   ```
  └── text: "_foo [bar_](/url)
"
   ```

✅ **cm_example_474**: `text`
   Input: `\*<img src="foo" title="\*"/>
`
   Parse Tree:
   ```
  └── text: "*<img src="foo" title="*"/>
"
   ```

✅ **cm_example_475**: `text`
   Input: `\*\*<a href="\*\*">
`
   Parse Tree:
   ```
  └── text: "**<a href="**">
"
   ```

✅ **cm_example_476**: `text`
   Input: `\_\_<a href="\_\_">
`
   Parse Tree:
   ```
  └── text: "__<a href="__">
"
   ```

✅ **cm_example_477**: `text`
   Input: `\*a \`\*\`\*
`
   Parse Tree:
   ```
  └── text: "*a `*`*
"
   ```

✅ **cm_example_478**: `text`
   Input: `\_a \`\_\`\_
`
   Parse Tree:
   ```
  └── text: "_a `_`_
"
   ```

✅ **cm_example_479**: `text`
   Input: `\*\*a<http://foo.bar/?q=\*\*>
`
   Parse Tree:
   ```
  └── text: "**a<http://foo.bar/?q=**>
"
   ```

✅ **cm_example_480**: `text`
   Input: `\_\_a<http://foo.bar/?q=\_\_>
`
   Parse Tree:
   ```
  └── text: "__a<http://foo.bar/?q=__>
"
   ```

## security_vectors

✅ **script_tag**: `inline_html`
   Input: `<script>alert('xss')</script>`
   Parse Tree:
   ```
  └── inline_html: "<script>"
   ```

✅ **script_src**: `inline_html`
   Input: `<script src="malicious.js"></script>`
   Parse Tree:
   ```
  └── inline_html: "<script src="malicious.js">"
   ```

✅ **onclick_handler**: `text`
   Input: `<div onclick="alert('xss')">click</div>`
   Parse Tree:
   ```
  └── text: "<div onclick="alert('xss')">click</div>"
   ```

❌ **javascript_url**: `inline_html` (Unexpected failure)
   Input: `\[click\](javascript:alert('xss'))`
   Error: ` --> 1:1
  |
1 | [click](javascript:alert('xss'))
  | ^---
  |
  = expected inline_html`

❌ **data_url**: `inline_link` (Unexpected failure)
   Input: `\[click\](data:text/html,<script>alert('xss')</script>)`
   Error: ` --> 1:1
  |
1 | [click](data:text/html,<script>alert('xss')</script>)
  | ^---
  |
  = expected inline_link`

✅ **mixed_xss_1**: `text`
   Input: `<img src=x onerror=alert('xss')>
\*\*bold\*\*`
   Parse Tree:
   ```
  └── text: "<img src=x onerror=alert('xss')>
**bold**"
   ```

✅ **mixed_xss_2**: `text`
   Input: `\*\*bold\*\* <script>alert('xss')</script>`
   Parse Tree:
   ```
  └── text: "**bold** <script>alert('xss')</script>"
   ```

✅ **mixed_xss_3**: `text`
   Input: `\[text\](<img src=x onerror=alert('xss')>)`
   Parse Tree:
   ```
  └── text: "[text](<img src=x onerror=alert('xss')>)"
   ```

✅ **ftp_protocol**: `text`
   Input: `\[link\](ftp://malicious.com)`
   Parse Tree:
   ```
  └── text: "[link](ftp://malicious.com)"
   ```

✅ **file_protocol**: `text`
   Input: `\[link\](file:///etc/passwd)`
   Parse Tree:
   ```
  └── text: "[link](file:///etc/passwd)"
   ```

✅ **custom_protocol**: `text`
   Input: `\[link\](custom://protocol)`
   Parse Tree:
   ```
  └── text: "[link](custom://protocol)"
   ```

❌ **url_with_credentials**: `inline_link` (Unexpected failure)
   Input: `https://user:pass@evil.com`
   Error: ` --> 1:1
  |
1 | https://user:pass@evil.com
  | ^---
  |
  = expected inline_link`

❌ **url_with_unicode**: `inline_link` (Unexpected failure)
   Input: `https://аpple.com`
   Error: ` --> 1:1
  |
1 | https://аpple.com
  | ^---
  |
  = expected inline_link`

❌ **url_with_path_traversal**: `inline_link` (Unexpected failure)
   Input: `file://../../etc/passwd`
   Error: ` --> 1:1
  |
1 | file://../../etc/passwd
  | ^---
  |
  = expected inline_link`

✅ **fake_attachment**: `text`
   Input: `\[download.pdf\](malicious.exe)`
   Parse Tree:
   ```
  └── text: "[download.pdf](malicious.exe)"
   ```

✅ **misleading_link**: `inline_link`
   Input: `\[google.com\](https://evil.com)`
   Parse Tree:
   ```
  ├── inline_link > "[google.com](https://evil.com)"
    └── bracket_link_without_title: "[google.com](https://evil.com)"
   ```

✅ **homograph_attack**: `text`
   Input: `\[аpple.com\](https://evil.com)`
   Parse Tree:
   ```
  └── text: "[аpple.com](https://evil.com)"
   ```

## commonmark_soft_line_breaks

✅ **cm_example_648**: `text`
   Input: `foo
baz
`
   Parse Tree:
   ```
  └── text: "foo
baz
"
   ```

✅ **cm_example_649**: `text`
   Input: `foo 
 baz
`
   Parse Tree:
   ```
  └── text: "foo 
 baz
"
   ```

## commonmark_edge_cases

✅ **link_vs_emphasis**: `inline_link`
   Input: `\[\*foo\*\](bar)`
   Parse Tree:
   ```
  ├── inline_link > "[*foo*](bar)"
    └── bracket_link_without_title: "[*foo*](bar)"
   ```

❌ **emphasis_vs_link**: `inline_link` (Unexpected failure)
   Input: `\*\[foo\](bar)\*`
   Error: ` --> 1:1
  |
1 | *[foo](bar)*
  | ^---
  |
  = expected inline_link`

❌ **code_vs_emphasis**: `emphasis` (Unexpected failure)
   Input: `\`\*foo\*\``
   Error: ` --> 1:1
  |
1 | `*foo*`
  | ^---
  |
  = expected emphasis`

❌ **html_vs_emphasis**: `emphasis` (Unexpected failure)
   Input: `<em>\*foo\*</em>`
   Error: ` --> 1:1
  |
1 | <em>*foo*</em>
  | ^---
  |
  = expected emphasis`

✅ **html_entities**: `text`
   Input: `&amp; &lt; &gt; &quot; &#39; &#x27;`
   Parse Tree:
   ```
  └── text: "&amp; &lt; &gt; &quot; &#39; &#x27;"
   ```

✅ **numeric_entities**: `text`
   Input: `&#65; &#x41; &#97; &#x61;`
   Parse Tree:
   ```
  └── text: "&#65; &#x41; &#97; &#x61;"
   ```

✅ **invalid_entities**: `text`
   Input: `&invalid; &; &#; &#x;`
   Parse Tree:
   ```
  └── text: "&invalid; &; &#; &#x;"
   ```

✅ **autolink_email**: `inline_link`
   Input: `<user@example.com>`
   Parse Tree:
   ```
  ├── inline_link > "<user@example.com>"
    ├── autolink > "<user@example.com>"
      ├── autolink_email > "<user@example.com>"
        └── EMAIL_LOCAL: "user"
        └── EMAIL_FULL_DOMAIN: "example.com"
   ```

✅ **autolink_url**: `inline_link`
   Input: `<http://example.com>`
   Parse Tree:
   ```
  ├── inline_link > "<http://example.com>"
    ├── autolink > "<http://example.com>"
      ├── autolink_url > "<http://example.com>"
        └── link_url: "http://example.com"
   ```

✅ **autolink_invalid**: `inline_link` (Expected failure)
   Input: `<not a url>`
   Error: ` --> 1:1
  |
1 | <not a url>
  | ^---
  |
  = expected inline_link`

✅ **autolink_nested**: `inline_link`
   Input: `\[<http://example.com>\](http://other.com)`
   Parse Tree:
   ```
  ├── inline_link > "[<http://example.com>](http://other.com)"
    └── bracket_link_without_title: "[<http://example.com>](http://other.com)"
   ```

✅ **hr_spaces_before**: `hr`
   Input: `   ---`
   Parse Tree:
   ```
  └── hr: "   ---"
   ```

✅ **hr_spaces_after**: `hr`
   Input: `---   `
   Parse Tree:
   ```
  └── hr: "---   "
   ```

❌ **hr_spaces_between**: `hr` (Unexpected failure)
   Input: `- - -`
   Error: ` --> 1:1
  |
1 | - - -
  | ^---
  |
  = expected hr`

✅ **hr_mixed_chars_invalid**: `hr` (Expected failure)
   Input: `-\*-`
   Error: ` --> 1:1
  |
1 | -*-
  | ^---
  |
  = expected hr`

❌ **hr_too_few_chars**: `hr` (Unexpected failure)
   Input: `--`
   Error: ` --> 1:1
  |
1 | --
  | ^---
  |
  = expected hr`

✅ **list_tight_vs_loose**: `list`
   Input: `- foo
- bar

- baz`
   Parse Tree:
   ```
  ├── list > "- foo
- bar

- baz"
    ├── list_item > "- foo"
      ├── regular_list_item > "- foo"
        └── list_marker: "-"
        └── list_item_content: "foo"
    ├── list_item > "- bar"
      ├── regular_list_item > "- bar"
        └── list_marker: "-"
        └── list_item_content: "bar"
    ├── list_item > "
- baz"
      ├── regular_list_item > "- baz"
        └── list_marker: "-"
        └── list_item_content: "baz"
   ```

✅ **list_marker_interruption**: `list`
   Input: `1. foo

2. bar`
   Parse Tree:
   ```
  ├── list > "1. foo

2. bar"
    ├── list_item > "1. foo"
      ├── regular_list_item > "1. foo"
        └── list_marker: "1."
        └── list_item_content: "foo"
    ├── list_item > "
2. bar"
      ├── regular_list_item > "2. bar"
        └── list_marker: "2."
        └── list_item_content: "bar"
   ```

✅ **list_continuation**: `list`
   Input: `1. foo

   continued`
   Parse Tree:
   ```
  ├── list > "1. foo
"
    ├── list_item > "1. foo"
      ├── regular_list_item > "1. foo"
        └── list_marker: "1."
        └── list_item_content: "foo"
   ```

✅ **list_lazy_continuation**: `list`
   Input: `1. foo
bar`
   Parse Tree:
   ```
  ├── list > "1. foo
"
    ├── list_item > "1. foo"
      ├── regular_list_item > "1. foo"
        └── list_marker: "1."
        └── list_item_content: "foo"
   ```

✅ **heading_no_space_after**: `heading`
   Input: `#foo`
   Parse Tree:
   ```
  ├── heading > "#foo"
    ├── H1 > "#foo"
      ├── heading_content > "foo"
        ├── heading_inline > "foo"
          └── word: "foo"
   ```

✅ **heading_space_before**: `heading`
   Input: ` # foo`
   Parse Tree:
   ```
  ├── heading > " # foo"
    ├── H1 > " # foo"
      ├── heading_content > "foo"
        ├── heading_inline > "foo"
          └── word: "foo"
   ```

✅ **heading_trailing_hashes**: `heading`
   Input: `# foo #`
   Parse Tree:
   ```
  ├── heading > "# foo "
    ├── H1 > "# foo "
      ├── heading_content > "foo "
        ├── heading_inline > "foo"
          └── word: "foo"
   ```

✅ **heading_trailing_hashes_mismatch**: `heading`
   Input: `# foo ###`
   Parse Tree:
   ```
  ├── heading > "# foo "
    ├── H1 > "# foo "
      ├── heading_content > "foo "
        ├── heading_inline > "foo"
          └── word: "foo"
   ```

✅ **heading_empty**: `heading` (Expected failure)
   Input: `#`
   Error: ` --> 1:2
  |
1 | #
  |  ^---
  |
  = expected heading_inline`

❌ **heading_only_hashes**: `heading` (Unexpected failure)
   Input: `######`
   Error: ` --> 1:7
  |
1 | ######
  |       ^---
  |
  = expected heading_inline`

✅ **setext_no_content**: `setext_h2` (Expected failure)
   Input: `
====`
   Error: ` --> 1:1
  |
1 | ␊
  | ^---
  |
  = expected heading_inline`

❌ **setext_spaces_before**: `setext_h2` (Unexpected failure)
   Input: `   foo
   ===`
   Error: ` --> 1:7
  |
1 |    foo␊
  |       ^---
  |
  = expected heading_inline`

❌ **setext_uneven_underline**: `setext_h2` (Unexpected failure)
   Input: `foo
======`
   Error: ` --> 1:4
  |
1 | foo␊
  |    ^---
  |
  = expected heading_inline`

## inline_images

✅ **image_basic**: `inline_image`
   Input: `!\[alt text\](image.jpg)`
   Parse Tree:
   ```
  ├── inline_image > "![alt text](image.jpg)"
    └── inline_link_text: "alt text"
    └── link_url: "image.jpg"
   ```

✅ **image_empty_alt**: `inline_image`
   Input: `!\[\](image.png)`
   Parse Tree:
   ```
  ├── inline_image > "![](image.png)"
    └── inline_link_text: ""
    └── link_url: "image.png"
   ```

✅ **image_with_url**: `inline_image`
   Input: `!\[remote\](https://example.com/image.png)`
   Parse Tree:
   ```
  ├── inline_image > "![remote](https://example.com/image.png)"
    └── inline_link_text: "remote"
    └── link_url: "https://example.com/image.png"
   ```

✅ **image_complex_alt**: `inline_image`
   Input: `!\[A very detailed alt text\](image.jpg)`
   Parse Tree:
   ```
  ├── inline_image > "![A very detailed alt text](image.jpg)"
    └── inline_link_text: "A very detailed alt text"
    └── link_url: "image.jpg"
   ```

✅ **image_no_extension**: `inline_image`
   Input: `!\[alt\](not\_an\_image)`
   Parse Tree:
   ```
  ├── inline_image > "![alt](not_an_image)"
    └── inline_link_text: "alt"
    └── link_url: "not_an_image"
   ```

❌ **image_unclosed**: `inline_image` (Unexpected failure)
   Input: `!\[alt\](image.jpg`
   Error: ` --> 1:1
  |
1 | ![alt](image.jpg
  | ^---
  |
  = expected inline_image`

## Summary

- **Total tests**: 1237
- **Passed**: 1145 ✅
- **Failed**: 92 ❌
  - Expected failures: 52 ✅
  - Unexpected failures: 92 ❌
- **Success rate**: 92.6%
