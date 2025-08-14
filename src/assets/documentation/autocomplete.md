# Auto-Close Function Using AST + Syntax Mapping

## Overview

The auto-close feature uses the **AST** and **syntax mapping** to detect when the user types an opening Markdown syntax (e.g., `*`, `**`, `` ` ``, `[`), and automatically inserts the corresponding closing syntax. It is aware of the **active line**, **current node**, and **Markdown variant**.

---

## Workflow

1. **User Input Detection**

   * On every keypress, detect if the character matches a `markdown_syntax` from your syntax mapping.
   * Example:

     ```rust
     let typed_char = '*';
     let node_type = syntax_mapping.get_node_by_syntax(typed_char);
     ```

2. **Active-Line AST Context**

   * Identify the AST node corresponding to the current cursor line:

     ```rust
     let active_node = ast.find_node_at_line(cursor_line);
     ```
   * If no closing syntax exists yet for this node, auto-close is triggered.

3. **Insert Closing Syntax**

   * Automatically insert the closing Markdown symbol at the cursor position:

     ```rust
     if let Some(syntax) = node_type.markdown_syntax {
         insert_text_at_cursor(syntax.clone());
         move_cursor_inside_closure(); // keep cursor between opening and closing
     }
     ```

4. **Update AST and Footer**

   * After auto-closing, update the AST:

     ```rust
     ast.update_node(cursor_line, active_node);
     ```
   * Refresh the active-line footer:

     ```
     Node: Strong
     Syntax: *
     Content: Bold text
     ```

---

## Example Scenarios

### Bold Text

* **User types:** `*Hello`
* **Auto-close inserts:** `*Hello*`
* **Footer shows:**

```
Node: Strong
Syntax: *
Content: Hello
```

### Inline Code

* **User types:** `` `fn main()` ``
* **Auto-close inserts:** `` `fn main()` ``
* **Footer shows:**

```
Node: InlineCode
Syntax: `
Content: fn main()
```

### Link

* **User types:** `[OpenAI](`
* **Auto-close inserts:** `[OpenAI](|)` (cursor inside parentheses)
* **Footer shows:**

```
Node: Link
Syntax: [text](url)
Content: OpenAI
```

---

## Benefits

* **Context-aware**: Uses AST to handle nested elements (`**_bold & italic_**`).
* **Variant-specific**: Works with CommonMark, GFM, Pandoc, etc.
* **Consistent**: Auto-closed syntax matches the syntax mapping rules.
* **Dynamic Footer**: Shows live node type, syntax, and content for the active line.

---

## Tips for Implementation

1. **Keep AST and Syntax Mapping Separate**

   * `ast.ron` defines the structure.
   * `syntax.ron` defines Markdown symbols.

2. **Always Parse on Keypress**

   * Update the AST to ensure proper nesting and correct node identification.

3. **Handle Cursor Position**

   * After auto-insertion, place the cursor between the opening and closing syntax.

4. **Support Multiple Variants**

   * Load variant-specific AST + syntax mapping files for GFM, Pandoc, etc.