# Marco Extra Syntax Features Test

This document tests all the extra markdown syntax features that have been integrated into Marco.

## HTML Entities Test

Here are some common HTML entities that can be inserted via the **Format > HTML Entities** menu:

- Copyright: &copy;
- Trademark: &trade;
- Registered: &reg;
- Non-breaking space: word&nbsp;word
- Ampersand: &amp;
- Less than: &lt;
- Greater than: &gt;
- Quotes: &quot;Hello&quot;
- Apostrophe: don&apos;t

## Admonitions Test

These can be inserted via the **Insert > Admonition** menu:

> :warning: **Warning:** This is a warning admonition with an emoji.

> :information_source: **Info:** This is an informational admonition.

> :heavy_check_mark: **Success:** This is a success admonition.

> :x: **Error:** This is an error admonition.

> :bulb: **Tip:** This is a helpful tip admonition.

## GitHub-Style Admonitions

> [!NOTE]
> This is a GitHub-style note admonition.

> [!WARNING]
> This is a GitHub-style warning admonition.

> [!TIP]
> This is a GitHub-style tip admonition.

## Extra Syntax Features

### Underline
<ins>This text should be underlined</ins>

### Center Text
<center>This text should be centered</center>

### Colored Text
<p style="color:red">This text should be red</p>
<p style="color:blue">This text should be blue</p>
<font color="green">This text should be green (deprecated syntax)</font>

### Comments
[This is a comment]: # (This should not be visible in preview)
[Another comment]: # 

### Image with Size Control
<img src="https://via.placeholder.com/150" width="100" height="100" alt="Resized image">

### Video Embedding
<iframe width="560" height="315" src="https://www.youtube.com/embed/dQw4w9WgXcQ" frameborder="0" allowfullscreen></iframe>

### Table with Line Breaks
| Column 1 | Column 2 |
|----------|----------|
| Line 1<br/>Line 2 | Data |
| More<br/>lines<br/>here | More data |

### Indented Text
<div style="margin-left: 20px;">
This text is indented 20px.
</div>

<div style="margin-left: 40px;">
This text is indented 40px.
</div>

## Menu Actions Available

You can test these via the menus:

### Format Menu
- **HTML Entities...** - Opens dialog to insert common HTML entities

### Insert Menu  
- **Admonition** - Opens dialog to insert styled admonitions

### View Menu
- **Refresh Syntax Highlighting** - Refreshes the syntax highlighting
- **Clear Extra Syntax** - Clears extra syntax formatting

### Symbols Menu
- **HTML Entity** - Quick insert of copyright symbol
- Various other symbols and entities

## Testing Instructions

1. Use **Format > HTML Entities...** to insert HTML entities
2. Use **Insert > Admonition** to create styled admonitions  
3. Use **View > Refresh Syntax Highlighting** to refresh highlighting
4. Use **View > Clear Extra Syntax** to clear extra formatting
5. Test the preview functionality to see how extra syntax renders

This completes the integration of extra markdown syntax features into Marco!
