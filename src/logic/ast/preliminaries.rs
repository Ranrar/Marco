/// Trait for visiting AST nodes in preliminaries.rs
pub trait AstVisitor {
    fn visit_character(&mut self, _character: &Character) {}
    fn visit_line(&mut self, line: &Line) {
        self.walk_line(line);
    }
    fn walk_line(&mut self, line: &Line) {
        for character in &line.chars {
            self.visit_character(character);
        }
        if let Some(ending) = &line.ending {
            self.visit_line_ending(ending);
        }
    }
    fn visit_line_ending(&mut self, _ending: &LineEnding) {}
    fn visit_blank_line(&mut self, blank: &BlankLine) {
        self.walk_blank_line(blank);
    }
    fn walk_blank_line(&mut self, blank: &BlankLine) {
        self.visit_line(&blank.line);
    }
    fn visit_character_class(&mut self, _class: &CharacterClass) {}
    fn visit_tab_expansion(&mut self, _tab: &TabExpansion) {}
    fn visit_block_structure_line(&mut self, block_line: &BlockStructureLine) {
        self.walk_block_structure_line(block_line);
    }
    fn walk_block_structure_line(&mut self, block_line: &BlockStructureLine) {
        for item in &block_line.chars {
            self.visit_character_or_tab_expansion(item);
        }
        if let Some(ending) = &block_line.ending {
            self.visit_line_ending(ending);
        }
    }
    fn visit_character_or_tab_expansion(&mut self, item: &CharacterOrTabExpansion) {
        match item {
            CharacterOrTabExpansion::Character(c) => self.visit_character(c),
            CharacterOrTabExpansion::TabExpansion(t) => self.visit_tab_expansion(t),
        }
    }
    fn visit_insecure_character_replacement(&mut self, _rep: &InsecureCharacterReplacement) {}
    fn visit_ast_character(&mut self, ast_char: &AstCharacter) {
        match ast_char {
            AstCharacter::Normal(c) => self.visit_character(c),
            AstCharacter::InsecureReplacement(rep) => self.visit_insecure_character_replacement(rep),
        }
    }
    fn visit_backslash_escape(&mut self, _escape: &BackslashEscape) {}
    fn visit_entity_or_char_ref(&mut self, _entity: &EntityOrCharRef) {}
}
// ============================================================================
use anyhow::Error;

/// Type alias for AST results with anyhow error handling.
pub type AstResult<T> = Result<T, Error>;

/// Example: minimal error-producing function for demonstration.
pub fn parse_character_safe(c: char) -> AstResult<Character> {
    if c == '\u{0000}' {
        Err(Error::msg("NULL character is not allowed"))
    } else {
        Ok(Character { codepoint: c })
    }
}
// CommonMark Spec Version 0.31.2 -- Preliminaries AST (Sections 2.1–2.5)
//
// This module defines the Abstract Syntax Tree (AST) node types for the
// "Preliminaries" chapter of the CommonMark specification (version 0.31.2):
//   2.1 Characters and lines
//   2.2 Tabs
//   2.3 Insecure characters
//   2.4 Backslash escapes
//   2.5 Entity and numeric character references
//
// Each struct/enum below is annotated with comments explaining its role and
// mapping to the relevant section of the spec. This AST is intended for use in
// a Markdown parser or analyzer that needs to reason about the low-level
// character and line structure of CommonMark documents.
//
// Node mapping:
//   - Character, Line, LineEnding, BlankLine, CharacterClass: Section 2.1
//   - TabExpansion, BlockStructureLine, CharacterOrTabExpansion: Section 2.2
//   - InsecureCharacterReplacement, AstCharacter: Section 2.3
//   - BackslashEscape: Section 2.4
//   - EntityOrCharRef: Section 2.5
//
// All nodes are thoroughly documented for clarity and future extension.
// --------------------------------------------------------------------------
// Visitor Pattern Usage (preliminaries.rs)
// --------------------------------------------------------------------------
// This module implements the visitor pattern for all AST node types, enabling
// double dispatch and recursive traversal. Each node type provides an `accept`
// method that takes a mutable reference to an `AstVisitor` trait object.
//
// To implement custom traversal or analysis, define a struct and implement
// `AstVisitor` for it. Override the relevant visit methods and use the provided
// walk methods for recursive traversal.
//
// Example:
//
// struct MyVisitor;
// impl AstVisitor for MyVisitor {
//     fn visit_character(&mut self, character: &Character) {
//         println!("Char: {}", character.codepoint);
//     }
//     fn visit_line(&mut self, line: &Line) {
//         self.walk_line(line); // Recursively visit children
//     }
// }
//
// let line = Line { chars: vec![Character { codepoint: 'a' }], ending: None };
// let mut visitor = MyVisitor;
// line.accept(&mut visitor);
//
// See the DebugPrinter and test_debug_printer_traversal for a full example.
// ============================================================================

// -----------------------------
// 2.1 Characters
// -----------------------------
/// Represents a Unicode code point (character) in a CommonMark document.
/// All Unicode code points are considered characters for the spec, including combining marks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Character {
    /// The Unicode code point value.
    pub codepoint: char,
}
impl Character {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_character(self);
    }
}

/// Represents a line in a CommonMark document.
/// A line is a sequence of zero or more characters (not including line endings),
/// followed by a line ending or end of file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// The characters in the line (excluding the line ending).
    pub chars: Vec<Character>,
    /// The type of line ending, or None if this is the last line in the file.
    pub ending: Option<LineEnding>,
}
impl Line {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_line(self);
    }
}
/// Represents the different types of line endings recognized by CommonMark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineEnding {
    /// Line feed (U+000A)
    LineFeed,
    /// Carriage return (U+000D) not followed by a line feed
    CarriageReturn,
    /// Carriage return followed by a line feed (CRLF)
    CarriageReturnLineFeed,
}
impl LineEnding {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_line_ending(self);
    }
}

/// Represents a blank line (a line containing no characters, or only spaces/tabs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlankLine {
    /// The original line (for position info, if needed)
    pub line: Line,
}
impl BlankLine {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_blank_line(self);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterClass {
    /// Unicode whitespace character (Zs), or tab, line feed, form feed, carriage return
    UnicodeWhitespace,
    /// Tab character (U+0009)
    Tab,
    /// Space character (U+0020)
    Space,
    /// ASCII control character (U+0000–U+001F, U+007F)
    AsciiControl,
    /// ASCII punctuation character (see spec for full set)
    AsciiPunctuation,
    /// Unicode punctuation or symbol (general categories P or S)
    UnicodePunctuationOrSymbol,
    /// Any other character
    Other,
}
impl CharacterClass {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_character_class(self);
    }
}

// -----------------------------
// 2.2 Tabs
// -----------------------------
/// AST nodes for tab handling in block structure contexts (section 2.2).
/// In CommonMark, tabs are not expanded to spaces in general, but in block structure contexts
/// (such as indentation for code blocks or list items), a tab is treated as advancing to the next tab stop (every 4 columns).

/// Represents the logical expansion of a tab character in block structure contexts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabExpansion {
    /// The position (column) where the tab occurs in the line.
    pub column: usize,
    /// The number of virtual spaces this tab represents (1 to 4).
    pub spaces: usize,
}
impl TabExpansion {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_tab_expansion(self);
    }
}


/// Represents a line after tab expansion for block structure analysis.
/// Used only for block structure parsing, not for literal text content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStructureLine {
    pub chars: Vec<CharacterOrTabExpansion>,
    pub ending: Option<LineEnding>,
}
impl BlockStructureLine {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_block_structure_line(self);
    }
}

/// Either a literal character or a virtual tab expansion (for block structure parsing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterOrTabExpansion {
    Character(Character),
    TabExpansion(TabExpansion),
}
impl CharacterOrTabExpansion {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_character_or_tab_expansion(self);
    }
}

/// Utility function to classify a character according to the CommonMark spec's character classes.
pub fn classify_character(c: char) -> CharacterClass {
    match c {
        '\u{0009}' => CharacterClass::Tab,
        '\u{0020}' => CharacterClass::Space,
        '\u{0000}'..='\u{001F}' | '\u{007F}' => CharacterClass::AsciiControl,
        '!'..='/' | ':'..='@' | '['..='`' | '{'..='~' => CharacterClass::AsciiPunctuation,
        _ if c.is_whitespace() => CharacterClass::UnicodeWhitespace,
        _ if unicode_categories::is_punctuation_or_symbol(c) => CharacterClass::UnicodePunctuationOrSymbol,
        _ => CharacterClass::Other,
    }
}

// -----------------------------
// 2.3 Insecure characters
// -----------------------------
/// Represents a character that was replaced for security reasons (U+0000 → U+FFFD).
/// In CommonMark, the Unicode character U+0000 (NULL) must be replaced with the REPLACEMENT CHARACTER (U+FFFD).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsecureCharacterReplacement {
    /// The original code point (should always be '\u{0000}').
    pub original: char,
    /// The replacement character (always '\u{FFFD}').
    pub replacement: char,
}
impl InsecureCharacterReplacement {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_insecure_character_replacement(self);
    }
}
/// AST node representing a character in the document, which may be a normal character or a replacement for an insecure character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstCharacter {
    /// A normal Unicode character.
    Normal(Character),
    /// A replacement for an insecure character (U+0000 → U+FFFD).
    InsecureReplacement(InsecureCharacterReplacement),
}
impl AstCharacter {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_ast_character(self);
    }
}

// -----------------------------
// 2.4 Backslash escapes
// -----------------------------
/// Represents a backslash escape in the source text (section 2.4).
/// Any ASCII punctuation character may be backslash-escaped, and the backslash is removed in parsing.
/// Backslashes before other characters are treated as literal backslashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackslashEscape {
    /// A backslash-escaped ASCII punctuation character (e.g., `\*` → `*`).
    EscapedPunctuation {
        /// The punctuation character that was escaped.
        punctuation: char,
    },
    /// A backslash before a non-punctuation character, treated as a literal backslash.
    LiteralBackslash {
        /// The character following the backslash.
        following: char,
    },
}
impl BackslashEscape {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_backslash_escape(self);
    }
}

// -----------------------------
// 2.5 Entity and numeric character references
// -----------------------------
/// Represents an entity or numeric character reference (section 2.5).
/// Entity references are &name; (from the HTML5 entity set),
/// numeric references are either decimal (e.g., &#123;) or hexadecimal (e.g., &#x1F4A9;).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityOrCharRef {
    /// A named HTML5 entity reference (e.g., &amp; &copy; &AElig;).
    NamedEntity {
        /// The entity name (without the leading & and trailing ;)
        name: String,
        /// The code point this entity resolves to (if known)
        codepoint: Option<char>,
    },
    /// A decimal numeric character reference (e.g., &#123;)
    DecimalNumeric {
        /// The numeric value (as written)
        value: u32,
        /// The code point this resolves to (if valid)
        codepoint: Option<char>,
    },
    /// A hexadecimal numeric character reference (e.g., &#x1F4A9;)
    HexNumeric {
        /// The numeric value (as written)
        value: u32,
        /// The code point this resolves to (if valid)
        codepoint: Option<char>,
    },
}
impl EntityOrCharRef {
    pub fn accept<V: AstVisitor>(&self, visitor: &mut V) {
        visitor.visit_entity_or_char_ref(self);
    }
}

/// Helper module for Unicode general category checks (P, S)
mod unicode_categories {
    use unicode_general_category::get_general_category;
    use unicode_general_category::GeneralCategory;

    pub fn is_punctuation_or_symbol(c: char) -> bool {
        matches!(get_general_category(c),
            GeneralCategory::ConnectorPunctuation |
            GeneralCategory::DashPunctuation |
            GeneralCategory::OpenPunctuation |
            GeneralCategory::ClosePunctuation |
            GeneralCategory::InitialPunctuation |
            GeneralCategory::FinalPunctuation |
            GeneralCategory::OtherPunctuation |
            GeneralCategory::MathSymbol |
            GeneralCategory::CurrencySymbol |
            GeneralCategory::ModifierSymbol |
            GeneralCategory::OtherSymbol)
    }
}

// --------------------------------------------------------------------------
// Sample Visitor: DebugPrinter
// --------------------------------------------------------------------------
pub struct DebugPrinter;

impl AstVisitor for DebugPrinter {
    fn visit_character(&mut self, character: &Character) {
        println!("Character: {:?}", character.codepoint);
    }
    fn visit_line(&mut self, line: &Line) {
        println!("Line: chars={} ending={:?}", line.chars.len(), line.ending);
        self.walk_line(line);
    }
    fn visit_line_ending(&mut self, ending: &LineEnding) {
        println!("LineEnding: {:?}", ending);
    }
    fn visit_blank_line(&mut self, blank: &BlankLine) {
        println!("BlankLine");
        self.walk_blank_line(blank);
    }
    fn visit_character_class(&mut self, class: &CharacterClass) {
        println!("CharacterClass: {:?}", class);
    }
    fn visit_tab_expansion(&mut self, tab: &TabExpansion) {
        println!("TabExpansion: column={} spaces={}", tab.column, tab.spaces);
    }
    fn visit_block_structure_line(&mut self, block_line: &BlockStructureLine) {
        println!("BlockStructureLine: chars={} ending={:?}", block_line.chars.len(), block_line.ending);
        self.walk_block_structure_line(block_line);
    }
    fn visit_character_or_tab_expansion(&mut self, item: &CharacterOrTabExpansion) {
        match item {
            CharacterOrTabExpansion::Character(c) => {
                println!("CharacterOrTabExpansion: Character");
                c.accept(self);
            }
            CharacterOrTabExpansion::TabExpansion(t) => {
                println!("CharacterOrTabExpansion: TabExpansion");
                t.accept(self);
            }
        }
    }
    fn visit_insecure_character_replacement(&mut self, rep: &InsecureCharacterReplacement) {
        println!("InsecureCharacterReplacement: original={:?} replacement={:?}", rep.original, rep.replacement);
    }
    fn visit_ast_character(&mut self, ast_char: &AstCharacter) {
        match ast_char {
            AstCharacter::Normal(c) => {
                println!("AstCharacter: Normal");
                c.accept(self);
            }
            AstCharacter::InsecureReplacement(rep) => {
                println!("AstCharacter: InsecureReplacement");
                rep.accept(self);
            }
        }
    }
    fn visit_backslash_escape(&mut self, escape: &BackslashEscape) {
        println!("BackslashEscape: {:?}", escape);
    }
    fn visit_entity_or_char_ref(&mut self, entity: &EntityOrCharRef) {
        println!("EntityOrCharRef: {:?}", entity);
    }
}

// --------------------------------------------------------------------------
// Test: Traversal with DebugPrinter
// --------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_safe() {
        let result = super::parse_character_safe('a');
        assert!(result.is_ok());
        let result = super::parse_character_safe('\u{0000}');
        assert!(result.is_err());
    }

    #[test]
    fn test_line_traversal() {
        let c = Character { codepoint: 'x' };
        let line = Line {
            chars: vec![c],
            ending: Some(LineEnding::LineFeed),
        };
        struct Printer;
        impl AstVisitor for Printer {
            fn visit_line(&mut self, line: &Line) {
                self.walk_line(line);
            }
            fn visit_character(&mut self, _character: &Character) {
                assert!(true);
            }
            fn visit_line_ending(&mut self, _ending: &LineEnding) {
                assert!(true);
            }
        }
        let mut printer = Printer;
        printer.visit_line(&line);
    }

    #[test]
    fn test_debug_printer_traversal() {
        let char_a = Character { codepoint: 'a' };
        let char_b = Character { codepoint: 'b' };
        let line = Line {
            chars: vec![char_a.clone(), char_b.clone()],
            ending: Some(LineEnding::LineFeed),
        };
        let blank = BlankLine { line: line.clone() };
        let tab = TabExpansion { column: 4, spaces: 2 };
        let block_line = BlockStructureLine {
            chars: vec![CharacterOrTabExpansion::Character(char_a.clone()), CharacterOrTabExpansion::TabExpansion(tab.clone())],
            ending: Some(LineEnding::CarriageReturn),
        };
        let insecure = InsecureCharacterReplacement { original: '\u{0000}', replacement: '\u{FFFD}' };
        let ast_char = AstCharacter::InsecureReplacement(insecure);
        let entity = EntityOrCharRef::NamedEntity { name: "amp".to_string(), codepoint: Some('&') };
        let backslash = BackslashEscape::EscapedPunctuation { punctuation: '*' };

        let mut visitor = DebugPrinter;
        // Test traversal for each node type
        char_a.accept(&mut visitor);
        line.accept(&mut visitor);
        blank.accept(&mut visitor);
        tab.accept(&mut visitor);
        block_line.accept(&mut visitor);
        ast_char.accept(&mut visitor);
        entity.accept(&mut visitor);
        backslash.accept(&mut visitor);
    }
}