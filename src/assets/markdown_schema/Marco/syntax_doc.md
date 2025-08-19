# Marco Markdown Syntax Schema

This document describes each syntax entry in the Marco markdown schema, including its purpose and example usage.

| Entry Name         | Node Type      | Markdown Syntax / Pattern                | Description                                                                 | Tested |
|--------------------|---------------|------------------------------------------|-----------------------------------------------------------------------------|:------:|
| Heading1–6 | heading | #, ##, ###, ####, #####, ###### | ATX headings, depth 1–6 | ❌ |
| Paragraph | paragraph | (empty string) | Standard paragraph | ❌ |
| Strong | strong | ** | Bold text | ❌ |
| Emphasis | emphasis | * | Italic text | ❌ |
| BlockQuote | blockquote | > | Blockquote | ❌ |
| ListUnordered | list | - | Unordered list | ❌ |
| ListOrdered | list | 1. | Ordered list | ❌ |
| ListItem | listItem | - | List item | ❌ |
| CodeBlock | codeBlock | ``` | Fenced code block | ❌ |
| InlineCode | inlineCode | ` | Inline code | ❌ |
| HorizontalRule | thematicBreak | --- | Horizontal rule | ❌ |
| Link | link | [text](url) | Markdown link | ❌ |
| Image | image | ![alt](url) | Markdown image | ❌ |
| Strikethrough | delete | ~~ | Strikethrough text | ✓ |
| TaskList | list | - [ ] / - [x] | Task list item (GFM) | ✓ |
| Table | table |  | col | ✓ |
| Footnote | footnote | [^1] | Footnote reference | ✓ |
| Autolink | link | <https://example.com> | Autolinked URL | ✓ |
| Emoji | emoji | :smile: | Emoji shortcode | ❌ |
| Mention | mention | @username | User mention | ❌ |
| MathInline | mathInline | $inline$ | Inline math | ❌ |
| MathBlock | mathBlock | $$block$$ | Block math | ❌ |
| HTMLIns | htmlInline | <ins>...</ins> | Inline HTML ins tag | ❌ |
| HTMLNbsp | htmlInline | &nbsp; | Non-breaking space entity | ❌ |
| HTMLCenter | htmlBlock | <center>...</center> | Centered HTML block | ❌ |
| HTMLPStyle | htmlBlock | <p style=...> | Paragraph with style attribute | ❌ |
| HTMLFont | htmlInline | <font color=...> | Font color HTML tag | ❌ |
| CommentHack | comment | [comment]: # | Markdown comment hack | ❌ |
| Admonition | admonition | > :warning: | Admonition block (info/warning) | ❌ |
| ImageSize | htmlBlock | re:<img\\s+[^>]*width=["']?\d+["']?[^>]*> | HTML image tag with width attribute (regex) | ❌ |
| FigureCaption | figure | re:<figure[\\s\\S]*?<figcaption[\\s\\S]*?>[\\s\\S]*?</figcaption> | HTML figure with caption (regex) | ❌ |
| LinkTarget | link | re:<a\\s+[^>]*href=\\\"[^\\\"]+\\\"[^>]*target=\\\"[^\\\"]+\\\"[^>]*> | HTML anchor with target attribute (regex) | ❌ |
| SymbolEntity | symbol | &copy; | &reg; | ❌ |
| TOCPlaceholder | toc | #### Table of Contents | Table of contents placeholder | ❌ |
| VideoEmbed | video | re:\[!\[.*?\]\(https?://img\\.youtube\\.com/vi/[A-Za-z0-9_-]+/0\\.jpg\)\]\(https?://(www\\.)?youtube\\.com/watch\?v=[A-Za-z0-9_-]+.*?\) | YouTube video embed (regex) | ❌ |

| SetextUnderline | setext_underline | re:^(?:=+ | -+)\s*$ | ❌ |
| LinkDefinition | link_def | re:^\s*\[(?P<id>[^\]]+)\]:\s*(?P<url>\S+)(?:\s+"(?P<title>.+?)")? | Reference-style link definition (e.g. `[id]: http://example "Title"`) | ❌ |
| LinkReference | link_reference | re:\[(?P<text>.*?)\]\[(?P<id>[^\]]*)\] | Reference-style link usage (`[text][id]` or `[text][]`) | ❌ |
| LinkShortcut | link_shortcut | re:\[(?P<text>[^\]]+)\]\s*\[\] | Shortcut/collapsed reference link (`[text][]`) | ❌ |
| HardBreak | hardbreak | re:\s{2}$ | Hard line break (two trailing spaces) | ❌ |
| HTMLBr | hardbreak | re:^<br\s*/?>\s*$ | Explicit HTML `<br>` element producing a hard break | ❌ |
| IndentedCode | codeBlock | re:^(?: {4} | \t).+ | ❌ |
| FrontMatterStart | frontmatter_start | re:^---\s*$ | YAML frontmatter start/end markers (delimited by `---`) | ❌ |
| FrontMatterEnd | frontmatter_end | re:^---\s*$ | YAML frontmatter end marker | ❌ |
| DefListTerm | def_term | re:^(?P<term>[^\n].+)\n(?=:\s) | Definition list term (extension: `Term\n: definition`) | ❌ |
| DefListDef | def_description | re:^:\s+(?P<desc>.+) | Definition list description line | ❌ |

## Entry Descriptions

- **Heading1–6**: ATX-style headings, depth 1–6 (`#`, `##`, etc.).
- **Paragraph**: Standard text block.
- **Strong**: Bold text (`**bold**`).
- **Emphasis**: Italic text (`*italic*`).
- **BlockQuote**: Quoted block (`> quote`).
- **ListUnordered**: Unordered list (`- item`).
- **ListOrdered**: Ordered list (`1. item`).
- **ListItem**: List item marker.
- **CodeBlock**: Fenced code block (triple backticks).
- **InlineCode**: Inline code (single backtick).
- **HorizontalRule**: Horizontal rule (`---`).
- **Link**: Markdown link (`[text](url)`).
- **Image**: Markdown image (`![alt](url)`).
- **Strikethrough**: Strikethrough text (`~~strike~~`).
- **TaskList**: Task list item (`- [ ]` or `- [x]`).
- **Table**: Table row (`| col |`).
- **Footnote**: Footnote reference (`[^1]`).
- **Autolink**: Autolinked URL (`<https://...>`).
- **Emoji**: Emoji shortcode (`:smile:`).
- **Mention**: User mention (`@username`).
- **MathInline**: Inline math (`$x^2$`).
- **MathBlock**: Block math (`$$x^2$$`).
- **HTMLIns**: Inline HTML ins tag.
- **HTMLNbsp**: Non-breaking space entity.
- **HTMLCenter**: Centered HTML block.
- **HTMLPStyle**: Paragraph with style attribute.
- **HTMLFont**: Font color HTML tag.
- **CommentHack**: Markdown comment hack.
- **Admonition**: Admonition block (e.g., warning).
- **ImageSize**: HTML image tag with width attribute (regex).
- **FigureCaption**: HTML figure with caption (regex).
- **LinkTarget**: HTML anchor with target attribute (regex).
- **SymbolEntity**: HTML symbol entities.
- **TOCPlaceholder**: Table of contents placeholder.
- **VideoEmbed**: YouTube video embed (regex).
