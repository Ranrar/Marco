Logical Framework for AST Validation and Display Hints

Step 1: Logic for AST vs Syntax validation

### 1. Basic principle

* `syntax.ron` describes **allowed node types** and their rules (e.g. whether they are block, inline, whether they can have children, which children, etc.).
* `ast.ron` is a **concrete tree**, where nodes occur in a certain structure.
* Validation is: *“Does each node in `ast` match the rules that its type has in `syntax`?”*

---

### 2. Each step in the validation

For each node in the AST:

1. **Find the node type in the syntax definition**

* Look up the node type in `syntax.ron`.
* If the type is not found in syntax → error.

2. **Check parent rules**

* See which parent node has in AST.
* Check in `syntax` whether this parent type can contain this node type.
* If not → error.

3. **Check children rules**

* Look at the node type in `syntax`.
* If the type says “may only have inline-children” → check that all children in the AST are inline types.
* If the type says “may not have children” → check that the list of children in the AST is empty.
* If the type says “may only have block-children” → check the same.

4. **Check order rules**

* Some types have requirements for placement (e.g. frontmatter first, or link definitions before paragraphs).
* Traverse siblings and check that the order matches the requirements of the syntax definition.

5. **Check attributes/metadata**

* If a node has extra fields (e.g. id, url, longcode), check that they are valid according to the syntax rules (e.g. correct format, mandatory field filled).

---

### 3. Example in logic

If we have:

* Syntax says:

* **Document** may have children of type *Block*.
* **Frontmatter** is Block, but only valid if it is the first child in Document.
* **Paragraph** is Block, and may only have Inline children.
* **Text** is Inline, and may only be a child of Block types.

* AST says:

* Document

* Paragraph

* Text("hello")
* Frontmatter

Validation logic gives:

* Document → ok (may have Blocks as children).
* First child is Paragraph → ok (may be in Document).
* Paragraph has a Text → ok (may have Inline).
* Second child is Frontmatter → error (syntax says that Frontmatter can only be first).

---

### 4. Output of validation

The validation gives a **list of errors and warnings**, e.g.:

* `Frontmatter node must be first child of Document`
* `LinkDef node not allowed inside Paragraph`
* `Image node missing required "src" attribute`

---

### 5. Advantage of this logical approach

* You don't need any hardcoded rules in the program code.
* The whole logic is: *AST asks syntax if it matches*.
* New node types are only added in `syntax.ron` → the validation still works.

Step 2: Logic to populate display_hints.ron
* **`syntax.ron`** → describes rules and types (what the language can do).
* **`ast.ron`** → is a concrete document.
* **`display_hints.ron`** → should be a “display guide” or presentation model that we can derive from the first two.

So we need to define **pure logic** for how to build `display_hints` automatically from `ast` + `syntax`.

---

## 🔎 Logic to populate `display_hints.ron`

### 1. Basic principle

* `ast` gives us **what is actually in the document**.
* `syntax` gives us **how each node should be understood and displayed**.
* `display_hints` is therefore a **projection**: *For each node in ast, look in syntax to find the appropriate presentation hint*.

---

### 2. Step-by-step logic

1. **Traverse the AST**

* Start from the root (`Document`).
* For each node → look up the node type in `syntax`.

2. **Lookup in syntax**

* Syntax can tell us metadata such as:

* whether a node is block or inline
* whether it has a visual marker (e.g. `#` for heading, `>` for blockquote, `-` for list)
* how its children should be displayed (indentation, separators, etc.).

3. **Build display-hint entry**

* For each node in the AST, a corresponding “hint” is constructed:

* **type** → the name of the node (e.g. `Paragraph`, `Heading`).
* **style** → retrieved from syntax (e.g. “block”, “inline”).
* **position** → possibly calculated from order in the AST (e.g. “first node”, “after frontmatter”).
* **decoration** → things from syntax (e.g. symbols, quotes, bullets).
* **children\_hints** → generated recursively from the node's children.

4. **Rules for inline vs block**

* If the node is block-type → create hints about paragraph/list block/heading.
* If the node is inline-type → create hints about text, link, emphasis, etc.

5. **Combine to display\_hints.ron**

* When the entire tree is traversed, we have a new structure parallel to the AST, but with a focus on **how it is displayed** rather than **what it is**.
* This structure is `display_hints.ron`.

---

### 3. Example in pure logic

Assume we have:

**AST**:

```
Document
├─ Frontmatter
├─ Heading(level=1, "Title")
└─ Paragraph
└─ Text("Hello world")
```

**Syntax** says:

* Frontmatter → not displayed (metadata only).
* Heading(level=1) → block, decorate with “# ” in front of the text.
* Paragraph → block, children are displayed inline.
* Text → inline, displayed as plain text.

**Display hints become logical**:

* Document → container.

* Heading → block, style: heading1, decoration: `# `.
* Paragraph → block, style: paragraph.

* Text → inline, style: plain.

---

### 4. Errors and omissions

During derivation, you can detect mismatches:

* If the AST contains a node that the syntax does not know → error.
* If the syntax requires something (e.g. heading must have level) but the AST lacks → error.

---

### 5. Final result

`display_hints.ron` is therefore **AST + Syntax = Display Model**, where each node in the AST has been assigned presentation information from the syntax.