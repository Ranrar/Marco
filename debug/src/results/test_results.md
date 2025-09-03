# Marco Grammar Test Results

Generated automatically from test_cases.toml

## commonmark_fenced_code_blocks

❌ **cm_example_119**: `text` (Unexpected failure)
   Input: `\`\`\`
<
 >
\`\`\`
`
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

✅ **cm_example_120**: `text`
   Input: `~~~
<
 >
~~~
`
   Parse Tree:
   ```
  └── text: "~~~
<
 >
~~~
"
   ```

❌ **cm_example_121**: `text` (Unexpected failure)
   Input: `\`\`
foo
\`\`
`
   Error: ` --> 1:1
  |
1 | ``
  | ^---
  |
  = expected text`

❌ **cm_example_122**: `text` (Unexpected failure)
   Input: `\`\`\`
aaa
~~~
\`\`\`
`
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

✅ **cm_example_123**: `text`
   Input: `~~~
aaa
\`\`\`
~~~
`
   Parse Tree:
   ```
  └── text: "~~~
aaa
"
   ```

❌ **cm_example_124**: `text` (Unexpected failure)
   Input: `\`\`\`\`
aaa
\`\`\`
\`\`\`\`\`\`
`
   Error: ` --> 1:1
  |
1 | ````
  | ^---
  |
  = expected text`

✅ **cm_example_125**: `text`
   Input: `~~~~
aaa
~~~
~~~~
`
   Parse Tree:
   ```
  └── text: "~~~~
aaa
~~~
~~~~
"
   ```

❌ **cm_example_126**: `text` (Unexpected failure)
   Input: `\`\`\`
`
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

❌ **cm_example_127**: `text` (Unexpected failure)
   Input: `\`\`\`\`\`

\`\`\`
aaa
`
   Error: ` --> 1:1
  |
1 | `````
  | ^---
  |
  = expected text`

❌ **cm_example_128**: `text` (Unexpected failure)
   Input: `> \`\`\`
> aaa

bbb
`
   Error: ` --> 1:1
  |
1 | > ```
  | ^---
  |
  = expected text`

❌ **cm_example_129**: `text` (Unexpected failure)
   Input: `\`\`\`

  
\`\`\`
`
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

❌ **cm_example_130**: `text` (Unexpected failure)
   Input: `\`\`\`
\`\`\`
`
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

❌ **cm_example_131**: `text` (Unexpected failure)
   Input: ` \`\`\`
 aaa
aaa
\`\`\`
`
   Error: ` --> 1:1
  |
1 |  ```
  | ^---
  |
  = expected text`

❌ **cm_example_132**: `text` (Unexpected failure)
   Input: `  \`\`\`
aaa
  aaa
aaa
  \`\`\`
`
   Error: ` --> 1:1
  |
1 |   ```
  | ^---
  |
  = expected text`

❌ **cm_example_133**: `text` (Unexpected failure)
   Input: `   \`\`\`
   aaa
    aaa
  aaa
   \`\`\`
`
   Error: ` --> 1:1
  |
1 |    ```
  | ^---
  |
  = expected text`

❌ **cm_example_134**: `text` (Unexpected failure)
   Input: `    \`\`\`
    aaa
    \`\`\`
`
   Error: ` --> 1:1
  |
1 |     ```
  | ^---
  |
  = expected text`

❌ **cm_example_135**: `text` (Unexpected failure)
   Input: `\`\`\`
aaa
  \`\`\`
`
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

❌ **cm_example_136**: `text` (Unexpected failure)
   Input: `   \`\`\`
aaa
  \`\`\`
`
   Error: ` --> 1:1
  |
1 |    ```
  | ^---
  |
  = expected text`

❌ **cm_example_137**: `text` (Unexpected failure)
   Input: `\`\`\`
aaa
    \`\`\`
`
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

❌ **cm_example_138**: `text` (Unexpected failure)
   Input: `\`\`\` \`\`\`
aaa
`
   Error: ` --> 1:1
  |
1 | ``` ```
  | ^---
  |
  = expected text`

✅ **cm_example_139**: `text`
   Input: `~~~~~~
aaa
~~~ ~~
`
   Parse Tree:
   ```
  └── text: "~~~~~~
aaa
~~~ ~~
"
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
~~~
bar
~~~
# baz
"
   ```

❌ **cm_example_142**: `text` (Unexpected failure)
   Input: `\`\`\`ruby
def foo(x)
  return 3
end
\`\`\`
`
   Error: ` --> 1:1
  |
1 | ```ruby
  | ^---
  |
  = expected text`

✅ **cm_example_143**: `text`
   Input: `~~~~    ruby startline=3 $%@#$
def foo(x)
  return 3
end
~~~~~~~
`
   Parse Tree:
   ```
  └── text: "~~~~    ruby startline=3 "
   ```

❌ **cm_example_144**: `text` (Unexpected failure)
   Input: `\`\`\`\`;
\`\`\`\`
`
   Error: ` --> 1:1
  |
1 | ````;
  | ^---
  |
  = expected text`

❌ **cm_example_145**: `text` (Unexpected failure)
   Input: `\`\`\` aa \`\`\`
foo
`
   Error: ` --> 1:1
  |
1 | ``` aa ```
  | ^---
  |
  = expected text`

✅ **cm_example_146**: `text`
   Input: `~~~ aa \`\`\` ~~~
foo
~~~
`
   Parse Tree:
   ```
  └── text: "~~~ aa "
   ```

❌ **cm_example_147**: `text` (Unexpected failure)
   Input: `\`\`\`
\`\`\` aaa
\`\`\`
`
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

## admonitions

✅ **note_simple**: `admonition_block`
   Input: `:::
note
This is a note
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
note
This is a note
:::"
    ├── admonition_open > ":::
note"
      ├── admonition_type > "note"
        └── KW_NOTE: "note"
    └── admonition_close: ":::"
   ```

✅ **tip_simple**: `admonition_block`
   Input: `:::
tip
This is a tip
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
tip
This is a tip
:::"
    ├── admonition_open > ":::
tip"
      ├── admonition_type > "tip"
        └── KW_TIP: "tip"
    └── admonition_close: ":::"
   ```

✅ **warning_simple**: `admonition_block`
   Input: `:::
warning
This is a warning
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
warning
This is a warning
:::"
    ├── admonition_open > ":::
warning"
      ├── admonition_type > "warning"
        └── KW_WARNING: "warning"
    └── admonition_close: ":::"
   ```

✅ **danger_simple**: `admonition_block`
   Input: `:::
danger
This is dangerous
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
danger
This is dangerous
:::"
    ├── admonition_open > ":::
danger"
      ├── admonition_type > "danger"
        └── KW_DANGER: "danger"
    └── admonition_close: ":::"
   ```

✅ **info_simple**: `admonition_block`
   Input: `:::
info
This is info
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
info
This is info
:::"
    ├── admonition_open > ":::
info"
      ├── admonition_type > "info"
        └── KW_INFO: "info"
    └── admonition_close: ":::"
   ```

✅ **note_with_title**: `admonition_block`
   Input: `:::
note\[Custom Title\]
Note content
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
note[Custom Title]
Note content
:::"
    ├── admonition_open > ":::
note[Custom Title]"
      ├── admonition_type > "note"
        └── KW_NOTE: "note"
    └── admonition_close: ":::"
   ```

✅ **warning_titled**: `admonition_block`
   Input: `:::
warning\[Important Warning\]
Warning content
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
warning[Important Warning]
Warning content
:::"
    ├── admonition_open > ":::
warning[Important Warning]"
      ├── admonition_type > "warning"
        └── KW_WARNING: "warning"
    └── admonition_close: ":::"
   ```

✅ **emoji_admonition**: `admonition_block`
   Input: `:::
\[💡\] Custom Emoji
Content here
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
[💡] Custom Emoji
Content here
:::"
    └── admonition_emoji: ":::
[💡] Custom Emoji"
    └── admonition_close: ":::"
   ```

✅ **note_uppercase**: `admonition_block`
   Input: `:::
NOTE
Uppercase note
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
NOTE
Uppercase note
:::"
    ├── admonition_open > ":::
NOTE"
      ├── admonition_type > "NOTE"
        └── KW_NOTE: "NOTE"
    └── admonition_close: ":::"
   ```

✅ **tip_mixed_case**: `admonition_block`
   Input: `:::
TiP
Mixed case tip
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
TiP
Mixed case tip
:::"
    ├── admonition_open > ":::
TiP"
      ├── admonition_type > "TiP"
        └── KW_TIP: "TiP"
    └── admonition_close: ":::"
   ```

❌ **admonition_unclosed**: `admonition_block` (Unexpected failure)
   Input: `:::
note
Unclosed admonition`
   Error: ` --> 3:20
  |
3 | Unclosed admonition
  |                    ^---
  |
  = expected admonition_close or admonition_block`

❌ **admonition_unknown**: `admonition_block` (Unexpected failure)
   Input: `:::
custom
Unknown type
:::`
   Error: ` --> 2:1
  |
2 | custom
  | ^---
  |
  = expected admonition_type`

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

❌ **cm_example_4**: `text` (Unexpected failure)
   Input: `  - foo

	bar
`
   Error: ` --> 1:1
  |
1 |   - foo
  | ^---
  |
  = expected text`

❌ **cm_example_5**: `text` (Unexpected failure)
   Input: `- foo

		bar
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

❌ **cm_example_6**: `text` (Unexpected failure)
   Input: `>		foo
`
   Error: ` --> 1:1
  |
1 | >		foo
  | ^---
  |
  = expected text`

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

❌ **cm_example_9**: `text` (Unexpected failure)
   Input: ` - foo
   - bar
	 - baz
`
   Error: ` --> 1:1
  |
1 |  - foo
  | ^---
  |
  = expected text`

✅ **cm_example_10**: `text`
   Input: `#	Foo
`
   Parse Tree:
   ```
  └── text: "#	Foo
"
   ```

❌ **cm_example_11**: `text` (Unexpected failure)
   Input: `\*	\*	\*	
`
   Error: ` --> 1:1
  |
1 | *	*	*	
  | ^---
  |
  = expected text`

## commonmark_textual_content

✅ **cm_example_650**: `text`
   Input: `hello $.;'there
`
   Parse Tree:
   ```
  └── text: "hello "
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

❌ **cm_example_31**: `text` (Unexpected failure)
   Input: `<a href=\"&ouml;&ouml;.html\">
`
   Error: ` --> 1:1
  |
1 | <a href=\"&ouml;&ouml;.html\">
  | ^---
  |
  = expected text`

❌ **cm_example_32**: `text` (Unexpected failure)
   Input: `\[foo\](/f&ouml;&ouml; \"f&ouml;&ouml;\")
`
   Error: ` --> 1:1
  |
1 | [foo](/f&ouml;&ouml; \"f&ouml;&ouml;\")
  | ^---
  |
  = expected text`

❌ **cm_example_33**: `text` (Unexpected failure)
   Input: `\[foo\]

\[foo\]: /f&ouml;&ouml; \"f&ouml;&ouml;\"
`
   Error: ` --> 1:1
  |
1 | [foo]
  | ^---
  |
  = expected text`

❌ **cm_example_34**: `text` (Unexpected failure)
   Input: `\`\`\` f&ouml;&ouml;
foo
\`\`\`
`
   Error: ` --> 1:1
  |
1 | ``` f&ouml;&ouml;
  | ^---
  |
  = expected text`

❌ **cm_example_35**: `text` (Unexpected failure)
   Input: `\`f&ouml;&ouml;\`
`
   Error: ` --> 1:1
  |
1 | `f&ouml;&ouml;`
  | ^---
  |
  = expected text`

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
"
   ```

✅ **cm_example_38**: `text`
   Input: `&#42; foo

\* foo
`
   Parse Tree:
   ```
  └── text: "&#42; foo

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

❌ **cm_example_41**: `text` (Unexpected failure)
   Input: `\[a\](url &quot;tit&quot;)
`
   Error: ` --> 1:1
  |
1 | [a](url &quot;tit&quot;)
  | ^---
  |
  = expected text`

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

## edge_cases

❌ **only_whitespace**: `text` (Unexpected failure)
   Input: `   	   `
   Error: ` --> 1:1
  |
1 |    	   
  | ^---
  |
  = expected text`

✅ **mixed_line_endings**: `text`
   Input: `text\r
more text
final text`
   Parse Tree:
   ```
  └── text: "text\r
more text
final text"
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
  └── text: "text\u200Bwith\u200Cinvisible\u200Dchars"
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

❌ **deeply_nested**: `text` (Unexpected failure)
   Input: `\*\*bold with \*italic and \`code\` inside\* text\*\*`
   Error: ` --> 1:1
  |
1 | **bold with *italic and `code` inside* text**
  | ^---
  |
  = expected text`

❌ **mixed_formatting**: `text` (Unexpected failure)
   Input: `\*\*bold\*\* and \*italic\* and \`code\` and ~~strike~~`
   Error: ` --> 1:1
  |
1 | **bold** and *italic* and `code` and ~~strike~~
  | ^---
  |
  = expected text`

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
   Input: `.,;:!?()\[\]{}\"'`
   Parse Tree:
   ```
  └── text: ".,;:!?()"
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
   Error: ` --> 1:1
  |
1 | -
  | ^---
  |
  = expected list_item`

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

## inline_links

✅ **link_http**: `inline_link`
   Input: `\[link\](https://example.com)`
   Parse Tree:
   ```
  ├── inline_link > "[link](https://example.com)"
    └── inline_link_text: "link"
    └── inline_url: "https://example.com"
   ```

✅ **link_https**: `inline_link`
   Input: `\[secure link\](https://secure.example.com)`
   Parse Tree:
   ```
  ├── inline_link > "[secure link](https://secure.example.com)"
    └── inline_link_text: "secure link"
    └── inline_url: "https://secure.example.com"
   ```

✅ **link_local**: `inline_link`
   Input: `\[local file\](./path/to/file.md)`
   Parse Tree:
   ```
  ├── inline_link > "[local file](./path/to/file.md)"
    └── inline_link_text: "local file"
    └── inline_url: "./path/to/file.md"
   ```

✅ **link_empty_text**: `inline_link`
   Input: `\[\](https://example.com)`
   Parse Tree:
   ```
  ├── inline_link > "[](https://example.com)"
    └── inline_link_text: ""
    └── inline_url: "https://example.com"
   ```

❌ **link_with_title**: `inline_link` (Unexpected failure)
   Input: `\[link\](https://example.com \"Title\")`
   Error: ` --> 1:1
  |
1 | [link](https://example.com \"Title\")
  | ^---
  |
  = expected inline_link`

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
    └── inline_link_text: "**bold link**"
    └── inline_url: "https://example.com"
   ```

✅ **link_unicode**: `inline_link`
   Input: `\[café link\](https://example.com)`
   Parse Tree:
   ```
  ├── inline_link > "[café link](https://example.com)"
    └── inline_link_text: "café link"
    └── inline_url: "https://example.com"
   ```

✅ **link_empty_url**: `inline_link` (Expected failure)
   Input: `\[text\]()`
   Error: ` --> 1:8
  |
1 | [text]()
  |        ^---
  |
  = expected inline_url`

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
   Error: ` --> 1:8
  |
1 | [text](missing closing paren
  |        ^---
  |
  = expected inline_url`

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

## commonmark_code_spans

❌ **cm_example_328**: `text` (Unexpected failure)
   Input: `\`foo\`
`
   Error: ` --> 1:1
  |
1 | `foo`
  | ^---
  |
  = expected text`

❌ **cm_example_329**: `text` (Unexpected failure)
   Input: `\`\` foo \` bar \`\`
`
   Error: ` --> 1:1
  |
1 | `` foo ` bar ``
  | ^---
  |
  = expected text`

❌ **cm_example_330**: `text` (Unexpected failure)
   Input: `\` \`\` \`
`
   Error: ` --> 1:1
  |
1 | ` `` `
  | ^---
  |
  = expected text`

❌ **cm_example_331**: `text` (Unexpected failure)
   Input: `\`  \`\`  \`
`
   Error: ` --> 1:1
  |
1 | `  ``  `
  | ^---
  |
  = expected text`

❌ **cm_example_332**: `text` (Unexpected failure)
   Input: `\` a\`
`
   Error: ` --> 1:1
  |
1 | ` a`
  | ^---
  |
  = expected text`

❌ **cm_example_333**: `text` (Unexpected failure)
   Input: `\` b \`
`
   Error: ` --> 1:1
  |
1 | ` b `
  | ^---
  |
  = expected text`

❌ **cm_example_334**: `text` (Unexpected failure)
   Input: `\` \`
\`  \`
`
   Error: ` --> 1:1
  |
1 | ` `
  | ^---
  |
  = expected text`

❌ **cm_example_335**: `text` (Unexpected failure)
   Input: `\`\`
foo
bar  
baz
\`\`
`
   Error: ` --> 1:1
  |
1 | ``
  | ^---
  |
  = expected text`

❌ **cm_example_336**: `text` (Unexpected failure)
   Input: `\`\`
foo 
\`\`
`
   Error: ` --> 1:1
  |
1 | ``
  | ^---
  |
  = expected text`

❌ **cm_example_337**: `text` (Unexpected failure)
   Input: `\`foo   bar 
baz\`
`
   Error: ` --> 1:1
  |
1 | `foo   bar 
  | ^---
  |
  = expected text`

❌ **cm_example_338**: `text` (Unexpected failure)
   Input: `\`foo\\\`bar\`
`
   Error: ` --> 1:1
  |
1 | `foo\\`bar`
  | ^---
  |
  = expected text`

❌ **cm_example_339**: `text` (Unexpected failure)
   Input: `\`\`foo\`bar\`\`
`
   Error: ` --> 1:1
  |
1 | ``foo`bar``
  | ^---
  |
  = expected text`

❌ **cm_example_340**: `text` (Unexpected failure)
   Input: `\` foo \`\` bar \`
`
   Error: ` --> 1:1
  |
1 | ` foo `` bar `
  | ^---
  |
  = expected text`

❌ **cm_example_341**: `text` (Unexpected failure)
   Input: `\*foo\`\*\`
`
   Error: ` --> 1:1
  |
1 | *foo`*`
  | ^---
  |
  = expected text`

❌ **cm_example_342**: `text` (Unexpected failure)
   Input: `\[not a \`link\](/foo\`)
`
   Error: ` --> 1:1
  |
1 | [not a `link](/foo`)
  | ^---
  |
  = expected text`

❌ **cm_example_343**: `text` (Unexpected failure)
   Input: `\`<a href=\"\`\">\`
`
   Error: ` --> 1:1
  |
1 | `<a href=\"`\">`
  | ^---
  |
  = expected text`

❌ **cm_example_344**: `text` (Unexpected failure)
   Input: `<a href=\"\`\">\`
`
   Error: ` --> 1:1
  |
1 | <a href=\"`\">`
  | ^---
  |
  = expected text`

❌ **cm_example_345**: `text` (Unexpected failure)
   Input: `\`<http://foo.bar.\`baz>\`
`
   Error: ` --> 1:1
  |
1 | `<http://foo.bar.`baz>`
  | ^---
  |
  = expected text`

❌ **cm_example_346**: `text` (Unexpected failure)
   Input: `<http://foo.bar.\`baz>\`
`
   Error: ` --> 1:1
  |
1 | <http://foo.bar.`baz>`
  | ^---
  |
  = expected text`

❌ **cm_example_347**: `text` (Unexpected failure)
   Input: `\`\`\`foo\`\`
`
   Error: ` --> 1:1
  |
1 | ```foo``
  | ^---
  |
  = expected text`

❌ **cm_example_348**: `text` (Unexpected failure)
   Input: `\`foo
`
   Error: ` --> 1:1
  |
1 | `foo
  | ^---
  |
  = expected text`

❌ **cm_example_349**: `text` (Unexpected failure)
   Input: `\`foo\`\`bar\`\`
`
   Error: ` --> 1:1
  |
1 | `foo``bar``
  | ^---
  |
  = expected text`

## commonmark_emphasis_and_strong_emphasis

❌ **cm_example_350**: `text` (Unexpected failure)
   Input: `\*foo bar\*
`
   Error: ` --> 1:1
  |
1 | *foo bar*
  | ^---
  |
  = expected text`

✅ **cm_example_351**: `text`
   Input: `a \* foo bar\*
`
   Parse Tree:
   ```
  └── text: "a "
   ```

✅ **cm_example_352**: `text`
   Input: `a\*\"foo\"\*
`
   Parse Tree:
   ```
  └── text: "a"
   ```

❌ **cm_example_353**: `text` (Unexpected failure)
   Input: `\* a \*
`
   Error: ` --> 1:1
  |
1 | * a *
  | ^---
  |
  = expected text`

✅ **cm_example_354**: `text`
   Input: `foo\*bar\*
`
   Parse Tree:
   ```
  └── text: "foo"
   ```

✅ **cm_example_355**: `text`
   Input: `5\*6\*78
`
   Parse Tree:
   ```
  └── text: "5"
   ```

❌ **cm_example_356**: `text` (Unexpected failure)
   Input: `\_foo bar\_
`
   Error: ` --> 1:1
  |
1 | _foo bar_
  | ^---
  |
  = expected text`

❌ **cm_example_357**: `text` (Unexpected failure)
   Input: `\_ foo bar\_
`
   Error: ` --> 1:1
  |
1 | _ foo bar_
  | ^---
  |
  = expected text`

✅ **cm_example_358**: `text`
   Input: `a\_\"foo\"\_
`
   Parse Tree:
   ```
  └── text: "a"
   ```

✅ **cm_example_359**: `text`
   Input: `foo\_bar\_
`
   Parse Tree:
   ```
  └── text: "foo"
   ```

✅ **cm_example_360**: `text`
   Input: `5\_6\_78
`
   Parse Tree:
   ```
  └── text: "5"
   ```

✅ **cm_example_361**: `text`
   Input: `пристаням\_стремятся\_
`
   Parse Tree:
   ```
  └── text: "пристаням"
   ```

✅ **cm_example_362**: `text`
   Input: `aa\_\"bb\"\_cc
`
   Parse Tree:
   ```
  └── text: "aa"
   ```

✅ **cm_example_363**: `text`
   Input: `foo-\_(bar)\_
`
   Parse Tree:
   ```
  └── text: "foo-"
   ```

❌ **cm_example_364**: `text` (Unexpected failure)
   Input: `\_foo\*
`
   Error: ` --> 1:1
  |
1 | _foo*
  | ^---
  |
  = expected text`

❌ **cm_example_365**: `text` (Unexpected failure)
   Input: `\*foo bar \*
`
   Error: ` --> 1:1
  |
1 | *foo bar *
  | ^---
  |
  = expected text`

❌ **cm_example_366**: `text` (Unexpected failure)
   Input: `\*foo bar
\*
`
   Error: ` --> 1:1
  |
1 | *foo bar
  | ^---
  |
  = expected text`

❌ **cm_example_367**: `text` (Unexpected failure)
   Input: `\*(\*foo)
`
   Error: ` --> 1:1
  |
1 | *(*foo)
  | ^---
  |
  = expected text`

❌ **cm_example_368**: `text` (Unexpected failure)
   Input: `\*(\*foo\*)\*
`
   Error: ` --> 1:1
  |
1 | *(*foo*)*
  | ^---
  |
  = expected text`

❌ **cm_example_369**: `text` (Unexpected failure)
   Input: `\*foo\*bar
`
   Error: ` --> 1:1
  |
1 | *foo*bar
  | ^---
  |
  = expected text`

❌ **cm_example_370**: `text` (Unexpected failure)
   Input: `\_foo bar \_
`
   Error: ` --> 1:1
  |
1 | _foo bar _
  | ^---
  |
  = expected text`

❌ **cm_example_371**: `text` (Unexpected failure)
   Input: `\_(\_foo)
`
   Error: ` --> 1:1
  |
1 | _(_foo)
  | ^---
  |
  = expected text`

❌ **cm_example_372**: `text` (Unexpected failure)
   Input: `\_(\_foo\_)\_
`
   Error: ` --> 1:1
  |
1 | _(_foo_)_
  | ^---
  |
  = expected text`

❌ **cm_example_373**: `text` (Unexpected failure)
   Input: `\_foo\_bar
`
   Error: ` --> 1:1
  |
1 | _foo_bar
  | ^---
  |
  = expected text`

❌ **cm_example_374**: `text` (Unexpected failure)
   Input: `\_пристаням\_стремятся
`
   Error: ` --> 1:1
  |
1 | _пристаням_стремятся
  | ^---
  |
  = expected text`

❌ **cm_example_375**: `text` (Unexpected failure)
   Input: `\_foo\_bar\_baz\_
`
   Error: ` --> 1:1
  |
1 | _foo_bar_baz_
  | ^---
  |
  = expected text`

❌ **cm_example_376**: `text` (Unexpected failure)
   Input: `\_(bar)\_.
`
   Error: ` --> 1:1
  |
1 | _(bar)_.
  | ^---
  |
  = expected text`

❌ **cm_example_377**: `text` (Unexpected failure)
   Input: `\*\*foo bar\*\*
`
   Error: ` --> 1:1
  |
1 | **foo bar**
  | ^---
  |
  = expected text`

❌ **cm_example_378**: `text` (Unexpected failure)
   Input: `\*\* foo bar\*\*
`
   Error: ` --> 1:1
  |
1 | ** foo bar**
  | ^---
  |
  = expected text`

✅ **cm_example_379**: `text`
   Input: `a\*\*\"foo\"\*\*
`
   Parse Tree:
   ```
  └── text: "a"
   ```

✅ **cm_example_380**: `text`
   Input: `foo\*\*bar\*\*
`
   Parse Tree:
   ```
  └── text: "foo"
   ```

❌ **cm_example_381**: `text` (Unexpected failure)
   Input: `\_\_foo bar\_\_
`
   Error: ` --> 1:1
  |
1 | __foo bar__
  | ^---
  |
  = expected text`

❌ **cm_example_382**: `text` (Unexpected failure)
   Input: `\_\_ foo bar\_\_
`
   Error: ` --> 1:1
  |
1 | __ foo bar__
  | ^---
  |
  = expected text`

❌ **cm_example_383**: `text` (Unexpected failure)
   Input: `\_\_
foo bar\_\_
`
   Error: ` --> 1:1
  |
1 | __
  | ^---
  |
  = expected text`

✅ **cm_example_384**: `text`
   Input: `a\_\_\"foo\"\_\_
`
   Parse Tree:
   ```
  └── text: "a"
   ```

✅ **cm_example_385**: `text`
   Input: `foo\_\_bar\_\_
`
   Parse Tree:
   ```
  └── text: "foo"
   ```

✅ **cm_example_386**: `text`
   Input: `5\_\_6\_\_78
`
   Parse Tree:
   ```
  └── text: "5"
   ```

✅ **cm_example_387**: `text`
   Input: `пристаням\_\_стремятся\_\_
`
   Parse Tree:
   ```
  └── text: "пристаням"
   ```

❌ **cm_example_388**: `text` (Unexpected failure)
   Input: `\_\_foo, \_\_bar\_\_, baz\_\_
`
   Error: ` --> 1:1
  |
1 | __foo, __bar__, baz__
  | ^---
  |
  = expected text`

✅ **cm_example_389**: `text`
   Input: `foo-\_\_(bar)\_\_
`
   Parse Tree:
   ```
  └── text: "foo-"
   ```

❌ **cm_example_390**: `text` (Unexpected failure)
   Input: `\*\*foo bar \*\*
`
   Error: ` --> 1:1
  |
1 | **foo bar **
  | ^---
  |
  = expected text`

❌ **cm_example_391**: `text` (Unexpected failure)
   Input: `\*\*(\*\*foo)
`
   Error: ` --> 1:1
  |
1 | **(**foo)
  | ^---
  |
  = expected text`

❌ **cm_example_392**: `text` (Unexpected failure)
   Input: `\*(\*\*foo\*\*)\*
`
   Error: ` --> 1:1
  |
1 | *(**foo**)*
  | ^---
  |
  = expected text`

❌ **cm_example_393**: `text` (Unexpected failure)
   Input: `\*\*Gomphocarpus (\*Gomphocarpus physocarpus\*, syn.
\*Asclepias physocarpa\*)\*\*
`
   Error: ` --> 1:1
  |
1 | **Gomphocarpus (*Gomphocarpus physocarpus*, syn.
  | ^---
  |
  = expected text`

❌ **cm_example_394**: `text` (Unexpected failure)
   Input: `\*\*foo \"\*bar\*\" foo\*\*
`
   Error: ` --> 1:1
  |
1 | **foo \"*bar*\" foo**
  | ^---
  |
  = expected text`

❌ **cm_example_395**: `text` (Unexpected failure)
   Input: `\*\*foo\*\*bar
`
   Error: ` --> 1:1
  |
1 | **foo**bar
  | ^---
  |
  = expected text`

❌ **cm_example_396**: `text` (Unexpected failure)
   Input: `\_\_foo bar \_\_
`
   Error: ` --> 1:1
  |
1 | __foo bar __
  | ^---
  |
  = expected text`

❌ **cm_example_397**: `text` (Unexpected failure)
   Input: `\_\_(\_\_foo)
`
   Error: ` --> 1:1
  |
1 | __(__foo)
  | ^---
  |
  = expected text`

❌ **cm_example_398**: `text` (Unexpected failure)
   Input: `\_(\_\_foo\_\_)\_
`
   Error: ` --> 1:1
  |
1 | _(__foo__)_
  | ^---
  |
  = expected text`

❌ **cm_example_399**: `text` (Unexpected failure)
   Input: `\_\_foo\_\_bar
`
   Error: ` --> 1:1
  |
1 | __foo__bar
  | ^---
  |
  = expected text`

❌ **cm_example_400**: `text` (Unexpected failure)
   Input: `\_\_пристаням\_\_стремятся
`
   Error: ` --> 1:1
  |
1 | __пристаням__стремятся
  | ^---
  |
  = expected text`

❌ **cm_example_401**: `text` (Unexpected failure)
   Input: `\_\_foo\_\_bar\_\_baz\_\_
`
   Error: ` --> 1:1
  |
1 | __foo__bar__baz__
  | ^---
  |
  = expected text`

❌ **cm_example_402**: `text` (Unexpected failure)
   Input: `\_\_(bar)\_\_.
`
   Error: ` --> 1:1
  |
1 | __(bar)__.
  | ^---
  |
  = expected text`

❌ **cm_example_403**: `text` (Unexpected failure)
   Input: `\*foo \[bar\](/url)\*
`
   Error: ` --> 1:1
  |
1 | *foo [bar](/url)*
  | ^---
  |
  = expected text`

❌ **cm_example_404**: `text` (Unexpected failure)
   Input: `\*foo
bar\*
`
   Error: ` --> 1:1
  |
1 | *foo
  | ^---
  |
  = expected text`

❌ **cm_example_405**: `text` (Unexpected failure)
   Input: `\_foo \_\_bar\_\_ baz\_
`
   Error: ` --> 1:1
  |
1 | _foo __bar__ baz_
  | ^---
  |
  = expected text`

❌ **cm_example_406**: `text` (Unexpected failure)
   Input: `\_foo \_bar\_ baz\_
`
   Error: ` --> 1:1
  |
1 | _foo _bar_ baz_
  | ^---
  |
  = expected text`

❌ **cm_example_407**: `text` (Unexpected failure)
   Input: `\_\_foo\_ bar\_
`
   Error: ` --> 1:1
  |
1 | __foo_ bar_
  | ^---
  |
  = expected text`

❌ **cm_example_408**: `text` (Unexpected failure)
   Input: `\*foo \*bar\*\*
`
   Error: ` --> 1:1
  |
1 | *foo *bar**
  | ^---
  |
  = expected text`

❌ **cm_example_409**: `text` (Unexpected failure)
   Input: `\*foo \*\*bar\*\* baz\*
`
   Error: ` --> 1:1
  |
1 | *foo **bar** baz*
  | ^---
  |
  = expected text`

❌ **cm_example_410**: `text` (Unexpected failure)
   Input: `\*foo\*\*bar\*\*baz\*
`
   Error: ` --> 1:1
  |
1 | *foo**bar**baz*
  | ^---
  |
  = expected text`

❌ **cm_example_411**: `text` (Unexpected failure)
   Input: `\*foo\*\*bar\*
`
   Error: ` --> 1:1
  |
1 | *foo**bar*
  | ^---
  |
  = expected text`

❌ **cm_example_412**: `text` (Unexpected failure)
   Input: `\*\*\*foo\*\* bar\*
`
   Error: ` --> 1:1
  |
1 | ***foo** bar*
  | ^---
  |
  = expected text`

❌ **cm_example_413**: `text` (Unexpected failure)
   Input: `\*foo \*\*bar\*\*\*
`
   Error: ` --> 1:1
  |
1 | *foo **bar***
  | ^---
  |
  = expected text`

❌ **cm_example_414**: `text` (Unexpected failure)
   Input: `\*foo\*\*bar\*\*\*
`
   Error: ` --> 1:1
  |
1 | *foo**bar***
  | ^---
  |
  = expected text`

✅ **cm_example_415**: `text`
   Input: `foo\*\*\*bar\*\*\*baz
`
   Parse Tree:
   ```
  └── text: "foo"
   ```

✅ **cm_example_416**: `text`
   Input: `foo\*\*\*\*\*\*bar\*\*\*\*\*\*\*\*\*baz
`
   Parse Tree:
   ```
  └── text: "foo"
   ```

❌ **cm_example_417**: `text` (Unexpected failure)
   Input: `\*foo \*\*bar \*baz\* bim\*\* bop\*
`
   Error: ` --> 1:1
  |
1 | *foo **bar *baz* bim** bop*
  | ^---
  |
  = expected text`

❌ **cm_example_418**: `text` (Unexpected failure)
   Input: `\*foo \[\*bar\*\](/url)\*
`
   Error: ` --> 1:1
  |
1 | *foo [*bar*](/url)*
  | ^---
  |
  = expected text`

❌ **cm_example_419**: `text` (Unexpected failure)
   Input: `\*\* is not an empty emphasis
`
   Error: ` --> 1:1
  |
1 | ** is not an empty emphasis
  | ^---
  |
  = expected text`

❌ **cm_example_420**: `text` (Unexpected failure)
   Input: `\*\*\*\* is not an empty strong emphasis
`
   Error: ` --> 1:1
  |
1 | **** is not an empty strong emphasis
  | ^---
  |
  = expected text`

❌ **cm_example_421**: `text` (Unexpected failure)
   Input: `\*\*foo \[bar\](/url)\*\*
`
   Error: ` --> 1:1
  |
1 | **foo [bar](/url)**
  | ^---
  |
  = expected text`

❌ **cm_example_422**: `text` (Unexpected failure)
   Input: `\*\*foo
bar\*\*
`
   Error: ` --> 1:1
  |
1 | **foo
  | ^---
  |
  = expected text`

❌ **cm_example_423**: `text` (Unexpected failure)
   Input: `\_\_foo \_bar\_ baz\_\_
`
   Error: ` --> 1:1
  |
1 | __foo _bar_ baz__
  | ^---
  |
  = expected text`

❌ **cm_example_424**: `text` (Unexpected failure)
   Input: `\_\_foo \_\_bar\_\_ baz\_\_
`
   Error: ` --> 1:1
  |
1 | __foo __bar__ baz__
  | ^---
  |
  = expected text`

❌ **cm_example_425**: `text` (Unexpected failure)
   Input: `\_\_\_\_foo\_\_ bar\_\_
`
   Error: ` --> 1:1
  |
1 | ____foo__ bar__
  | ^---
  |
  = expected text`

❌ **cm_example_426**: `text` (Unexpected failure)
   Input: `\*\*foo \*\*bar\*\*\*\*
`
   Error: ` --> 1:1
  |
1 | **foo **bar****
  | ^---
  |
  = expected text`

❌ **cm_example_427**: `text` (Unexpected failure)
   Input: `\*\*foo \*bar\* baz\*\*
`
   Error: ` --> 1:1
  |
1 | **foo *bar* baz**
  | ^---
  |
  = expected text`

❌ **cm_example_428**: `text` (Unexpected failure)
   Input: `\*\*foo\*bar\*baz\*\*
`
   Error: ` --> 1:1
  |
1 | **foo*bar*baz**
  | ^---
  |
  = expected text`

❌ **cm_example_429**: `text` (Unexpected failure)
   Input: `\*\*\*foo\* bar\*\*
`
   Error: ` --> 1:1
  |
1 | ***foo* bar**
  | ^---
  |
  = expected text`

❌ **cm_example_430**: `text` (Unexpected failure)
   Input: `\*\*foo \*bar\*\*\*
`
   Error: ` --> 1:1
  |
1 | **foo *bar***
  | ^---
  |
  = expected text`

❌ **cm_example_431**: `text` (Unexpected failure)
   Input: `\*\*foo \*bar \*\*baz\*\*
bim\* bop\*\*
`
   Error: ` --> 1:1
  |
1 | **foo *bar **baz**
  | ^---
  |
  = expected text`

❌ **cm_example_432**: `text` (Unexpected failure)
   Input: `\*\*foo \[\*bar\*\](/url)\*\*
`
   Error: ` --> 1:1
  |
1 | **foo [*bar*](/url)**
  | ^---
  |
  = expected text`

❌ **cm_example_433**: `text` (Unexpected failure)
   Input: `\_\_ is not an empty emphasis
`
   Error: ` --> 1:1
  |
1 | __ is not an empty emphasis
  | ^---
  |
  = expected text`

❌ **cm_example_434**: `text` (Unexpected failure)
   Input: `\_\_\_\_ is not an empty strong emphasis
`
   Error: ` --> 1:1
  |
1 | ____ is not an empty strong emphasis
  | ^---
  |
  = expected text`

✅ **cm_example_435**: `text`
   Input: `foo \*\*\*
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

✅ **cm_example_436**: `text`
   Input: `foo \*\\\*\*
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

✅ **cm_example_437**: `text`
   Input: `foo \*\_\*
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

✅ **cm_example_438**: `text`
   Input: `foo \*\*\*\*\*
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

✅ **cm_example_439**: `text`
   Input: `foo \*\*\\\*\*\*
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

✅ **cm_example_440**: `text`
   Input: `foo \*\*\_\*\*
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

❌ **cm_example_441**: `text` (Unexpected failure)
   Input: `\*\*foo\*
`
   Error: ` --> 1:1
  |
1 | **foo*
  | ^---
  |
  = expected text`

❌ **cm_example_442**: `text` (Unexpected failure)
   Input: `\*foo\*\*
`
   Error: ` --> 1:1
  |
1 | *foo**
  | ^---
  |
  = expected text`

❌ **cm_example_443**: `text` (Unexpected failure)
   Input: `\*\*\*foo\*\*
`
   Error: ` --> 1:1
  |
1 | ***foo**
  | ^---
  |
  = expected text`

❌ **cm_example_444**: `text` (Unexpected failure)
   Input: `\*\*\*\*foo\*
`
   Error: ` --> 1:1
  |
1 | ****foo*
  | ^---
  |
  = expected text`

❌ **cm_example_445**: `text` (Unexpected failure)
   Input: `\*\*foo\*\*\*
`
   Error: ` --> 1:1
  |
1 | **foo***
  | ^---
  |
  = expected text`

❌ **cm_example_446**: `text` (Unexpected failure)
   Input: `\*foo\*\*\*\*
`
   Error: ` --> 1:1
  |
1 | *foo****
  | ^---
  |
  = expected text`

✅ **cm_example_447**: `text`
   Input: `foo \_\_\_
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

✅ **cm_example_448**: `text`
   Input: `foo \_\\\_\_
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

✅ **cm_example_449**: `text`
   Input: `foo \_\*\_
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

✅ **cm_example_450**: `text`
   Input: `foo \_\_\_\_\_
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

✅ **cm_example_451**: `text`
   Input: `foo \_\_\\\_\_\_
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

✅ **cm_example_452**: `text`
   Input: `foo \_\_\*\_\_
`
   Parse Tree:
   ```
  └── text: "foo "
   ```

❌ **cm_example_453**: `text` (Unexpected failure)
   Input: `\_\_foo\_
`
   Error: ` --> 1:1
  |
1 | __foo_
  | ^---
  |
  = expected text`

❌ **cm_example_454**: `text` (Unexpected failure)
   Input: `\_foo\_\_
`
   Error: ` --> 1:1
  |
1 | _foo__
  | ^---
  |
  = expected text`

❌ **cm_example_455**: `text` (Unexpected failure)
   Input: `\_\_\_foo\_\_
`
   Error: ` --> 1:1
  |
1 | ___foo__
  | ^---
  |
  = expected text`

❌ **cm_example_456**: `text` (Unexpected failure)
   Input: `\_\_\_\_foo\_
`
   Error: ` --> 1:1
  |
1 | ____foo_
  | ^---
  |
  = expected text`

❌ **cm_example_457**: `text` (Unexpected failure)
   Input: `\_\_foo\_\_\_
`
   Error: ` --> 1:1
  |
1 | __foo___
  | ^---
  |
  = expected text`

❌ **cm_example_458**: `text` (Unexpected failure)
   Input: `\_foo\_\_\_\_
`
   Error: ` --> 1:1
  |
1 | _foo____
  | ^---
  |
  = expected text`

❌ **cm_example_459**: `text` (Unexpected failure)
   Input: `\*\*foo\*\*
`
   Error: ` --> 1:1
  |
1 | **foo**
  | ^---
  |
  = expected text`

❌ **cm_example_460**: `text` (Unexpected failure)
   Input: `\*\_foo\_\*
`
   Error: ` --> 1:1
  |
1 | *_foo_*
  | ^---
  |
  = expected text`

❌ **cm_example_461**: `text` (Unexpected failure)
   Input: `\_\_foo\_\_
`
   Error: ` --> 1:1
  |
1 | __foo__
  | ^---
  |
  = expected text`

❌ **cm_example_462**: `text` (Unexpected failure)
   Input: `\_\*foo\*\_
`
   Error: ` --> 1:1
  |
1 | _*foo*_
  | ^---
  |
  = expected text`

❌ **cm_example_463**: `text` (Unexpected failure)
   Input: `\*\*\*\*foo\*\*\*\*
`
   Error: ` --> 1:1
  |
1 | ****foo****
  | ^---
  |
  = expected text`

❌ **cm_example_464**: `text` (Unexpected failure)
   Input: `\_\_\_\_foo\_\_\_\_
`
   Error: ` --> 1:1
  |
1 | ____foo____
  | ^---
  |
  = expected text`

❌ **cm_example_465**: `text` (Unexpected failure)
   Input: `\*\*\*\*\*\*foo\*\*\*\*\*\*
`
   Error: ` --> 1:1
  |
1 | ******foo******
  | ^---
  |
  = expected text`

❌ **cm_example_466**: `text` (Unexpected failure)
   Input: `\*\*\*foo\*\*\*
`
   Error: ` --> 1:1
  |
1 | ***foo***
  | ^---
  |
  = expected text`

❌ **cm_example_467**: `text` (Unexpected failure)
   Input: `\_\_\_\_\_foo\_\_\_\_\_
`
   Error: ` --> 1:1
  |
1 | _____foo_____
  | ^---
  |
  = expected text`

❌ **cm_example_468**: `text` (Unexpected failure)
   Input: `\*foo \_bar\* baz\_
`
   Error: ` --> 1:1
  |
1 | *foo _bar* baz_
  | ^---
  |
  = expected text`

❌ **cm_example_469**: `text` (Unexpected failure)
   Input: `\*foo \_\_bar \*baz bim\_\_ bam\*
`
   Error: ` --> 1:1
  |
1 | *foo __bar *baz bim__ bam*
  | ^---
  |
  = expected text`

❌ **cm_example_470**: `text` (Unexpected failure)
   Input: `\*\*foo \*\*bar baz\*\*
`
   Error: ` --> 1:1
  |
1 | **foo **bar baz**
  | ^---
  |
  = expected text`

❌ **cm_example_471**: `text` (Unexpected failure)
   Input: `\*foo \*bar baz\*
`
   Error: ` --> 1:1
  |
1 | *foo *bar baz*
  | ^---
  |
  = expected text`

❌ **cm_example_472**: `text` (Unexpected failure)
   Input: `\*\[bar\*\](/url)
`
   Error: ` --> 1:1
  |
1 | *[bar*](/url)
  | ^---
  |
  = expected text`

❌ **cm_example_473**: `text` (Unexpected failure)
   Input: `\_foo \[bar\_\](/url)
`
   Error: ` --> 1:1
  |
1 | _foo [bar_](/url)
  | ^---
  |
  = expected text`

❌ **cm_example_474**: `text` (Unexpected failure)
   Input: `\*<img src=\"foo\" title=\"\*\"/>
`
   Error: ` --> 1:1
  |
1 | *<img src=\"foo\" title=\"*\"/>
  | ^---
  |
  = expected text`

❌ **cm_example_475**: `text` (Unexpected failure)
   Input: `\*\*<a href=\"\*\*\">
`
   Error: ` --> 1:1
  |
1 | **<a href=\"**\">
  | ^---
  |
  = expected text`

❌ **cm_example_476**: `text` (Unexpected failure)
   Input: `\_\_<a href=\"\_\_\">
`
   Error: ` --> 1:1
  |
1 | __<a href=\"__\">
  | ^---
  |
  = expected text`

❌ **cm_example_477**: `text` (Unexpected failure)
   Input: `\*a \`\*\`\*
`
   Error: ` --> 1:1
  |
1 | *a `*`*
  | ^---
  |
  = expected text`

❌ **cm_example_478**: `text` (Unexpected failure)
   Input: `\_a \`\_\`\_
`
   Error: ` --> 1:1
  |
1 | _a `_`_
  | ^---
  |
  = expected text`

❌ **cm_example_479**: `text` (Unexpected failure)
   Input: `\*\*a<http://foo.bar/?q=\*\*>
`
   Error: ` --> 1:1
  |
1 | **a<http://foo.bar/?q=**>
  | ^---
  |
  = expected text`

❌ **cm_example_480**: `text` (Unexpected failure)
   Input: `\_\_a<http://foo.bar/?q=\_\_>
`
   Error: ` --> 1:1
  |
1 | __a<http://foo.bar/?q=__>
  | ^---
  |
  = expected text`

## commonmark_blank_lines

✅ **cm_example_227**: `text`
   Input: `  

aaa
  

# aaa

  
`
   Parse Tree:
   ```
  └── text: "  

aaa
  

# aaa

  
"
   ```

## task_lists

❌ **task_incomplete**: `task_list_item` (Unexpected failure)
   Input: `- \[ \] Todo item`
   Error: ` --> 1:1
  |
1 | - [ ] Todo item
  | ^---
  |
  = expected task_list_item`

❌ **task_complete**: `task_list_item` (Unexpected failure)
   Input: `- \[x\] Done item`
   Error: ` --> 1:1
  |
1 | - [x] Done item
  | ^---
  |
  = expected task_list_item`

❌ **task_uppercase**: `task_list_item` (Unexpected failure)
   Input: `- \[X\] Also done`
   Error: ` --> 1:1
  |
1 | - [X] Also done
  | ^---
  |
  = expected task_list_item`

❌ **task_with_meta**: `task_list_item` (Unexpected failure)
   Input: `- \[ \] Task (priority: high)`
   Error: ` --> 1:1
  |
1 | - [ ] Task (priority: high)
  | ^---
  |
  = expected task_list_item`

❌ **task_complete_meta**: `task_list_item` (Unexpected failure)
   Input: `- \[x\] Completed (assignee: john)`
   Error: ` --> 1:1
  |
1 | - [x] Completed (assignee: john)
  | ^---
  |
  = expected task_list_item`

✅ **task_no_space**: `task_list_item` (Expected failure)
   Input: `-\[ \] No space`
   Error: ` --> 1:1
  |
1 | -[ ] No space
  | ^---
  |
  = expected task_list_item`

❌ **task_multiple_spaces**: `task_list_item` (Unexpected failure)
   Input: `-   \[x\]   Multiple spaces`
   Error: ` --> 1:1
  |
1 | -   [x]   Multiple spaces
  | ^---
  |
  = expected task_list_item`

✅ **task_invalid_marker**: `task_list_item` (Expected failure)
   Input: `- \[?\] Invalid marker`
   Error: ` --> 1:1
  |
1 | - [?] Invalid marker
  | ^---
  |
  = expected task_list_item`

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

## commonmark_link_reference_definitions

❌ **cm_example_192**: `text` (Unexpected failure)
   Input: `\[foo\]: /url \"title\"

\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]: /url \"title\"
  | ^---
  |
  = expected text`

❌ **cm_example_193**: `text` (Unexpected failure)
   Input: `   \[foo\]: 
      /url  
           'the title'  

\[foo\]
`
   Error: ` --> 1:1
  |
1 |    [foo]: 
  | ^---
  |
  = expected text`

❌ **cm_example_194**: `text` (Unexpected failure)
   Input: `\[Foo\*bar\\\]\]:my\_(url) 'title (with parens)'

\[Foo\*bar\\\]\]
`
   Error: ` --> 1:1
  |
1 | [Foo*bar\\]]:my_(url) 'title (with parens)'
  | ^---
  |
  = expected text`

❌ **cm_example_195**: `text` (Unexpected failure)
   Input: `\[Foo bar\]:
<my url>
'title'

\[Foo bar\]
`
   Error: ` --> 1:1
  |
1 | [Foo bar]:
  | ^---
  |
  = expected text`

❌ **cm_example_196**: `text` (Unexpected failure)
   Input: `\[foo\]: /url '
title
line1
line2
'

\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]: /url '
  | ^---
  |
  = expected text`

❌ **cm_example_197**: `text` (Unexpected failure)
   Input: `\[foo\]: /url 'title

with blank line'

\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]: /url 'title
  | ^---
  |
  = expected text`

❌ **cm_example_198**: `text` (Unexpected failure)
   Input: `\[foo\]:
/url

\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]:
  | ^---
  |
  = expected text`

❌ **cm_example_199**: `text` (Unexpected failure)
   Input: `\[foo\]:

\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]:
  | ^---
  |
  = expected text`

❌ **cm_example_200**: `text` (Unexpected failure)
   Input: `\[foo\]: <>

\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]: <>
  | ^---
  |
  = expected text`

❌ **cm_example_201**: `text` (Unexpected failure)
   Input: `\[foo\]: <bar>(baz)

\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]: <bar>(baz)
  | ^---
  |
  = expected text`

❌ **cm_example_202**: `text` (Unexpected failure)
   Input: `\[foo\]: /url\\bar\\\*baz \"foo\\\"bar\\baz\"

\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]: /url\\bar\\*baz \"foo\\\"bar\\baz\"
  | ^---
  |
  = expected text`

❌ **cm_example_203**: `text` (Unexpected failure)
   Input: `\[foo\]

\[foo\]: url
`
   Error: ` --> 1:1
  |
1 | [foo]
  | ^---
  |
  = expected text`

❌ **cm_example_204**: `text` (Unexpected failure)
   Input: `\[foo\]

\[foo\]: first
\[foo\]: second
`
   Error: ` --> 1:1
  |
1 | [foo]
  | ^---
  |
  = expected text`

❌ **cm_example_205**: `text` (Unexpected failure)
   Input: `\[FOO\]: /url

\[Foo\]
`
   Error: ` --> 1:1
  |
1 | [FOO]: /url
  | ^---
  |
  = expected text`

❌ **cm_example_206**: `text` (Unexpected failure)
   Input: `\[ΑΓΩ\]: /φου

\[αγω\]
`
   Error: ` --> 1:1
  |
1 | [ΑΓΩ]: /φου
  | ^---
  |
  = expected text`

❌ **cm_example_207**: `text` (Unexpected failure)
   Input: `\[foo\]: /url
`
   Error: ` --> 1:1
  |
1 | [foo]: /url
  | ^---
  |
  = expected text`

❌ **cm_example_208**: `text` (Unexpected failure)
   Input: `\[
foo
\]: /url
bar
`
   Error: ` --> 1:1
  |
1 | [
  | ^---
  |
  = expected text`

❌ **cm_example_209**: `text` (Unexpected failure)
   Input: `\[foo\]: /url \"title\" ok
`
   Error: ` --> 1:1
  |
1 | [foo]: /url \"title\" ok
  | ^---
  |
  = expected text`

❌ **cm_example_210**: `text` (Unexpected failure)
   Input: `\[foo\]: /url
\"title\" ok
`
   Error: ` --> 1:1
  |
1 | [foo]: /url
  | ^---
  |
  = expected text`

❌ **cm_example_211**: `text` (Unexpected failure)
   Input: `    \[foo\]: /url \"title\"

\[foo\]
`
   Error: ` --> 1:1
  |
1 |     [foo]: /url \"title\"
  | ^---
  |
  = expected text`

❌ **cm_example_212**: `text` (Unexpected failure)
   Input: `\`\`\`
\[foo\]: /url
\`\`\`

\[foo\]
`
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

✅ **cm_example_213**: `text`
   Input: `Foo
\[bar\]: /baz

\[bar\]
`
   Parse Tree:
   ```
  └── text: "Foo
"
   ```

❌ **cm_example_214**: `text` (Unexpected failure)
   Input: `# \[Foo\]
\[foo\]: /url
> bar
`
   Error: ` --> 1:1
  |
1 | # [Foo]
  | ^---
  |
  = expected text`

❌ **cm_example_215**: `text` (Unexpected failure)
   Input: `\[foo\]: /url
bar
===
\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]: /url
  | ^---
  |
  = expected text`

❌ **cm_example_216**: `text` (Unexpected failure)
   Input: `\[foo\]: /url
===
\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]: /url
  | ^---
  |
  = expected text`

❌ **cm_example_217**: `text` (Unexpected failure)
   Input: `\[foo\]: /foo-url \"foo\"
\[bar\]: /bar-url
  \"bar\"
\[baz\]: /baz-url

\[foo\],
\[bar\],
\[baz\]
`
   Error: ` --> 1:1
  |
1 | [foo]: /foo-url \"foo\"
  | ^---
  |
  = expected text`

❌ **cm_example_218**: `text` (Unexpected failure)
   Input: `\[foo\]

> \[foo\]: /url
`
   Error: ` --> 1:1
  |
1 | [foo]
  | ^---
  |
  = expected text`

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
      ├── inline > "Quote with "
        ├── inline_core > "Quote with "
          └── text: "Quote with "
      ├── inline > "`code`"
        ├── inline_core > "`code`"
          └── code_inline: "`code`"
   ```

✅ **quote_with_link**: `blockquote`
   Input: `> Quote with \[link\](url)`
   Parse Tree:
   ```
  ├── blockquote > "> Quote with "
    ├── blockquote_line > "> Quote with "
      ├── inline > "Quote with "
        ├── inline_core > "Quote with "
          └── text: "Quote with "
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
  └── text: "According to Smith et al. (2023)"
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

❌ **api_doc**: `text` (Unexpected failure)
   Input: `\`GET /api/v1/users/{id}\` returns user data`
   Error: ` --> 1:1
  |
1 | `GET /api/v1/users/{id}` returns user data
  | ^---
  |
  = expected text`

✅ **code_with_backticks**: `text`
   Input: `Use \`\\\`\` to escape backticks in code`
   Parse Tree:
   ```
  └── text: "Use "
   ```

✅ **regex_example**: `text`
   Input: `Pattern: \`/^\[a-zA-Z0-9\]+$/g\``
   Parse Tree:
   ```
  └── text: "Pattern: "
   ```

✅ **code_switching**: `text`
   Input: `In Python, use \`print()\`, but in Rust use \`println!()\``
   Parse Tree:
   ```
  └── text: "In Python, use "
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

❌ **perf_nested_structures**: `text` (Unexpected failure)
   Input: `> Quote with \*\*bold\*\* and \*italic\*
> 
> Another line`
   Error: ` --> 1:1
  |
1 | > Quote with **bold** and *italic*
  | ^---
  |
  = expected text`

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
            ├── safe_inline > "Project"
              └── word: "Project"
            ├── safe_inline > "Title"
              └── word: "Title"
    ├── block > "[![Build Status](badge.svg)](link)"
      └── unknown_block: "[![Build Status](badge.svg)](link)"
    ├── block > "## Description"
      ├── heading > "## Description"
        ├── H2 > "## Description"
          ├── heading_content > "Description"
            ├── safe_inline > "Description"
              └── word: "Description"
    ├── block > "This project does amazing things.

### Installation

"
      ├── paragraph > "This project does amazing things.

### Installation

"
        ├── paragraph_line > "This project does amazing things.

### Installation

"
          ├── inline > "This project does amazing things.

### Installation

"
            ├── inline_core > "This project does amazing things.

### Installation

"
              └── text: "This project does amazing things.

### Installation

"
    ├── block > "```bash
npm install
```"
      ├── code_block > "```bash
npm install
```"
        ├── fenced_code > "```bash
npm install
```"
          └── language_id: "bash"
    ├── block > "### Usage"
      ├── heading > "### Usage"
        ├── H3 > "### Usage"
          ├── heading_content > "Usage"
            ├── safe_inline > "Usage"
              └── word: "Usage"
    ├── block > "```javascript
const lib = require('lib');
```"
      ├── code_block > "```javascript
const lib = require('lib');
```"
        ├── fenced_code > "```javascript
const lib = require('lib');
```"
          └── language_id: "javascript"
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
            ├── safe_inline > "Abstract"
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
          ├── inline > "This paper presents novel findings"
            ├── inline_core > "This paper presents novel findings"
              └── text: "This paper presents novel findings"
          ├── inline > "[^1]"
            ├── inline_core > "[^1]"
              ├── footnote_ref > "[^1]"
                └── footnote_label: "1"
          ├── inline > ".

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

"
            ├── inline_core > ".

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

"
              └── text: ".

## Introduction

According to Smith et al. (2023), this is important.

## Methodology

We used the following approach:

1. Data collection
2. Analysis
3. Validation

"
          ├── inline > "[^1]"
            ├── inline_core > "[^1]"
              ├── footnote_ref > "[^1]"
                └── footnote_label: "1"
          ├── inline > ": Important reference here"
            ├── inline_core > ": Important reference here"
              └── text: ": Important reference here"
   ```

❌ **perf_many_small_elements**: `text` (Unexpected failure)
   Input: `\`code1\` \`code2\` \`code3\` \`code4\` \`code5\` \`code6\` \`code7\` \`code8\` \`code9\` \`code10\``
   Error: ` --> 1:1
  |
1 | `code1` `code2` `code3` `code4` `code5` `code6` `code7` `code8` `code9` `code10`
  | ^---
  |
  = expected text`

❌ **perf_few_large_elements**: `text` (Unexpected failure)
   Input: `\`\`\`
very long code block with lots of content
that spans multiple lines and contains
various programming constructs and
other text that needs to be parsed
efficiently by the parser
\`\`\``
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

❌ **perf_shallow_wide**: `text` (Unexpected failure)
   Input: `\*\*bold1\*\* \*\*bold2\*\* \*\*bold3\*\* \*\*bold4\*\* \*\*bold5\*\* \*\*bold6\*\* \*\*bold7\*\* \*\*bold8\*\*`
   Error: ` --> 1:1
  |
1 | **bold1** **bold2** **bold3** **bold4** **bold5** **bold6** **bold7** **bold8**
  | ^---
  |
  = expected text`

❌ **perf_deep_narrow**: `text` (Unexpected failure)
   Input: `\*\*bold \*italic \`code\` italic\* bold\*\*`
   Error: ` --> 1:1
  |
1 | **bold *italic `code` italic* bold**
  | ^---
  |
  = expected text`

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
  └── inline_url: "https://example.com/path/to/page"
   ```

✅ **url_with_query**: `inline_url`
   Input: `https://example.com/search?q=test&lang=en`
   Parse Tree:
   ```
  └── inline_url: "https://example.com/search?q=test&lang=en"
   ```

✅ **url_with_fragment**: `inline_url`
   Input: `https://example.com/page#section`
   Parse Tree:
   ```
  └── inline_url: "https://example.com/page#section"
   ```

✅ **url_complex**: `inline_url`
   Input: `https://subdomain.example.com:8080/path/to/page?param1=value1&param2=value2#section`
   Parse Tree:
   ```
  └── inline_url: "https://subdomain.example.com:8080/path/to/page?param1=value1&param2=value2#section"
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

## specification_compliance

❌ **gfm_table_basic**: `table` (Unexpected failure)
   Input: `| foo | bar |
| --- | --- |
| baz | bim |`
   Error: ` --> 3:14
  |
3 | | baz | bim |
  |              ^---
  |
  = expected inline_core`

❌ **gfm_table_alignment**: `table` (Unexpected failure)
   Input: `| left | center | right |
|:-----|:------:|------:|
| L    | C      | R     |`
   Error: ` --> 3:26
  |
3 | | L    | C      | R     |
  |                          ^---
  |
  = expected inline_core`

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

❌ **gfm_task_list**: `task_list_item` (Unexpected failure)
   Input: `- \[x\] foo
  - \[ \] bar
  - \[x\] baz
- \[ \] bim`
   Error: ` --> 1:1
  |
1 | - [x] foo
  | ^---
  |
  = expected task_list_item`

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

❌ **mmd_table_caption**: `table` (Unexpected failure)
   Input: `| foo | bar |
|-----|-----|
| baz | bim |
\[Table caption\]`
   Error: ` --> 4:2
  |
4 | [Table caption]
  |  ^---
  |
  = expected KW_PAGE or KW_TOC`

✅ **mmd_footnote_inline**: `text`
   Input: `Here is some text^\[and a footnote\]`
   Parse Tree:
   ```
  └── text: "Here is some text^"
   ```

## commonmark_images

✅ **cm_example_571**: `text`
   Input: `!\[foo\](/url \"title\")
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_572**: `text`
   Input: `!\[foo \*bar\*\]

\[foo \*bar\*\]: train.jpg \"train & tracks\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_573**: `text`
   Input: `!\[foo !\[bar\](/url)\](/url2)
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_574**: `text`
   Input: `!\[foo \[bar\](/url)\](/url2)
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_575**: `text`
   Input: `!\[foo \*bar\*\]\[\]

\[foo \*bar\*\]: train.jpg \"train & tracks\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_576**: `text`
   Input: `!\[foo \*bar\*\]\[foobar\]

\[FOOBAR\]: train.jpg \"train & tracks\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_577**: `text`
   Input: `!\[foo\](train.jpg)
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_578**: `text`
   Input: `My !\[foo bar\](/path/to/train.jpg  \"title\"   )
`
   Parse Tree:
   ```
  └── text: "My !"
   ```

✅ **cm_example_579**: `text`
   Input: `!\[foo\](<url>)
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_580**: `text`
   Input: `!\[\](/url)
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_581**: `text`
   Input: `!\[foo\]\[bar\]

\[bar\]: /url
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_582**: `text`
   Input: `!\[foo\]\[bar\]

\[BAR\]: /url
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_583**: `text`
   Input: `!\[foo\]\[\]

\[foo\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_584**: `text`
   Input: `!\[\*foo\* bar\]\[\]

\[\*foo\* bar\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_585**: `text`
   Input: `!\[Foo\]\[\]

\[foo\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_586**: `text`
   Input: `!\[foo\] 
\[\]

\[foo\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_587**: `text`
   Input: `!\[foo\]

\[foo\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_588**: `text`
   Input: `!\[\*foo\* bar\]

\[\*foo\* bar\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_589**: `text`
   Input: `!\[\[foo\]\]

\[\[foo\]\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_590**: `text`
   Input: `!\[Foo\]

\[foo\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "!"
   ```

✅ **cm_example_591**: `text`
   Input: `!\\\[foo\]

\[foo\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "!\\"
   ```

✅ **cm_example_592**: `text`
   Input: `\\!\[foo\]

\[foo\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "\\!"
   ```

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

✅ **ref_def_basic**: `reference_definition`
   Input: `\[ref\]: https://example.com`
   Parse Tree:
   ```
  ├── reference_definition > "[ref]: https://example.com"
    └── block_caption: "ref"
    └── inline_url: "https://example.com"
    └── EOI: ""
   ```

❌ **ref_def_with_title**: `reference_definition` (Unexpected failure)
   Input: `\[ref\]: https://example.com \"Title\"`
   Error: ` --> 1:28
  |
1 | [ref]: https://example.com \"Title\"
  |                            ^---
  |
  = expected EOI`

❌ **ref_def_with_spaces**: `reference_definition` (Unexpected failure)
   Input: `\[ref\]:   https://example.com   \"Title\"   `
   Error: ` --> 1:32
  |
1 | [ref]:   https://example.com   \"Title\"   
  |                                ^---
  |
  = expected EOI`

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

## commonmark_conformance

✅ **cm_atx_basic**: `heading`
   Input: `# foo`
   Parse Tree:
   ```
  ├── heading > "# foo"
    ├── H1 > "# foo"
      ├── heading_content > "foo"
        ├── safe_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_multiple**: `heading`
   Input: `## foo`
   Parse Tree:
   ```
  ├── heading > "## foo"
    ├── H2 > "## foo"
      ├── heading_content > "foo"
        ├── safe_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_max_level**: `heading`
   Input: `###### foo`
   Parse Tree:
   ```
  ├── heading > "###### foo"
    ├── H6 > "###### foo"
      ├── heading_content > "foo"
        ├── safe_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_no_space**: `heading`
   Input: `#5 bolt`
   Parse Tree:
   ```
  ├── heading > "#5 bolt"
    ├── H1 > "#5 bolt"
      ├── heading_content > "5 bolt"
        ├── safe_inline > "5"
          └── word: "5"
        ├── safe_inline > "bolt"
          └── word: "bolt"
   ```

❌ **cm_atx_escaped**: `heading` (Unexpected failure)
   Input: `\\## foo`
   Error: ` --> 1:3
  |
1 | \\## foo
  |   ^---
  |
  = expected safe_inline`

✅ **cm_atx_content_formatting**: `heading`
   Input: `# foo \*bar\* \\\*baz\\\*`
   Parse Tree:
   ```
  ├── heading > "# foo "
    ├── H1 > "# foo "
      ├── heading_content > "foo "
        ├── safe_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_spaces_after**: `heading`
   Input: `#                  foo                     `
   Parse Tree:
   ```
  ├── heading > "#                  foo                     "
    ├── H1 > "#                  foo                     "
      ├── heading_content > "foo                     "
        ├── safe_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_trailing_spaces**: `heading`
   Input: `### foo ### `
   Parse Tree:
   ```
  ├── heading > "### foo "
    ├── H3 > "### foo "
      ├── heading_content > "foo "
        ├── safe_inline > "foo"
          └── word: "foo"
   ```

✅ **cm_atx_trailing_hash_count**: `heading`
   Input: `### foo #### `
   Parse Tree:
   ```
  ├── heading > "### foo "
    ├── H3 > "### foo "
      ├── heading_content > "foo "
        ├── safe_inline > "foo"
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
      ├── safe_inline > "Foo"
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
      ├── safe_inline > "Foo"
        └── word: "Foo"
   ```

❌ **cm_setext_content**: `setext_h2` (Unexpected failure)
   Input: `Foo \*bar\*
=========`
   Error: ` --> 1:5
  |
1 | Foo *bar*
  |     ^---
  |
  = expected safe_inline`

❌ **cm_setext_underline_count**: `setext_h2` (Unexpected failure)
   Input: `Foo
=========================`
   Error: ` --> 1:4
  |
1 | Foo␊
  |    ^---
  |
  = expected safe_inline`

✅ **cm_setext_spaces**: `setext_h2`
   Input: `   Foo
---`
   Parse Tree:
   ```
  ├── setext_h2 > "   Foo
---"
    ├── heading_content > "   Foo"
      └── safe_inline: " "
      ├── safe_inline > "Foo"
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
  = expected safe_inline`

❌ **cm_setext_lazy**: `setext_h2` (Unexpected failure)
   Input: `Foo
Bar
---`
   Error: ` --> 1:4
  |
1 | Foo␊
  |    ^---
  |
  = expected safe_inline`

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

❌ **cm_emphasis_newline_fail**: `emphasis` (Unexpected failure)
   Input: `\*foo
bar\*`
   Error: ` --> 1:1
  |
1 | *foo
  | ^---
  |
  = expected emphasis`

✅ **cm_link_basic**: `inline_link`
   Input: `\[link\](/uri)`
   Parse Tree:
   ```
  ├── inline_link > "[link](/uri)"
    └── inline_link_text: "link"
    └── inline_url: "/uri"
   ```

✅ **cm_link_title**: `inline_link`
   Input: `\[link\](/uri \"title\")`
   Parse Tree:
   ```
  ├── inline_link > "[link](/uri \"title\")"
    └── inline_link_text: "link"
    └── inline_url: "/uri \"title\""
   ```

✅ **cm_link_empty**: `inline_link` (Expected failure)
   Input: `\[\]()`
   Error: ` --> 1:4
  |
1 | []()
  |    ^---
  |
  = expected inline_url`

✅ **cm_link_with_parens**: `inline_link`
   Input: `\[link\](/uri(and(nested)))`
   Parse Tree:
   ```
  ├── inline_link > "[link](/uri(and(nested)"
    └── inline_link_text: "link"
    └── inline_url: "/uri(and(nested"
   ```

✅ **cm_link_escaped_parens**: `inline_link`
   Input: `\[link\](/uri\\(paren\\))`
   Parse Tree:
   ```
  ├── inline_link > "[link](/uri\\(paren\\)"
    └── inline_link_text: "link"
    └── inline_url: "/uri\\(paren\\"
   ```

❌ **cm_autolink_uri**: `inline_link` (Unexpected failure)
   Input: `<http://foo.bar.baz>`
   Error: ` --> 1:1
  |
1 | <http://foo.bar.baz>
  | ^---
  |
  = expected inline_link`

❌ **cm_autolink_email**: `inline_link` (Unexpected failure)
   Input: `<foo@bar.example.com>`
   Error: ` --> 1:1
  |
1 | <foo@bar.example.com>
  | ^---
  |
  = expected inline_link`

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

## commonmark_raw_html

❌ **cm_example_612**: `text` (Unexpected failure)
   Input: `<a><bab><c2c>
`
   Error: ` --> 1:1
  |
1 | <a><bab><c2c>
  | ^---
  |
  = expected text`

❌ **cm_example_613**: `text` (Unexpected failure)
   Input: `<a/><b2/>
`
   Error: ` --> 1:1
  |
1 | <a/><b2/>
  | ^---
  |
  = expected text`

❌ **cm_example_614**: `text` (Unexpected failure)
   Input: `<a  /><b2
data=\"foo\" >
`
   Error: ` --> 1:1
  |
1 | <a  /><b2
  | ^---
  |
  = expected text`

❌ **cm_example_615**: `text` (Unexpected failure)
   Input: `<a foo=\"bar\" bam = 'baz <em>\"</em>'
\_boolean zoop:33=zoop:33 />
`
   Error: ` --> 1:1
  |
1 | <a foo=\"bar\" bam = 'baz <em>\"</em>'
  | ^---
  |
  = expected text`

✅ **cm_example_616**: `text`
   Input: `Foo <responsive-image src=\"foo.jpg\" />
`
   Parse Tree:
   ```
  └── text: "Foo <responsive-image src=\"foo.jpg\" />
"
   ```

✅ **cm_example_617**: `text`
   Input: `<33> <\_\_>
`
   Parse Tree:
   ```
  └── text: "<33> <"
   ```

❌ **cm_example_618**: `text` (Unexpected failure)
   Input: `<a h\*#ref=\"hi\">
`
   Error: ` --> 1:1
  |
1 | <a h*#ref=\"hi\">
  | ^---
  |
  = expected text`

❌ **cm_example_619**: `text` (Unexpected failure)
   Input: `<a href=\"hi'> <a href=hi'>
`
   Error: ` --> 1:1
  |
1 | <a href=\"hi'> <a href=hi'>
  | ^---
  |
  = expected text`

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

❌ **cm_example_621**: `text` (Unexpected failure)
   Input: `<a href='bar'title=title>
`
   Error: ` --> 1:1
  |
1 | <a href='bar'title=title>
  | ^---
  |
  = expected text`

✅ **cm_example_622**: `text`
   Input: `</a></foo >
`
   Parse Tree:
   ```
  └── text: "</a></foo >
"
   ```

✅ **cm_example_623**: `text`
   Input: `</a href=\"foo\">
`
   Parse Tree:
   ```
  └── text: "</a href=\"foo\">
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
  └── text: "foo <?php echo "
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
  └── text: "foo <!"
   ```

✅ **cm_example_630**: `text`
   Input: `foo <a href=\"&ouml;\">
`
   Parse Tree:
   ```
  └── text: "foo <a href=\"&ouml;\">
"
   ```

✅ **cm_example_631**: `text`
   Input: `foo <a href=\"\\\*\">
`
   Parse Tree:
   ```
  └── text: "foo <a href=\"\\"
   ```

❌ **cm_example_632**: `text` (Unexpected failure)
   Input: `<a href=\"\\\"\">
`
   Error: ` --> 1:1
  |
1 | <a href=\"\\\"\">
  | ^---
  |
  = expected text`

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
  └── text: "!@#"
   ```

❌ **markdown_specials**: `text` (Unexpected failure)
   Input: `\*\_\`#\[\]~>|$@^=-`
   Error: ` --> 1:1
  |
1 | *_`#[]~>|$@^=-
  | ^---
  |
  = expected text`

✅ **empty_string**: `text` (Expected failure)
   Input: ``
   Error: ` --> 1:1
  |
1 | 
  | ^---
  |
  = expected text`

❌ **only_spaces**: `text` (Unexpected failure)
   Input: `   `
   Error: ` --> 1:1
  |
1 |    
  | ^---
  |
  = expected text`

❌ **only_tabs**: `text` (Unexpected failure)
   Input: `		`
   Error: ` --> 1:1
  |
1 | 		
  | ^---
  |
  = expected text`

❌ **mixed_whitespace**: `text` (Unexpected failure)
   Input: ` 	 	 `
   Error: ` --> 1:1
  |
1 |  	 	 
  | ^---
  |
  = expected text`

✅ **very_long_text**: `text`
   Input: `This is a very long text string that should test how the parser handles extended content without any special formatting or markdown syntax just plain text that goes on and on and should continue to parse correctly even with this much content`
   Parse Tree:
   ```
  └── text: "This is a very long text string that should test how the parser handles extended content without any special formatting or markdown syntax just plain text that goes on and on and should continue to parse correctly even with this much content"
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
  └── text: "foo\\
baz
"
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
  └── text: "foo\\
     bar
"
   ```

❌ **cm_example_638**: `text` (Unexpected failure)
   Input: `\*foo  
bar\*
`
   Error: ` --> 1:1
  |
1 | *foo  
  | ^---
  |
  = expected text`

❌ **cm_example_639**: `text` (Unexpected failure)
   Input: `\*foo\\
bar\*
`
   Error: ` --> 1:1
  |
1 | *foo\\
  | ^---
  |
  = expected text`

❌ **cm_example_640**: `text` (Unexpected failure)
   Input: `\`code  
span\`
`
   Error: ` --> 1:1
  |
1 | `code  
  | ^---
  |
  = expected text`

❌ **cm_example_641**: `text` (Unexpected failure)
   Input: `\`code\\
span\`
`
   Error: ` --> 1:1
  |
1 | `code\\
  | ^---
  |
  = expected text`

❌ **cm_example_642**: `text` (Unexpected failure)
   Input: `<a href=\"foo  
bar\">
`
   Error: ` --> 1:1
  |
1 | <a href=\"foo  
  | ^---
  |
  = expected text`

❌ **cm_example_643**: `text` (Unexpected failure)
   Input: `<a href=\"foo\\
bar\">
`
   Error: ` --> 1:1
  |
1 | <a href=\"foo\\
  | ^---
  |
  = expected text`

✅ **cm_example_644**: `text`
   Input: `foo\\
`
   Parse Tree:
   ```
  └── text: "foo\\
"
   ```

✅ **cm_example_645**: `text`
   Input: `foo  
`
   Parse Tree:
   ```
  └── text: "foo  
"
   ```

❌ **cm_example_646**: `text` (Unexpected failure)
   Input: `### foo\\
`
   Error: ` --> 1:1
  |
1 | ### foo\\
  | ^---
  |
  = expected text`

❌ **cm_example_647**: `text` (Unexpected failure)
   Input: `### foo  
`
   Error: ` --> 1:1
  |
1 | ### foo  
  | ^---
  |
  = expected text`

## escaped_characters

❌ **escaped_asterisk**: `escaped_char` (Unexpected failure)
   Input: `\\\*not bold\\\*`
   Error: ` --> 1:1
  |
1 | \\*not bold\\*
  | ^---
  |
  = expected escaped_char`

❌ **escaped_underscore**: `escaped_char` (Unexpected failure)
   Input: `\\\_not italic\\\_`
   Error: ` --> 1:1
  |
1 | \\_not italic\\_
  | ^---
  |
  = expected escaped_char`

❌ **escaped_backtick**: `escaped_char` (Unexpected failure)
   Input: `\\\`not code\\\``
   Error: ` --> 1:1
  |
1 | \\`not code\\`
  | ^---
  |
  = expected escaped_char`

❌ **escaped_hash**: `escaped_char` (Unexpected failure)
   Input: `\\# not heading`
   Error: ` --> 1:1
  |
1 | \\# not heading
  | ^---
  |
  = expected escaped_char`

❌ **escaped_bracket**: `escaped_char` (Unexpected failure)
   Input: `\\\[not link\\\]`
   Error: ` --> 1:1
  |
1 | \\[not link\\]
  | ^---
  |
  = expected escaped_char`

❌ **escaped_tilde**: `escaped_char` (Unexpected failure)
   Input: `\\~not strikethrough\\~`
   Error: ` --> 1:1
  |
1 | \\~not strikethrough\\~
  | ^---
  |
  = expected escaped_char`

❌ **escaped_greater**: `escaped_char` (Unexpected failure)
   Input: `\\> not blockquote`
   Error: ` --> 1:1
  |
1 | \\> not blockquote
  | ^---
  |
  = expected escaped_char`

❌ **escaped_pipe**: `escaped_char` (Unexpected failure)
   Input: `\\| not table`
   Error: ` --> 1:1
  |
1 | \\| not table
  | ^---
  |
  = expected escaped_char`

❌ **escaped_dollar**: `escaped_char` (Unexpected failure)
   Input: `\\$ not math`
   Error: ` --> 1:1
  |
1 | \\$ not math
  | ^---
  |
  = expected escaped_char`

❌ **escaped_at**: `escaped_char` (Unexpected failure)
   Input: `\\@ not mention`
   Error: ` --> 1:1
  |
1 | \\@ not mention
  | ^---
  |
  = expected escaped_char`

❌ **escaped_caret**: `escaped_char` (Unexpected failure)
   Input: `\\^ not superscript`
   Error: ` --> 1:1
  |
1 | \\^ not superscript
  | ^---
  |
  = expected escaped_char`

❌ **escaped_equals**: `escaped_char` (Unexpected failure)
   Input: `\\= not highlight`
   Error: ` --> 1:1
  |
1 | \\= not highlight
  | ^---
  |
  = expected escaped_char`

❌ **escaped_dash**: `escaped_char` (Unexpected failure)
   Input: `\\- not list`
   Error: ` --> 1:1
  |
1 | \\- not list
  | ^---
  |
  = expected escaped_char`

❌ **multiple_escapes**: `escaped_char` (Unexpected failure)
   Input: `\\\*\\\*not bold\\\*\\\*`
   Error: ` --> 1:1
  |
1 | \\*\\*not bold\\*\\*
  | ^---
  |
  = expected escaped_char`

❌ **escaped_in_text**: `escaped_char` (Unexpected failure)
   Input: `This is \\\*not\\\* bold text`
   Error: ` --> 1:1
  |
1 | This is \\*not\\* bold text
  | ^---
  |
  = expected escaped_char`

## commonmark_links

❌ **cm_example_481**: `text` (Unexpected failure)
   Input: `\[link\](/uri \"title\")
`
   Error: ` --> 1:1
  |
1 | [link](/uri \"title\")
  | ^---
  |
  = expected text`

❌ **cm_example_482**: `text` (Unexpected failure)
   Input: `\[link\](/uri)
`
   Error: ` --> 1:1
  |
1 | [link](/uri)
  | ^---
  |
  = expected text`

❌ **cm_example_483**: `text` (Unexpected failure)
   Input: `\[\](./target.md)
`
   Error: ` --> 1:1
  |
1 | [](./target.md)
  | ^---
  |
  = expected text`

❌ **cm_example_484**: `text` (Unexpected failure)
   Input: `\[link\]()
`
   Error: ` --> 1:1
  |
1 | [link]()
  | ^---
  |
  = expected text`

❌ **cm_example_485**: `text` (Unexpected failure)
   Input: `\[link\](<>)
`
   Error: ` --> 1:1
  |
1 | [link](<>)
  | ^---
  |
  = expected text`

❌ **cm_example_486**: `text` (Unexpected failure)
   Input: `\[\]()
`
   Error: ` --> 1:1
  |
1 | []()
  | ^---
  |
  = expected text`

❌ **cm_example_487**: `text` (Unexpected failure)
   Input: `\[link\](/my uri)
`
   Error: ` --> 1:1
  |
1 | [link](/my uri)
  | ^---
  |
  = expected text`

❌ **cm_example_488**: `text` (Unexpected failure)
   Input: `\[link\](</my uri>)
`
   Error: ` --> 1:1
  |
1 | [link](</my uri>)
  | ^---
  |
  = expected text`

❌ **cm_example_489**: `text` (Unexpected failure)
   Input: `\[link\](foo
bar)
`
   Error: ` --> 1:1
  |
1 | [link](foo
  | ^---
  |
  = expected text`

❌ **cm_example_490**: `text` (Unexpected failure)
   Input: `\[link\](<foo
bar>)
`
   Error: ` --> 1:1
  |
1 | [link](<foo
  | ^---
  |
  = expected text`

❌ **cm_example_491**: `text` (Unexpected failure)
   Input: `\[a\](<b)c>)
`
   Error: ` --> 1:1
  |
1 | [a](<b)c>)
  | ^---
  |
  = expected text`

❌ **cm_example_492**: `text` (Unexpected failure)
   Input: `\[link\](<foo\\>)
`
   Error: ` --> 1:1
  |
1 | [link](<foo\\>)
  | ^---
  |
  = expected text`

❌ **cm_example_493**: `text` (Unexpected failure)
   Input: `\[a\](<b)c
\[a\](<b)c>
\[a\](<b>c)
`
   Error: ` --> 1:1
  |
1 | [a](<b)c
  | ^---
  |
  = expected text`

❌ **cm_example_494**: `text` (Unexpected failure)
   Input: `\[link\](\\(foo\\))
`
   Error: ` --> 1:1
  |
1 | [link](\\(foo\\))
  | ^---
  |
  = expected text`

❌ **cm_example_495**: `text` (Unexpected failure)
   Input: `\[link\](foo(and(bar)))
`
   Error: ` --> 1:1
  |
1 | [link](foo(and(bar)))
  | ^---
  |
  = expected text`

❌ **cm_example_496**: `text` (Unexpected failure)
   Input: `\[link\](foo(and(bar))
`
   Error: ` --> 1:1
  |
1 | [link](foo(and(bar))
  | ^---
  |
  = expected text`

❌ **cm_example_497**: `text` (Unexpected failure)
   Input: `\[link\](foo\\(and\\(bar\\))
`
   Error: ` --> 1:1
  |
1 | [link](foo\\(and\\(bar\\))
  | ^---
  |
  = expected text`

❌ **cm_example_498**: `text` (Unexpected failure)
   Input: `\[link\](<foo(and(bar)>)
`
   Error: ` --> 1:1
  |
1 | [link](<foo(and(bar)>)
  | ^---
  |
  = expected text`

❌ **cm_example_499**: `text` (Unexpected failure)
   Input: `\[link\](foo\\)\\:)
`
   Error: ` --> 1:1
  |
1 | [link](foo\\)\\:)
  | ^---
  |
  = expected text`

❌ **cm_example_500**: `text` (Unexpected failure)
   Input: `\[link\](#fragment)

\[link\](http://example.com#fragment)

\[link\](http://example.com?foo=3#frag)
`
   Error: ` --> 1:1
  |
1 | [link](#fragment)
  | ^---
  |
  = expected text`

❌ **cm_example_501**: `text` (Unexpected failure)
   Input: `\[link\](foo\\bar)
`
   Error: ` --> 1:1
  |
1 | [link](foo\\bar)
  | ^---
  |
  = expected text`

❌ **cm_example_502**: `text` (Unexpected failure)
   Input: `\[link\](foo%20b&auml;)
`
   Error: ` --> 1:1
  |
1 | [link](foo%20b&auml;)
  | ^---
  |
  = expected text`

❌ **cm_example_503**: `text` (Unexpected failure)
   Input: `\[link\](\"title\")
`
   Error: ` --> 1:1
  |
1 | [link](\"title\")
  | ^---
  |
  = expected text`

❌ **cm_example_504**: `text` (Unexpected failure)
   Input: `\[link\](/url \"title\")
\[link\](/url 'title')
\[link\](/url (title))
`
   Error: ` --> 1:1
  |
1 | [link](/url \"title\")
  | ^---
  |
  = expected text`

❌ **cm_example_505**: `text` (Unexpected failure)
   Input: `\[link\](/url \"title \\\"&quot;\")
`
   Error: ` --> 1:1
  |
1 | [link](/url \"title \\\"&quot;\")
  | ^---
  |
  = expected text`

❌ **cm_example_506**: `text` (Unexpected failure)
   Input: `\[link\](/url \"title\")
`
   Error: ` --> 1:1
  |
1 | [link](/url \"title\")
  | ^---
  |
  = expected text`

❌ **cm_example_507**: `text` (Unexpected failure)
   Input: `\[link\](/url \"title \"and\" title\")
`
   Error: ` --> 1:1
  |
1 | [link](/url \"title \"and\" title\")
  | ^---
  |
  = expected text`

❌ **cm_example_508**: `text` (Unexpected failure)
   Input: `\[link\](/url 'title \"and\" title')
`
   Error: ` --> 1:1
  |
1 | [link](/url 'title \"and\" title')
  | ^---
  |
  = expected text`

❌ **cm_example_509**: `text` (Unexpected failure)
   Input: `\[link\](   /uri
  \"title\"  )
`
   Error: ` --> 1:1
  |
1 | [link](   /uri
  | ^---
  |
  = expected text`

❌ **cm_example_510**: `text` (Unexpected failure)
   Input: `\[link\] (/uri)
`
   Error: ` --> 1:1
  |
1 | [link] (/uri)
  | ^---
  |
  = expected text`

❌ **cm_example_511**: `text` (Unexpected failure)
   Input: `\[link \[foo \[bar\]\]\](/uri)
`
   Error: ` --> 1:1
  |
1 | [link [foo [bar]]](/uri)
  | ^---
  |
  = expected text`

❌ **cm_example_512**: `text` (Unexpected failure)
   Input: `\[link\] bar\](/uri)
`
   Error: ` --> 1:1
  |
1 | [link] bar](/uri)
  | ^---
  |
  = expected text`

❌ **cm_example_513**: `text` (Unexpected failure)
   Input: `\[link \[bar\](/uri)
`
   Error: ` --> 1:1
  |
1 | [link [bar](/uri)
  | ^---
  |
  = expected text`

❌ **cm_example_514**: `text` (Unexpected failure)
   Input: `\[link \\\[bar\](/uri)
`
   Error: ` --> 1:1
  |
1 | [link \\[bar](/uri)
  | ^---
  |
  = expected text`

❌ **cm_example_515**: `text` (Unexpected failure)
   Input: `\[link \*foo \*\*bar\*\* \`#\`\*\](/uri)
`
   Error: ` --> 1:1
  |
1 | [link *foo **bar** `#`*](/uri)
  | ^---
  |
  = expected text`

❌ **cm_example_516**: `text` (Unexpected failure)
   Input: `\[!\[moon\](moon.jpg)\](/uri)
`
   Error: ` --> 1:1
  |
1 | [![moon](moon.jpg)](/uri)
  | ^---
  |
  = expected text`

❌ **cm_example_517**: `text` (Unexpected failure)
   Input: `\[foo \[bar\](/uri)\](/uri)
`
   Error: ` --> 1:1
  |
1 | [foo [bar](/uri)](/uri)
  | ^---
  |
  = expected text`

❌ **cm_example_518**: `text` (Unexpected failure)
   Input: `\[foo \*\[bar \[baz\](/uri)\](/uri)\*\](/uri)
`
   Error: ` --> 1:1
  |
1 | [foo *[bar [baz](/uri)](/uri)*](/uri)
  | ^---
  |
  = expected text`

✅ **cm_example_519**: `text`
   Input: `!\[\[\[foo\](uri1)\](uri2)\](uri3)
`
   Parse Tree:
   ```
  └── text: "!"
   ```

❌ **cm_example_520**: `text` (Unexpected failure)
   Input: `\*\[foo\*\](/uri)
`
   Error: ` --> 1:1
  |
1 | *[foo*](/uri)
  | ^---
  |
  = expected text`

❌ **cm_example_521**: `text` (Unexpected failure)
   Input: `\[foo \*bar\](baz\*)
`
   Error: ` --> 1:1
  |
1 | [foo *bar](baz*)
  | ^---
  |
  = expected text`

❌ **cm_example_522**: `text` (Unexpected failure)
   Input: `\*foo \[bar\* baz\]
`
   Error: ` --> 1:1
  |
1 | *foo [bar* baz]
  | ^---
  |
  = expected text`

❌ **cm_example_523**: `text` (Unexpected failure)
   Input: `\[foo <bar attr=\"\](baz)\">
`
   Error: ` --> 1:1
  |
1 | [foo <bar attr=\"](baz)\">
  | ^---
  |
  = expected text`

❌ **cm_example_524**: `text` (Unexpected failure)
   Input: `\[foo\`\](/uri)\`
`
   Error: ` --> 1:1
  |
1 | [foo`](/uri)`
  | ^---
  |
  = expected text`

❌ **cm_example_525**: `text` (Unexpected failure)
   Input: `\[foo<http://example.com/?search=\](uri)>
`
   Error: ` --> 1:1
  |
1 | [foo<http://example.com/?search=](uri)>
  | ^---
  |
  = expected text`

❌ **cm_example_526**: `text` (Unexpected failure)
   Input: `\[foo\]\[bar\]

\[bar\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [foo][bar]
  | ^---
  |
  = expected text`

❌ **cm_example_527**: `text` (Unexpected failure)
   Input: `\[link \[foo \[bar\]\]\]\[ref\]

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | [link [foo [bar]]][ref]
  | ^---
  |
  = expected text`

❌ **cm_example_528**: `text` (Unexpected failure)
   Input: `\[link \\\[bar\]\[ref\]

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | [link \\[bar][ref]
  | ^---
  |
  = expected text`

❌ **cm_example_529**: `text` (Unexpected failure)
   Input: `\[link \*foo \*\*bar\*\* \`#\`\*\]\[ref\]

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | [link *foo **bar** `#`*][ref]
  | ^---
  |
  = expected text`

❌ **cm_example_530**: `text` (Unexpected failure)
   Input: `\[!\[moon\](moon.jpg)\]\[ref\]

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | [![moon](moon.jpg)][ref]
  | ^---
  |
  = expected text`

❌ **cm_example_531**: `text` (Unexpected failure)
   Input: `\[foo \[bar\](/uri)\]\[ref\]

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | [foo [bar](/uri)][ref]
  | ^---
  |
  = expected text`

❌ **cm_example_532**: `text` (Unexpected failure)
   Input: `\[foo \*bar \[baz\]\[ref\]\*\]\[ref\]

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | [foo *bar [baz][ref]*][ref]
  | ^---
  |
  = expected text`

❌ **cm_example_533**: `text` (Unexpected failure)
   Input: `\*\[foo\*\]\[ref\]

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | *[foo*][ref]
  | ^---
  |
  = expected text`

❌ **cm_example_534**: `text` (Unexpected failure)
   Input: `\[foo \*bar\]\[ref\]\*

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | [foo *bar][ref]*
  | ^---
  |
  = expected text`

❌ **cm_example_535**: `text` (Unexpected failure)
   Input: `\[foo <bar attr=\"\]\[ref\]\">

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | [foo <bar attr=\"][ref]\">
  | ^---
  |
  = expected text`

❌ **cm_example_536**: `text` (Unexpected failure)
   Input: `\[foo\`\]\[ref\]\`

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | [foo`][ref]`
  | ^---
  |
  = expected text`

❌ **cm_example_537**: `text` (Unexpected failure)
   Input: `\[foo<http://example.com/?search=\]\[ref\]>

\[ref\]: /uri
`
   Error: ` --> 1:1
  |
1 | [foo<http://example.com/?search=][ref]>
  | ^---
  |
  = expected text`

❌ **cm_example_538**: `text` (Unexpected failure)
   Input: `\[foo\]\[BaR\]

\[bar\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [foo][BaR]
  | ^---
  |
  = expected text`

❌ **cm_example_539**: `text` (Unexpected failure)
   Input: `\[ẞ\]

\[SS\]: /url
`
   Error: ` --> 1:1
  |
1 | [ẞ]
  | ^---
  |
  = expected text`

❌ **cm_example_540**: `text` (Unexpected failure)
   Input: `\[Foo
  bar\]: /url

\[Baz\]\[Foo bar\]
`
   Error: ` --> 1:1
  |
1 | [Foo
  | ^---
  |
  = expected text`

❌ **cm_example_541**: `text` (Unexpected failure)
   Input: `\[foo\] \[bar\]

\[bar\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [foo] [bar]
  | ^---
  |
  = expected text`

❌ **cm_example_542**: `text` (Unexpected failure)
   Input: `\[foo\]
\[bar\]

\[bar\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [foo]
  | ^---
  |
  = expected text`

❌ **cm_example_543**: `text` (Unexpected failure)
   Input: `\[foo\]: /url1

\[foo\]: /url2

\[bar\]\[foo\]
`
   Error: ` --> 1:1
  |
1 | [foo]: /url1
  | ^---
  |
  = expected text`

❌ **cm_example_544**: `text` (Unexpected failure)
   Input: `\[bar\]\[foo\\!\]

\[foo!\]: /url
`
   Error: ` --> 1:1
  |
1 | [bar][foo\\!]
  | ^---
  |
  = expected text`

❌ **cm_example_545**: `text` (Unexpected failure)
   Input: `\[foo\]\[ref\[\]

\[ref\[\]: /uri
`
   Error: ` --> 1:1
  |
1 | [foo][ref[]
  | ^---
  |
  = expected text`

❌ **cm_example_546**: `text` (Unexpected failure)
   Input: `\[foo\]\[ref\[bar\]\]

\[ref\[bar\]\]: /uri
`
   Error: ` --> 1:1
  |
1 | [foo][ref[bar]]
  | ^---
  |
  = expected text`

❌ **cm_example_547**: `text` (Unexpected failure)
   Input: `\[\[\[foo\]\]\]

\[\[\[foo\]\]\]: /url
`
   Error: ` --> 1:1
  |
1 | [[[foo]]]
  | ^---
  |
  = expected text`

❌ **cm_example_548**: `text` (Unexpected failure)
   Input: `\[foo\]\[ref\\\[\]

\[ref\\\[\]: /uri
`
   Error: ` --> 1:1
  |
1 | [foo][ref\\[]
  | ^---
  |
  = expected text`

❌ **cm_example_549**: `text` (Unexpected failure)
   Input: `\[bar\\\\\]: /uri

\[bar\\\\\]
`
   Error: ` --> 1:1
  |
1 | [bar\\\\]: /uri
  | ^---
  |
  = expected text`

❌ **cm_example_550**: `text` (Unexpected failure)
   Input: `\[\]

\[\]: /uri
`
   Error: ` --> 1:1
  |
1 | []
  | ^---
  |
  = expected text`

❌ **cm_example_551**: `text` (Unexpected failure)
   Input: `\[
 \]

\[
 \]: /uri
`
   Error: ` --> 1:1
  |
1 | [
  | ^---
  |
  = expected text`

❌ **cm_example_552**: `text` (Unexpected failure)
   Input: `\[foo\]\[\]

\[foo\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [foo][]
  | ^---
  |
  = expected text`

❌ **cm_example_553**: `text` (Unexpected failure)
   Input: `\[\*foo\* bar\]\[\]

\[\*foo\* bar\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [*foo* bar][]
  | ^---
  |
  = expected text`

❌ **cm_example_554**: `text` (Unexpected failure)
   Input: `\[Foo\]\[\]

\[foo\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [Foo][]
  | ^---
  |
  = expected text`

❌ **cm_example_555**: `text` (Unexpected failure)
   Input: `\[foo\] 
\[\]

\[foo\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [foo] 
  | ^---
  |
  = expected text`

❌ **cm_example_556**: `text` (Unexpected failure)
   Input: `\[foo\]

\[foo\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [foo]
  | ^---
  |
  = expected text`

❌ **cm_example_557**: `text` (Unexpected failure)
   Input: `\[\*foo\* bar\]

\[\*foo\* bar\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [*foo* bar]
  | ^---
  |
  = expected text`

❌ **cm_example_558**: `text` (Unexpected failure)
   Input: `\[\[\*foo\* bar\]\]

\[\*foo\* bar\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [[*foo* bar]]
  | ^---
  |
  = expected text`

❌ **cm_example_559**: `text` (Unexpected failure)
   Input: `\[\[bar \[foo\]

\[foo\]: /url
`
   Error: ` --> 1:1
  |
1 | [[bar [foo]
  | ^---
  |
  = expected text`

❌ **cm_example_560**: `text` (Unexpected failure)
   Input: `\[Foo\]

\[foo\]: /url \"title\"
`
   Error: ` --> 1:1
  |
1 | [Foo]
  | ^---
  |
  = expected text`

❌ **cm_example_561**: `text` (Unexpected failure)
   Input: `\[foo\] bar

\[foo\]: /url
`
   Error: ` --> 1:1
  |
1 | [foo] bar
  | ^---
  |
  = expected text`

✅ **cm_example_562**: `text`
   Input: `\\\[foo\]

\[foo\]: /url \"title\"
`
   Parse Tree:
   ```
  └── text: "\\"
   ```

❌ **cm_example_563**: `text` (Unexpected failure)
   Input: `\[foo\*\]: /url

\*\[foo\*\]
`
   Error: ` --> 1:1
  |
1 | [foo*]: /url
  | ^---
  |
  = expected text`

❌ **cm_example_564**: `text` (Unexpected failure)
   Input: `\[foo\]\[bar\]

\[foo\]: /url1
\[bar\]: /url2
`
   Error: ` --> 1:1
  |
1 | [foo][bar]
  | ^---
  |
  = expected text`

❌ **cm_example_565**: `text` (Unexpected failure)
   Input: `\[foo\]\[\]

\[foo\]: /url1
`
   Error: ` --> 1:1
  |
1 | [foo][]
  | ^---
  |
  = expected text`

❌ **cm_example_566**: `text` (Unexpected failure)
   Input: `\[foo\]()

\[foo\]: /url1
`
   Error: ` --> 1:1
  |
1 | [foo]()
  | ^---
  |
  = expected text`

❌ **cm_example_567**: `text` (Unexpected failure)
   Input: `\[foo\](not a link)

\[foo\]: /url1
`
   Error: ` --> 1:1
  |
1 | [foo](not a link)
  | ^---
  |
  = expected text`

❌ **cm_example_568**: `text` (Unexpected failure)
   Input: `\[foo\]\[bar\]\[baz\]

\[baz\]: /url
`
   Error: ` --> 1:1
  |
1 | [foo][bar][baz]
  | ^---
  |
  = expected text`

❌ **cm_example_569**: `text` (Unexpected failure)
   Input: `\[foo\]\[bar\]\[baz\]

\[baz\]: /url1
\[bar\]: /url2
`
   Error: ` --> 1:1
  |
1 | [foo][bar][baz]
  | ^---
  |
  = expected text`

❌ **cm_example_570**: `text` (Unexpected failure)
   Input: `\[foo\]\[bar\]\[baz\]

\[baz\]: /url1
\[foo\]: /url2
`
   Error: ` --> 1:1
  |
1 | [foo][bar][baz]
  | ^---
  |
  = expected text`

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

## commonmark_inlines

❌ **cm_example_327**: `text` (Unexpected failure)
   Input: `\`hi\`lo\`
`
   Error: ` --> 1:1
  |
1 | `hi`lo`
  | ^---
  |
  = expected text`

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

✅ **malformed_bold**: `bold` (Expected failure)
   Input: `\*\*missing closing`
   Error: ` --> 1:1
  |
1 | **missing closing
  | ^---
  |
  = expected bold`

✅ **malformed_italic**: `italic` (Expected failure)
   Input: `\*missing closing`
   Error: ` --> 1:1
  |
1 | *missing closing
  | ^---
  |
  = expected italic`

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
  = expected safe_inline`

✅ **invalid_list_marker**: `text`
   Input: `? Not a list`
   Parse Tree:
   ```
  └── text: "? Not a list"
   ```

✅ **invalid_table**: `table` (Expected failure)
   Input: `"| A | B |
| 1 | 2 | 3 |"        # mismatched columns`
   Error: ` --> 1:1
  |
1 | "| A | B |
  | ^---
  |
  = expected table_header`

✅ **invalid_footnote**: `footnote_ref` (Expected failure)
   Input: `\[^invalid label with spaces\]`
   Error: ` --> 1:1
  |
1 | [^invalid label with spaces]
  | ^---
  |
  = expected footnote_ref`

✅ **invalid_reference**: `text` (Expected failure)
   Input: `\[ref with spaces\]: url`
   Error: ` --> 1:1
  |
1 | [ref with spaces]: url
  | ^---
  |
  = expected text`

❌ **nested_conflict_1**: `text` (Unexpected failure)
   Input: `\*\*bold with \`code\*\* inside\``
   Error: ` --> 1:1
  |
1 | **bold with `code** inside`
  | ^---
  |
  = expected text`

❌ **nested_conflict_2**: `text` (Unexpected failure)
   Input: `\*italic with \*\*bold\* text\*\*`
   Error: ` --> 1:1
  |
1 | *italic with **bold* text**
  | ^---
  |
  = expected text`

✅ **nested_conflict_3**: `text`
   Input: `~~strike with \*\*bold~~ text\*\*`
   Parse Tree:
   ```
  └── text: "~~strike with "
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
   Error: ` --> 2:1
  |
2 | custom_type
  | ^---
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

❌ **many_emphasis_markers**: `text` (Unexpected failure)
   Input: `\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*`
   Error: ` --> 1:1
  |
1 | *****************************************************************************
  | ^---
  |
  = expected text`

❌ **alternating_chars**: `text` (Unexpected failure)
   Input: `\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*\_\*`
   Error: ` --> 1:1
  |
1 | *_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*_*
  | ^---
  |
  = expected text`

❌ **quadratic_blowup**: `text` (Unexpected failure)
   Input: `\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[`
   Error: ` --> 1:1
  |
1 | [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
  | ^---
  |
  = expected text`

✅ **mixed_line_endings_complex**: `text`
   Input: `Line 1\r
Line 2
Line 3\r
Line 4
`
   Parse Tree:
   ```
  └── text: "Line 1\r
Line 2
Line 3\r
Line 4
"
   ```

✅ **binary_like_data**: `text`
   Input: `\\u0000\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\u0008\\u0009\
\\u000B\\u000C\\r\\u000E\\u000F`
   Parse Tree:
   ```
  └── text: "\\u0000\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\u0008\\u0009\
\\u000B\\u000C\\r\\u000E\\u000F"
   ```

❌ **massive_nested_brackets**: `text` (Unexpected failure)
   Input: `\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]`
   Error: ` --> 1:1
  |
1 | [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
  | ^---
  |
  = expected text`

## tables

❌ **table_simple**: `table` (Unexpected failure)
   Input: `| Col1 | Col2 |
|------|------|
| A    | B    |`
   Error: ` --> 3:16
  |
3 | | A    | B    |
  |                ^---
  |
  = expected inline_core`

❌ **table_with_alignment**: `table` (Unexpected failure)
   Input: `| Left | Center | Right |
|:-----|:------:|------:|
| A    | B      | C     |`
   Error: ` --> 3:26
  |
3 | | A    | B      | C     |
  |                          ^---
  |
  = expected inline_core`

❌ **table_minimal**: `table` (Unexpected failure)
   Input: `|A|B|
|-|-|
|1|2|`
   Error: ` --> 3:6
  |
3 | |1|2|
  |      ^---
  |
  = expected inline_core`

❌ **table_with_formatting**: `table` (Unexpected failure)
   Input: `| \*\*Bold\*\* | \*Italic\* |
|----------|----------|
| \`code\`   | \[link\](url) |`
   Error: ` --> 3:21
  |
3 | | `code`   | [link](url) |
  |                     ^---
  |
  = expected inline_url`

❌ **table_with_pipes**: `table` (Unexpected failure)
   Input: `| Text | With \\| Pipe |
|------|------------|
| A    | B          |`
   Error: ` --> 3:22
  |
3 | | A    | B          |
  |                      ^---
  |
  = expected inline_core`

✅ **table_empty_cells**: `table` (Expected failure)
   Input: `| | |
|-|-|
| | |`
   Error: ` --> 3:6
  |
3 | | | |
  |      ^---
  |
  = expected inline_core`

❌ **table_uneven_columns**: `table` (Unexpected failure)
   Input: `| A | B | C |
|---|---|
| 1 | 2 |`
   Error: ` --> 3:10
  |
3 | | 1 | 2 |
  |          ^---
  |
  = expected inline_core`

✅ **table_no_separator**: `table` (Expected failure)
   Input: `| A | B |
| 1 | 2 |`
   Error: ` --> 2:10
  |
2 | | 1 | 2 |
  |          ^---
  |
  = expected inline_core`

✅ **table_malformed**: `table` (Expected failure)
   Input: `| A | B
|---|
| 1 | 2 |`
   Error: ` --> 3:10
  |
3 | | 1 | 2 |
  |          ^---
  |
  = expected inline_core`

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

❌ **bug_code_in_link**: `inline_link` (Unexpected failure)
   Input: `\[\`code\` in link\](url)`
   Error: ` --> 1:18
  |
1 | [`code` in link](url)
  |                  ^---
  |
  = expected inline_url`

✅ **bug_nested_quotes**: `blockquote`
   Input: `> > > Quote with \`code\` and \*emphasis\*`
   Parse Tree:
   ```
  ├── blockquote > "> > > Quote with `code` and *emphasis*"
    ├── blockquote_line > "> > > Quote with `code` and *emphasis*"
      ├── inline > "> > Quote with "
        ├── inline_core > "> > Quote with "
          └── text: "> > Quote with "
      ├── inline > "`code`"
        ├── inline_core > "`code`"
          └── code_inline: "`code`"
      ├── inline > "and "
        ├── inline_core > "and "
          └── text: "and "
      ├── inline > "*emphasis*"
        ├── inline_core > "*emphasis*"
          ├── emphasis > "*emphasis*"
            ├── italic > "*emphasis*"
              └── italic_asterisk: "*emphasis*"
   ```

❌ **bug_table_alignment**: `table` (Unexpected failure)
   Input: `| Left | Center | Right |
|:-----|:------:|------:|
| A | B | C |`
   Error: ` --> 3:14
  |
3 | | A | B | C |
  |              ^---
  |
  = expected inline_core`

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
   Error: ` --> 1:1
  |
1 | *Emphasized* heading
  | ^---
  |
  = expected safe_inline`

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

❌ **cm_link_title_quotes**: `inline_link` (Unexpected failure)
   Input: `\[link\](url \"title with 'quotes'\")`
   Error: ` --> 1:8
  |
1 | [link](url \"title with 'quotes'\")
  |        ^---
  |
  = expected inline_url`

❌ **cm_reference_case_insensitive**: `text` (Unexpected failure)
   Input: `\[FOO\]\[bar\]
\[bar\]: /url`
   Error: ` --> 1:1
  |
1 | [FOO][bar]
  | ^---
  |
  = expected text`

❌ **cm_autolink_scheme_case**: `inline_link` (Unexpected failure)
   Input: `<HTTP://EXAMPLE.COM>`
   Error: ` --> 1:1
  |
1 | <HTTP://EXAMPLE.COM>
  | ^---
  |
  = expected inline_link`

❌ **cm_entity_in_link**: `inline_link` (Unexpected failure)
   Input: `\[link\](url?param=value&amp;other=2)`
   Error: ` --> 1:8
  |
1 | [link](url?param=value&amp;other=2)
  |        ^---
  |
  = expected inline_url`

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

## commonmark_html_blocks

❌ **cm_example_148**: `text` (Unexpected failure)
   Input: `<table><tr><td>
<pre>
\*\*Hello\*\*,

\_world\_.
</pre>
</td></tr></table>
`
   Error: ` --> 1:1
  |
1 | <table><tr><td>
  | ^---
  |
  = expected text`

❌ **cm_example_149**: `text` (Unexpected failure)
   Input: `<table>
  <tr>
    <td>
           hi
    </td>
  </tr>
</table>

okay.
`
   Error: ` --> 1:1
  |
1 | <table>
  | ^---
  |
  = expected text`

❌ **cm_example_150**: `text` (Unexpected failure)
   Input: ` <div>
  \*hello\*
         <foo><a>
`
   Error: ` --> 1:1
  |
1 |  <div>
  | ^---
  |
  = expected text`

✅ **cm_example_151**: `text`
   Input: `</div>
\*foo\*
`
   Parse Tree:
   ```
  └── text: "</div>
"
   ```

❌ **cm_example_152**: `text` (Unexpected failure)
   Input: `<DIV CLASS=\"foo\">

\*Markdown\*

</DIV>
`
   Error: ` --> 1:1
  |
1 | <DIV CLASS=\"foo\">
  | ^---
  |
  = expected text`

❌ **cm_example_153**: `text` (Unexpected failure)
   Input: `<div id=\"foo\"
  class=\"bar\">
</div>
`
   Error: ` --> 1:1
  |
1 | <div id=\"foo\"
  | ^---
  |
  = expected text`

❌ **cm_example_154**: `text` (Unexpected failure)
   Input: `<div id=\"foo\" class=\"bar
  baz\">
</div>
`
   Error: ` --> 1:1
  |
1 | <div id=\"foo\" class=\"bar
  | ^---
  |
  = expected text`

❌ **cm_example_155**: `text` (Unexpected failure)
   Input: `<div>
\*foo\*

\*bar\*
`
   Error: ` --> 1:1
  |
1 | <div>
  | ^---
  |
  = expected text`

❌ **cm_example_156**: `text` (Unexpected failure)
   Input: `<div id=\"foo\"
\*hi\*
`
   Error: ` --> 1:1
  |
1 | <div id=\"foo\"
  | ^---
  |
  = expected text`

❌ **cm_example_157**: `text` (Unexpected failure)
   Input: `<div class
foo
`
   Error: ` --> 1:1
  |
1 | <div class
  | ^---
  |
  = expected text`

❌ **cm_example_158**: `text` (Unexpected failure)
   Input: `<div \*???-&&&-<---
\*foo\*
`
   Error: ` --> 1:1
  |
1 | <div *???-&&&-<---
  | ^---
  |
  = expected text`

❌ **cm_example_159**: `text` (Unexpected failure)
   Input: `<div><a href=\"bar\">\*foo\*</a></div>
`
   Error: ` --> 1:1
  |
1 | <div><a href=\"bar\">*foo*</a></div>
  | ^---
  |
  = expected text`

❌ **cm_example_160**: `text` (Unexpected failure)
   Input: `<table><tr><td>
foo
</td></tr></table>
`
   Error: ` --> 1:1
  |
1 | <table><tr><td>
  | ^---
  |
  = expected text`

❌ **cm_example_161**: `text` (Unexpected failure)
   Input: `<div></div>
\`\`\` c
int x = 33;
\`\`\`
`
   Error: ` --> 1:1
  |
1 | <div></div>
  | ^---
  |
  = expected text`

❌ **cm_example_162**: `text` (Unexpected failure)
   Input: `<a href=\"foo\">
\*bar\*
</a>
`
   Error: ` --> 1:1
  |
1 | <a href=\"foo\">
  | ^---
  |
  = expected text`

❌ **cm_example_163**: `text` (Unexpected failure)
   Input: `<Warning>
\*bar\*
</Warning>
`
   Error: ` --> 1:1
  |
1 | <Warning>
  | ^---
  |
  = expected text`

❌ **cm_example_164**: `text` (Unexpected failure)
   Input: `<i class=\"foo\">
\*bar\*
</i>
`
   Error: ` --> 1:1
  |
1 | <i class=\"foo\">
  | ^---
  |
  = expected text`

✅ **cm_example_165**: `text`
   Input: `</ins>
\*bar\*
`
   Parse Tree:
   ```
  └── text: "</ins>
"
   ```

❌ **cm_example_166**: `text` (Unexpected failure)
   Input: `<del>
\*foo\*
</del>
`
   Error: ` --> 1:1
  |
1 | <del>
  | ^---
  |
  = expected text`

❌ **cm_example_167**: `text` (Unexpected failure)
   Input: `<del>

\*foo\*

</del>
`
   Error: ` --> 1:1
  |
1 | <del>
  | ^---
  |
  = expected text`

❌ **cm_example_168**: `text` (Unexpected failure)
   Input: `<del>\*foo\*</del>
`
   Error: ` --> 1:1
  |
1 | <del>*foo*</del>
  | ^---
  |
  = expected text`

❌ **cm_example_169**: `text` (Unexpected failure)
   Input: `<pre language=\"haskell\"><code>
import Text.HTML.TagSoup

main :: IO ()
main = print $ parseTags tags
</code></pre>
okay
`
   Error: ` --> 1:1
  |
1 | <pre language=\"haskell\"><code>
  | ^---
  |
  = expected text`

❌ **cm_example_170**: `text` (Unexpected failure)
   Input: `<script type=\"text/javascript\">
// JavaScript example

document.getElementById(\"demo\").innerHTML = \"Hello JavaScript!\";
</script>
okay
`
   Error: ` --> 1:1
  |
1 | <script type=\"text/javascript\">
  | ^---
  |
  = expected text`

❌ **cm_example_171**: `text` (Unexpected failure)
   Input: `<textarea>

\*foo\*

\_bar\_

</textarea>
`
   Error: ` --> 1:1
  |
1 | <textarea>
  | ^---
  |
  = expected text`

❌ **cm_example_172**: `text` (Unexpected failure)
   Input: `<style
  type=\"text/css\">
h1 {color:red;}

p {color:blue;}
</style>
okay
`
   Error: ` --> 1:1
  |
1 | <style
  | ^---
  |
  = expected text`

❌ **cm_example_173**: `text` (Unexpected failure)
   Input: `<style
  type=\"text/css\">

foo
`
   Error: ` --> 1:1
  |
1 | <style
  | ^---
  |
  = expected text`

❌ **cm_example_174**: `text` (Unexpected failure)
   Input: `> <div>
> foo

bar
`
   Error: ` --> 1:1
  |
1 | > <div>
  | ^---
  |
  = expected text`

❌ **cm_example_175**: `text` (Unexpected failure)
   Input: `- <div>
- foo
`
   Error: ` --> 1:1
  |
1 | - <div>
  | ^---
  |
  = expected text`

❌ **cm_example_176**: `text` (Unexpected failure)
   Input: `<style>p{color:red;}</style>
\*foo\*
`
   Error: ` --> 1:1
  |
1 | <style>p{color:red;}</style>
  | ^---
  |
  = expected text`

✅ **cm_example_177**: `text`
   Input: `<!-- foo -->\*bar\*
\*baz\*
`
   Parse Tree:
   ```
  └── text: "<!-- foo -->"
   ```

❌ **cm_example_178**: `text` (Unexpected failure)
   Input: `<script>
foo
</script>1. \*bar\*
`
   Error: ` --> 1:1
  |
1 | <script>
  | ^---
  |
  = expected text`

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
  └── text: "<!"
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

❌ **cm_example_184**: `text` (Unexpected failure)
   Input: `  <div>

    <div>
`
   Error: ` --> 1:1
  |
1 |   <div>
  | ^---
  |
  = expected text`

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

❌ **cm_example_186**: `text` (Unexpected failure)
   Input: `<div>
bar
</div>
\*foo\*
`
   Error: ` --> 1:1
  |
1 | <div>
  | ^---
  |
  = expected text`

✅ **cm_example_187**: `text`
   Input: `Foo
<a href=\"bar\">
baz
`
   Parse Tree:
   ```
  └── text: "Foo
<a href=\"bar\">
baz
"
   ```

❌ **cm_example_188**: `text` (Unexpected failure)
   Input: `<div>

\*Emphasized\* text.

</div>
`
   Error: ` --> 1:1
  |
1 | <div>
  | ^---
  |
  = expected text`

❌ **cm_example_189**: `text` (Unexpected failure)
   Input: `<div>
\*Emphasized\* text.
</div>
`
   Error: ` --> 1:1
  |
1 | <div>
  | ^---
  |
  = expected text`

❌ **cm_example_190**: `text` (Unexpected failure)
   Input: `<table>

<tr>

<td>
Hi
</td>

</tr>

</table>
`
   Error: ` --> 1:1
  |
1 | <table>
  | ^---
  |
  = expected text`

❌ **cm_example_191**: `text` (Unexpected failure)
   Input: `<table>

  <tr>

    <td>
      Hi
    </td>

  </tr>

</table>
`
   Error: ` --> 1:1
  |
1 | <table>
  | ^---
  |
  = expected text`

## commonmark_backslash_escapes

✅ **cm_example_12**: `text`
   Input: `\\!\\\"\\#\\$\\%\\&\\'\\(\\)\\\*\\+\\,\\-\\.\\/\\:\\;\\<\\=\\>\\?\\@\\\[\\\\\\\]\\^\\\_\\\`\\{\\|\\}\\~
`
   Parse Tree:
   ```
  └── text: "\\!\\\"\\#\\"
   ```

✅ **cm_example_13**: `text`
   Input: `\\	\\A\\a\\ \\3\\φ\\«
`
   Parse Tree:
   ```
  └── text: "\\	\\A\\a\\ \\3\\φ\\«
"
   ```

✅ **cm_example_14**: `text`
   Input: `\\\*not emphasized\*
\\<br/> not a tag
\\\[not a link\](/foo)
\\\`not code\`
1\\. not a list
\\\* not a list
\\# not a heading
\\\[foo\]: /url \"not a reference\"
\\&ouml; not a character entity
`
   Parse Tree:
   ```
  └── text: "\\"
   ```

✅ **cm_example_15**: `text`
   Input: `\\\\\*emphasis\*
`
   Parse Tree:
   ```
  └── text: "\\\\"
   ```

✅ **cm_example_16**: `text`
   Input: `foo\\
bar
`
   Parse Tree:
   ```
  └── text: "foo\\
bar
"
   ```

❌ **cm_example_17**: `text` (Unexpected failure)
   Input: `\`\` \\\[\\\` \`\`
`
   Error: ` --> 1:1
  |
1 | `` \\[\\` ``
  | ^---
  |
  = expected text`

✅ **cm_example_18**: `text`
   Input: `    \\\[\\\]
`
   Parse Tree:
   ```
  └── text: "    \\"
   ```

✅ **cm_example_19**: `text`
   Input: `~~~
\\\[\\\]
~~~
`
   Parse Tree:
   ```
  └── text: "~~~
\\"
   ```

❌ **cm_example_20**: `text` (Unexpected failure)
   Input: `<http://example.com?find=\\\*>
`
   Error: ` --> 1:1
  |
1 | <http://example.com?find=\\*>
  | ^---
  |
  = expected text`

❌ **cm_example_21**: `text` (Unexpected failure)
   Input: `<a href=\"/bar\\/)\">
`
   Error: ` --> 1:1
  |
1 | <a href=\"/bar\\/)\">
  | ^---
  |
  = expected text`

❌ **cm_example_22**: `text` (Unexpected failure)
   Input: `\[foo\](/bar\\\* \"ti\\\*tle\")
`
   Error: ` --> 1:1
  |
1 | [foo](/bar\\* \"ti\\*tle\")
  | ^---
  |
  = expected text`

❌ **cm_example_23**: `text` (Unexpected failure)
   Input: `\[foo\]

\[foo\]: /bar\\\* \"ti\\\*tle\"
`
   Error: ` --> 1:1
  |
1 | [foo]
  | ^---
  |
  = expected text`

❌ **cm_example_24**: `text` (Unexpected failure)
   Input: `\`\`\` foo\\+bar
foo
\`\`\`
`
   Error: ` --> 1:1
  |
1 | ``` foo\\+bar
  | ^---
  |
  = expected text`

## headings_setext

✅ **setext_h1_simple**: `setext_h1`
   Input: `Heading
=======`
   Parse Tree:
   ```
  ├── setext_h1 > "Heading
======="
    ├── heading_content > "Heading"
      ├── safe_inline > "Heading"
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
      ├── safe_inline > "Heading"
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
      ├── safe_inline > "Long"
        └── word: "Long"
      ├── safe_inline > "Heading"
        └── word: "Heading"
      ├── safe_inline > "Text"
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
      ├── safe_inline > "Subheading"
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
      ├── safe_inline > "Subheading"
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
  = expected safe_inline`

✅ **setext_no_text**: `heading` (Expected failure)
   Input: `
======`
   Error: ` --> 1:1
  |
1 | ␊
  | ^---
  |
  = expected heading`

## marco_stress_tests

✅ **nested_admonitions**: `admonition_block`
   Input: `:::
note
Outer note
:::
warning
Inner warning
:::
:::`
   Parse Tree:
   ```
  ├── admonition_block > ":::
note
Outer note
:::
warning
Inner warning
:::
:::"
    ├── admonition_open > ":::
note"
      ├── admonition_type > "note"
        └── KW_NOTE: "note"
    ├── admonition_block > ":::
warning
Inner warning
:::"
      ├── admonition_open > ":::
warning"
        ├── admonition_type > "warning"
          └── KW_WARNING: "warning"
      └── admonition_close: ":::"
    └── admonition_close: ":::"
   ```

✅ **run_multiline_complex**: `run_block_fenced`
   Input: `\`\`\`run@bash
for i in {1..10}; do
  echo \"Line $i\"
  if \[ $i -eq 5 \]; then
    break
  fi
done
\`\`\``
   Parse Tree:
   ```
  ├── run_block_fenced > "```run@bash
for i in {1..10}; do
  echo \"Line $i\"
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

❌ **tabs_with_code**: `tabs_block` (Unexpected failure)
   Input: `:::
tabs Code Examples
@tab Python
\`\`\`python
print('hello')
\`\`\`
@tab Rust
\`\`\`rust
fn main() {}
\`\`\`
:::`
   Error: ` --> 1:1
  |
1 | :::
  | ^---
  |
  = expected tabs_block`

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

## unicode_advanced

✅ **rtl_arabic**: `text`
   Input: `مرحبا بالعالم \*\*نص عريض\*\* \*نص مائل\*`
   Parse Tree:
   ```
  └── text: "مرحبا بالعالم "
   ```

✅ **rtl_hebrew**: `text`
   Input: `שלום עולם \*\*טקסט מודגש\*\* \*טקסט נטוי\*`
   Parse Tree:
   ```
  └── text: "שלום עולם "
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

❌ **emoji_in_formatting**: `text` (Unexpected failure)
   Input: `\*\*👍 bold emoji\*\* \*🎉 italic emoji\*`
   Error: ` --> 1:1
  |
1 | **👍 bold emoji** *🎉 italic emoji*
  | ^---
  |
  = expected text`

❌ **emoji_in_links**: `text` (Unexpected failure)
   Input: `\[🔗 emoji link\](https://example.com)`
   Error: ` --> 1:1
  |
1 | [🔗 emoji link](https://example.com)
  | ^---
  |
  = expected text`

✅ **zero_width_joiner**: `text`
   Input: `text\u200Dwith\u200Dzwj`
   Parse Tree:
   ```
  └── text: "text\u200Dwith\u200Dzwj"
   ```

✅ **zero_width_non_joiner**: `text`
   Input: `text\u200Cwith\u200Cznj`
   Parse Tree:
   ```
  └── text: "text\u200Cwith\u200Cznj"
   ```

✅ **zero_width_space**: `text`
   Input: `text\u200Bwith\u200Bzws`
   Parse Tree:
   ```
  └── text: "text\u200Bwith\u200Bzws"
   ```

✅ **combining_diacritics**: `text`
   Input: `"e\u0301\u0302\u0303\u0304"  # e with multiple combining marks`
   Parse Tree:
   ```
  └── text: ""e\u0301\u0302\u0303\u0304"  # e with multiple combining marks"
   ```

✅ **normalization_test**: `text`
   Input: `"café vs cafe\u0301"  # NFC vs NFD`
   Parse Tree:
   ```
  └── text: ""café vs cafe\u0301"  # NFC vs NFD"
   ```

✅ **astral_symbols**: `text`
   Input: `"𝕳𝖊𝖑𝖑𝖔 𝖜𝖔𝖗𝖑𝖉"  # Mathematical bold fraktur`
   Parse Tree:
   ```
  └── text: ""𝕳𝖊𝖑𝖑𝖔 𝖜𝖔𝖗𝖑𝖉"  # Mathematical bold fraktur"
   ```

✅ **musical_symbols**: `text`
   Input: `𝄞 𝄢 𝅘𝅥 𝅘𝅥𝅮`
   Parse Tree:
   ```
  └── text: "𝄞 𝄢 𝅘𝅥 𝅘𝅥𝅮"
   ```

## commonmark_setext_headings

✅ **cm_example_80**: `text`
   Input: `Foo \*bar\*
=========

Foo \*bar\*
---------
`
   Parse Tree:
   ```
  └── text: "Foo "
   ```

✅ **cm_example_81**: `text`
   Input: `Foo \*bar
baz\*
====
`
   Parse Tree:
   ```
  └── text: "Foo "
   ```

✅ **cm_example_82**: `text`
   Input: `  Foo \*bar
baz\*	
====
`
   Parse Tree:
   ```
  └── text: "  Foo "
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
  └── text: "Foo\\
----
"
   ```

❌ **cm_example_91**: `text` (Unexpected failure)
   Input: `\`Foo
----
\`

<a title=\"a lot
---
of dashes\"/>
`
   Error: ` --> 1:1
  |
1 | `Foo
  | ^---
  |
  = expected text`

❌ **cm_example_92**: `text` (Unexpected failure)
   Input: `> Foo
---
`
   Error: ` --> 1:1
  |
1 | > Foo
  | ^---
  |
  = expected text`

❌ **cm_example_93**: `text` (Unexpected failure)
   Input: `> foo
bar
===
`
   Error: ` --> 1:1
  |
1 | > foo
  | ^---
  |
  = expected text`

❌ **cm_example_94**: `text` (Unexpected failure)
   Input: `- Foo
---
`
   Error: ` --> 1:1
  |
1 | - Foo
  | ^---
  |
  = expected text`

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

❌ **cm_example_96**: `text` (Unexpected failure)
   Input: `---
Foo
---
Bar
---
Baz
`
   Error: ` --> 1:1
  |
1 | ---
  | ^---
  |
  = expected text`

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

❌ **cm_example_98**: `text` (Unexpected failure)
   Input: `---
---
`
   Error: ` --> 1:1
  |
1 | ---
  | ^---
  |
  = expected text`

❌ **cm_example_99**: `text` (Unexpected failure)
   Input: `- foo
-----
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

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

❌ **cm_example_101**: `text` (Unexpected failure)
   Input: `> foo
-----
`
   Error: ` --> 1:1
  |
1 | > foo
  | ^---
  |
  = expected text`

✅ **cm_example_102**: `text`
   Input: `\\> foo
------
`
   Parse Tree:
   ```
  └── text: "\\> foo
------
"
   ```

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
\\---
baz
"
   ```

## commonmark_thematic_breaks

❌ **cm_example_43**: `text` (Unexpected failure)
   Input: `\*\*\*
---
\_\_\_
`
   Error: ` --> 1:1
  |
1 | ***
  | ^---
  |
  = expected text`

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
"
   ```

❌ **cm_example_47**: `text` (Unexpected failure)
   Input: ` \*\*\*
  \*\*\*
   \*\*\*
`
   Error: ` --> 1:1
  |
1 |  ***
  | ^---
  |
  = expected text`

❌ **cm_example_48**: `text` (Unexpected failure)
   Input: `    \*\*\*
`
   Error: ` --> 1:1
  |
1 |     ***
  | ^---
  |
  = expected text`

✅ **cm_example_49**: `text`
   Input: `Foo
    \*\*\*
`
   Parse Tree:
   ```
  └── text: "Foo
    "
   ```

❌ **cm_example_50**: `text` (Unexpected failure)
   Input: `\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_\_
`
   Error: ` --> 1:1
  |
1 | _____________________________________
  | ^---
  |
  = expected text`

❌ **cm_example_51**: `text` (Unexpected failure)
   Input: ` - - -
`
   Error: ` --> 1:1
  |
1 |  - - -
  | ^---
  |
  = expected text`

❌ **cm_example_52**: `text` (Unexpected failure)
   Input: ` \*\*  \* \*\* \* \*\* \* \*\*
`
   Error: ` --> 1:1
  |
1 |  **  * ** * ** * **
  | ^---
  |
  = expected text`

❌ **cm_example_53**: `text` (Unexpected failure)
   Input: `-     -      -      -
`
   Error: ` --> 1:1
  |
1 | -     -      -      -
  | ^---
  |
  = expected text`

❌ **cm_example_54**: `text` (Unexpected failure)
   Input: `- - - -    
`
   Error: ` --> 1:1
  |
1 | - - - -    
  | ^---
  |
  = expected text`

❌ **cm_example_55**: `text` (Unexpected failure)
   Input: `\_ \_ \_ \_ a

a------

---a---
`
   Error: ` --> 1:1
  |
1 | _ _ _ _ a
  | ^---
  |
  = expected text`

❌ **cm_example_56**: `text` (Unexpected failure)
   Input: ` \*-\*
`
   Error: ` --> 1:1
  |
1 |  *-*
  | ^---
  |
  = expected text`

❌ **cm_example_57**: `text` (Unexpected failure)
   Input: `- foo
\*\*\*
- bar
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

✅ **cm_example_58**: `text`
   Input: `Foo
\*\*\*
bar
`
   Parse Tree:
   ```
  └── text: "Foo
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

❌ **cm_example_60**: `text` (Unexpected failure)
   Input: `\* Foo
\* \* \*
\* Bar
`
   Error: ` --> 1:1
  |
1 | * Foo
  | ^---
  |
  = expected text`

❌ **cm_example_61**: `text` (Unexpected failure)
   Input: `- Foo
- \* \* \*
`
   Error: ` --> 1:1
  |
1 | - Foo
  | ^---
  |
  = expected text`

## commonmark_edge_cases

❌ **link_vs_emphasis**: `inline_link` (Unexpected failure)
   Input: `\[\*foo\*\](bar)`
   Error: ` --> 1:9
  |
1 | [*foo*](bar)
  |         ^---
  |
  = expected inline_url`

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

❌ **autolink_email**: `inline_link` (Unexpected failure)
   Input: `<user@example.com>`
   Error: ` --> 1:1
  |
1 | <user@example.com>
  | ^---
  |
  = expected inline_link`

❌ **autolink_url**: `inline_link` (Unexpected failure)
   Input: `<http://example.com>`
   Error: ` --> 1:1
  |
1 | <http://example.com>
  | ^---
  |
  = expected inline_link`

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
    └── inline_link_text: "<http://example.com>"
    └── inline_url: "http://other.com"
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
        ├── safe_inline > "foo"
          └── word: "foo"
   ```

❌ **heading_space_before**: `heading` (Unexpected failure)
   Input: ` # foo`
   Error: ` --> 1:2
  |
1 |  # foo
  |  ^---
  |
  = expected safe_inline`

✅ **heading_trailing_hashes**: `heading`
   Input: `# foo #`
   Parse Tree:
   ```
  ├── heading > "# foo "
    ├── H1 > "# foo "
      ├── heading_content > "foo "
        ├── safe_inline > "foo"
          └── word: "foo"
   ```

✅ **heading_trailing_hashes_mismatch**: `heading`
   Input: `# foo ###`
   Parse Tree:
   ```
  ├── heading > "# foo "
    ├── H1 > "# foo "
      ├── heading_content > "foo "
        ├── safe_inline > "foo"
          └── word: "foo"
   ```

✅ **heading_empty**: `heading` (Expected failure)
   Input: `#`
   Error: ` --> 1:2
  |
1 | #
  |  ^---
  |
  = expected safe_inline`

❌ **heading_only_hashes**: `heading` (Unexpected failure)
   Input: `######`
   Error: ` --> 1:7
  |
1 | ######
  |       ^---
  |
  = expected safe_inline`

✅ **setext_no_content**: `setext_h2` (Expected failure)
   Input: `
====`
   Error: ` --> 1:1
  |
1 | ␊
  | ^---
  |
  = expected safe_inline`

❌ **setext_spaces_before**: `setext_h2` (Unexpected failure)
   Input: `   foo
   ===`
   Error: ` --> 1:7
  |
1 |    foo␊
  |       ^---
  |
  = expected safe_inline`

❌ **setext_uneven_underline**: `setext_h2` (Unexpected failure)
   Input: `foo
======`
   Error: ` --> 1:4
  |
1 | foo␊
  |    ^---
  |
  = expected safe_inline`

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

❌ **bold_asterisk_multiline_fail**: `bold` (Unexpected failure)
   Input: `\*\*bold
text\*\*`
   Error: ` --> 1:1
  |
1 | **bold
  | ^---
  |
  = expected bold`

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

❌ **bold_mismatched**: `bold` (Unexpected failure)
   Input: `\*\*bold with underscore\_\_`
   Error: ` --> 1:1
  |
1 | **bold with underscore__
  | ^---
  |
  = expected bold`

❌ **bold_unclosed**: `bold` (Unexpected failure)
   Input: `\*\*missing closing`
   Error: ` --> 1:1
  |
1 | **missing closing
  | ^---
  |
  = expected bold`

## error_recovery

❌ **partial_bold_recovery**: `bold` (Unexpected failure)
   Input: `\*\*bold but not closed and more text`
   Error: ` --> 1:1
  |
1 | **bold but not closed and more text
  | ^---
  |
  = expected bold`

❌ **partial_link_recovery**: `inline_link` (Unexpected failure)
   Input: `\[link text but no closing and more text`
   Error: ` --> 1:1
  |
1 | [link text but no closing and more text
  | ^---
  |
  = expected inline_link`

❌ **mixed_delimiters_recovery**: `text` (Unexpected failure)
   Input: `\*\*bold \_italic\* underscore\_\_`
   Error: ` --> 1:1
  |
1 | **bold _italic* underscore__
  | ^---
  |
  = expected text`

✅ **malformed_table_recovery**: `table` (Expected failure)
   Input: `| A | B |
|---|
| 1 | 2 | 3 |`
   Error: ` --> 3:14
  |
3 | | 1 | 2 | 3 |
  |              ^---
  |
  = expected inline_core`

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
   Error: ` --> 2:1
  |
2 | custom_type
  | ^---
  |
  = expected admonition_type`

✅ **invalid_macro_syntax**: `text` (Expected failure)
   Input: `\[invalid:syntax\](no closing`
   Error: ` --> 1:1
  |
1 | [invalid:syntax](no closing
  | ^---
  |
  = expected text`

✅ **empty_inline_code**: `fenced_code` (Expected failure)
   Input: `\`\``
   Error: ` --> 1:1
  |
1 | ``
  | ^---
  |
  = expected fenced_code`

✅ **empty_emphasis**: `text` (Expected failure)
   Input: `\*\*\*\*`
   Error: ` --> 1:1
  |
1 | ****
  | ^---
  |
  = expected text`

✅ **empty_link_text**: `inline_link` (Expected failure)
   Input: `\[\](url)`
   Error: ` --> 1:4
  |
1 | [](url)
  |    ^---
  |
  = expected inline_url`

✅ **empty_image_alt**: `text`
   Input: `!\[\](image.png)`
   Parse Tree:
   ```
  └── text: "!"
   ```

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

✅ **malformed_utf8_high_surrogate**: `text`
   Input: `\\uD800`
   Parse Tree:
   ```
  └── text: "\\uD800"
   ```

✅ **malformed_utf8_low_surrogate**: `text`
   Input: `\\uDFFF`
   Parse Tree:
   ```
  └── text: "\\uDFFF"
   ```

✅ **malformed_utf8_overlong**: `text`
   Input: `\\u0000`
   Parse Tree:
   ```
  └── text: "\\u0000"
   ```

✅ **random_ascii_control**: `text`
   Input: `\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\u0008`
   Parse Tree:
   ```
  └── text: "\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\u0008"
   ```

✅ **random_ascii_printable**: `text`
   Input: `!@#$%^&\*()\_+{}|:<>?\[\];',./`
   Parse Tree:
   ```
  └── text: "!@#"
   ```

✅ **random_ascii_extended**: `text`
   Input: `¡¢£¤¥¦§¨©ª«¬­®¯°±²³´µ¶·¸¹º»¼½¾¿`
   Parse Tree:
   ```
  └── text: "¡¢£¤¥¦§¨©ª«¬­®¯°±²³´µ¶·¸¹º»¼½¾¿"
   ```

❌ **chaos_markdown_soup**: `text` (Unexpected failure)
   Input: `\*\_\`#\[\]()~>|$@^=-\\\*\*\_\`#\[\]()~>|$@^=-\\\*`
   Error: ` --> 1:1
  |
1 | *_`#[]()~>|$@^=-\\**_`#[]()~>|$@^=-\\*
  | ^---
  |
  = expected text`

✅ **chaos_nested_delimiters**: `text`
   Input: `(\[{<>}\])((\[{<>}\]))(((\[{<>}\])))`
   Parse Tree:
   ```
  └── text: "("
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

## memory_stress

❌ **huge_document_headings**: `text` (Unexpected failure)
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
   Error: ` --> 1:1
  |
1 | # Heading 1
  | ^---
  |
  = expected text`

❌ **many_bold_words**: `text` (Unexpected failure)
   Input: `\*\*word1\*\* \*\*word2\*\* \*\*word3\*\* \*\*word4\*\* \*\*word5\*\* \*\*word6\*\* \*\*word7\*\* \*\*word8\*\* \*\*word9\*\* \*\*word10\*\* \*\*word11\*\* \*\*word12\*\* \*\*word13\*\* \*\*word14\*\* \*\*word15\*\* \*\*word16\*\* \*\*word17\*\* \*\*word18\*\* \*\*word19\*\* \*\*word20\*\*`
   Error: ` --> 1:1
  |
1 | **word1** **word2** **word3** **word4** **word5** **word6** **word7** **word8** **word9** **word10** **word11** **word12** **word13** **word14** **word15** **word16** **word17** **word18** **word19** **word20**
  | ^---
  |
  = expected text`

❌ **many_links**: `text` (Unexpected failure)
   Input: `\[link1\](url1) \[link2\](url2) \[link3\](url3) \[link4\](url4) \[link5\](url5) \[link6\](url6) \[link7\](url7) \[link8\](url8) \[link9\](url9) \[link10\](url10)`
   Error: ` --> 1:1
  |
1 | [link1](url1) [link2](url2) [link3](url3) [link4](url4) [link5](url5) [link6](url6) [link7](url7) [link8](url8) [link9](url9) [link10](url10)
  | ^---
  |
  = expected text`

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

❌ **exponential_nesting**: `text` (Unexpected failure)
   Input: `\*\*bold \*italic \`code \*\*bold \*italic \`code \*\*bold \*italic \`code\` italic\* bold\*\* code\` italic\* bold\*\* \`code\` italic\* bold\*\*`
   Error: ` --> 1:1
  |
1 | **bold *italic `code **bold *italic `code **bold *italic `code` italic* bold** code` italic* bold** `code` italic* bold**
  | ^---
  |
  = expected text`

✅ **parse_tree_explosion**: `text`
   Input: `((((((((((nested parentheses))))))))))`
   Parse Tree:
   ```
  └── text: "((((((((((nested parentheses))))))))))"
   ```

❌ **large_table_data**: `table` (Unexpected failure)
   Input: `| Col1 | Col2 | Col3 | Col4 | Col5 | Col6 | Col7 | Col8 |
|------|------|------|------|------|------|------|------|
| Data1| Data2| Data3| Data4| Data5| Data6| Data7| Data8|
| Data9| Data10| Data11| Data12| Data13| Data14| Data15| Data16|
| Data17| Data18| Data19| Data20| Data21| Data22| Data23| Data24|
| Data25| Data26| Data27| Data28| Data29| Data30| Data31| Data32|`
   Error: ` --> 6:66
  |
6 | | Data25| Data26| Data27| Data28| Data29| Data30| Data31| Data32|
  |                                                                  ^---
  |
  = expected inline_core`

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
  └── subscript: "˅sub˅"
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
   Input: `run@bash(echo \"hello world\")`
   Parse Tree:
   ```
  ├── run_inline > "run@bash(echo \"hello world\")"
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
echo \"done\"
\`\`\``
   Parse Tree:
   ```
  ├── run_block_fenced > "```run@bash
ls -la
echo \"done\"
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
    println!(\"Hello, world!\");
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
    println!(\"Hello, world!\");
}
```

> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
    ├── block > "# How to Use Marco"
      ├── heading > "# How to Use Marco"
        ├── H1 > "# How to Use Marco"
          ├── heading_content > "How to Use Marco"
            ├── safe_inline > "How"
              └── word: "How"
            ├── safe_inline > "to"
              └── word: "to"
            ├── safe_inline > "Use"
              └── word: "Use"
            ├── safe_inline > "Marco"
              └── word: "Marco"
    ├── block > "**Marco** is a powerful *markdown* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

"
      ├── paragraph > "**Marco** is a powerful *markdown* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

"
        ├── paragraph_line > "**Marco** is a powerful *markdown* processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

"
          ├── inline > "**Marco**"
            ├── inline_core > "**Marco**"
              ├── emphasis > "**Marco**"
                ├── bold > "**Marco**"
                  └── bold_asterisk: "**Marco**"
          ├── inline > "is a powerful "
            ├── inline_core > "is a powerful "
              └── text: "is a powerful "
          ├── inline > "*markdown*"
            ├── inline_core > "*markdown*"
              ├── emphasis > "*markdown*"
                ├── italic > "*markdown*"
                  └── italic_asterisk: "*markdown*"
          ├── inline > "processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

"
            ├── inline_core > "processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

"
              └── text: "processor.

## Features

- Fast parsing
- Extensions support
- Real-time preview

"
    ├── block > "```rust
fn main() {
    println!(\"Hello, world!\");
}
```"
      ├── code_block > "```rust
fn main() {
    println!(\"Hello, world!\");
}
```"
        ├── fenced_code > "```rust
fn main() {
    println!(\"Hello, world!\");
}
```"
          └── language_id: "rust"
    ├── block > "> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
      ├── blockquote > "> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
        ├── blockquote_line > "> Marco makes markdown easy!

Visit [our website](https://example.com) for more info."
          ├── inline > "Marco makes markdown easy!

Visit "
            ├── inline_core > "Marco makes markdown easy!

Visit "
              └── text: "Marco makes markdown easy!

Visit "
          ├── inline > "[our website](https://example.com)"
            ├── inline_core > "[our website](https://example.com)"
              ├── inline_link > "[our website](https://example.com)"
                └── inline_link_text: "our website"
                └── inline_url: "https://example.com"
          ├── inline > "for more info."
            ├── inline_core > "for more info."
              └── text: "for more info."
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
            ├── safe_inline > "API"
              └── word: "API"
            ├── safe_inline > "Reference"
              └── word: "Reference"
    ├── block > "## Authentication"
      ├── heading > "## Authentication"
        ├── H2 > "## Authentication"
          ├── heading_content > "Authentication"
            ├── safe_inline > "Authentication"
              └── word: "Authentication"
    ├── block > "Use JWT tokens:

"
      ├── paragraph > "Use JWT tokens:

"
        ├── paragraph_line > "Use JWT tokens:

"
          ├── inline > "Use JWT tokens:

"
            ├── inline_core > "Use JWT tokens:

"
              └── text: "Use JWT tokens:

"
    ├── block > "```http
GET /api/users
Authorization: Bearer <token>
```"
      ├── code_block > "```http
GET /api/users
Authorization: Bearer <token>
```"
        ├── fenced_code > "```http
GET /api/users
Authorization: Bearer <token>
```"
          └── language_id: "http"
    ├── block > "### Response"
      ├── heading > "### Response"
        ├── H3 > "### Response"
          ├── heading_content > "Response"
            ├── safe_inline > "Response"
              └── word: "Response"
    ├── block > "| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
      ├── paragraph > "| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
        ├── paragraph_line > "| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
          ├── inline > "| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
            ├── inline_core > "| Field | Type | Description |
|-------|------|-----------|
| id    | int  | User ID   |
| name  | str  | Full name |

:::
warning
Tokens expire after 24 hours
:::"
              └── text: "| Field | Type | Description |
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
            ├── safe_inline > "My"
              └── word: "My"
            ├── safe_inline > "Project"
              └── word: "Project"
    ├── block > "[![CI](https://img.shields.io/badge/CI-passing-green)](https://example.com)

## Quick Start

1. Install dependencies: `npm install`
2. Run tests: `npm test`
3. Build: `npm run build`

### Configuration

Create a `.env` file:

"
      ├── paragraph > "[![CI](https://img.shields.io/badge/CI-passing-green)](https://example.com)

## Quick Start

1. Install dependencies: `npm install`
2. Run tests: `npm test`
3. Build: `npm run build`

### Configuration

Create a `.env` file:

"
        ├── paragraph_line > "[![CI](https://img.shields.io/badge/CI-passing-green)](https://example.com)

## Quick Start

1. Install dependencies: `npm install`"
          ├── inline > "[![CI](https://img.shields.io/badge/CI-passing-green)"
            ├── inline_core > "[![CI](https://img.shields.io/badge/CI-passing-green)"
              ├── inline_link > "[![CI](https://img.shields.io/badge/CI-passing-green)"
                └── inline_link_text: "![CI"
                └── inline_url: "https://img.shields.io/badge/CI-passing-green"
          ├── inline > "](https://example.com)

## Quick Start

1. Install dependencies: "
            ├── inline_core > "](https://example.com)

## Quick Start

1. Install dependencies: "
              └── text: "](https://example.com)

## Quick Start

1. Install dependencies: "
          ├── inline > "`npm install`"
            ├── inline_core > "`npm install`"
              └── code_inline: "`npm install`"
        ├── paragraph_line > "2. Run tests: `npm test`"
          ├── inline > "2. Run tests: "
            ├── inline_core > "2. Run tests: "
              └── text: "2. Run tests: "
          ├── inline > "`npm test`"
            ├── inline_core > "`npm test`"
              └── code_inline: "`npm test`"
        ├── paragraph_line > "3. Build: `npm run build`

### Configuration

Create a `.env` file:

"
          ├── inline > "3. Build: "
            ├── inline_core > "3. Build: "
              └── text: "3. Build: "
          ├── inline > "`npm run build`"
            ├── inline_core > "`npm run build`"
              └── code_inline: "`npm run build`"
          ├── inline > "

### Configuration

Create a "
            ├── inline_core > "

### Configuration

Create a "
              └── text: "

### Configuration

Create a "
          ├── inline > "`.env`"
            ├── inline_core > "`.env`"
              └── code_inline: "`.env`"
          ├── inline > "file:

"
            ├── inline_core > "file:

"
              └── text: "file:

"
    ├── block > "```bash
API_KEY=your_key_here
DEBUG=true
```"
      ├── code_block > "```bash
API_KEY=your_key_here
DEBUG=true
```"
        ├── fenced_code > "```bash
API_KEY=your_key_here
DEBUG=true
```"
          └── language_id: "bash"
    ├── block > "## Contributing"
      ├── heading > "## Contributing"
        ├── H2 > "## Contributing"
          ├── heading_content > "Contributing"
            ├── safe_inline > "Contributing"
              └── word: "Contributing"
    ├── block > "- [x] Write tests
- [ ] Update docs
- [ ] Add examples
"
      ├── list > "- [x] Write tests
- [ ] Update docs
- [ ] Add examples
"
        ├── list_item > "- [x] Write tests"
          ├── regular_list_item > "- [x] Write tests"
            └── list_marker: "-"
            └── list_item_content: "[x] Write tests"
        ├── list_item > "- [ ] Update docs"
          ├── regular_list_item > "- [ ] Update docs"
            └── list_marker: "-"
            └── list_item_content: "[ ] Update docs"
        ├── list_item > "- [ ] Add examples"
          ├── regular_list_item > "- [ ] Add examples"
            └── list_marker: "-"
            └── list_item_content: "[ ] Add examples"
    ├── block > "**Note**: Please follow our "
      ├── paragraph > "**Note**: Please follow our "
        ├── paragraph_line > "**Note**: Please follow our "
          ├── inline > "**Note**"
            ├── inline_core > "**Note**"
              ├── emphasis > "**Note**"
                ├── bold > "**Note**"
                  └── bold_asterisk: "**Note**"
          ├── inline > ": Please follow our "
            ├── inline_core > ": Please follow our "
              └── text: ": Please follow our "
    ├── block > "[style guide](STYLE.md)."
      └── unknown_block: "[style guide](STYLE.md)."
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

✅ **almost_empty**: `text` (Expected failure)
   Input: ` `
   Error: ` --> 1:1
  |
1 |  
  | ^---
  |
  = expected text`

✅ **just_newlines**: `text`
   Input: `




`
   Parse Tree:
   ```
  └── text: "




"
   ```

❌ **only_markdown_chars**: `text` (Unexpected failure)
   Input: `\*\_\`#\[\]~>|$@^=-`
   Error: ` --> 1:1
  |
1 | *_`#[]~>|$@^=-
  | ^---
  |
  = expected text`

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
  └── inline_url: "https://example.com/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
   ```

✅ **url_with_many_params**: `inline_url`
   Input: `https://example.com?param0=value0&param1=value1&param2=value2&param3=value3&param4=value4&param5=value5&param6=value6&param7=value7&param8=value8&param9=value9&param10=value10&param11=value11&param12=value12&param13=value13&param14=value14&param15=value15&param16=value16&param17=value17&param18=value18&param19=value19`
   Parse Tree:
   ```
  └── inline_url: "https://example.com?param0=value0&param1=value1&param2=value2&param3=value3&param4=value4&param5=value5&param6=value6&param7=value7&param8=value8&param9=value9&param10=value10&param11=value11&param12=value12&param13=value13&param14=value14&param15=value15&param16=value16&param17=value17&param18=value18&param19=value19"
   ```

❌ **ipv6_url**: `inline_url` (Unexpected failure)
   Input: `http://\[2001:db8::1\]:8080/path`
   Error: ` --> 1:1
  |
1 | http://[2001:db8::1]:8080/path
  | ^---
  |
  = expected inline_url`

✅ **localhost_variants**: `text`
   Input: `http://127.0.0.1:8080/path`
   Parse Tree:
   ```
  └── text: "http://127.0.0.1:8080/path"
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

❌ **cm_example_254**: `text` (Unexpected failure)
   Input: `1.  A paragraph
    with two lines.

        indented code

    > A block quote.
`
   Error: ` --> 1:1
  |
1 | 1.  A paragraph
  | ^---
  |
  = expected text`

❌ **cm_example_255**: `text` (Unexpected failure)
   Input: `- one

 two
`
   Error: ` --> 1:1
  |
1 | - one
  | ^---
  |
  = expected text`

❌ **cm_example_256**: `text` (Unexpected failure)
   Input: `- one

  two
`
   Error: ` --> 1:1
  |
1 | - one
  | ^---
  |
  = expected text`

❌ **cm_example_257**: `text` (Unexpected failure)
   Input: ` -    one

     two
`
   Error: ` --> 1:1
  |
1 |  -    one
  | ^---
  |
  = expected text`

❌ **cm_example_258**: `text` (Unexpected failure)
   Input: ` -    one

      two
`
   Error: ` --> 1:1
  |
1 |  -    one
  | ^---
  |
  = expected text`

❌ **cm_example_259**: `text` (Unexpected failure)
   Input: `   > > 1.  one
>>
>>     two
`
   Error: ` --> 1:1
  |
1 |    > > 1.  one
  | ^---
  |
  = expected text`

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

❌ **cm_example_262**: `text` (Unexpected failure)
   Input: `- foo


  bar
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

❌ **cm_example_263**: `text` (Unexpected failure)
   Input: `1.  foo

    \`\`\`
    bar
    \`\`\`

    baz

    > bam
`
   Error: ` --> 1:1
  |
1 | 1.  foo
  | ^---
  |
  = expected text`

❌ **cm_example_264**: `text` (Unexpected failure)
   Input: `- Foo

      bar


      baz
`
   Error: ` --> 1:1
  |
1 | - Foo
  | ^---
  |
  = expected text`

❌ **cm_example_265**: `text` (Unexpected failure)
   Input: `123456789. ok
`
   Error: ` --> 1:1
  |
1 | 123456789. ok
  | ^---
  |
  = expected text`

❌ **cm_example_266**: `text` (Unexpected failure)
   Input: `1234567890. not ok
`
   Error: ` --> 1:1
  |
1 | 1234567890. not ok
  | ^---
  |
  = expected text`

❌ **cm_example_267**: `text` (Unexpected failure)
   Input: `0. ok
`
   Error: ` --> 1:1
  |
1 | 0. ok
  | ^---
  |
  = expected text`

❌ **cm_example_268**: `text` (Unexpected failure)
   Input: `003. ok
`
   Error: ` --> 1:1
  |
1 | 003. ok
  | ^---
  |
  = expected text`

✅ **cm_example_269**: `text`
   Input: `-1. not ok
`
   Parse Tree:
   ```
  └── text: "-1. not ok
"
   ```

❌ **cm_example_270**: `text` (Unexpected failure)
   Input: `- foo

      bar
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

❌ **cm_example_271**: `text` (Unexpected failure)
   Input: `  10.  foo

           bar
`
   Error: ` --> 1:1
  |
1 |   10.  foo
  | ^---
  |
  = expected text`

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

❌ **cm_example_273**: `text` (Unexpected failure)
   Input: `1.     indented code

   paragraph

       more code
`
   Error: ` --> 1:1
  |
1 | 1.     indented code
  | ^---
  |
  = expected text`

❌ **cm_example_274**: `text` (Unexpected failure)
   Input: `1.      indented code

   paragraph

       more code
`
   Error: ` --> 1:1
  |
1 | 1.      indented code
  | ^---
  |
  = expected text`

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

❌ **cm_example_276**: `text` (Unexpected failure)
   Input: `-    foo

  bar
`
   Error: ` --> 1:1
  |
1 | -    foo
  | ^---
  |
  = expected text`

❌ **cm_example_277**: `text` (Unexpected failure)
   Input: `-  foo

   bar
`
   Error: ` --> 1:1
  |
1 | -  foo
  | ^---
  |
  = expected text`

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
  "
   ```

❌ **cm_example_279**: `text` (Unexpected failure)
   Input: `-   
  foo
`
   Error: ` --> 1:1
  |
1 | -   
  | ^---
  |
  = expected text`

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

❌ **cm_example_281**: `text` (Unexpected failure)
   Input: `- foo
-
- bar
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

❌ **cm_example_282**: `text` (Unexpected failure)
   Input: `- foo
-   
- bar
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

❌ **cm_example_283**: `text` (Unexpected failure)
   Input: `1. foo
2.
3. bar
`
   Error: ` --> 1:1
  |
1 | 1. foo
  | ^---
  |
  = expected text`

❌ **cm_example_284**: `text` (Unexpected failure)
   Input: `\*
`
   Error: ` --> 1:1
  |
1 | *
  | ^---
  |
  = expected text`

✅ **cm_example_285**: `text`
   Input: `foo
\*

foo
1.
`
   Parse Tree:
   ```
  └── text: "foo
"
   ```

❌ **cm_example_286**: `text` (Unexpected failure)
   Input: ` 1.  A paragraph
     with two lines.

         indented code

     > A block quote.
`
   Error: ` --> 1:1
  |
1 |  1.  A paragraph
  | ^---
  |
  = expected text`

❌ **cm_example_287**: `text` (Unexpected failure)
   Input: `  1.  A paragraph
      with two lines.

          indented code

      > A block quote.
`
   Error: ` --> 1:1
  |
1 |   1.  A paragraph
  | ^---
  |
  = expected text`

❌ **cm_example_288**: `text` (Unexpected failure)
   Input: `   1.  A paragraph
       with two lines.

           indented code

       > A block quote.
`
   Error: ` --> 1:1
  |
1 |    1.  A paragraph
  | ^---
  |
  = expected text`

❌ **cm_example_289**: `text` (Unexpected failure)
   Input: `    1.  A paragraph
        with two lines.

            indented code

        > A block quote.
`
   Error: ` --> 1:1
  |
1 |     1.  A paragraph
  | ^---
  |
  = expected text`

❌ **cm_example_290**: `text` (Unexpected failure)
   Input: `  1.  A paragraph
with two lines.

          indented code

      > A block quote.
`
   Error: ` --> 1:1
  |
1 |   1.  A paragraph
  | ^---
  |
  = expected text`

❌ **cm_example_291**: `text` (Unexpected failure)
   Input: `  1.  A paragraph
    with two lines.
`
   Error: ` --> 1:1
  |
1 |   1.  A paragraph
  | ^---
  |
  = expected text`

❌ **cm_example_292**: `text` (Unexpected failure)
   Input: `> 1. > Blockquote
continued here.
`
   Error: ` --> 1:1
  |
1 | > 1. > Blockquote
  | ^---
  |
  = expected text`

❌ **cm_example_293**: `text` (Unexpected failure)
   Input: `> 1. > Blockquote
> continued here.
`
   Error: ` --> 1:1
  |
1 | > 1. > Blockquote
  | ^---
  |
  = expected text`

❌ **cm_example_294**: `text` (Unexpected failure)
   Input: `- foo
  - bar
    - baz
      - boo
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

❌ **cm_example_295**: `text` (Unexpected failure)
   Input: `- foo
 - bar
  - baz
   - boo
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

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

❌ **cm_example_298**: `text` (Unexpected failure)
   Input: `- - foo
`
   Error: ` --> 1:1
  |
1 | - - foo
  | ^---
  |
  = expected text`

❌ **cm_example_299**: `text` (Unexpected failure)
   Input: `1. - 2. foo
`
   Error: ` --> 1:1
  |
1 | 1. - 2. foo
  | ^---
  |
  = expected text`

❌ **cm_example_300**: `text` (Unexpected failure)
   Input: `- # Foo
- Bar
  ---
  baz
`
   Error: ` --> 1:1
  |
1 | - # Foo
  | ^---
  |
  = expected text`

## commonmark_atx_headings

❌ **cm_example_62**: `text` (Unexpected failure)
   Input: `# foo
## foo
### foo
#### foo
##### foo
###### foo
`
   Error: ` --> 1:1
  |
1 | # foo
  | ^---
  |
  = expected text`

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

✅ **cm_example_65**: `text`
   Input: `\\## foo
`
   Parse Tree:
   ```
  └── text: "\\## foo
"
   ```

❌ **cm_example_66**: `text` (Unexpected failure)
   Input: `# foo \*bar\* \\\*baz\\\*
`
   Error: ` --> 1:1
  |
1 | # foo *bar* \\*baz\\*
  | ^---
  |
  = expected text`

❌ **cm_example_67**: `text` (Unexpected failure)
   Input: `#                  foo                     
`
   Error: ` --> 1:1
  |
1 | #                  foo                     
  | ^---
  |
  = expected text`

❌ **cm_example_68**: `text` (Unexpected failure)
   Input: ` ### foo
  ## foo
   # foo
`
   Error: ` --> 1:1
  |
1 |  ### foo
  | ^---
  |
  = expected text`

❌ **cm_example_69**: `text` (Unexpected failure)
   Input: `    # foo
`
   Error: ` --> 1:1
  |
1 |     # foo
  | ^---
  |
  = expected text`

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

❌ **cm_example_71**: `text` (Unexpected failure)
   Input: `## foo ##
  ###   bar    ###
`
   Error: ` --> 1:1
  |
1 | ## foo ##
  | ^---
  |
  = expected text`

❌ **cm_example_72**: `text` (Unexpected failure)
   Input: `# foo ##################################
##### foo ##
`
   Error: ` --> 1:1
  |
1 | # foo ##################################
  | ^---
  |
  = expected text`

❌ **cm_example_73**: `text` (Unexpected failure)
   Input: `### foo ###     
`
   Error: ` --> 1:1
  |
1 | ### foo ###     
  | ^---
  |
  = expected text`

❌ **cm_example_74**: `text` (Unexpected failure)
   Input: `### foo ### b
`
   Error: ` --> 1:1
  |
1 | ### foo ### b
  | ^---
  |
  = expected text`

❌ **cm_example_75**: `text` (Unexpected failure)
   Input: `# foo#
`
   Error: ` --> 1:1
  |
1 | # foo#
  | ^---
  |
  = expected text`

❌ **cm_example_76**: `text` (Unexpected failure)
   Input: `### foo \\###
## foo #\\##
# foo \\#
`
   Error: ` --> 1:1
  |
1 | ### foo \\###
  | ^---
  |
  = expected text`

❌ **cm_example_77**: `text` (Unexpected failure)
   Input: `\*\*\*\*
## foo
\*\*\*\*
`
   Error: ` --> 1:1
  |
1 | ****
  | ^---
  |
  = expected text`

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

❌ **cm_example_79**: `text` (Unexpected failure)
   Input: `## 
#
### ###
`
   Error: ` --> 1:1
  |
1 | ## 
  | ^---
  |
  = expected text`

## performance_tests

❌ **backtrack_emphasis**: `text` (Unexpected failure)
   Input: `\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*a\*`
   Error: ` --> 1:1
  |
1 | *a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*a*
  | ^---
  |
  = expected text`

❌ **backtrack_links**: `text` (Unexpected failure)
   Input: `\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[\[not a link\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]\]`
   Error: ` --> 1:1
  |
1 | [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[not a link]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
  | ^---
  |
  = expected text`

❌ **backtrack_code**: `text` (Unexpected failure)
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
   Error: ` --> 1:1
  |
1 | ```
  | ^---
  |
  = expected text`

❌ **large_table**: `table` (Unexpected failure)
   Input: `| A | B | C | D | E | F | G | H |
|---|---|---|---|---|---|---|---|
| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
| 9 | 10| 11| 12| 13| 14| 15| 16|
| 17| 18| 19| 20| 21| 22| 23| 24|`
   Error: ` --> 5:34
  |
5 | | 17| 18| 19| 20| 21| 22| 23| 24|
  |                                  ^---
  |
  = expected inline_core`

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

## commonmark_lists

❌ **cm_example_301**: `text` (Unexpected failure)
   Input: `- foo
- bar
+ baz
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

❌ **cm_example_302**: `text` (Unexpected failure)
   Input: `1. foo
2. bar
3) baz
`
   Error: ` --> 1:1
  |
1 | 1. foo
  | ^---
  |
  = expected text`

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

❌ **cm_example_306**: `text` (Unexpected failure)
   Input: `- foo

- bar


- baz
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

❌ **cm_example_307**: `text` (Unexpected failure)
   Input: `- foo
  - bar
    - baz


      bim
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

❌ **cm_example_308**: `text` (Unexpected failure)
   Input: `- foo
- bar

<!-- -->

- baz
- bim
`
   Error: ` --> 1:1
  |
1 | - foo
  | ^---
  |
  = expected text`

❌ **cm_example_309**: `text` (Unexpected failure)
   Input: `-   foo

    notcode

-   foo

<!-- -->

    code
`
   Error: ` --> 1:1
  |
1 | -   foo
  | ^---
  |
  = expected text`

❌ **cm_example_310**: `text` (Unexpected failure)
   Input: `- a
 - b
  - c
   - d
  - e
 - f
- g
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

❌ **cm_example_311**: `text` (Unexpected failure)
   Input: `1. a

  2. b

   3. c
`
   Error: ` --> 1:1
  |
1 | 1. a
  | ^---
  |
  = expected text`

❌ **cm_example_312**: `text` (Unexpected failure)
   Input: `- a
 - b
  - c
   - d
    - e
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

❌ **cm_example_313**: `text` (Unexpected failure)
   Input: `1. a

  2. b

    3. c
`
   Error: ` --> 1:1
  |
1 | 1. a
  | ^---
  |
  = expected text`

❌ **cm_example_314**: `text` (Unexpected failure)
   Input: `- a
- b

- c
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

❌ **cm_example_315**: `text` (Unexpected failure)
   Input: `\* a
\*

\* c
`
   Error: ` --> 1:1
  |
1 | * a
  | ^---
  |
  = expected text`

❌ **cm_example_316**: `text` (Unexpected failure)
   Input: `- a
- b

  c
- d
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

❌ **cm_example_317**: `text` (Unexpected failure)
   Input: `- a
- b

  \[ref\]: /url
- d
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

❌ **cm_example_318**: `text` (Unexpected failure)
   Input: `- a
- \`\`\`
  b


  \`\`\`
- c
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

❌ **cm_example_319**: `text` (Unexpected failure)
   Input: `- a
  - b

    c
- d
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

❌ **cm_example_320**: `text` (Unexpected failure)
   Input: `\* a
  > b
  >
\* c
`
   Error: ` --> 1:1
  |
1 | * a
  | ^---
  |
  = expected text`

❌ **cm_example_321**: `text` (Unexpected failure)
   Input: `- a
  > b
  \`\`\`
  c
  \`\`\`
- d
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

❌ **cm_example_322**: `text` (Unexpected failure)
   Input: `- a
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

❌ **cm_example_323**: `text` (Unexpected failure)
   Input: `- a
  - b
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

❌ **cm_example_324**: `text` (Unexpected failure)
   Input: `1. \`\`\`
   foo
   \`\`\`

   bar
`
   Error: ` --> 1:1
  |
1 | 1. ```
  | ^---
  |
  = expected text`

❌ **cm_example_325**: `text` (Unexpected failure)
   Input: `\* foo
  \* bar

  baz
`
   Error: ` --> 1:1
  |
1 | * foo
  | ^---
  |
  = expected text`

❌ **cm_example_326**: `text` (Unexpected failure)
   Input: `- a
  - b
  - c

- d
  - e
  - f
`
   Error: ` --> 1:1
  |
1 | - a
  | ^---
  |
  = expected text`

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
    println!(\"hello\");
}
\`\`\``
   Parse Tree:
   ```
  ├── fenced_code > "```rust
fn main() {
    println!(\"hello\");
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

## security_vectors

✅ **script_tag**: `inline_html`
   Input: `<script>alert('xss')</script>`
   Parse Tree:
   ```
  └── inline_html: "<script>"
   ```

✅ **script_src**: `inline_html`
   Input: `<script src=\"malicious.js\"></script>`
   Parse Tree:
   ```
  └── inline_html: "<script src=\"malicious.js\">"
   ```

❌ **onclick_handler**: `text` (Unexpected failure)
   Input: `<div onclick=\"alert('xss')\">click</div>`
   Error: ` --> 1:1
  |
1 | <div onclick=\"alert('xss')\">click</div>
  | ^---
  |
  = expected text`

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
   Error: ` --> 1:9
  |
1 | [click](data:text/html,<script>alert('xss')</script>)
  |         ^---
  |
  = expected inline_url`

❌ **mixed_xss_1**: `text` (Unexpected failure)
   Input: `<img src=x onerror=alert('xss')>
\*\*bold\*\*`
   Error: ` --> 1:1
  |
1 | <img src=x onerror=alert('xss')>
  | ^---
  |
  = expected text`

❌ **mixed_xss_2**: `text` (Unexpected failure)
   Input: `\*\*bold\*\* <script>alert('xss')</script>`
   Error: ` --> 1:1
  |
1 | **bold** <script>alert('xss')</script>
  | ^---
  |
  = expected text`

❌ **mixed_xss_3**: `text` (Unexpected failure)
   Input: `\[text\](<img src=x onerror=alert('xss')>)`
   Error: ` --> 1:1
  |
1 | [text](<img src=x onerror=alert('xss')>)
  | ^---
  |
  = expected text`

❌ **ftp_protocol**: `text` (Unexpected failure)
   Input: `\[link\](ftp://malicious.com)`
   Error: ` --> 1:1
  |
1 | [link](ftp://malicious.com)
  | ^---
  |
  = expected text`

❌ **file_protocol**: `text` (Unexpected failure)
   Input: `\[link\](file:///etc/passwd)`
   Error: ` --> 1:1
  |
1 | [link](file:///etc/passwd)
  | ^---
  |
  = expected text`

❌ **custom_protocol**: `text` (Unexpected failure)
   Input: `\[link\](custom://protocol)`
   Error: ` --> 1:1
  |
1 | [link](custom://protocol)
  | ^---
  |
  = expected text`

❌ **url_with_credentials**: `inline_link` (Unexpected failure)
   Input: `https://user:pass@evil.com`
   Error: ` --> 1:1
  |
1 | https://user:pass@evil.com
  | ^---
  |
  = expected inline_link`

❌ **url_with_unicode**: `inline_link` (Unexpected failure)
   Input: `"https://аpple.com"  # Punycode attack`
   Error: ` --> 1:1
  |
1 | "https://аpple.com"  # Punycode attack
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

❌ **fake_attachment**: `text` (Unexpected failure)
   Input: `\[download.pdf\](malicious.exe)`
   Error: ` --> 1:1
  |
1 | [download.pdf](malicious.exe)
  | ^---
  |
  = expected text`

✅ **misleading_link**: `inline_link`
   Input: `\[google.com\](https://evil.com)`
   Parse Tree:
   ```
  ├── inline_link > "[google.com](https://evil.com)"
    └── inline_link_text: "google.com"
    └── inline_url: "https://evil.com"
   ```

✅ **homograph_attack**: `text`
   Input: `"\[аpple.com\](https://evil.com)"  # Cyrillic 'а'`
   Parse Tree:
   ```
  └── text: """
   ```

## tabs

❌ **tabs_simple**: `tabs_block` (Unexpected failure)
   Input: `:::
tabs
General content
@tab Tab 1
Content 1
@tab Tab 2
Content 2
:::`
   Error: ` --> 3:1
  |
3 | General content
  | ^---
  |
  = expected tab_line`

❌ **tabs_with_title**: `tabs_block` (Unexpected failure)
   Input: `:::
tabs Main Tabs
@tab First
First content
@tab Second
Second content
:::`
   Error: ` --> 1:1
  |
1 | :::
  | ^---
  |
  = expected tabs_block`

❌ **tabs_formatted**: `tabs_block` (Unexpected failure)
   Input: `:::
tabs
@tab \*\*Bold Tab\*\*
Content with \*\*formatting\*\*
@tab \*Italic Tab\*
More content
:::`
   Error: ` --> 3:2
  |
3 | @tab **Bold Tab**
  |  ^---
  |
  = unexpected KW_TAB`

✅ **tabs_empty_content**: `tabs_block` (Expected failure)
   Input: `:::
tabs
@tab Empty
@tab Also Empty
:::`
   Error: ` --> 3:2
  |
3 | @tab Empty
  |  ^---
  |
  = unexpected KW_TAB`

✅ **tabs_no_general**: `tabs_block` (Expected failure)
   Input: `:::
tabs
@tab Only Tab
Only content
:::`
   Error: ` --> 3:2
  |
3 | @tab Only Tab
  |  ^---
  |
  = unexpected KW_TAB`

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

❌ **cm_example_108**: `text` (Unexpected failure)
   Input: `  - foo

    bar
`
   Error: ` --> 1:1
  |
1 |   - foo
  | ^---
  |
  = expected text`

❌ **cm_example_109**: `text` (Unexpected failure)
   Input: `1.  foo

    - bar
`
   Error: ` --> 1:1
  |
1 | 1.  foo
  | ^---
  |
  = expected text`

❌ **cm_example_110**: `text` (Unexpected failure)
   Input: `    <a/>
    \*hi\*

    - one
`
   Error: ` --> 1:1
  |
1 |     <a/>
  | ^---
  |
  = expected text`

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

❌ **cm_example_115**: `text` (Unexpected failure)
   Input: `# Heading
    foo
Heading
------
    foo
----
`
   Error: ` --> 1:1
  |
1 | # Heading
  | ^---
  |
  = expected text`

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

## commonmark_precedence

❌ **cm_example_42**: `text` (Unexpected failure)
   Input: `- \`one
- two\`
`
   Error: ` --> 1:1
  |
1 | - `one
  | ^---
  |
  = expected text`

## headings_atx

✅ **h1_simple**: `H1`
   Input: `# Hello`
   Parse Tree:
   ```
  ├── H1 > "# Hello"
    ├── heading_content > "Hello"
      ├── safe_inline > "Hello"
        └── word: "Hello"
   ```

✅ **h1_no_space**: `H1`
   Input: `#NoSpace`
   Parse Tree:
   ```
  ├── H1 > "#NoSpace"
    ├── heading_content > "NoSpace"
      ├── safe_inline > "NoSpace"
        └── word: "NoSpace"
   ```

✅ **h1_multiple_spaces**: `H1`
   Input: `#   Multiple   Spaces   `
   Parse Tree:
   ```
  ├── H1 > "#   Multiple   Spaces"
    ├── heading_content > "Multiple   Spaces"
      ├── safe_inline > "Multiple"
        └── word: "Multiple"
      ├── safe_inline > "Spaces"
        └── word: "Spaces"
   ```

❌ **h1_with_formatting**: `H1` (Unexpected failure)
   Input: `# \*\*Bold\*\* and \*italic\* heading`
   Error: ` --> 1:3
  |
1 | # **Bold** and *italic* heading
  |   ^---
  |
  = expected safe_inline`

✅ **h1_unicode**: `H1`
   Input: `# Café & Résumé`
   Parse Tree:
   ```
  ├── H1 > "# Café & Résumé"
    ├── heading_content > "Café & Résumé"
      ├── safe_inline > "Café"
        └── word: "Café"
      ├── safe_inline > "&"
        └── safe_punct: "&"
      ├── safe_inline > "Résumé"
        └── word: "Résumé"
   ```

✅ **h1_numbers**: `H1`
   Input: `# Chapter 1: Introduction`
   Parse Tree:
   ```
  ├── H1 > "# Chapter 1: Introduction"
    ├── heading_content > "Chapter 1: Introduction"
      ├── safe_inline > "Chapter"
        └── word: "Chapter"
      ├── safe_inline > "1"
        └── word: "1"
      ├── safe_inline > ":"
        └── safe_punct: ":"
      ├── safe_inline > "Introduction"
        └── word: "Introduction"
   ```

✅ **h2_simple**: `H2`
   Input: `## Section`
   Parse Tree:
   ```
  ├── H2 > "## Section"
    ├── heading_content > "Section"
      ├── safe_inline > "Section"
        └── word: "Section"
   ```

✅ **h2_empty**: `H2` (Expected failure)
   Input: `##`
   Error: ` --> 1:3
  |
1 | ##
  |   ^---
  |
  = expected safe_inline`

❌ **h2_only_spaces**: `H2` (Unexpected failure)
   Input: `##   `
   Error: ` --> 1:6
  |
1 | ##   
  |      ^---
  |
  = expected safe_inline`

✅ **h2_long**: `H2`
   Input: `## This is a very long heading that should still parse correctly`
   Parse Tree:
   ```
  ├── H2 > "## This is a very long heading that should still parse correctly"
    ├── heading_content > "This is a very long heading that should still parse correctly"
      ├── safe_inline > "This"
        └── word: "This"
      ├── safe_inline > "is"
        └── word: "is"
      ├── safe_inline > "a"
        └── word: "a"
      ├── safe_inline > "very"
        └── word: "very"
      ├── safe_inline > "long"
        └── word: "long"
      ├── safe_inline > "heading"
        └── word: "heading"
      ├── safe_inline > "that"
        └── word: "that"
      ├── safe_inline > "should"
        └── word: "should"
      ├── safe_inline > "still"
        └── word: "still"
      ├── safe_inline > "parse"
        └── word: "parse"
      ├── safe_inline > "correctly"
        └── word: "correctly"
   ```

✅ **h3_simple**: `H3`
   Input: `### Subsection`
   Parse Tree:
   ```
  ├── H3 > "### Subsection"
    ├── heading_content > "Subsection"
      ├── safe_inline > "Subsection"
        └── word: "Subsection"
   ```

✅ **h4_simple**: `H4`
   Input: `#### Sub-subsection`
   Parse Tree:
   ```
  ├── H4 > "#### Sub-subsection"
    ├── heading_content > "Sub-subsection"
      ├── safe_inline > "Sub-subsection"
        └── word: "Sub-subsection"
   ```

✅ **h5_simple**: `H5`
   Input: `##### Deep Section`
   Parse Tree:
   ```
  ├── H5 > "##### Deep Section"
    ├── heading_content > "Deep Section"
      ├── safe_inline > "Deep"
        └── word: "Deep"
      ├── safe_inline > "Section"
        └── word: "Section"
   ```

✅ **h6_simple**: `H6`
   Input: `###### Deepest Section`
   Parse Tree:
   ```
  ├── H6 > "###### Deepest Section"
    ├── heading_content > "Deepest Section"
      ├── safe_inline > "Deepest"
        └── word: "Deepest"
      ├── safe_inline > "Section"
        └── word: "Section"
   ```

✅ **h7_invalid**: `heading` (Expected failure)
   Input: `####### Too Many Hashes`
   Error: ` --> 1:7
  |
1 | ####### Too Many Hashes
  |       ^---
  |
  = expected safe_inline`

✅ **h8_invalid**: `heading` (Expected failure)
   Input: `######## Even More Hashes`
   Error: ` --> 1:7
  |
1 | ######## Even More Hashes
  |       ^---
  |
  = expected safe_inline`

✅ **no_hash**: `heading` (Expected failure)
   Input: `Not a heading`
   Error: ` --> 1:14
  |
1 | Not a heading
  |              ^---
  |
  = expected safe_inline`

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

❌ **italic_unclosed**: `italic` (Unexpected failure)
   Input: `\*missing closing`
   Error: ` --> 1:1
  |
1 | *missing closing
  | ^---
  |
  = expected italic`

## inline_images

❌ **image_basic**: `inline_image` (Unexpected failure)
   Input: `!\[alt text\](image.jpg)`
   Error: ` --> 1:13
  |
1 | ![alt text](image.jpg)
  |             ^---
  |
  = expected inline_url`

✅ **image_empty_alt**: `inline_image` (Expected failure)
   Input: `!\[\](image.png)`
   Error: ` --> 1:5
  |
1 | ![](image.png)
  |     ^---
  |
  = expected inline_url`

✅ **image_with_url**: `inline_image`
   Input: `!\[remote\](https://example.com/image.png)`
   Parse Tree:
   ```
  ├── inline_image > "![remote](https://example.com/image.png)"
    └── inline_link_text: "remote"
    └── inline_url: "https://example.com/image.png"
   ```

❌ **image_complex_alt**: `inline_image` (Unexpected failure)
   Input: `!\[A very detailed alt text\](image.jpg)`
   Error: ` --> 1:29
  |
1 | ![A very detailed alt text](image.jpg)
  |                             ^---
  |
  = expected inline_url`

✅ **image_no_extension**: `inline_image` (Expected failure)
   Input: `!\[alt\](not\_an\_image)`
   Error: ` --> 1:8
  |
1 | ![alt](not_an_image)
  |        ^---
  |
  = expected inline_url`

❌ **image_unclosed**: `inline_image` (Unexpected failure)
   Input: `!\[alt\](image.jpg`
   Error: ` --> 1:8
  |
1 | ![alt](image.jpg
  |        ^---
  |
  = expected inline_url`

## commonmark_block_quotes

❌ **cm_example_228**: `text` (Unexpected failure)
   Input: `> # Foo
> bar
> baz
`
   Error: ` --> 1:1
  |
1 | > # Foo
  | ^---
  |
  = expected text`

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

❌ **cm_example_230**: `text` (Unexpected failure)
   Input: `   > # Foo
   > bar
 > baz
`
   Error: ` --> 1:1
  |
1 |    > # Foo
  | ^---
  |
  = expected text`

❌ **cm_example_231**: `text` (Unexpected failure)
   Input: `    > # Foo
    > bar
    > baz
`
   Error: ` --> 1:1
  |
1 |     > # Foo
  | ^---
  |
  = expected text`

❌ **cm_example_232**: `text` (Unexpected failure)
   Input: `> # Foo
> bar
baz
`
   Error: ` --> 1:1
  |
1 | > # Foo
  | ^---
  |
  = expected text`

❌ **cm_example_233**: `text` (Unexpected failure)
   Input: `> bar
baz
> foo
`
   Error: ` --> 1:1
  |
1 | > bar
  | ^---
  |
  = expected text`

❌ **cm_example_234**: `text` (Unexpected failure)
   Input: `> foo
---
`
   Error: ` --> 1:1
  |
1 | > foo
  | ^---
  |
  = expected text`

❌ **cm_example_235**: `text` (Unexpected failure)
   Input: `> - foo
- bar
`
   Error: ` --> 1:1
  |
1 | > - foo
  | ^---
  |
  = expected text`

❌ **cm_example_236**: `text` (Unexpected failure)
   Input: `>     foo
    bar
`
   Error: ` --> 1:1
  |
1 | >     foo
  | ^---
  |
  = expected text`

❌ **cm_example_237**: `text` (Unexpected failure)
   Input: `> \`\`\`
foo
\`\`\`
`
   Error: ` --> 1:1
  |
1 | > ```
  | ^---
  |
  = expected text`

❌ **cm_example_238**: `text` (Unexpected failure)
   Input: `> foo
    - bar
`
   Error: ` --> 1:1
  |
1 | > foo
  | ^---
  |
  = expected text`

❌ **cm_example_239**: `text` (Unexpected failure)
   Input: `>
`
   Error: ` --> 1:1
  |
1 | >
  | ^---
  |
  = expected text`

❌ **cm_example_240**: `text` (Unexpected failure)
   Input: `>
>  
> 
`
   Error: ` --> 1:1
  |
1 | >
  | ^---
  |
  = expected text`

❌ **cm_example_241**: `text` (Unexpected failure)
   Input: `>
> foo
>  
`
   Error: ` --> 1:1
  |
1 | >
  | ^---
  |
  = expected text`

❌ **cm_example_242**: `text` (Unexpected failure)
   Input: `> foo

> bar
`
   Error: ` --> 1:1
  |
1 | > foo
  | ^---
  |
  = expected text`

❌ **cm_example_243**: `text` (Unexpected failure)
   Input: `> foo
> bar
`
   Error: ` --> 1:1
  |
1 | > foo
  | ^---
  |
  = expected text`

❌ **cm_example_244**: `text` (Unexpected failure)
   Input: `> foo
>
> bar
`
   Error: ` --> 1:1
  |
1 | > foo
  | ^---
  |
  = expected text`

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

❌ **cm_example_246**: `text` (Unexpected failure)
   Input: `> aaa
\*\*\*
> bbb
`
   Error: ` --> 1:1
  |
1 | > aaa
  | ^---
  |
  = expected text`

❌ **cm_example_247**: `text` (Unexpected failure)
   Input: `> bar
baz
`
   Error: ` --> 1:1
  |
1 | > bar
  | ^---
  |
  = expected text`

❌ **cm_example_248**: `text` (Unexpected failure)
   Input: `> bar

baz
`
   Error: ` --> 1:1
  |
1 | > bar
  | ^---
  |
  = expected text`

❌ **cm_example_249**: `text` (Unexpected failure)
   Input: `> bar
>
baz
`
   Error: ` --> 1:1
  |
1 | > bar
  | ^---
  |
  = expected text`

❌ **cm_example_250**: `text` (Unexpected failure)
   Input: `> > > foo
bar
`
   Error: ` --> 1:1
  |
1 | > > > foo
  | ^---
  |
  = expected text`

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

❌ **cm_example_252**: `text` (Unexpected failure)
   Input: `>     code

>    not code
`
   Error: ` --> 1:1
  |
1 | >     code
  | ^---
  |
  = expected text`

## commonmark_autolinks

❌ **cm_example_593**: `text` (Unexpected failure)
   Input: `<http://foo.bar.baz>
`
   Error: ` --> 1:1
  |
1 | <http://foo.bar.baz>
  | ^---
  |
  = expected text`

❌ **cm_example_594**: `text` (Unexpected failure)
   Input: `<http://foo.bar.baz/test?q=hello&id=22&boolean>
`
   Error: ` --> 1:1
  |
1 | <http://foo.bar.baz/test?q=hello&id=22&boolean>
  | ^---
  |
  = expected text`

❌ **cm_example_595**: `text` (Unexpected failure)
   Input: `<irc://foo.bar:2233/baz>
`
   Error: ` --> 1:1
  |
1 | <irc://foo.bar:2233/baz>
  | ^---
  |
  = expected text`

❌ **cm_example_596**: `text` (Unexpected failure)
   Input: `<MAILTO:FOO@BAR.BAZ>
`
   Error: ` --> 1:1
  |
1 | <MAILTO:FOO@BAR.BAZ>
  | ^---
  |
  = expected text`

❌ **cm_example_597**: `text` (Unexpected failure)
   Input: `<a+b+c:d>
`
   Error: ` --> 1:1
  |
1 | <a+b+c:d>
  | ^---
  |
  = expected text`

❌ **cm_example_598**: `text` (Unexpected failure)
   Input: `<made-up-scheme://foo,bar>
`
   Error: ` --> 1:1
  |
1 | <made-up-scheme://foo,bar>
  | ^---
  |
  = expected text`

❌ **cm_example_599**: `text` (Unexpected failure)
   Input: `<http://../>
`
   Error: ` --> 1:1
  |
1 | <http://../>
  | ^---
  |
  = expected text`

❌ **cm_example_600**: `text` (Unexpected failure)
   Input: `<localhost:5001/foo>
`
   Error: ` --> 1:1
  |
1 | <localhost:5001/foo>
  | ^---
  |
  = expected text`

❌ **cm_example_601**: `text` (Unexpected failure)
   Input: `<http://foo.bar/baz bim>
`
   Error: ` --> 1:1
  |
1 | <http://foo.bar/baz bim>
  | ^---
  |
  = expected text`

❌ **cm_example_602**: `text` (Unexpected failure)
   Input: `<http://example.com/\\\[\\>
`
   Error: ` --> 1:1
  |
1 | <http://example.com/\\[\\>
  | ^---
  |
  = expected text`

❌ **cm_example_603**: `text` (Unexpected failure)
   Input: `<foo@bar.example.com>
`
   Error: ` --> 1:1
  |
1 | <foo@bar.example.com>
  | ^---
  |
  = expected text`

❌ **cm_example_604**: `text` (Unexpected failure)
   Input: `<foo+special@Bar.baz-bar0.com>
`
   Error: ` --> 1:1
  |
1 | <foo+special@Bar.baz-bar0.com>
  | ^---
  |
  = expected text`

❌ **cm_example_605**: `text` (Unexpected failure)
   Input: `<foo\\+@bar.example.com>
`
   Error: ` --> 1:1
  |
1 | <foo\\+@bar.example.com>
  | ^---
  |
  = expected text`

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

❌ **cm_example_608**: `text` (Unexpected failure)
   Input: `<m:abc>
`
   Error: ` --> 1:1
  |
1 | <m:abc>
  | ^---
  |
  = expected text`

❌ **cm_example_609**: `text` (Unexpected failure)
   Input: `<foo.bar.baz>
`
   Error: ` --> 1:1
  |
1 | <foo.bar.baz>
  | ^---
  |
  = expected text`

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
   Input: `<a href=\"url\">link</a>`
   Parse Tree:
   ```
  └── inline_html: "<a href=\"url\">"
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
   Input: `<div class=\"container\">
<p>Paragraph</p>
</div>`
   Parse Tree:
   ```
  └── inline_html: "<div class=\"container\">"
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

## Summary

- **Total tests**: 1225
- **Passed**: 592 ✅
- **Failed**: 633 ❌
- **Success rate**: 48.3%
