// Diagnostics: parse errors, broken links, etc.

use crate::parser::{Document, Node, NodeKind, Position, Span};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub span: Span,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticsProfile {
    /// Emit all diagnostics (full analysis mode).
    All,
    /// Emit only critical diagnostics (currently severity=Error).
    CriticalOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticsOptions {
    pub profile: DiagnosticsProfile,
    /// Optional cap to avoid flooding downstream consumers.
    pub max_diagnostics: Option<usize>,
}

impl DiagnosticsOptions {
    pub const fn all() -> Self {
        Self {
            profile: DiagnosticsProfile::All,
            max_diagnostics: None,
        }
    }

    pub const fn critical_only() -> Self {
        Self {
            profile: DiagnosticsProfile::CriticalOnly,
            max_diagnostics: None,
        }
    }
}

impl Default for DiagnosticsOptions {
    fn default() -> Self {
        // Product direction: prefer minimal critical diagnostics by default.
        Self::critical_only()
    }
}

/// Stable diagnostic codes for markdown intelligence.
///
/// These identifiers are intended to be stable across releases so frontend
/// components (status panes, filters, telemetry) can rely on them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    // Parse / ingestion (MD0xx)
    ParseFailure,

    // Headings (MD1xx)
    InvalidHeadingLevel,
    EmptyHeadingText,
    DuplicateHeadingId,
    HeadingTooLong,

    // Links (MD2xx)
    EmptyLinkUrl,
    UnsafeLinkProtocol,
    InsecureLinkProtocol,
    UnresolvedLinkReference,
    EmptyLinkReferenceLabel,

    // Code blocks (MD3xx)
    EmptyCodeBlock,
    MissingCodeBlockLanguage,

    // Images (MD-4xx)
    EmptyImageUrl,
    ImageMissingAltText,
    UnsafeImageProtocol,

    // Inline HTML (MD-5xx)
    InlineHtmlContainsScript,
    InlineHtmlJavascriptUrl,
    InlineHtmlUnsafeEventHandler,

    // Block HTML (MD6xx)
    HtmlBlockContainsScript,
    HtmlBlockJavascriptUrl,
    EmptyHtmlBlock,
    HtmlBlockMismatchedAngles,
    HtmlBlockUnsafeEventHandler,

    // Structural blocks (MD7xx)
    EmptyList,
    EmptyListItem,
    MalformedTaskCheckbox,
    EmptyTaskListItem,
    EmptyBlockquote,
    EmptyDefinitionList,
    EmptyDefinitionTerm,
    EmptyDefinitionDescription,
    EmptyTableCell,

    // Footnotes (MD8xx)
    MissingFootnoteDefinition,
    DuplicateFootnoteDefinition,
    UnusedFootnoteDefinition,

    // Extended blocks & rich content (MD9xx)
    EmptyTabGroup,
    EmptyTabTitle,
    DuplicateTabTitle,
    EmptyTabPanel,
    EmptySliderDeck,
    EmptySlide,
    EmptyAdmonitionBody,
    EmptyMathExpression,
    EmptyMermaidDiagram,
    EmptyAdmonitionTitle,
    UnknownAdmonitionKind,
    InvalidSliderTimer,
    EmptyPlatformMentionUsername,
    UnknownPlatformMentionPlatform,
    UnknownEmojiShortcode,
    EmptyPlatformMentionDisplayName,
}

impl DiagnosticCode {
    pub fn catalog_key(self) -> String {
        format!("{self:?}")
    }

    pub fn as_str(self) -> &'static str {
        self.catalog_entry()
            .map(|entry| entry.code.as_str())
            .unwrap_or_else(|| {
                crate::intelligence::catalog::diagnostics_catalog_settings()
                    .unknown_code_fallback
                    .as_str()
            })
    }

    /// Default user-facing diagnostic message sourced from embedded catalog metadata.
    pub fn default_message(self) -> &'static str {
        self.message_template()
    }

    /// Catalog-provided message template (or title when template is absent).
    pub fn message_template(self) -> &'static str {
        self.catalog_entry()
            .map(|entry| {
                entry
                    .message_template
                    .as_deref()
                    .unwrap_or(entry.title.as_str())
            })
            .unwrap_or_else(|| {
                crate::intelligence::catalog::diagnostics_catalog_settings()
                    .unknown_message_fallback
                    .as_str()
            })
    }

    /// Resolve the default diagnostic severity from catalog metadata.
    pub fn default_severity(self) -> DiagnosticSeverity {
        self.catalog_entry()
            .and_then(|entry| DiagnosticSeverity::from_catalog_str(&entry.default_severity))
            .unwrap_or(DiagnosticSeverity::Warning)
    }

    /// Format message template placeholders like `{protocol}` with values.
    pub fn format_message(self, pairs: &[(&str, String)]) -> String {
        let mut message = self.message_template().to_string();
        for (key, value) in pairs {
            let placeholder = format!("{{{}}}", key);
            message = message.replace(&placeholder, value);
        }
        message
    }

    /// Optional embedded catalog entry for this diagnostic code.
    pub fn catalog_entry(
        self,
    ) -> Option<&'static crate::intelligence::catalog::DiagnosticsCatalogEntry> {
        let key = self.catalog_key();
        crate::intelligence::catalog::find_catalog_entry_by_key(&key)
    }

    /// Fix suggestion sourced from the embedded diagnostics catalog.
    pub fn fix_suggestion(self) -> &'static str {
        self.catalog_entry()
            .map(|entry| entry.fix_suggestion.as_str())
            .unwrap_or_else(|| {
                crate::intelligence::catalog::diagnostics_catalog_settings()
                    .unknown_fix_suggestion_fallback
                    .as_str()
            })
    }

    /// Resolve fix suggestion as a `Cow` for UI integration.
    pub fn fix_suggestion_resolved(self) -> Cow<'static, str> {
        Cow::Borrowed(self.fix_suggestion())
    }
}

impl Diagnostic {
    /// Stable external code id from catalog (e.g. `MD103`) for this diagnostic.
    pub fn code_id(&self) -> &'static str {
        self.code.as_str()
    }

    /// Stable quick fix suggestion associated with this diagnostic code.
    pub fn fix_suggestion(&self) -> &'static str {
        self.code.fix_suggestion()
    }

    /// Optional embedded catalog entry for this diagnostic.
    pub fn catalog_entry(
        &self,
    ) -> Option<&'static crate::intelligence::catalog::DiagnosticsCatalogEntry> {
        self.code.catalog_entry()
    }

    /// Human title from embedded catalog if present.
    pub fn title_resolved(&self) -> Option<&'static str> {
        self.catalog_entry().map(|entry| entry.title.as_str())
    }

    /// Rich description from embedded catalog if present.
    pub fn description_resolved(&self) -> Option<&'static str> {
        self.catalog_entry().map(|entry| entry.description.as_str())
    }

    /// Resolve fix suggestion from embedded catalog when available,
    /// with a stable in-code fallback.
    pub fn fix_suggestion_resolved(&self) -> Cow<'static, str> {
        self.code.fix_suggestion_resolved()
    }

    /// Build a parse-failure diagnostic anchored at a specific position.
    pub fn parse_error_at(position: Position, message: impl Into<String>) -> Self {
        let span = Span {
            start: position,
            end: Position {
                line: position.line,
                column: position.column.saturating_add(1),
                offset: position.offset.saturating_add(1),
            },
        };

        Self {
            code: DiagnosticCode::ParseFailure,
            span,
            severity: DiagnosticCode::ParseFailure.default_severity(),
            message: message.into(),
        }
    }

    /// Build a parse-failure diagnostic at a safe default location (1:1).
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::parse_error_at(
            Position {
                line: 1,
                column: 1,
                offset: 0,
            },
            message,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

impl DiagnosticSeverity {
    pub fn from_catalog_str(value: &str) -> Option<Self> {
        match value {
            "Error" => Some(Self::Error),
            "Warning" => Some(Self::Warning),
            "Info" => Some(Self::Info),
            "Hint" => Some(Self::Hint),
            _ => None,
        }
    }

    fn sort_rank(self) -> u8 {
        match self {
            // Higher severity first when multiple diagnostics target the same span.
            Self::Error => 0,
            Self::Warning => 1,
            Self::Info => 2,
            Self::Hint => 3,
        }
    }
}

fn sort_and_dedup_diagnostics(diagnostics: &mut Vec<Diagnostic>) {
    diagnostics.sort_by(|a, b| {
        (
            a.span.start.offset,
            a.span.end.offset,
            a.severity.sort_rank(),
            a.code.as_str(),
            a.message.as_str(),
        )
            .cmp(&(
                b.span.start.offset,
                b.span.end.offset,
                b.severity.sort_rank(),
                b.code.as_str(),
                b.message.as_str(),
            ))
    });

    diagnostics.dedup_by(|a, b| {
        a.span == b.span && a.severity == b.severity && a.code == b.code && a.message == b.message
    });
}

fn diag(
    diagnostics: &mut Vec<Diagnostic>,
    code: DiagnosticCode,
    span: Span,
    severity: DiagnosticSeverity,
    message: impl Into<String>,
) {
    diagnostics.push(Diagnostic {
        code,
        span,
        severity,
        message: message.into(),
    });
}

fn diag_catalog(diagnostics: &mut Vec<Diagnostic>, code: DiagnosticCode, span: Span) {
    diag(
        diagnostics,
        code,
        span,
        code.default_severity(),
        code.default_message(),
    );
}

fn diag_catalog_message(
    diagnostics: &mut Vec<Diagnostic>,
    code: DiagnosticCode,
    span: Span,
    message: impl Into<String>,
) {
    diag(
        diagnostics,
        code,
        span,
        code.default_severity(),
        message.into(),
    );
}

fn has_disallowed_scheme(url_lower: &str, disallowed_schemes: &[String]) -> bool {
    let scheme = url_lower
        .split_once(':')
        .map(|(prefix, _)| prefix)
        .unwrap_or_default();

    !scheme.is_empty() && disallowed_schemes.iter().any(|item| item == scheme)
}

fn starts_with_any_prefix(url_lower: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|prefix| url_lower.starts_with(prefix))
}

fn contains_unsafe_protocol_marker(text_lower: &str, protocols: &[String]) -> bool {
    protocols
        .iter()
        .map(|scheme| format!("{}:", scheme))
        .any(|needle| text_lower.contains(&needle))
}

fn contains_any_marker(text_lower: &str, markers: &[String]) -> bool {
    markers.iter().any(|marker| text_lower.contains(marker))
}

fn contains_unsafe_event_handler_attr(text_lower: &str) -> bool {
    const EVENT_ATTRS: &[&str] = &[
        "onabort",
        "onanimationend",
        "onanimationiteration",
        "onanimationstart",
        "onauxclick",
        "onbeforeinput",
        "onbeforeunload",
        "onblur",
        "oncancel",
        "oncanplay",
        "oncanplaythrough",
        "onchange",
        "onclick",
        "onclose",
        "oncontextmenu",
        "oncopy",
        "oncuechange",
        "oncut",
        "ondblclick",
        "ondrag",
        "ondragend",
        "ondragenter",
        "ondragleave",
        "ondragover",
        "ondragstart",
        "ondrop",
        "ondurationchange",
        "onended",
        "onerror",
        "onfocus",
        "onfocusin",
        "onfocusout",
        "onformdata",
        "oninput",
        "oninvalid",
        "onkeydown",
        "onkeypress",
        "onkeyup",
        "onload",
        "onloadeddata",
        "onloadedmetadata",
        "onloadstart",
        "onmousedown",
        "onmouseenter",
        "onmouseleave",
        "onmousemove",
        "onmouseout",
        "onmouseover",
        "onmouseup",
        "onpaste",
        "onpause",
        "onplay",
        "onplaying",
        "onprogress",
        "onratechange",
        "onreset",
        "onresize",
        "onscroll",
        "onsecuritypolicyviolation",
        "onseeked",
        "onseeking",
        "onselect",
        "onslotchange",
        "onstalled",
        "onsubmit",
        "onsuspend",
        "ontimeupdate",
        "ontoggle",
        "ontransitionend",
        "onunload",
        "onvolumechange",
        "onwaiting",
        "onwheel",
    ];

    EVENT_ATTRS.iter().any(|attr| {
        text_lower.contains(&format!(" {}=", attr))
            || text_lower.contains(&format!("\n{}=", attr))
            || text_lower.contains(&format!("\t{}=", attr))
            || text_lower.contains(&format!("<{}=", attr))
    })
}

fn is_known_platform(platform_lower: &str) -> bool {
    matches!(
        platform_lower,
        "github"
            | "gitlab"
            | "codeberg"
            | "twitter"
            | "x"
            | "mastodon"
            | "bluesky"
            | "linkedin"
            | "xing"
            | "medium"
            | "dribbble"
            | "behance"
            | "reddit"
            | "discord"
            | "telegram"
            | "youtube"
            | "twitch"
    )
}

fn list_item_has_malformed_task_marker(node: &Node) -> bool {
    if node
        .children
        .iter()
        .any(|child| matches!(child.kind, NodeKind::TaskCheckbox { .. }))
    {
        return false;
    }

    let Some(first_child) = node.children.first() else {
        return false;
    };

    let candidate_text = match &first_child.kind {
        NodeKind::Text(text) => Some(text.as_str()),
        NodeKind::Paragraph => first_child
            .children
            .iter()
            .find_map(|inline| match &inline.kind {
                NodeKind::Text(text) => Some(text.as_str()),
                _ => None,
            }),
        _ => None,
    };

    let Some(text) = candidate_text else {
        return false;
    };

    let trimmed = text.trim_start();
    if !trimmed.starts_with('[') {
        return false;
    }

    let Some(close_idx) = trimmed.find(']') else {
        return false;
    };

    let marker_body = trimmed[1..close_idx].trim();
    if marker_body.is_empty() {
        // "[ ]" is valid; empty marker body after trimming means this is a valid checkbox marker.
        return false;
    }

    !matches!(marker_body, "x" | "X")
}

fn known_admonition_kind(marker_kind_upper: &str) -> bool {
    matches!(
        marker_kind_upper,
        "NOTE" | "TIP" | "IMPORTANT" | "WARNING" | "CAUTION"
    )
}

fn blockquote_has_unknown_admonition_marker(node: &Node) -> bool {
    let Some(first_block) = node.children.first() else {
        return false;
    };

    if !matches!(first_block.kind, NodeKind::Paragraph) {
        return false;
    }

    let mut raw = String::new();
    for inline in &first_block.children {
        match &inline.kind {
            NodeKind::Text(text) => raw.push_str(text),
            NodeKind::SoftBreak | NodeKind::HardBreak => break,
            _ => return false,
        }
    }

    let trimmed = raw.trim();
    if !trimmed.starts_with("[!") {
        return false;
    }

    let Some(close_idx) = trimmed.find(']') else {
        return false;
    };

    let marker = &trimmed[2..close_idx].trim();
    if marker.is_empty() {
        return false;
    }

    let marker_upper = marker.to_ascii_uppercase();
    !known_admonition_kind(&marker_upper)
}

fn known_emoji_shortcodes() -> &'static HashSet<String> {
    static SHORTCODES: OnceLock<HashSet<String>> = OnceLock::new();

    SHORTCODES.get_or_init(|| {
        crate::logic::text_completion::emoji_shortcodes_for_completion()
            .iter()
            .map(|shortcode| shortcode.to_ascii_lowercase())
            .collect::<HashSet<_>>()
    })
}

fn is_shortcode_body_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '+' || ch == '-'
}

fn strip_surrounding_shortcode_wrappers(token: &str) -> &str {
    token.trim_matches(|c: char| {
        matches!(
            c,
            ',' | '.'
                | ';'
                | '!'
                | '?'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '<'
                | '>'
                | '"'
                | '\''
                | '`'
        )
    })
}

fn shortcode_candidate_from_token(token: &str) -> Option<&str> {
    let trimmed = strip_surrounding_shortcode_wrappers(token);

    if trimmed.len() < 3 || !trimmed.starts_with(':') || !trimmed.ends_with(':') {
        return None;
    }

    let body = &trimmed[1..trimmed.len() - 1];
    if body.is_empty() || !body.chars().all(is_shortcode_body_char) {
        return None;
    }

    if !body.chars().any(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }

    // Enforce token-like boundaries inside the candidate itself.
    // This avoids odd cases like ":-name:" / ":name-:" / ":name--x:".
    if body.starts_with(['-', '_', '+']) || body.ends_with(['-', '_', '+']) {
        return None;
    }

    if body.contains("--") || body.contains("__") || body.contains("++") {
        return None;
    }

    Some(trimmed)
}

fn text_has_unknown_emoji_shortcode(text: &str) -> bool {
    let known = known_emoji_shortcodes();

    text.split_whitespace().any(|token| {
        shortcode_candidate_from_token(token)
            .map(|candidate| !known.contains(&candidate.to_ascii_lowercase()))
            .unwrap_or(false)
    })
}

// Compute diagnostics for document
pub fn compute_diagnostics(document: &Document) -> Vec<Diagnostic> {
    compute_diagnostics_with_options(document, DiagnosticsOptions::all())
}

/// Compute diagnostics using configurable policy controls.
pub fn compute_diagnostics_with_options(
    document: &Document,
    options: DiagnosticsOptions,
) -> Vec<Diagnostic> {
    log::debug!(
        "Computing diagnostics for {} nodes",
        document.children.len()
    );

    let mut diagnostics = Vec::new();

    for node in &document.children {
        collect_diagnostics(node, &mut diagnostics);
    }

    collect_document_level_diagnostics(document, &mut diagnostics);
    sort_and_dedup_diagnostics(&mut diagnostics);

    match options.profile {
        DiagnosticsProfile::All => {}
        DiagnosticsProfile::CriticalOnly => {
            diagnostics.retain(|d| matches!(d.severity, DiagnosticSeverity::Error));
        }
    }

    if let Some(max) = options.max_diagnostics {
        diagnostics.truncate(max);
    }

    log::info!("Found {} diagnostics", diagnostics.len());
    diagnostics
}

/// Compute only critical diagnostics (errors) using the default policy profile.
pub fn compute_diagnostics_critical(document: &Document) -> Vec<Diagnostic> {
    compute_diagnostics_with_options(document, DiagnosticsOptions::critical_only())
}

fn collect_document_level_diagnostics(document: &Document, diagnostics: &mut Vec<Diagnostic>) {
    // Detect duplicate explicit heading IDs (e.g. "{#id}").
    // We intentionally diagnose the second and subsequent occurrences.
    let mut seen: HashMap<String, Span> = HashMap::new();
    collect_duplicate_heading_ids(&document.children, &mut seen, diagnostics);

    collect_footnote_consistency_diagnostics(&document.children, diagnostics);
    collect_link_reference_consistency_diagnostics(
        &document.children,
        &document.references,
        diagnostics,
    );
}

fn normalize_label_for_diagnostics(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn collect_footnote_consistency_diagnostics(nodes: &[Node], diagnostics: &mut Vec<Diagnostic>) {
    let mut definitions: HashMap<String, Span> = HashMap::new();
    let mut references: Vec<(String, Span)> = Vec::new();

    collect_footnote_nodes(nodes, &mut definitions, &mut references, diagnostics);

    let mut reference_counts: HashMap<String, usize> = HashMap::new();
    for (normalized_label, span) in references {
        *reference_counts
            .entry(normalized_label.clone())
            .or_insert(0) += 1;
        if !definitions.contains_key(&normalized_label) {
            diag_catalog(diagnostics, DiagnosticCode::MissingFootnoteDefinition, span);
        }
    }

    for (label, span) in definitions {
        if !reference_counts.contains_key(&label) {
            diag_catalog(diagnostics, DiagnosticCode::UnusedFootnoteDefinition, span);
        }
    }
}

fn collect_footnote_nodes(
    nodes: &[Node],
    definitions: &mut HashMap<String, Span>,
    references: &mut Vec<(String, Span)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for node in nodes {
        if let Some(span) = node.span {
            match &node.kind {
                NodeKind::FootnoteDefinition { label } => {
                    let normalized = normalize_label_for_diagnostics(label);
                    if let std::collections::hash_map::Entry::Vacant(entry) =
                        definitions.entry(normalized)
                    {
                        entry.insert(span);
                    } else {
                        diag_catalog(
                            diagnostics,
                            DiagnosticCode::DuplicateFootnoteDefinition,
                            span,
                        );
                    }
                }
                NodeKind::FootnoteReference { label } => {
                    references.push((normalize_label_for_diagnostics(label), span));
                }
                _ => {}
            }
        }

        if !node.children.is_empty() {
            collect_footnote_nodes(&node.children, definitions, references, diagnostics);
        }
    }
}

fn node_has_meaningful_content(node: &Node) -> bool {
    match &node.kind {
        NodeKind::Text(text) => !text.trim().is_empty(),
        NodeKind::CodeSpan(code) => !code.trim().is_empty(),
        NodeKind::InlineHtml(html) => !html.trim().is_empty(),
        _ => node.children.iter().any(node_has_meaningful_content),
    }
}

fn collect_duplicate_heading_ids(
    nodes: &[Node],
    seen: &mut HashMap<String, Span>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for node in nodes {
        if let (NodeKind::Heading { id: Some(id), .. }, Some(span)) = (&node.kind, node.span) {
            let key = id.trim().to_lowercase();
            if !key.is_empty() {
                if let Some(first_span) = seen.get(&key) {
                    diag(
                        diagnostics,
                        DiagnosticCode::DuplicateHeadingId,
                        span,
                        DiagnosticCode::DuplicateHeadingId.default_severity(),
                        DiagnosticCode::DuplicateHeadingId.format_message(&[
                            ("id", id.clone()),
                            ("line", first_span.start.line.to_string()),
                        ]),
                    );
                } else {
                    seen.insert(key, span);
                }
            }
        }

        if !node.children.is_empty() {
            collect_duplicate_heading_ids(&node.children, seen, diagnostics);
        }
    }
}

// Recursively collect diagnostics from a node and its children
fn collect_diagnostics(node: &Node, diagnostics: &mut Vec<Diagnostic>) {
    if let Some(span) = &node.span {
        match &node.kind {
            NodeKind::Heading { level, text, .. } => {
                if *level > 6 {
                    diag(
                        diagnostics,
                        DiagnosticCode::InvalidHeadingLevel,
                        *span,
                        DiagnosticCode::InvalidHeadingLevel.default_severity(),
                        DiagnosticCode::InvalidHeadingLevel
                            .format_message(&[("level", level.to_string())]),
                    );
                }

                if text.trim().is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyHeadingText, *span);
                }

                // Friendly style guardrail for very long headings.
                if text.chars().count()
                    > crate::intelligence::catalog::diagnostics_catalog_settings()
                        .heading_too_long_threshold
                {
                    diag_catalog(diagnostics, DiagnosticCode::HeadingTooLong, *span);
                }
            }
            NodeKind::Link { url, .. } => {
                if url.trim().is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyLinkUrl, *span);
                }

                let lower_url = url.to_lowercase();
                let settings = crate::intelligence::catalog::diagnostics_catalog_settings();
                if has_disallowed_scheme(&lower_url, &settings.unsafe_protocols) {
                    let protocol = url
                        .split_once(':')
                        .map(|(prefix, _)| prefix)
                        .unwrap_or(settings.unknown_protocol_label.as_str())
                        .to_string();
                    diag_catalog_message(
                        diagnostics,
                        DiagnosticCode::UnsafeLinkProtocol,
                        *span,
                        DiagnosticCode::UnsafeLinkProtocol
                            .format_message(&[("protocol", protocol)]),
                    );
                }

                if starts_with_any_prefix(&lower_url, &settings.insecure_link_prefixes) {
                    diag_catalog(diagnostics, DiagnosticCode::InsecureLinkProtocol, *span);
                }
            }
            NodeKind::LinkReference { .. } => {}
            NodeKind::CodeBlock { language, code } => {
                if code.trim().is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyCodeBlock, *span);
                }

                if !code.trim().is_empty() && language.is_none() {
                    diag_catalog(diagnostics, DiagnosticCode::MissingCodeBlockLanguage, *span);
                }
            }
            NodeKind::Image { url, alt } => {
                if url.trim().is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyImageUrl, *span);
                }

                if alt.trim().is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::ImageMissingAltText, *span);
                }

                let lower_url = url.to_lowercase();
                let settings = crate::intelligence::catalog::diagnostics_catalog_settings();
                if has_disallowed_scheme(&lower_url, &settings.unsafe_protocols) {
                    let protocol = url
                        .split_once(':')
                        .map(|(prefix, _)| prefix)
                        .unwrap_or(settings.unknown_protocol_label.as_str())
                        .to_string();
                    diag_catalog_message(
                        diagnostics,
                        DiagnosticCode::UnsafeImageProtocol,
                        *span,
                        DiagnosticCode::UnsafeImageProtocol
                            .format_message(&[("protocol", protocol)]),
                    );
                }
            }
            NodeKind::InlineHtml(html) => {
                let lower_html = html.to_lowercase();
                let settings = crate::intelligence::catalog::diagnostics_catalog_settings();
                if contains_any_marker(&lower_html, &settings.script_tag_markers) {
                    diag_catalog(diagnostics, DiagnosticCode::InlineHtmlContainsScript, *span);
                }

                if contains_unsafe_protocol_marker(&lower_html, &settings.unsafe_protocols) {
                    diag_catalog(diagnostics, DiagnosticCode::InlineHtmlJavascriptUrl, *span);
                }

                if contains_unsafe_event_handler_attr(&lower_html) {
                    diag_catalog(
                        diagnostics,
                        DiagnosticCode::InlineHtmlUnsafeEventHandler,
                        *span,
                    );
                }
            }
            NodeKind::List { .. } => {
                if node.children.is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyList, *span);
                }
            }
            NodeKind::ListItem => {
                if node.children.is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyListItem, *span);
                }

                let has_task_checkbox = node
                    .children
                    .iter()
                    .any(|child| matches!(child.kind, NodeKind::TaskCheckbox { .. }));

                if has_task_checkbox {
                    let has_task_content = node.children.iter().any(|child| {
                        !matches!(child.kind, NodeKind::TaskCheckbox { .. })
                            && node_has_meaningful_content(child)
                    });

                    if !has_task_content {
                        diag_catalog(diagnostics, DiagnosticCode::EmptyTaskListItem, *span);
                    }
                } else if list_item_has_malformed_task_marker(node) {
                    diag_catalog(diagnostics, DiagnosticCode::MalformedTaskCheckbox, *span);
                }
            }
            NodeKind::HtmlBlock { html } => {
                let lower_html = html.to_lowercase();
                let settings = crate::intelligence::catalog::diagnostics_catalog_settings();

                if contains_any_marker(&lower_html, &settings.script_tag_markers) {
                    diag_catalog(diagnostics, DiagnosticCode::HtmlBlockContainsScript, *span);
                }

                if contains_unsafe_protocol_marker(&lower_html, &settings.unsafe_protocols) {
                    diag_catalog(diagnostics, DiagnosticCode::HtmlBlockJavascriptUrl, *span);
                }

                if html.trim().is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyHtmlBlock, *span);
                }

                let open_angles = html.matches('<').count();
                let close_angles = html.matches('>').count();
                if open_angles != close_angles {
                    diag_catalog(
                        diagnostics,
                        DiagnosticCode::HtmlBlockMismatchedAngles,
                        *span,
                    );
                }

                if contains_unsafe_event_handler_attr(&lower_html) {
                    diag_catalog(
                        diagnostics,
                        DiagnosticCode::HtmlBlockUnsafeEventHandler,
                        *span,
                    );
                }
            }
            NodeKind::Blockquote => {
                if node.children.is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyBlockquote, *span);
                }

                if blockquote_has_unknown_admonition_marker(node) {
                    diag_catalog(diagnostics, DiagnosticCode::UnknownAdmonitionKind, *span);
                }
            }
            NodeKind::DefinitionList => {
                if node.children.is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyDefinitionList, *span);
                }
            }
            NodeKind::DefinitionTerm => {
                if !node_has_meaningful_content(node) {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyDefinitionTerm, *span);
                }
            }
            NodeKind::DefinitionDescription => {
                if !node_has_meaningful_content(node) {
                    diag_catalog(
                        diagnostics,
                        DiagnosticCode::EmptyDefinitionDescription,
                        *span,
                    );
                }
            }
            NodeKind::TableCell { .. } => {
                if !node_has_meaningful_content(node) {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyTableCell, *span);
                }
            }
            NodeKind::TabGroup => {
                if node.children.is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyTabGroup, *span);
                }

                let mut seen_titles: HashMap<String, Span> = HashMap::new();
                for child in &node.children {
                    if let (NodeKind::TabItem { title }, Some(tab_span)) = (&child.kind, child.span)
                    {
                        let normalized = title.trim().to_lowercase();

                        if normalized.is_empty() {
                            diag_catalog(diagnostics, DiagnosticCode::EmptyTabTitle, tab_span);
                        }

                        if !normalized.is_empty() {
                            if let std::collections::hash_map::Entry::Vacant(entry) =
                                seen_titles.entry(normalized)
                            {
                                entry.insert(tab_span);
                            } else {
                                diag_catalog(
                                    diagnostics,
                                    DiagnosticCode::DuplicateTabTitle,
                                    tab_span,
                                );
                            }
                        }

                        if !node_has_meaningful_content(child) {
                            diag_catalog(diagnostics, DiagnosticCode::EmptyTabPanel, tab_span);
                        }
                    }
                }
            }
            NodeKind::SliderDeck { timer_seconds } => {
                if node.children.is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptySliderDeck, *span);
                }

                if timer_seconds.is_some_and(|value| value == 0) {
                    diag_catalog(diagnostics, DiagnosticCode::InvalidSliderTimer, *span);
                }
            }
            NodeKind::Slide { .. } => {
                if !node_has_meaningful_content(node) {
                    diag_catalog(diagnostics, DiagnosticCode::EmptySlide, *span);
                }
            }
            NodeKind::Admonition { title, .. } => {
                if node.children.is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyAdmonitionBody, *span);
                }

                if let Some(custom_title) = title {
                    if custom_title.trim().is_empty() {
                        diag_catalog(diagnostics, DiagnosticCode::EmptyAdmonitionTitle, *span);
                    }
                }
            }
            NodeKind::InlineMath { content } | NodeKind::DisplayMath { content } => {
                if content.trim().is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyMathExpression, *span);
                }
            }
            NodeKind::MermaidDiagram { content } => {
                if content.trim().is_empty() {
                    diag_catalog(diagnostics, DiagnosticCode::EmptyMermaidDiagram, *span);
                }
            }
            NodeKind::PlatformMention {
                username,
                platform,
                display,
            } => {
                if username.trim().is_empty() {
                    diag_catalog(
                        diagnostics,
                        DiagnosticCode::EmptyPlatformMentionUsername,
                        *span,
                    );
                }

                if !is_known_platform(&platform.trim().to_lowercase()) {
                    diag_catalog(
                        diagnostics,
                        DiagnosticCode::UnknownPlatformMentionPlatform,
                        *span,
                    );
                }

                if display.as_ref().is_some_and(|d| d.trim().is_empty()) {
                    diag_catalog(
                        diagnostics,
                        DiagnosticCode::EmptyPlatformMentionDisplayName,
                        *span,
                    );
                }
            }
            NodeKind::Text(text) => {
                if text_has_unknown_emoji_shortcode(text) {
                    diag_catalog(diagnostics, DiagnosticCode::UnknownEmojiShortcode, *span);
                }
            }
            _ => {}
        }
    }

    for child in &node.children {
        collect_diagnostics(child, diagnostics);
    }
}

fn collect_link_reference_consistency_diagnostics(
    nodes: &[Node],
    references: &crate::parser::ReferenceMap,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for node in nodes {
        if let (NodeKind::LinkReference { label, .. }, Some(span)) = (&node.kind, node.span) {
            let normalized = normalize_label_for_diagnostics(label);

            if normalized.is_empty() {
                diag_catalog(diagnostics, DiagnosticCode::EmptyLinkReferenceLabel, span);
            } else if !references.contains(label) {
                diag_catalog(diagnostics, DiagnosticCode::UnresolvedLinkReference, span);
            }
        }

        if !node.children.is_empty() {
            collect_link_reference_consistency_diagnostics(&node.children, references, diagnostics);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Position;

    fn span(line: usize, start_col: usize, end_col: usize, start_offset: usize) -> Span {
        Span {
            start: Position {
                line,
                column: start_col,
                offset: start_offset,
            },
            end: Position {
                line,
                column: end_col,
                offset: start_offset + (end_col.saturating_sub(start_col)),
            },
        }
    }

    #[test]
    fn smoke_test_codes_are_stable_strings() {
        for code in [
            DiagnosticCode::ParseFailure,
            DiagnosticCode::InvalidHeadingLevel,
            DiagnosticCode::DuplicateHeadingId,
            DiagnosticCode::UnresolvedLinkReference,
            DiagnosticCode::EmptyLinkReferenceLabel,
            DiagnosticCode::MissingCodeBlockLanguage,
            DiagnosticCode::ImageMissingAltText,
            DiagnosticCode::InlineHtmlUnsafeEventHandler,
            DiagnosticCode::HtmlBlockUnsafeEventHandler,
            DiagnosticCode::EmptyDefinitionList,
            DiagnosticCode::MissingFootnoteDefinition,
            DiagnosticCode::EmptyTaskListItem,
            DiagnosticCode::InvalidSliderTimer,
            DiagnosticCode::EmptyPlatformMentionUsername,
            DiagnosticCode::UnknownPlatformMentionPlatform,
            DiagnosticCode::UnknownEmojiShortcode,
            DiagnosticCode::EmptyPlatformMentionDisplayName,
            DiagnosticCode::DuplicateTabTitle,
            DiagnosticCode::EmptyMathExpression,
            DiagnosticCode::EmptyAdmonitionTitle,
            DiagnosticCode::UnknownAdmonitionKind,
        ] {
            let id = code.as_str();
            assert!(
                id.starts_with("MD") || id.starts_with("MO") || id.starts_with("MG"),
                "unexpected diagnostic namespace for code id: {}",
                id
            );
            assert_eq!(id.len(), 5);
        }
    }

    #[test]
    fn smoke_test_all_diagnostic_codes_are_in_catalog() {
        let all_codes = [
            DiagnosticCode::ParseFailure,
            DiagnosticCode::InvalidHeadingLevel,
            DiagnosticCode::EmptyHeadingText,
            DiagnosticCode::DuplicateHeadingId,
            DiagnosticCode::HeadingTooLong,
            DiagnosticCode::EmptyLinkUrl,
            DiagnosticCode::UnsafeLinkProtocol,
            DiagnosticCode::InsecureLinkProtocol,
            DiagnosticCode::UnresolvedLinkReference,
            DiagnosticCode::EmptyLinkReferenceLabel,
            DiagnosticCode::EmptyCodeBlock,
            DiagnosticCode::MissingCodeBlockLanguage,
            DiagnosticCode::EmptyImageUrl,
            DiagnosticCode::ImageMissingAltText,
            DiagnosticCode::UnsafeImageProtocol,
            DiagnosticCode::InlineHtmlContainsScript,
            DiagnosticCode::InlineHtmlJavascriptUrl,
            DiagnosticCode::InlineHtmlUnsafeEventHandler,
            DiagnosticCode::HtmlBlockContainsScript,
            DiagnosticCode::HtmlBlockJavascriptUrl,
            DiagnosticCode::EmptyHtmlBlock,
            DiagnosticCode::HtmlBlockMismatchedAngles,
            DiagnosticCode::HtmlBlockUnsafeEventHandler,
            DiagnosticCode::EmptyList,
            DiagnosticCode::EmptyListItem,
            DiagnosticCode::MalformedTaskCheckbox,
            DiagnosticCode::EmptyTaskListItem,
            DiagnosticCode::EmptyBlockquote,
            DiagnosticCode::EmptyDefinitionList,
            DiagnosticCode::EmptyDefinitionTerm,
            DiagnosticCode::EmptyDefinitionDescription,
            DiagnosticCode::EmptyTableCell,
            DiagnosticCode::MissingFootnoteDefinition,
            DiagnosticCode::DuplicateFootnoteDefinition,
            DiagnosticCode::UnusedFootnoteDefinition,
            DiagnosticCode::EmptyTabGroup,
            DiagnosticCode::EmptyTabTitle,
            DiagnosticCode::DuplicateTabTitle,
            DiagnosticCode::EmptyTabPanel,
            DiagnosticCode::EmptySliderDeck,
            DiagnosticCode::EmptySlide,
            DiagnosticCode::EmptyAdmonitionBody,
            DiagnosticCode::EmptyMathExpression,
            DiagnosticCode::EmptyMermaidDiagram,
            DiagnosticCode::EmptyAdmonitionTitle,
            DiagnosticCode::UnknownAdmonitionKind,
            DiagnosticCode::InvalidSliderTimer,
            DiagnosticCode::EmptyPlatformMentionUsername,
            DiagnosticCode::UnknownPlatformMentionPlatform,
            DiagnosticCode::UnknownEmojiShortcode,
            DiagnosticCode::EmptyPlatformMentionDisplayName,
        ];

        for code in all_codes {
            assert!(
                code.catalog_entry().is_some(),
                "missing catalog entry for {:?}",
                code
            );
        }
    }

    #[test]
    fn smoke_test_fix_suggestions_are_available() {
        assert!(DiagnosticCode::DuplicateHeadingId
            .fix_suggestion()
            .contains("unique"));
        assert!(DiagnosticCode::MissingCodeBlockLanguage
            .fix_suggestion()
            .contains("```"));
        assert!(DiagnosticCode::MissingFootnoteDefinition
            .fix_suggestion()
            .contains("[^label]:"));
        assert!(DiagnosticCode::EmptySliderDeck
            .fix_suggestion()
            .contains("@slidestart"));
    }

    #[test]
    fn smoke_test_diagnostic_methods_expose_fixit_metadata() {
        let d = Diagnostic {
            code: DiagnosticCode::ImageMissingAltText,
            span: span(1, 1, 10, 0),
            severity: DiagnosticSeverity::Warning,
            message: "Image missing alt text".to_string(),
        };

        assert!(
            d.code_id().starts_with("MD")
                || d.code_id().starts_with("MO")
                || d.code_id().starts_with("MG")
        );
        assert!(d.fix_suggestion().contains("alt text"));
    }

    #[test]
    fn smoke_test_resolved_catalog_metadata_available_for_seed_code() {
        let d = Diagnostic {
            code: DiagnosticCode::EmptyImageUrl,
            span: span(1, 1, 5, 0),
            severity: DiagnosticSeverity::Error,
            message: "Empty image URL".to_string(),
        };

        assert_eq!(d.title_resolved(), Some("Empty image URL"));
        assert!(d
            .description_resolved()
            .expect("expected embedded catalog description")
            .contains("cannot render an image"));
    }

    #[test]
    fn smoke_test_resolved_fix_suggestion_uses_catalog_override_when_present() {
        let d = Diagnostic {
            code: DiagnosticCode::ImageMissingAltText,
            span: span(1, 1, 10, 0),
            severity: DiagnosticSeverity::Warning,
            message: "Image missing alt text".to_string(),
        };

        assert_eq!(
            d.fix_suggestion_resolved(),
            "Add descriptive alt text between '[' and ']' for accessibility and better screen-reader output."
        );
    }

    #[test]
    fn smoke_test_parse_error_diagnostic_builder() {
        let d = Diagnostic::parse_error("Parse failed");
        assert_eq!(d.code, DiagnosticCode::ParseFailure);
        assert_eq!(d.severity, DiagnosticSeverity::Error);
        assert!(d.code_id().starts_with("MD"));
        assert_eq!(d.span.start.line, 1);
        assert_eq!(d.span.start.column, 1);
    }

    #[test]
    fn smoke_test_diagnostics_options_critical_only_filters_non_errors() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Heading {
                    level: 1,
                    text: "This heading is intentionally very long to trigger an informational diagnostic while remaining syntactically valid and useful for filtering checks".to_string(),
                    id: None,
                },
                span: Some(span(1, 1, 20, 0)),
                children: vec![],
            }],
            ..Default::default()
        };

        let all = compute_diagnostics_with_options(&doc, DiagnosticsOptions::all());
        let critical = compute_diagnostics_with_options(&doc, DiagnosticsOptions::critical_only());

        assert!(all
            .iter()
            .any(|d| matches!(d.severity, DiagnosticSeverity::Info)));
        assert!(critical.is_empty());
    }

    #[test]
    fn smoke_test_diagnostics_options_max_limit_is_applied() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 10,
                        text: "".to_string(),
                        id: None,
                    },
                    span: Some(span(1, 1, 2, 0)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Image {
                        url: "".to_string(),
                        alt: "".to_string(),
                    },
                    span: Some(span(2, 1, 3, 10)),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics_with_options(
            &doc,
            DiagnosticsOptions {
                profile: DiagnosticsProfile::All,
                max_diagnostics: Some(2),
            },
        );

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn smoke_test_duplicate_heading_ids_diagnosed() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 2,
                        text: "A".to_string(),
                        id: Some("dup-id".to_string()),
                    },
                    span: Some(span(1, 1, 5, 0)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Heading {
                        level: 2,
                        text: "B".to_string(),
                        id: Some("dup-id".to_string()),
                    },
                    span: Some(span(3, 1, 5, 20)),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);
        assert!(diagnostics.iter().any(|d| {
            d.code == DiagnosticCode::DuplicateHeadingId
                && d.severity == DiagnosticSeverity::Warning
        }));
    }

    #[test]
    fn smoke_test_missing_language_and_http_link_rules() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::CodeBlock {
                        language: None,
                        code: "let x = 1;".to_string(),
                    },
                    span: Some(span(1, 1, 4, 0)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Paragraph,
                    span: Some(span(3, 1, 30, 30)),
                    children: vec![Node {
                        kind: NodeKind::Link {
                            url: ["http", "://example.com"].concat(),
                            title: None,
                        },
                        span: Some(span(3, 5, 20, 34)),
                        children: vec![],
                    }],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::MissingCodeBlockLanguage));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::InsecureLinkProtocol));
    }

    #[test]
    fn smoke_test_footnote_consistency_rules() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Paragraph,
                    span: Some(span(1, 1, 20, 0)),
                    children: vec![Node {
                        kind: NodeKind::FootnoteReference {
                            label: "missing".to_string(),
                        },
                        span: Some(span(1, 10, 19, 9)),
                        children: vec![],
                    }],
                },
                Node {
                    kind: NodeKind::FootnoteDefinition {
                        label: "dup".to_string(),
                    },
                    span: Some(span(3, 1, 10, 30)),
                    children: vec![Node {
                        kind: NodeKind::Paragraph,
                        span: Some(span(3, 5, 14, 34)),
                        children: vec![Node {
                            kind: NodeKind::Text("def one".to_string()),
                            span: Some(span(3, 5, 11, 34)),
                            children: vec![],
                        }],
                    }],
                },
                Node {
                    kind: NodeKind::FootnoteDefinition {
                        label: "DUP".to_string(),
                    },
                    span: Some(span(5, 1, 10, 60)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::FootnoteDefinition {
                        label: "unused".to_string(),
                    },
                    span: Some(span(7, 1, 12, 90)),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);

        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::MissingFootnoteDefinition));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::DuplicateFootnoteDefinition));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnusedFootnoteDefinition));
    }

    #[test]
    fn smoke_test_empty_table_cell_and_definition_entries() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::DefinitionList,
                    span: Some(span(1, 1, 4, 0)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Table {
                        alignments: vec![crate::parser::TableAlignment::None],
                    },
                    span: Some(span(3, 1, 4, 20)),
                    children: vec![Node {
                        kind: NodeKind::TableRow { header: false },
                        span: Some(span(3, 1, 4, 20)),
                        children: vec![Node {
                            kind: NodeKind::TableCell {
                                header: false,
                                alignment: crate::parser::TableAlignment::None,
                            },
                            span: Some(span(3, 2, 3, 21)),
                            children: vec![Node {
                                kind: NodeKind::Text("   ".to_string()),
                                span: Some(span(3, 2, 3, 21)),
                                children: vec![],
                            }],
                        }],
                    }],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptyDefinitionList));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptyTableCell));
    }

    #[test]
    fn smoke_test_tab_group_and_slider_rules() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::TabGroup,
                    span: Some(span(1, 1, 10, 0)),
                    children: vec![
                        Node {
                            kind: NodeKind::TabItem {
                                title: "One".to_string(),
                            },
                            span: Some(span(2, 1, 8, 11)),
                            children: vec![],
                        },
                        Node {
                            kind: NodeKind::TabItem {
                                title: " one ".to_string(),
                            },
                            span: Some(span(3, 1, 10, 20)),
                            children: vec![],
                        },
                    ],
                },
                Node {
                    kind: NodeKind::SliderDeck {
                        timer_seconds: Some(5),
                    },
                    span: Some(span(5, 1, 12, 40)),
                    children: vec![Node {
                        kind: NodeKind::Slide { vertical: false },
                        span: Some(span(6, 1, 8, 50)),
                        children: vec![Node {
                            kind: NodeKind::Text("  ".to_string()),
                            span: Some(span(6, 1, 3, 50)),
                            children: vec![],
                        }],
                    }],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);

        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::DuplicateTabTitle));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptyTabPanel));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptySlide));
    }

    #[test]
    fn smoke_test_empty_admonition_math_and_mermaid_rules() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Admonition {
                        kind: crate::parser::AdmonitionKind::Note,
                        title: Some("".to_string()),
                        icon: None,
                        style: crate::parser::AdmonitionStyle::Alert,
                    },
                    span: Some(span(1, 1, 10, 0)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Paragraph,
                    span: Some(span(3, 1, 12, 20)),
                    children: vec![
                        Node {
                            kind: NodeKind::InlineMath {
                                content: "   ".to_string(),
                            },
                            span: Some(span(3, 2, 6, 21)),
                            children: vec![],
                        },
                        Node {
                            kind: NodeKind::DisplayMath {
                                content: "\n\t".to_string(),
                            },
                            span: Some(span(3, 7, 11, 26)),
                            children: vec![],
                        },
                    ],
                },
                Node {
                    kind: NodeKind::MermaidDiagram {
                        content: "".to_string(),
                    },
                    span: Some(span(5, 1, 4, 40)),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);

        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptyAdmonitionBody));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptyMathExpression));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptyMermaidDiagram));
    }

    #[test]
    fn smoke_test_link_reference_and_html_event_handler_rules() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Paragraph,
                    span: Some(span(1, 1, 24, 0)),
                    children: vec![Node {
                        kind: NodeKind::LinkReference {
                            label: "missing-ref".to_string(),
                            suffix: "[missing-ref]".to_string(),
                        },
                        span: Some(span(1, 2, 20, 1)),
                        children: vec![Node {
                            kind: NodeKind::Text("Guide".to_string()),
                            span: Some(span(1, 3, 8, 2)),
                            children: vec![],
                        }],
                    }],
                },
                Node {
                    kind: NodeKind::InlineHtml("<a onclick=\"x()\">x</a>".to_string()),
                    span: Some(span(2, 1, 22, 25)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::HtmlBlock {
                        html: "<img onerror=\"x()\" src=\"/a.png\">".to_string(),
                    },
                    span: Some(span(3, 1, 30, 48)),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);

        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnresolvedLinkReference));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::InlineHtmlUnsafeEventHandler));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::HtmlBlockUnsafeEventHandler));
    }

    #[test]
    fn smoke_test_task_item_and_platform_mention_rules() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::ListItem,
                    span: Some(span(1, 1, 6, 0)),
                    children: vec![Node {
                        kind: NodeKind::TaskCheckbox { checked: false },
                        span: Some(span(1, 3, 5, 2)),
                        children: vec![],
                    }],
                },
                Node {
                    kind: NodeKind::PlatformMention {
                        username: "   ".to_string(),
                        platform: "unknownplatform".to_string(),
                        display: Some("   ".to_string()),
                    },
                    span: Some(span(2, 1, 22, 8)),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);

        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptyTaskListItem));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptyPlatformMentionUsername));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnknownPlatformMentionPlatform));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EmptyPlatformMentionDisplayName));
    }

    #[test]
    fn smoke_test_malformed_task_unknown_admonition_and_unknown_emoji_rules() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::ListItem,
                    span: Some(span(1, 1, 16, 0)),
                    children: vec![Node {
                        kind: NodeKind::Paragraph,
                        span: Some(span(1, 3, 16, 2)),
                        children: vec![Node {
                            kind: NodeKind::Text("[maybe] investigate".to_string()),
                            span: Some(span(1, 3, 16, 2)),
                            children: vec![],
                        }],
                    }],
                },
                Node {
                    kind: NodeKind::Blockquote,
                    span: Some(span(2, 1, 24, 20)),
                    children: vec![Node {
                        kind: NodeKind::Paragraph,
                        span: Some(span(2, 3, 24, 22)),
                        children: vec![Node {
                            kind: NodeKind::Text("[!CUSTOM] body".to_string()),
                            span: Some(span(2, 3, 24, 22)),
                            children: vec![],
                        }],
                    }],
                },
                Node {
                    kind: NodeKind::Paragraph,
                    span: Some(span(3, 1, 18, 45)),
                    children: vec![Node {
                        kind: NodeKind::Text("Status :not_an_emoji:".to_string()),
                        span: Some(span(3, 8, 23, 52)),
                        children: vec![],
                    }],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);

        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::MalformedTaskCheckbox));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnknownAdmonitionKind));
        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnknownEmojiShortcode));
    }

    #[test]
    fn smoke_test_unknown_emoji_shortcode_avoids_common_false_positives() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: Some(span(1, 1, 80, 0)),
                children: vec![Node {
                    kind: NodeKind::Text(
                        "Visit https://example.com:8080/path, ratio a:b:c, and valid :smile:."
                            .to_string(),
                    ),
                    span: Some(span(1, 1, 80, 0)),
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);

        assert!(diagnostics
            .iter()
            .all(|d| d.code != DiagnosticCode::UnknownEmojiShortcode));
    }

    #[test]
    fn smoke_test_unknown_emoji_shortcode_detects_punctuation_wrapped_token() {
        let doc = Document {
            children: vec![Node {
                kind: NodeKind::Paragraph,
                span: Some(span(1, 1, 42, 0)),
                children: vec![Node {
                    kind: NodeKind::Text("Please review (:not_an_emoji:) now.".to_string()),
                    span: Some(span(1, 1, 42, 0)),
                    children: vec![],
                }],
            }],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);

        assert!(diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnknownEmojiShortcode));
    }

    #[test]
    fn smoke_test_diagnostics_are_sorted_for_editor_stability() {
        let doc = Document {
            children: vec![
                Node {
                    kind: NodeKind::Heading {
                        level: 10,
                        text: "".to_string(),
                        id: None,
                    },
                    span: Some(span(2, 1, 2, 20)),
                    children: vec![],
                },
                Node {
                    kind: NodeKind::Image {
                        url: "".to_string(),
                        alt: "".to_string(),
                    },
                    span: Some(span(1, 1, 3, 0)),
                    children: vec![],
                },
            ],
            ..Default::default()
        };

        let diagnostics = compute_diagnostics(&doc);
        for window in diagnostics.windows(2) {
            let left = &window[0];
            let right = &window[1];
            let l_key = (
                left.span.start.offset,
                left.span.end.offset,
                left.severity.sort_rank(),
                left.code.as_str(),
                left.message.as_str(),
            );
            let r_key = (
                right.span.start.offset,
                right.span.end.offset,
                right.severity.sort_rank(),
                right.code.as_str(),
                right.message.as_str(),
            );
            assert!(
                l_key <= r_key,
                "diagnostics must be sorted for stable editor rendering"
            );
        }
    }

    #[test]
    fn smoke_test_sort_and_dedup_diagnostics_removes_exact_duplicates() {
        let mut diagnostics = vec![
            Diagnostic {
                code: DiagnosticCode::EmptyImageUrl,
                span: span(1, 1, 3, 0),
                severity: DiagnosticSeverity::Error,
                message: "Empty image URL".to_string(),
            },
            Diagnostic {
                code: DiagnosticCode::EmptyImageUrl,
                span: span(1, 1, 3, 0),
                severity: DiagnosticSeverity::Error,
                message: "Empty image URL".to_string(),
            },
            Diagnostic {
                code: DiagnosticCode::ImageMissingAltText,
                span: span(1, 1, 3, 0),
                severity: DiagnosticSeverity::Warning,
                message: "Image missing alt text".to_string(),
            },
        ];

        sort_and_dedup_diagnostics(&mut diagnostics);

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].code, DiagnosticCode::EmptyImageUrl);
        assert_eq!(diagnostics[1].code, DiagnosticCode::ImageMissingAltText);
    }
}
