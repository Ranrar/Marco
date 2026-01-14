# GFM disallowed raw HTML (tagfilter) showcase

Planned feature: match GFM tagfilter behavior by escaping the opening `<` for specific tags.

Typical tag list:
- iframe
- noembed
- noframes
- plaintext
- script
- style
- title
- textarea
- xmp

## Canonical examples

<script>alert('xss')</script>
<style>body { background: red; }</style>
<iframe src="https://example.com"></iframe>
<textarea>not a form</textarea>
<title>title</title>
<xmp>raw</xmp>

## Edge cases

### Case and whitespace

<SCRIPT>alert(1)</SCRIPT>
<script >alert(1)</script>
<script	>alert(1)</script>

### Attributes

<script type="text/javascript">alert(1)</script>

### In code spans/blocks (must stay literal)

`<script>alert(1)</script>`

```html
<script>alert(1)</script>
```

### Allowed HTML (should remain as HTML)

<div class="ok">allowed?</div>
<span>inline</span>
