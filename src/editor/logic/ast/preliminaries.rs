// ============================================================================
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

/// Represents a blank line (a line containing no characters, or only spaces/tabs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlankLine {
    /// The original line (for position info, if needed)
    pub line: Line,
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


/// Represents a line after tab expansion for block structure analysis.
/// Used only for block structure parsing, not for literal text content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStructureLine {
    pub chars: Vec<CharacterOrTabExpansion>,
    pub ending: Option<LineEnding>,
}

/// Either a literal character or a virtual tab expansion (for block structure parsing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterOrTabExpansion {
    Character(Character),
    TabExpansion(TabExpansion),
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
/// AST node representing a character in the document, which may be a normal character or a replacement for an insecure character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstCharacter {
    /// A normal Unicode character.
    Normal(Character),
    /// A replacement for an insecure character (U+0000 → U+FFFD).
    InsecureReplacement(InsecureCharacterReplacement),
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