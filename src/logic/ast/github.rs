use anyhow::Error;

/// Type alias for AST results with anyhow error handling.
pub type AstResult<T> = Result<T, Error>;

/// Example: minimal error-producing function for demonstration.
pub fn parse_table_row_safe(is_valid: bool) -> AstResult<TableRow> {
    if !is_valid {
        Err(Error::msg("Invalid table row"))
    } else {
        Ok(TableRow { cells: vec![] })
    }
}
impl Table {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_table(self);
    }
}

impl TableRow {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_table_row(self);
    }
}

impl TableCell {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_table_cell(self);
    }
}

impl TableAlignment {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_table_alignment(self);
    }
}

impl TaskListItem {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_task_list_item(self);
    }
}
/// Trait for visiting AST nodes in github.rs
pub trait AstVisitor {
    fn visit_table(&mut self, table: &Table) {
        self.walk_table(table);
    }

    fn walk_table(&mut self, table: &Table) {
        self.visit_table_row(&table.header);
        for row in &table.rows {
            self.visit_table_row(row);
        }
    }

    fn visit_table_row(&mut self, row: &TableRow) {
        self.walk_table_row(row);
    }

    fn walk_table_row(&mut self, row: &TableRow) {
        for cell in &row.cells {
            self.visit_table_cell(cell);
        }
    }

    fn visit_table_cell(&mut self, cell: &TableCell) {
        self.walk_table_cell(cell);
    }

    fn walk_table_cell(&mut self, _cell: &TableCell) {
        // Traverse inlines if needed (handled in inlines visitor)
    }

    fn visit_table_alignment(&mut self, _alignment: &TableAlignment) {
        // No children to traverse
    }

    fn visit_task_list_item(&mut self, item: &TaskListItem) {
        self.walk_task_list_item(item);
    }

    fn walk_task_list_item(&mut self, _item: &TaskListItem) {
        // Traverse blocks (handled in blocks visitor)
    }
}
use crate::logic::ast::blocks_and_inlines::Block;

// -----------------------------------------------------------------------------
// 4.10 Tables (GFM Extension)
//
// GFM enables the table extension (section 4.10), which adds a table block type. A table consists of a header row, a delimiter row (which determines column alignment), and zero or more data rows. Each row is made up of cells containing inline content. Block-level elements are not allowed inside tables.
//
// Spec reference: https://github.github.com/gfm/#tables-extension-
// -----------------------------------------------------------------------------

/// Represents the alignment of a table column, as determined by the delimiter row (GFM 4.10).
///
/// - None: No alignment specified (default, left-aligned in HTML)
/// - Left: Delimiter starts with ':' (e.g., ":---")
/// - Center: Delimiter starts and ends with ':' (e.g., ":---:")
/// - Right: Delimiter ends with ':' (e.g., "---:")
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableAlignment {
    None,
    Left,
    Center,
    Right,
}

/// Represents a single cell in a table row (GFM 4.10).
/// Each cell contains inline content (parsed as a sequence of inlines).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell {
    /// The inline content of the cell (e.g., text, emphasis, code, etc.).
    pub content: Vec<(crate::logic::ast::inlines::Inline, crate::logic::core::event_types::SourcePos)>,
}

/// Represents a row in a table (either header or data row) (GFM 4.10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    /// The cells in this row, in column order.
    pub cells: Vec<TableCell>,
}

/// Represents a GFM table block (GFM 4.10).
/// A table consists of a header row, a vector of column alignments, and zero or more data rows.
/// The header row and delimiter row must have the same number of cells; data rows may have fewer (missing cells are treated as empty) or more (excess are ignored).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// The header row (first row in the table).
    pub header: TableRow,
    /// The alignment for each column, as determined by the delimiter row.
    pub alignments: Vec<TableAlignment>,
    /// The data rows (zero or more rows following the header).
    pub rows: Vec<TableRow>,
}

// -----------------------------------------------------------------------------
// 5.3 Task List Items (GFM Extension)
//
// GFM enables the tasklist extension (section 5.3), which adds task list items. A task list item is a list item whose first block is a paragraph beginning with a task list item marker (e.g., [ ] or [x]). The marker indicates whether the item is checked or unchecked. Task list items can be arbitrarily nested.
//
// Spec reference: https://github.github.com/gfm/#task-list-items-extension-
// -----------------------------------------------------------------------------

/// Represents the marker for a GFM task list item (GFM 5.3).
/// The marker is always of the form [ ] (unchecked) or [x]/[X] (checked), possibly with leading spaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskListMarker {
    /// Whether the task is checked (true for [x] or [X], false for [ ]).
    pub checked: bool,
}

/// Represents a GFM task list item (GFM 5.3).
/// This is a special kind of list item whose first block is a paragraph beginning with a task list marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskListItem {
    /// The marker indicating checked/unchecked state.
    pub marker: TaskListMarker,
    /// The content of the task list item (inline content after the marker, plus any nested blocks).
    pub content: Vec<Block>,
}

// -----------------------------------------------------------------------------
// 6.5 Strikethrough (GFM Extension)
//
// GFM enables the strikethrough extension (section 6.5), which adds strikethrough as an emphasis type. Strikethrough text is any text wrapped in a matching pair of one or two tildes (~). Three or more tildes do not create a strikethrough.
//
// 6.9 Autolinks (GFM Extension) and 6.11 Disallowed Raw HTML (GFM Extension) are also included as inline variants below.
//
// Spec reference: https://github.github.com/gfm/#strikethrough-extension-
// -----------------------------------------------------------------------------

/// Represents the kind of autolink recognized by GFM (section 6.9, autolink extension).
///
/// All URL autolinks (including www-prefixed and explicit http URLs) are represented as HTTPS links.
/// This enforces the user requirement: only use HTTPS, never HTTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutolinkKind {
    /// An HTTPS URL autolink (must start with "https://" or is a www-prefixed domain, or explicit http URLs coerced to https).
    ///
    /// Examples (GFM 6.9):
    ///   - https://example.com
    ///   - https://www.example.com
    ///   - https://example.com/path?query
    HttpsUrl(String),
    /// An email address autolink (mailto: is implied).
    /// Example (GFM 6.9): foo@bar.baz
    Email(String),
    /// An XMPP protocol autolink (must start with "xmpp:").
    /// Example (GFM 6.9): xmpp:foo@bar.baz
    Xmpp(String),
}

/// Represents an inline element (text, emphasis, code, strikethrough, autolink, raw HTML, etc.)
/// (CommonMark/GFM inline AST, including GFM 6.5, 6.9, 6.11 extensions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// Plain text content.
    Text(String),
    /// Strikethrough content, delimited by one or two tildes (~) (GFM 6.5).
    /// Contains a vector of child inlines (with source positions) that are struck through.
    /// Example: ~~Hi~~ or ~there~
    Strikethrough(Vec<(Inline, crate::logic::core::event_types::SourcePos)>),
    /// Autolink content (HTTPS URLs, emails, or XMPP links only) (GFM 6.9).
    /// All URLs (including www-prefixed and explicit http URLs) are coerced to HTTPS.
    /// Example: Autolink(AutolinkKind::HttpsUrl("https://www.example.com".to_string()))
    Autolink(AutolinkKind),
    /// Raw HTML content (inline or block) (GFM 6.11).
    /// If the tag is disallowed (see DisallowedHtmlTag), the renderer should filter it by replacing
    /// the leading '<' with '&lt;'. All other HTML tags are left untouched.
    /// Example: RawHtml { tag: "<script>", disallowed: Some(DisallowedHtmlTag::Script) }
    RawHtml {
        /// The raw HTML tag or content.
        tag: String,
        /// If Some, this tag is disallowed and should be filtered; otherwise, it is allowed.
        disallowed: Option<DisallowedHtmlTag>,
    },
    // ... other inline variants (emphasis, code, links, etc.)
}

// -----------------------------------------------------------------------------
// 6.11 Disallowed Raw HTML (GFM Extension)
//
// GFM enables the tagfilter extension (section 6.11), which filters the following HTML tags when rendering HTML output:
// <title>, <textarea>, <style>, <xmp>, <iframe>, <noembed>, <noframes>, <script>, <plaintext>.
// Filtering is done by replacing the leading '<' with '&lt;'. These tags are chosen because they change how HTML is interpreted in a way unique to them.
// All other HTML tags are left untouched.
//
// Spec reference: https://github.github.com/gfm/#disallowed-raw-html-extension-
// -----------------------------------------------------------------------------

/// Represents a disallowed HTML tag as per the GFM tagfilter extension (section 6.11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisallowedHtmlTag {
    Title,
    Textarea,
    Style,
    Xmp,
    Iframe,
    Noembed,
    Noframes,
    Script,
    Plaintext,
}
