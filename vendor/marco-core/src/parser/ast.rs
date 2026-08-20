//! AST node definitions consumed by parser, renderer, and intelligence layers.

use crate::parser::Span;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
/// Map of normalized link reference labels to `(url, optional_title)`.
pub struct ReferenceMap {
    // Key: normalized label (case-folded, whitespace collapsed), Value: (url, optional title)
    defs: HashMap<String, (String, Option<String>)>,
}

impl ReferenceMap {
    /// Create an empty reference map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a link reference definition
    pub fn insert(&mut self, label: &str, url: String, title: Option<String>) {
        let normalized = normalize_label(label);
        // CommonMark: when multiple definitions normalize to the same label,
        // the first definition takes precedence.
        self.defs.entry(normalized).or_insert((url, title));
    }

    /// Lookup a link reference by label
    pub fn get(&self, label: &str) -> Option<&(String, Option<String>)> {
        let normalized = normalize_label(label);
        self.defs.get(&normalized)
    }

    /// Check if a label exists
    pub fn contains(&self, label: &str) -> bool {
        let normalized = normalize_label(label);
        self.defs.contains_key(&normalized)
    }
}

/// Normalize label according to CommonMark spec:
/// - Apply Unicode case-folding semantics (best-effort)
/// - Collapse consecutive whitespace to single space
/// - Trim leading/trailing whitespace
fn normalize_label(label: &str) -> String {
    // Build a whitespace-collapsed string directly without allocating a Vec.
    let mut collapsed = String::with_capacity(label.len());
    let mut first = true;
    for word in label.split_whitespace() {
        if !first {
            collapsed.push(' ');
        }
        collapsed.push_str(word);
        first = false;
    }

    // NOTE:
    // Rust doesn't provide full Unicode case-folding in std. We apply
    // to_lowercase() plus the critical sharp-s expansion so labels like
    // "ẞ" and "SS" normalize identically, matching CommonMark examples.
    let mut out = String::with_capacity(collapsed.len());
    for ch in collapsed.chars() {
        for lower in ch.to_lowercase() {
            if lower == 'ß' {
                out.push('s');
                out.push('s');
            } else {
                out.push(lower);
            }
        }
    }

    out
}

#[derive(Debug, Clone, Default)]
/// Root parsed Markdown document.
pub struct Document {
    /// Top-level AST children in source order.
    pub children: Vec<Node>,
    /// Collected link reference definitions.
    pub references: ReferenceMap,
}

#[derive(Debug, Clone)]
/// Generic AST node.
pub struct Node {
    /// Semantic node kind.
    pub kind: NodeKind,
    /// Optional source span for this node.
    pub span: Option<Span>,
    /// Child nodes for hierarchical constructs.
    pub children: Vec<Node>,
}

/// Table column alignment (GFM tables extension).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableAlignment {
    /// No explicit alignment.
    #[default]
    None,
    /// Left-aligned column.
    Left,
    /// Center-aligned column.
    Center,
    /// Right-aligned column.
    Right,
}

/// GitHub-style admonitions / alerts (GFM extension).
///
/// Syntax is based on blockquotes, e.g.
///
/// `> [!NOTE]`
/// `> body...`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmonitionKind {
    /// Note admonition kind.
    Note,
    /// Tip admonition kind.
    Tip,
    /// Important admonition kind.
    Important,
    /// Warning admonition kind.
    Warning,
    /// Caution admonition kind.
    Caution,
}

/// Rendering style for admonitions.
///
/// - `Alert`: Standard GitHub-style alert coloring (NOTE/TIP/WARNING/etc).
/// - `Quote`: Quote-colored styling (neutral border/colors like regular blockquotes) while
///   keeping the admonition title layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmonitionStyle {
    /// GitHub alert style.
    Alert,
    /// Quote-like neutral style.
    Quote,
}

#[derive(Debug, Clone)]
/// All supported block and inline AST node kinds.
pub enum NodeKind {
    /// ATX or setext heading node.
    Heading {
        /// Heading level, typically in range 1..=6.
        level: u8,
        /// Plain heading text content.
        text: String,
        /// Explicit heading id, e.g. `### Title {#custom-id}`.
        ///
        /// When present, the renderer should emit it as `id="..."` on the
        /// heading element.
        id: Option<String>,
    },
    /// Paragraph container.
    Paragraph,
    /// Fenced or indented code block.
    CodeBlock {
        /// Optional language/info string.
        language: Option<String>,
        /// Raw code block contents.
        code: String,
    },
    /// Horizontal rule (`---`, `***`, `___`).
    ThematicBreak,
    /// Ordered or unordered list container.
    List {
        /// Whether this is an ordered list.
        ordered: bool,
        /// Starting number for ordered lists.
        start: Option<u32>,
        /// Whether list items are tight (no blank separators).
        tight: bool,
    },
    /// List item container.
    ListItem,

    /// Extended definition lists (Markdown Guide / Markdown Extra-style).
    ///
    /// Rendering convention:
    /// - A `DefinitionList` contains alternating `DefinitionTerm` (`<dt>`) and
    ///   `DefinitionDescription` (`<dd>`) children.
    /// - `DefinitionTerm` should contain inline children.
    /// - `DefinitionDescription` should contain block children.
    DefinitionList,
    /// Definition term (`dt`) item.
    DefinitionTerm,
    /// Definition description (`dd`) item.
    DefinitionDescription,
    /// GFM task list checkbox marker for a list item.
    ///
    /// This is emitted by the list parser when a list item begins with
    /// `[ ]` or `[x]` / `[X]`.
    ///
    /// Rendering convention:
    /// - This node is expected to appear as the first child inside a `ListItem`.
    /// - The HTML renderer will convert it into a themed checkbox icon.
    TaskCheckbox {
        /// Whether checkbox is checked.
        checked: bool,
    },
    /// Blockquote container.
    Blockquote,
    /// GitHub-style admonition / alert (GFM extension).
    ///
    /// This is created by a post-parse transformation that recognizes a special
    /// first line inside a blockquote (e.g. `[!NOTE]`) and removes that marker.
    Admonition {
        /// Admonition semantic kind.
        kind: AdmonitionKind,
        /// Optional custom title for the admonition header.
        ///
        /// Used by extended GFM-style admonitions (e.g. `> [😂 Happy Header]`).
        title: Option<String>,
        /// Optional custom icon content (typically a Unicode emoji) for the title.
        ///
        /// Rendered as text (not SVG) and must be styled by CSS.
        icon: Option<String>,
        /// Render variant.
        style: AdmonitionStyle,
    },

    /// Extended tab blocks.
    ///
    /// Syntax (container + items):
    /// ```text
    /// :::tab
    /// @tab Title
    /// Content...
    /// :::
    /// ```
    ///
    /// Children convention:
    /// - A `TabGroup` contains one or more `TabItem` children.
    /// - Each `TabItem` contains block children representing the tab panel content.
    TabGroup,
    /// A single tab item inside a tab group.
    TabItem {
        /// User-visible tab title.
        title: String,
    },

    /// Extended slide decks (Reveal.js-like syntax, rendered as a simple slideshow).
    ///
    /// Syntax:
    /// ```text
    /// @slidestart
    /// slide 1
    /// ---
    /// slide 2
    /// @slideend
    /// ```
    ///
    /// Optional timer (seconds per slide): `@slidestart:t5`.
    ///
    /// Children convention:
    /// - A `SliderDeck` contains one or more `Slide` children.
    /// - Each `Slide` contains block children representing the slide content.
    SliderDeck {
        /// Optional per-slide timer value in seconds.
        timer_seconds: Option<u32>,
    },
    /// A single slide inside a slider deck.
    Slide {
        /// True if this slide started after a vertical separator (`--`).
        ///
        /// The current viewer treats slides as a single linear sequence
        /// (left/right). This flag is preserved for future vertical navigation.
        vertical: bool,
    },
    /// GFM table (pipe table extension).
    ///
    /// Children convention:
    /// - Each child is a `TableRow`.
    /// - Each `TableRow` contains `TableCell` children.
    Table {
        /// Per-column alignments.
        alignments: Vec<TableAlignment>,
    },
    /// A single table row.
    TableRow {
        /// Whether this row is part of the table header.
        header: bool,
    },
    /// A single table cell.
    TableCell {
        /// Whether this cell is in a header row.
        header: bool,
        /// Effective alignment for this cell.
        alignment: TableAlignment,
    },
    /// Raw block-level HTML fragment.
    HtmlBlock {
        /// Raw HTML source.
        html: String,
    },

    /// GFM-style footnote definition (extension).
    ///
    /// Syntax:
    /// - `[^label]: definition text`
    /// - Continuation lines may be indented.
    ///
    /// Rendering convention:
    /// - This node should not be rendered in place.
    /// - Instead, the renderer collects referenced footnotes and emits a
    ///   footnotes section at the end of the document.
    FootnoteDefinition {
        /// Footnote label (without `[^`/`]`).
        label: String,
    },

    /// Plain text inline content.
    Text(String),
    /// Inline task checkbox marker (extension).
    ///
    /// This is emitted when a paragraph begins with a task marker like
    /// `[ ] ` / `[x] ` / `[X] `.
    ///
    /// Rendering convention:
    /// - The HTML renderer converts it into the same themed SVG checkbox icon
    ///   used for task list items.
    TaskCheckboxInline {
        /// Whether checkbox is checked.
        checked: bool,
    },
    /// Emphasis inline container.
    Emphasis,
    /// Strong emphasis inline container.
    Strong,
    /// Combined strong+emphasis, e.g. `***text***` or `___text___`.
    ///
    /// This is parsed as a single inline node to avoid leaving dangling
    /// delimiters that would otherwise be treated as plain text.
    StrongEmphasis,
    /// Strikethrough (extension), e.g. `~~text~~`.
    Strikethrough,
    /// Highlight/mark (extension), e.g. `==text==`.
    Mark,
    /// Superscript (extension), e.g. `^text^`.
    Superscript,
    /// Subscript (extension), e.g. `~text~`.
    Subscript,
    /// Inline link node.
    Link {
        /// Link destination URL.
        url: String,
        /// Optional link title.
        title: Option<String>,
    },
    /// Reference-style link placeholder (CommonMark): `[text][label]`, `[label][]`, `[label]`.
    ///
    /// These cannot be fully resolved during inline parsing because reference
    /// definitions may appear later in the document. The top-level `parse()`
    /// performs a post-processing pass that converts this into a `Link` when a
    /// matching definition exists in `Document.references`.
    ///
    /// If no matching definition is found, this should be rendered as literal
    /// bracketed text (preserving the already-parsed `children` for the first
    /// bracketed segment).
    LinkReference {
        /// Label used for reference resolution (will be normalized when looked up).
        label: String,
        /// Extra literal suffix after the first `]` (e.g. `"[]"` or `"[label]"`).
        /// Empty for shortcut reference links.
        suffix: String,
    },

    /// GFM-style footnote reference (extension), e.g. `[^label]`.
    ///
    /// Rendering convention:
    /// - If a matching `FootnoteDefinition` exists, this renders as a numbered
    ///   superscript link.
    /// - Otherwise it should fall back to literal text.
    FootnoteReference {
        /// Referenced footnote label.
        label: String,
    },
    /// Inline image node.
    Image {
        /// Image source URL.
        url: String,
        /// Image alt text.
        alt: String,
    },
    /// Inline code span.
    CodeSpan(String),
    /// Inline HTML fragment.
    InlineHtml(String),
    /// Hard line break (two spaces + newline or backslash + newline).
    HardBreak,
    /// Soft line break.
    SoftBreak,

    /// Extended user mentions.
    ///
    /// Syntax:
    /// - `@username[platform]`
    /// - `@username[platform](Display Name)`
    ///
    /// Rendering policy:
    /// - The renderer may convert this to an external profile link based on
    ///   a platform mapping table.
    PlatformMention {
        /// Platform username/handle.
        username: String,
        /// Platform key, for example `github`.
        platform: String,
        /// Optional display label override.
        display: Option<String>,
    },
    /// Inline math (LaTeX), e.g. `$E = mc^2$`.
    ///
    /// Rendering policy:
    /// - Rendered using KaTeX in inline mode.
    /// - Content is raw LaTeX source code.
    InlineMath {
        /// Raw inline LaTeX content.
        content: String,
    },

    /// Display math (LaTeX), e.g. `$$\int_0^\infty e^{-x^2} dx$$`.
    ///
    /// Rendering policy:
    /// - Rendered using KaTeX in display mode.
    /// - Content is raw LaTeX source code.
    DisplayMath {
        /// Raw display LaTeX content.
        content: String,
    },

    /// Mermaid diagram (code block with language="mermaid").
    ///
    /// Rendering policy:
    /// - Rendered using mermaid-rs-renderer to SVG.
    /// - Content is raw Mermaid diagram source code.
    ///
    /// This is created during parsing when a fenced code block has
    /// info string "mermaid".
    MermaidDiagram {
        /// Raw Mermaid diagram source.
        content: String,
    },
}

impl Document {
    /// Create an empty document.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of top-level nodes in the document.
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns `true` when the document has no top-level nodes.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::ReferenceMap;

    #[test]
    fn smoke_test_reference_map_first_definition_wins() {
        let mut refs = ReferenceMap::new();
        refs.insert("foo", "https://first.example".to_string(), None);
        refs.insert("foo", "https://second.example".to_string(), None);

        let (url, title) = refs.get("foo").expect("reference not found");
        assert_eq!(url, "https://first.example");
        assert_eq!(title, &None);
    }

    #[test]
    fn smoke_test_reference_map_casefold_sharp_s() {
        let mut refs = ReferenceMap::new();
        refs.insert("SS", "/url".to_string(), None);

        let (url, _) = refs.get("ẞ").expect("reference not found");
        assert_eq!(url, "/url");
    }

    #[test]
    fn smoke_test_reference_map_whitespace_collapse() {
        let mut refs = ReferenceMap::new();
        refs.insert("Foo\n\t  bar", "/url".to_string(), None);

        assert!(refs.contains("foo bar"));
        assert!(refs.contains("  FOO   BAR  "));
    }
}
