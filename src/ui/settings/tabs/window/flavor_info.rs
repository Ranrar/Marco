// Info HTML for the Markdown Variant Selection dialog
pub const FLAVOR_INFO_HTML: &str = r#"
<html>
<head>
<style>
h1, h2, h3 {
  color: #2a4a7b;
}
html, body {
  font-family: system-ui, sans-serif;
  color: #222;
  background: transparent !important;
  margin: 0;
  padding: 1.5em;
  font-size: 1.05em;
}
ul, ol {
  margin-left: 1.2em;
}
table {
  border-collapse: collapse;
  margin: 1em 0;
  width: 100%;
  background: #fafbfc;
}
th, td {
  border: 1px solid #d0d7de;
  padding: 0.5em 0.8em;
  text-align: center;
}
th {
  background: #eaeef2;
  font-weight: 600;
}
tr:nth-child(even) td {
  background: #f6f8fa;
}
code {
  background: #f3f3f3;
  border-radius: 3px;
  padding: 0.1em 0.3em;
  font-size: 0.98em;
}
.variant {
  font-weight: bold;
  color: #2a4a7b;
}
</style>
</head>
<body>
<h1>Markdown Variant Selection</h1>
<p>You can enable one or more Markdown variants. Only compatible variants can be enabled together.</p>
<ul>
  <li>Enabling <b>Marco</b> enables <b>all</b> variants.</li>
  <li>Disabling <b>Marco</b> restores your previous selection.</li>
  <li>At least one variant must always be selected.</li>
  <li>Disabling the last remaining variant has no effect.</li>
  <li>Enabling a variant filters out incompatible ones.</li>
  <li>Disabling a variant may allow others to reappear.</li>
  <li><b>Marco</b> acts as a "Select All" toggle.</li>
</ul>
<hr />
<h2>Compatibility Matrix</h2>
<table>
  <tr><th>Variant</th><th>Can be toggled with...</th></tr>
  <tr><td class="variant">CommonMark</td><td>GFM, Markdig, Marco</td></tr>
  <tr><td class="variant">GFM</td><td>CommonMark, Markdig, Marco</td></tr>
  <tr><td class="variant">Pandoc</td><td>Obsidian, Typora, Markdown Extra, Marco</td></tr>
  <tr><td class="variant">Obsidian</td><td>Pandoc, Typora, Markdown Extra, Marco</td></tr>
  <tr><td class="variant">Typora</td><td>Pandoc, Obsidian, Markdown Extra, Markdig, Marco</td></tr>
  <tr><td class="variant">Markdown Extra</td><td>Pandoc, Obsidian, Typora, Marco</td></tr>
  <tr><td class="variant">Markdig</td><td>CommonMark, GFM, Typora, Marco</td></tr>
  <tr><td class="variant">Marco</td><td><b>All variants</b></td></tr>
</table>
</table>
<h2>Feature Support Table</h2>
<table>
  <tr>
    <th>Feature</th><th>CommonMark</th><th>GFM</th><th>Marco</th><th>Obsidian</th><th>Pandoc</th><th>Typora</th><th>Markdown Extra</th><th>Markdig</th>
  </tr>
  <tr><td><b>Tables</b></td>                           <td>❌</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Task Lists</b></td>                       <td>❌</td><td>✅</td><td>✅</td><td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Strikethrough</b></td>                    <td>❌</td><td>✅</td><td>✅</td><td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Frontmatter (YAML)</b></td>               <td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Footnotes</b></td>                        <td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Wiki Links (<code>[[Page]]</code>)</b></td> <td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>❌</td><td>❌</td><td>❌</td><td>❌</td></tr>
  <tr><td><b>Math / LaTeX (<code>$x^2$</code>, <code>$$</code>)</b></td> <td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td></tr>
  <tr><td><b>Auto-links</b></td>                       <td>✅</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Attribute Lists (<code>{#id .class}</code>)</b></td> <td>❌</td><td>❌</td><td>✅</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Definition Lists</b></td>                 <td>❌</td><td>❌</td><td>✅</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Abbreviations</b></td>                    <td>❌</td><td>❌</td><td>✅</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Highlight (<code>==text==</code>)</b></td> <td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td></tr>
  <tr><td><b>Inline HTML</b></td>                      <td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>TOC (auto gen headings)</b></td>          <td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Underline (<code>_text_</code>)</b></td>  <td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Superscript (<code>^text</code>)</b></td> <td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Subscript (<code>~text</code>)</b></td>   <td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Callouts / Admonitions</b></td>           <td>❌</td><td>❌</td><td>🧩</td><td>✅</td><td>🧩</td><td>✅</td><td>❌</td><td>🧩</td></tr>
  <tr><td><b>Diagrams (Mermaid, Graphviz)</b></td>     <td>❌</td><td>❌</td><td>🧩</td><td>✅</td><td>✅</td><td>✅</td><td>❌</td><td>🧩</td></tr>
  <tr><td><b>Emoji Shortcodes (<code>:smile:</code>)</b></td> <td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td></tr>
  <tr><td><b>Inline Comments</b></td>                  <td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td></tr>
  <tr><td><b>Custom Containers</b></td>                <td>❌</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td></tr>
  <tr><td><b>Line Breaks on Return</b></td>            <td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>Escaped Pipes in Tables</b></td>          <td>❌</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td></tr>
  <tr><td><b>Smart Punctuation</b></td>                <td>❌</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>HTML Entity Expansion</b></td>            <td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td><td>✅</td></tr>
  <tr><td><b>HTML Sanitization</b></td>                <td>❌</td><td>Partial</td><td>✅</td><td>❌</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td></tr>
  <tr><td><b>@include(file.md)</b></td>                <td>❌</td><td>❌</td><td>🧩</td><td>❌</td><td>🧩</td><td>❌</td><td>❌</td><td>🧩</td></tr>
  <tr><td><b>@toc (Auto TOC)</b></td>                  <td>❌</td><td>❌</td><td>🧩</td><td>🧩</td><td>🧩</td><td>🧩</td><td>❌</td><td>🧩</td></tr>
  <tr><td><b>@lint (markdown spellcheck)</b></td>      <td>❌</td><td>❌</td><td>🧩</td><td>❌</td><td>❌</td><td>🧩</td><td>❌</td><td>❌</td></tr>
  <tr><td><b>@mail (mailto: + subject)</b></td>        <td>❌</td><td>❌</td><td>🧩</td><td>❌</td><td>❌</td><td>❌</td><td>❌</td><td>❌</td></tr>
  <tr><td><b>@if (conditional content)</b></td>        <td>❌</td><td>❌</td><td>🧩</td><td>❌</td><td>🧩</td><td>❌</td><td>❌</td><td>❌</td></tr>
  <tr><td><b>@run (terminal command)</b></td>          <td>❌</td><td>❌</td><td>🧩</td><td>❌</td><td>❌</td><td>❌</td><td>❌</td><td>❌</td></tr>
  <tr><td><b>MathJax (advanced math)</b></td>          <td>❌</td><td>❌</td><td>🧩</td><td>🧩</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td></tr>
  <tr><td><b>KaTeX (fast math rendering)</b></td>      <td>❌</td><td>❌</td><td>🧩</td><td>🧩</td><td>✅</td><td>✅</td><td>❌</td><td>✅</td></tr>
  <tr><td><b>PlantUML (UML diagrams)</b></td>          <td>❌</td><td>❌</td><td>🧩</td><td>🧩</td><td>🧩</td><td>❌</td><td>❌</td><td>🧩</td></tr>
  <tr><td><b>Graphviz/Dot Graphs</b></td>              <td>❌</td><td>❌</td><td>🧩</td><td>🧩</td><td>✅</td><td>❌</td><td>❌</td><td>🧩</td></tr>
</table>
<p style="margin-top:1em;font-size:0.98em;">
✅ = Standard Support<br />
🧩 = Extension/Plugin<br />
❌ = Not Supported
</p>

<h2>Feature Descriptions</h2>
<table>
  <tr><th>Name</th><th>Description</th></tr>
  <tr><td><b>Tables</b></td><td>Pipe-based tables with header alignment and multi-line support.</td></tr>
  <tr><td><b>Task Lists</b></td><td><code>[ ]</code>, <code>[x]</code> checkboxes for todo-style lists.</td></tr>
  <tr><td><b>Strikethrough</b></td><td>Use <code>~~text~~</code> or <code>--text--</code> for crossed-out text.</td></tr>
  <tr><td><b>Frontmatter (YAML)</b></td><td>Metadata at the top of the document using <code>---</code> and YAML syntax.</td></tr>
  <tr><td><b>Footnotes</b></td><td>Academic-style notes like <code>[^1]</code> and <code>[1]:</code> rendered at the bottom with reference links.</td></tr>
  <tr><td><b>Wiki Links</b></td><td><code>[[Page Name]]</code> wiki-style links (e.g., Obsidian format).</td></tr>
  <tr><td><b>Math / LaTeX</b></td><td>Inline (<code>$...$</code>) and block (<code>$$...$$</code>) mathematical notation.</td></tr>
  <tr><td><b>Auto-links</b></td><td>Automatically link plain URLs or emails.</td></tr>
  <tr><td><b>Attribute Lists</b></td><td>Add attributes to elements (IDs, classes) via <code>{#id .class}</code>.</td></tr>
  <tr><td><b>Definition Lists</b></td><td>Term–definition style lists.</td></tr>
  <tr><td><b>Abbreviations</b></td><td>Define <code>*[HTML]: HyperText Markup Language</code> abbreviations.</td></tr>
  <tr><td><b>Highlighting</b></td><td>Use <code>==highlight==</code> for text highlighting.</td></tr>
  <tr><td><b>Inline HTML</b></td><td>Embed raw HTML inside Markdown.</td></tr>
  <tr><td><b>TOC (@toc)</b></td><td>Auto-generate a Table of Contents from document headings.</td></tr>
  <tr><td><b>Underline</b></td><td><code>_text_</code> or HTML <code>&lt;u&gt;</code> to underline text.</td></tr>
  <tr><td><b>Superscript</b></td><td><code>x^2</code> → render <code>2</code> as superscript.</td></tr>
  <tr><td><b>Subscript</b></td><td><code>H~2~O</code> → render <code>2</code> as subscript.</td></tr>
  <tr><td><b>Callouts</b></td><td>Blocks like <code>::: info</code>, <code>::: warning</code>, or <code>&gt; [!NOTE]</code> with icons and styled borders.</td></tr>
  <tr><td><b>Diagrams</b></td><td>Render diagrams using <b>Mermaid</b>, <b>PlantUML</b>, <b>Graphviz/DOT</b> syntax.</td></tr>
  <tr><td><b>Emoji Shortcodes</b></td><td><code>:smile:</code> style emoji converted to unicode emoji.</td></tr>
  <tr><td><b>Inline Comments</b></td><td>Special syntax to hide/show developer comments.</td></tr>
  <tr><td><b>Custom Containers</b></td><td>Define custom block styles and layouts with extended syntax.</td></tr>
  <tr><td><b>Line Breaks (Hard)</b></td><td>Treat single line breaks as <code>&lt;br&gt;</code> (like GFM).</td></tr>
  <tr><td><b>Escaped Pipes</b></td><td>Use <code>\|</code> in tables to allow inline pipes without breaking layout.</td></tr>
  <tr><td><b>Smart Typography</b></td><td>Converts straight quotes and dashes to smart/curly ones.</td></tr>
  <tr><td><b>HTML Entity Support</b></td><td>Parses HTML named/decimal entities like <code>&amp;copy;</code>, <code>&amp;#169;</code>.</td></tr>
  <tr><td><b>HTML Sanitization</b></td><td>Remove unsafe HTML tags/attributes (sandboxed mode).</td></tr>
  <tr><td><b>@include(file.md)</b></td><td>Modularize documents by including external <code>.md</code> files.</td></tr>
  <tr><td><b>@toc</b></td><td>Insert Table of Contents automatically at the specified position.</td></tr>
  <tr><td><b>@lint</b></td><td>Markdown linting + spellcheck with red underlines for incorrect or broken formatting.</td></tr>
  <tr><td><b>@mail</b></td><td>Render email links with subject/body preset for one-click mail composition.</td></tr>
  <tr><td><b>@if</b></td><td>Conditional rendering for multi-language or flavor variants (e.g., <code>CommonMark</code>, <code>GFM</code>).</td></tr>
  <tr><td><b>@run</b></td><td>Execute a shell or terminal command (safe-mode sandboxed by default).</td></tr>
  <tr><td><b>MathJax</b></td><td>Advanced LaTeX math rendering engine (for complex formulas).</td></tr>
  <tr><td><b>KaTeX</b></td><td>Lightweight fast LaTeX renderer (subset of MathJax).</td></tr>
  <tr><td><b>PlantUML</b></td><td>Create sequence/class/activity diagrams via <code>@startuml</code> syntax.</td></tr>
  <tr><td><b>Graphviz/Dot</b></td><td>Generate graphs with <code>dot</code> syntax (<code>digraph G {}</code>), rendered as SVG.</td></tr>
</table>
</body>
</html>
"#;