# LSP Syntax Highlighting Refactoring Plan

**Line/Column Positioning Approach**

Date: October 25, 2025 22:40
Status: Planning Phase
Target: Parser and LSP integration refactoring for live Markdown highlighting

---

## Overview

**Goal:** Refactor the parser and LSP integration to use line/column positioning instead of absolute byte offsets, eliminating coordinate system mismatch bugs and ensuring robust live highlighting.

**Key Concept:**

* Parser already tracks line and column positions via nom_locate.
* Highlights are defined as spans with start/end positions and a tag type (e.g., Emphasis, Strong, Link, CodeBlock).
* Byte offsets are kept for debugging/logging but are not used for GTK positioning.

## Current Issues

1. Parser produces byte offsets relative to the file.
2. GTK TextBuffer uses character offsets.
3. Current conversion is buggy and fragile, causing highlight shifts.
4. Multi-byte UTF-8 characters, emojis, and combining characters produce cumulative offset errors.

**Outcome:** Highlights may appear at incorrect positions during live editing.

## Proposed Architecture

**New Flow:**

1. Parser produces AST with spans and tags.
2. Each span uses line/column coordinates instead of absolute offsets.
3. Conversion to GTK TextIter is done **per-line**, converting parser byte column to character offset for the line.
4. Highlights are applied to the TextBuffer using tags corresponding to each token type.
5. The editor updates highlights live while typing using a debounced parser in a background worker.

**Advantages:**

* Robust: avoids cumulative offset errors.
* UTF-8 safe: multi-byte characters and emojis handled correctly.
* Efficient: only affected lines are processed, reducing overhead.
* Debuggable: line-based conversion is easy to inspect.
* Natural fit: parser already tracks line/column positions.

## Core Parser Changes

* **Position Struct:** line/column tracked; byte offset used only for logging.
* **Span Verification:** ensure start/end lines and columns match expected positions, especially for multi-line content and UTF-8 characters.
* **Audit Parser Modules:** verify all inline and block parsers propagate span information correctly.
* **Testing:** unit tests for single-line, multi-line, UTF-8, and emoji scenarios.

## LSP Integration Changes

* Implement line-based conversion: parser line/column → GTK TextIter.
* Span conversion handles start and end positions independently.
* Remove old absolute offset conversion logic.
* Ensure tags are correctly applied to nested highlights.
* Background parser communicates highlight data to the GTK main thread safely.

## Live Highlighting Flow

1. User edits the buffer.
2. Changes trigger a debounced parser update in the background.
3. Parser produces highlights (spans + tags).
4. Highlights are sent to the main GTK thread.
5. Positions are converted line-by-line to TextIter ranges.
6. Tags are applied to TextBuffer.
7. SourceView5 renders highlights in real-time.

**Loop:** This happens continuously for live editing, only updating affected lines.

## Edge Cases

* Empty lines: byte column 1 → char column 0.
* Line endings: end positions should stay within the line.
* Document boundaries: first and last lines handled carefully.
* Multi-byte characters: parser must not produce positions inside a multi-byte or zero-width character.
* Combining accents and emojis: ensure parser produces positions at valid character boundaries.

## Testing Strategy

* **Unit Tests:** ASCII, UTF-8, emoji, multi-line content.
* **Integration Tests:** simulate live editing, verify tags update correctly.
* **Manual Tests:** a set of test files covering ASCII, UTF-8, emoji, and multi-line content.
* **Stress Tests:** large documents, random content, and simulated rapid edits.

## Performance Considerations

* Line-based conversion reduces time complexity from O(n) to O(m) per line.
* Space complexity is limited to the line being processed.
* Optional optimizations: line caching, batch line processing, lazy evaluation.
* Expected improvement: faster highlighting for medium and large documents with no visible lag.

## Implementation Steps

1. Preparation: feature branch, backup, set up test files.
2. Parser Verification: audit spans, add tests, ensure UTF-8 correctness.
3. LSP Refactor: implement line/column conversion, simplify span conversion, remove old logic.
4. Testing: unit, integration, manual.
5. Cleanup and Documentation: remove unused code, update comments.
6. Verification: test with real-world and large documents, profile performance.

## Future Enhancements

* Incremental highlighting: re-parse only changed lines.
* Highlight caching: reuse previous line conversions.
* Async highlighting: background parsing to avoid UI blocking.
* Error recovery: handle invalid parser positions gracefully.
* Performance metrics: log parse and highlight times for optimization.

## Success Criteria

* Highlights appear at correct positions.
* Works with ASCII, UTF-8, emojis, and multi-line content.
* No shifting of highlights during live editing.
* Performance is smooth for large documents.
* Nested highlights render correctly.
* Parser and editor code is easier to maintain and debug.

---

## **1. color highlighting**

* Parser provides `Highlight` spans with start/end positions and a `HighlightTag` type (Emphasis, Strong, Link, etc.).
* For each type, you only need a **foreground color**.
* No bold, italic, underline, or font changes—just color.

## **2. Setup: Color Tags**

* Create a `GtkSourceTag` for each `HighlightTag` type.
  Example mapping:

  * Emphasis → light blue
  * Strong → green
  * Link → cyan
  * CodeBlock → gray
* You can store these in a **tag dictionary**: `tag_map[HighlightTag] → GtkSourceTag`.
* These tags only define the color property: `foreground = "#HEXCOLOR"`.

## **3. Conversion: Parser Span → TextIter**

1. Take parser `Span` with start/end line & column.
2. Convert start line/column → `GtkTextIter`.

   * Lines: 1-based → 0-based.
   * Columns: byte offset → character offset per line.
3. Convert end line/column → `GtkTextIter` similarly.
4. Clamp iterators to valid buffer positions.

**Note:** Do **per-line conversion** to handle multi-byte characters and emojis correctly.

## **4. Applying Color Highlights**

1. For each `Highlight`:

   * Retrieve the `GtkSourceTag` corresponding to its `HighlightTag`.
   * Apply tag to buffer range: `buffer.apply_tag(tag, start_iter, end_iter)`.
2. Before applying new highlights, clear color tags on affected lines to avoid leftover colors.
3. Only update lines that changed for efficiency.

## **5. Live Updates**

* Run parser in **debounced background** thread while typing.
* Send highlights to main GTK thread.
* Apply new highlights using color tags only on modified lines.
* Avoid unnecessary redraws for unchanged lines.

## **6. Edge Cases**

* Empty lines → byte column 1 maps to char column 0.
* Multi-line highlights → apply color to each line separately.
* Multi-byte characters → ensure parser spans align with valid character boundaries.
* Document start/end → clamp iterators to buffer bounds.

## **7. Performance Considerations**

* Cache **line conversions** from byte → char offsets.
* Reuse color tags to avoid creating them repeatedly.
* Batch updates if multiple lines are highlighted in the same operation.

### **Summary**

* Only foreground color is applied via tags.
* Per-line byte → char conversion ensures UTF-8 and emoji safety.
* Live highlighting works without affecting font weight, style, or other visual properties.
* Efficient for large files using caching and incremental updates.


---

## **Marker-Based Highlighting**

1. **Syntax Markers (punctuation)**

   * Example: `**` in bold, `*` in italic, `#` in heading
   * **Color**: Use full VS Code token color (bright/primary color)
   * **Effect**: Makes the structure obvious, like VS Code

2. **Content (inner text)**

   * Example: `bold` in `**bold**`
   * **Color**: Slightly **lighter/darker shade** of the marker color
   * **Effect**: Still visible, less emphasis than markers, keeps visual hierarchy

3. **Benefits**

   * Visual clarity of Markdown structure
   * Readers see both syntax and content, but markers pop out
   * Works well for headings, emphasis, links, code, etc.


## **Example Mapping Table**

| Highlight Type        | Example / Tokens    | Marker Color (Primary) | Content Color (Shade) | Notes                                               |
| --------------------- | ------------------- | ---------------------- | --------------------- | --------------------------------------------------- |
| **Heading**           | `# Heading`         | Blue                   | Light Blue            | Marker `#` bright, text lighter shade               |
| **Emphasis (italic)** | `*italic*`          | Light Blue             | Pale Blue             | Only `*` is bright, text is softer                  |
| **Strong (bold)**     | `**bold**`          | Green                  | Pale Green            | Only `**` bright, text slightly dimmed              |
| **Strong + Emphasis** | `***bold italic***` | Teal                   | Light Teal            | Marker brighter than text                           |
| **Code Span**         | `` `inline` ``      | Gray                   | Light Gray            | Backticks bright, code slightly dim                 |
| **Code Block**        | ` ``` `             | Dark Gray              | Gray                  | Markers ` ``` ` bright, block text slightly lighter |
| **Blockquote**        | `> quote`           | Purple                 | Lavender              | `>` bright, quote text shaded                       |
| **List Item**         | `- item`            | Orange                 | Light Orange          | Marker `-` bright, text dimmed                      |
| **Link Text**         | `[text](url)`       | Cyan                   | Light Cyan            | `[ ]` bright, text shaded                           |
| **Link URL**          | `[text](url)`       | Light Cyan             | Pale Cyan             | `( )` bright, URL text dimmed                       |
| **Strikethrough**     | `~~text~~`          | Dark Red               | Light Red             | `~~` bright, content shaded                         |

## **Implementation Notes for SourceView5**

1. **Create two GtkSourceTags per token**

   * **Marker tag** → primary color
   * **Content tag** → shaded version

2. **Parsing**

   * For inline nodes like emphasis or strong:

     * Track positions of **opening and closing markers**
     * Track positions of **inner content** separately

3. **Apply Tags**

   * Apply marker tag to marker positions
   * Apply content tag to inner text positions

4. **Shading Strategy**

   * Use **25-40% lighter/darker** than marker color
   * Maintain readability and VS Code-style visual hierarchy

5. **UTF-8 Safety**

   * Convert parser byte offsets → character offsets per line
   * Apply tags per line for efficiency

6. **Optional Enhancements**

   * Nested emphasis: blend shades for content
   * Inline code inside emphasis: code color overrides text shade
   * Live editing: update affected lines only