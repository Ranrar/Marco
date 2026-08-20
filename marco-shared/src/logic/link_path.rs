//! Resolution layer for the paths Markdown links and images point at.
//!
//! A Markdown destination is one of four things — an absolute local path, a
//! path relative to the document, a remote URL, or a data URI — and only the
//! first two are filesystem paths at all. Everything that needs to reason
//! about "which file does this reference mean" goes through [`LinkReference`]
//! rather than pattern-matching path strings at the call site.
//!
//! Images (`![alt](dest)`), inline links (`[text](dest)`) and the link
//! definitions both kinds of reference share (`[label]: dest`) are all handled
//! the same way — a local path is a local path regardless of what points at
//! it, which is also why definitions need no owner to be identified.
//!
//! Two rules shape the rest of the module:
//!
//! 1. Filesystem work uses [`Path`]/[`PathBuf`] components, never manual `/`
//!    or `\` splicing, so Windows drive letters and UNC prefixes behave.
//! 2. Markdown paths always use `/` and percent-encode the characters that
//!    would otherwise terminate a link destination (space, parentheses, `#`),
//!    on every platform.
//!
//! An unsaved document has no directory for a relative path to resolve
//! against, so references inserted into one are stored as absolute paths. When
//! the document later gets a location, [`plan_path_rebase`] resolves every
//! local reference against the *old* location (or, for an unsaved document,
//! takes the absolute path as-is) and rewrites it relative to the new one, so
//! the same physical file stays referenced across Save As and moves.

use std::path::{Component, Path, PathBuf};

/// What a Markdown link/image destination points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    /// An absolute filesystem path (`/home/kim/a.png`, `C:/Users/Kim/a.png`,
    /// `file:///home/kim/a.png`).
    LocalAbsolute,
    /// A filesystem path relative to the document (`../Pictures/a.png`).
    LocalRelative,
    /// Anything with a non-`file` scheme, plus protocol-relative `//host/…`.
    Remote,
    /// `data:image/png;base64,…`
    DataUri,
    /// A bare in-document anchor (`#section`). Not a file reference.
    Fragment,
}

impl TargetKind {
    /// Whether this kind participates in relative-path conversion.
    pub fn is_local(self) -> bool {
        matches!(self, TargetKind::LocalAbsolute | TargetKind::LocalRelative)
    }
}

/// A single image destination, as written in the Markdown and as resolved on
/// the filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkReference {
    /// The destination exactly as it appears in the Markdown, including any
    /// `#fragment` or `?query` suffix and any percent-encoding.
    pub markdown_path: String,
    /// The resolved filesystem path — absolute and lexically normalized —
    /// when this is a local reference that could be resolved. `None` for
    /// remote URLs, data URIs, anchors, and for relative paths with no
    /// document directory to resolve against.
    ///
    /// The file is *not* required to exist: a reference to a deleted image
    /// still resolves, so it can be rebased rather than silently dropped.
    pub resolved_path: Option<PathBuf>,
    /// What the destination is.
    pub kind: TargetKind,
}

impl LinkReference {
    /// Classify and resolve `markdown_path`.
    ///
    /// `document_dir` is the directory of the document the Markdown lives in
    /// — `None` for a document that has never been saved, in which case
    /// relative references cannot be resolved (and are left alone) while
    /// absolute ones still resolve.
    pub fn parse(markdown_path: &str, document_dir: Option<&Path>) -> Self {
        let trimmed = markdown_path.trim();
        let kind = classify(trimmed);

        let resolved_path = if kind.is_local() {
            let (path_part, _suffix) = split_suffix(trimmed);
            let decoded = percent_decode(strip_file_scheme(path_part));
            let path = PathBuf::from(decoded);

            match kind {
                // A path this platform does not consider absolute is
                // absolute *somewhere else* — a `C:/…` path in a document
                // opened on Linux, or `/home/…` opened on Windows. It cannot
                // be resolved here, and rebasing it would produce nonsense,
                // so it is left untouched.
                TargetKind::LocalAbsolute => path.is_absolute().then(|| normalize(&path)),
                TargetKind::LocalRelative => document_dir.map(|dir| normalize(&dir.join(&path))),
                _ => None,
            }
        } else {
            None
        };

        Self {
            markdown_path: markdown_path.to_string(),
            resolved_path,
            kind,
        }
    }

    /// Whether this reference is a filesystem path.
    pub fn is_local(&self) -> bool {
        self.kind.is_local()
    }

    /// The `#fragment` / `?query` suffix of the destination, or `""`.
    pub fn suffix(&self) -> &str {
        split_suffix(&self.markdown_path).1
    }
}

/// Classify a Markdown link/image destination.
pub fn classify(target: &str) -> TargetKind {
    let trimmed = target.trim();

    if trimmed.is_empty() {
        return TargetKind::LocalRelative;
    }
    if trimmed.starts_with('#') {
        return TargetKind::Fragment;
    }
    // Protocol-relative URLs (`//cdn.example.com/x.png`) are remote.
    if trimmed.starts_with("//") {
        return TargetKind::Remote;
    }
    // A drive-absolute Windows path (`C:/…`) would otherwise look like a
    // one-letter URL scheme, so it has to be recognized before scheme parsing.
    if is_windows_drive_path(trimmed) {
        return TargetKind::LocalAbsolute;
    }

    if let Some(scheme) = leading_scheme(trimmed) {
        if scheme.eq_ignore_ascii_case("data") {
            return TargetKind::DataUri;
        }
        if scheme.eq_ignore_ascii_case("file") {
            return TargetKind::LocalAbsolute;
        }
        return TargetKind::Remote;
    }

    // UNC paths (`\\server\share\a.png`) and POSIX absolute paths.
    if trimmed.starts_with('/') || trimmed.starts_with('\\') {
        return TargetKind::LocalAbsolute;
    }

    TargetKind::LocalRelative
}

/// The URL scheme at the start of `target`, if it has a well-formed one.
fn leading_scheme(target: &str) -> Option<&str> {
    let colon = target.find(':')?;
    let scheme = &target[..colon];
    let valid = !scheme.is_empty()
        && scheme
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'));
    valid.then_some(scheme)
}

/// Whether `path` starts with a Windows drive designator (`C:\` or `C:/`).
pub fn is_windows_drive_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

/// Strip a `file://` prefix, leaving a plain filesystem path.
///
/// `file:///home/kim/a.png` → `/home/kim/a.png`, and
/// `file:///C:/Users/Kim/a.png` → `C:/Users/Kim/a.png` (the extra leading
/// slash Windows drive paths carry in a `file:` URI is dropped).
fn strip_file_scheme(target: &str) -> &str {
    let has_scheme = target
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("file://"));
    if !has_scheme {
        return target;
    }
    let rest = &target[7..];

    // `file:///C:/Users/Kim/a.png` carries an extra leading slash before the
    // drive letter; POSIX paths keep theirs.
    if let Some(stripped) = rest.strip_prefix('/') {
        if is_windows_drive_path(stripped) {
            return stripped;
        }
    }
    rest
}

/// Split a destination into its filesystem part and its `#fragment` /
/// `?query` suffix. Generated paths percent-encode `#` and `?`, so the first
/// literal occurrence is always the real separator.
pub fn split_suffix(target: &str) -> (&str, &str) {
    let cut = target.find(['#', '?']).unwrap_or(target.len());
    (&target[..cut], &target[cut..])
}

/// Lexically normalize a path: resolve `.` and `..` components without
/// touching the filesystem, so references to files that do not exist (yet, or
/// any more) still resolve.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                // Only pop a real directory name; `..` above a root or after
                // another `..` has to be kept verbatim.
                let pops = matches!(out.components().next_back(), Some(Component::Normal(_)));
                if pops {
                    out.pop();
                } else {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    if out.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        out
    }
}

/// The Markdown destination to write for the local file `target`.
///
/// Relative to `document_dir` when there is one and a relative path exists;
/// otherwise the absolute path — which is the correct answer for an unsaved
/// document, and for a Windows image on a different drive than the document
/// (there is no valid `../..` path across drives).
pub fn markdown_path_for_file(target: &Path, document_dir: Option<&Path>) -> String {
    if let Some(dir) = document_dir {
        if let Some(relative) = relative_path(target, dir) {
            return ensure_explicit_relative_prefix(&encode_markdown_path(&to_slash_path(
                &relative,
            )));
        }

        log::debug!(
            "[link_path] no relative path from '{}' to '{}'; keeping the absolute path",
            dir.display(),
            target.display()
        );
    }

    encode_markdown_path(&to_slash_path(target))
}

/// The path of `target` relative to `base`, or `None` when the two share no
/// common prefix (different Windows drives, or one of them relative).
pub fn relative_path(target: &Path, base: &Path) -> Option<PathBuf> {
    let target = normalize(target);
    let base = normalize(base);

    let target_components: Vec<Component<'_>> = target.components().collect();
    let base_components: Vec<Component<'_>> = base.components().collect();

    let mut common = 0usize;
    let shared = target_components.len().min(base_components.len());
    while common < shared && components_equal(target_components[common], base_components[common]) {
        common += 1;
    }

    // Two absolute paths with nothing in common are on different roots
    // (`C:\` vs `D:\`, or a UNC share vs a drive) — no relative path exists.
    if common == 0 && (target.is_absolute() || base.is_absolute()) {
        return None;
    }

    let mut relative = PathBuf::new();
    for component in &base_components[common..] {
        match component {
            Component::Normal(_) => relative.push(".."),
            // A `..` left in the base after normalization means the base
            // itself escaped its root; we cannot invert that step.
            Component::ParentDir => return None,
            _ => {}
        }
    }
    for component in &target_components[common..] {
        relative.push(component.as_os_str());
    }

    if relative.as_os_str().is_empty() {
        Some(PathBuf::from("."))
    } else {
        Some(relative)
    }
}

#[cfg(target_os = "windows")]
fn components_equal(left: Component<'_>, right: Component<'_>) -> bool {
    left.as_os_str()
        .to_string_lossy()
        .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
}

#[cfg(not(target_os = "windows"))]
fn components_equal(left: Component<'_>, right: Component<'_>) -> bool {
    left == right
}

/// Render a filesystem path with `/` separators, as Markdown requires on
/// every platform.
pub fn to_slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Give a relative Markdown path an explicit `./` so it can never be mistaken
/// for a scheme or a reference label.
pub fn ensure_explicit_relative_prefix(path: &str) -> String {
    if path.starts_with("./") || path.starts_with("../") || path.starts_with('/') {
        return path.to_string();
    }
    if path.is_empty() {
        "./".to_string()
    } else {
        format!("./{path}")
    }
}

/// Percent-encode the characters that would otherwise end (or re-interpret) a
/// Markdown link destination. Non-ASCII bytes are left alone — every WebView
/// backend this feeds handles them, and encoding them makes paths unreadable
/// in the editor.
pub fn encode_markdown_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for ch in path.chars() {
        match ch {
            ' ' | '%' | '#' | '?' | '(' | ')' | '<' | '>' | '"' | '\'' | '\\' => {
                let mut buf = [0u8; 4];
                for byte in ch.encode_utf8(&mut buf).as_bytes() {
                    out.push_str(&format!("%{byte:02X}"));
                }
            }
            ch if (ch as u32) < 0x20 => {
                out.push_str(&format!("%{:02X}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out
}

/// Decode percent-escapes back into a filesystem path. Malformed escapes are
/// left verbatim rather than dropped.
pub fn percent_decode(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push((hi * 16 + lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Which piece of Markdown syntax a destination was written in.
///
/// Rebasing treats all three identically — a local path is a local path — but
/// a diagnostic about a missing file has to say whether a *link* or an *image*
/// is broken, so the distinction is carried out of the scanner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOrigin {
    /// `[text](dest)`
    InlineLink,
    /// `![alt](dest)`
    InlineImage,
    /// `[label]: dest` — shared by reference-style links and images, with
    /// nothing in the definition itself saying which kind will use it.
    ReferenceDefinition,
}

/// A byte range in a Markdown document holding one image destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkTargetSpan {
    /// Byte offset of the first byte of the destination token.
    pub start: usize,
    /// Byte offset one past the last byte of the destination token.
    pub end: usize,
    /// The destination itself, without the `<…>` wrapper if it had one.
    pub target: String,
    /// Whether the destination was written in angle-bracket form.
    pub angle_bracketed: bool,
    /// The syntax the destination appeared in.
    pub origin: TargetOrigin,
}

/// A local link or image destination whose file is not on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingTarget {
    /// Byte offset of the first byte of the destination token.
    pub start: usize,
    /// Byte offset one past the last byte of the destination token.
    pub end: usize,
    /// The destination exactly as written in the Markdown.
    pub markdown_path: String,
    /// Where that destination resolves to, and where nothing was found.
    pub resolved_path: PathBuf,
    /// The syntax the destination appeared in.
    pub origin: TargetOrigin,
}

/// Every local link, image and link-definition destination in `markdown` that
/// resolves to a path with no file (or directory) at it.
///
/// `document_dir` is the directory the document lives in, or `None` for one
/// that has never been saved — in which case relative destinations have
/// nothing to resolve against and are left unjudged rather than reported as
/// missing. Remote URLs, data URIs and in-document anchors are never
/// filesystem paths and are skipped for the same reason, as is a destination
/// absolute on another platform (a `C:/…` path in a document opened on Linux),
/// which this host cannot check.
///
/// Each distinct path is stat'ed once however many times it is referenced.
pub fn find_missing_local_targets(
    markdown: &str,
    document_dir: Option<&Path>,
) -> Vec<MissingTarget> {
    let mut seen: std::collections::HashMap<PathBuf, bool> = std::collections::HashMap::new();
    let mut missing = Vec::new();

    for span in find_link_targets(markdown) {
        let reference = LinkReference::parse(&span.target, document_dir);
        let Some(resolved) = reference.resolved_path else {
            continue;
        };
        // An empty destination resolves to the document's own directory, which
        // always exists; `EmptyLinkUrl` already covers that case anyway.
        let exists = *seen
            .entry(resolved.clone())
            .or_insert_with(|| resolved.exists());
        if exists {
            continue;
        }

        missing.push(MissingTarget {
            start: span.start,
            end: span.end,
            markdown_path: span.target,
            resolved_path: resolved,
            origin: span.origin,
        });
    }

    missing
}

/// The 1-based line and 1-based *byte* column of `byte_offset` within `text`.
///
/// Byte columns rather than character columns: this is the convention
/// `marco_core`'s diagnostic positions use, so spans from here and from the
/// parser can be mapped to editor iters by the same code.
pub fn line_and_byte_column(text: &str, byte_offset: usize) -> (usize, usize) {
    let offset = byte_offset.min(text.len());
    let before = &text[..offset];
    let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    (line, offset - line_start + 1)
}

/// A replacement to apply to a Markdown document, as a byte range and its new
/// text. Ranges never overlap and are returned in ascending order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetEdit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

/// Rewrite every local image path in `markdown` so it keeps pointing at the
/// same physical file after the document moves from `old_document` to
/// `new_document`.
///
/// `old_document` is `None` for a document that has never been saved; its
/// absolute image paths are used as-is. Remote URLs, data URIs, anchors, and
/// relative paths that cannot be resolved are left untouched, as are
/// references that already have the right form.
pub fn plan_path_rebase(
    markdown: &str,
    old_document: Option<&Path>,
    new_document: &Path,
) -> Vec<TargetEdit> {
    let old_dir = old_document.and_then(|p| p.parent()).map(Path::to_path_buf);
    let new_dir = new_document
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut edits = Vec::new();
    for span in find_link_targets(markdown) {
        let reference = LinkReference::parse(&span.target, old_dir.as_deref());
        let Some(resolved) = reference.resolved_path.as_deref() else {
            continue;
        };

        let suffix = reference.suffix();
        let rebased = format!(
            "{}{}",
            markdown_path_for_file(resolved, Some(&new_dir)),
            suffix
        );

        // The rebased form never needs angle brackets (it is percent-encoded),
        // so an unchanged path in bracket form is still left alone.
        if rebased == span.target {
            continue;
        }

        edits.push(TargetEdit {
            start: span.start,
            end: span.end,
            replacement: rebased,
        });
    }

    edits
}

/// Apply `edits` (as produced by [`plan_path_rebase`]) to `markdown`.
pub fn apply_edits(markdown: &str, edits: &[TargetEdit]) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut cursor = 0usize;
    for edit in edits {
        if edit.start < cursor || edit.end > markdown.len() {
            continue;
        }
        out.push_str(&markdown[cursor..edit.start]);
        out.push_str(&edit.replacement);
        cursor = edit.end;
    }
    out.push_str(&markdown[cursor..]);
    out
}

/// Locate every destination in `markdown` a rebase could touch: inline images
/// (`![alt](dest)`), inline links (`[text](dest)`), and the link definitions
/// (`[label]: dest`) that reference-style images and links share.
///
/// Definitions are covered without working out which kind of reference uses
/// them — a local path is rebased the same way either way, so the ambiguity
/// never has to be resolved.
///
/// Fenced code blocks and inline code spans are skipped so their contents are
/// never rewritten.
pub fn find_link_targets(markdown: &str) -> Vec<LinkTargetSpan> {
    let mut spans = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut line_start = 0usize;

    while line_start <= markdown.len() {
        let line_end = markdown[line_start..]
            .find('\n')
            .map(|offset| line_start + offset)
            .unwrap_or(markdown.len());
        let line = &markdown[line_start..line_end];

        match fence {
            Some(open) => {
                if closes_fence(line, open) {
                    fence = None;
                }
            }
            None => match opening_fence(line) {
                Some(open) => fence = Some(open),
                // A definition line holds nothing but the definition, so
                // there is no inline content left on it to scan.
                None => match parse_reference_definition(line) {
                    Some(span) => spans.push(offset_span(span, line_start)),
                    None => scan_line_for_links(line, line_start, &mut spans),
                },
            },
        }

        if line_end >= markdown.len() {
            break;
        }
        line_start = line_end + 1;
    }

    spans
}

/// The fence character and run length if `line` opens a fenced code block.
fn opening_fence(line: &str) -> Option<(u8, usize)> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let marker = trimmed.as_bytes().first().copied()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let run = trimmed.bytes().take_while(|b| *b == marker).count();
    if run < 3 {
        return None;
    }
    // An info string on a backtick fence may not contain a backtick.
    if marker == b'`' && trimmed[run..].contains('`') {
        return None;
    }
    Some((marker, run))
}

/// Whether `line` closes the fence opened with `(marker, run)`.
fn closes_fence(line: &str, (marker, run): (u8, usize)) -> bool {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return false;
    }
    let closing = trimmed.bytes().take_while(|b| *b == marker).count();
    closing >= run && trimmed[closing..].trim().is_empty()
}

/// Move a span parsed within a single line onto document coordinates.
fn offset_span(span: LinkTargetSpan, base: usize) -> LinkTargetSpan {
    LinkTargetSpan {
        start: base + span.start,
        end: base + span.end,
        ..span
    }
}

/// Scan one line (outside any fence) for inline images and links, pushing
/// their destination spans — offset by `base` — onto `spans`.
fn scan_line_for_links(line: &str, base: usize, spans: &mut Vec<LinkTargetSpan>) {
    let bytes = line.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'`' => {
                let run = bytes[i..].iter().take_while(|b| **b == b'`').count();
                match find_backtick_run(bytes, i + run, run) {
                    Some(close) => i = close + run,
                    None => i += run,
                }
            }
            // `![` is an image and `[` a link; the destination is parsed the
            // same way for both, so they differ only in where the label
            // starts. A `!` whose image parse fails advances by one so the
            // `[` still gets its chance as a plain link.
            b'!' if bytes.get(i + 1) == Some(&b'[') => match parse_inline_link_at(line, i, 2) {
                Some((span, next)) => {
                    spans.push(offset_span(span, base));
                    i = next;
                }
                None => i += 1,
            },
            b'[' => match parse_inline_link_at(line, i, 1) {
                Some((span, next)) => {
                    spans.push(offset_span(span, base));
                    i = next;
                }
                None => i += 1,
            },
            _ => i += 1,
        }
    }
}

/// Parse a link definition line (`[label]: destination "title"`), returning
/// the destination's span within `line`.
///
/// Only a line that is *entirely* a definition qualifies: a trailing
/// remainder means this was ordinary text that merely looked like one.
fn parse_reference_definition(line: &str) -> Option<LinkTargetSpan> {
    let bytes = line.as_bytes();

    let indent = line.len() - line.trim_start_matches(' ').len();
    // Four spaces of indent make this an indented code block, not a definition.
    if indent > 3 || bytes.get(indent) != Some(&b'[') {
        return None;
    }
    // `[^label]: …` is a footnote definition, whose body is prose. Reading its
    // first word as a destination would rewrite the footnote's own text.
    if bytes.get(indent + 1) == Some(&b'^') {
        return None;
    }

    // Label: runs to the first unescaped `]`, and may not be empty.
    let mut i = indent + 1;
    while i < bytes.len() && bytes[i] != b']' {
        if bytes[i] == b'\\' {
            i += 1;
        }
        i += 1;
    }
    if bytes.get(i) != Some(&b']') || i == indent + 1 {
        return None;
    }

    i += 1;
    while matches!(bytes.get(i), Some(b' ' | b'\t')) {
        i += 1;
    }
    if bytes.get(i) != Some(&b':') {
        return None;
    }
    i += 1;
    while matches!(bytes.get(i), Some(b' ' | b'\t')) {
        i += 1;
    }

    // A definition's destination ends at whitespace, never at `)`.
    let (span, mut i) = parse_destination(line, i, false, TargetOrigin::ReferenceDefinition)?;
    if span.target.is_empty() {
        return None;
    }

    i = skip_optional_title(line, i)?;
    line[i..].trim().is_empty().then_some(span)
}

/// Skip an optional link title starting at `i`, returning the index after it
/// (or after nothing, if there is no title). `None` if a title is opened but
/// never closed on this line.
fn skip_optional_title(line: &str, mut i: usize) -> Option<usize> {
    let bytes = line.as_bytes();

    while matches!(bytes.get(i), Some(b' ' | b'\t')) {
        i += 1;
    }

    let close = match bytes.get(i).copied() {
        Some(b'"') => b'"',
        Some(b'\'') => b'\'',
        Some(b'(') => b')',
        _ => return Some(i),
    };

    i += 1;
    while i < bytes.len() && bytes[i] != close {
        if bytes[i] == b'\\' {
            i += 1;
        }
        i += 1;
    }
    if bytes.get(i) != Some(&close) {
        return None;
    }
    i += 1;

    while matches!(bytes.get(i), Some(b' ' | b'\t')) {
        i += 1;
    }
    Some(i)
}

/// Parse a link destination starting at `i` — either `<…>` or a bare run —
/// returning its span within `line` and the index just past it.
///
/// `stop_at_paren` ends a bare destination at an unbalanced `)`, which is what
/// an inline `(…)` destination needs and a definition must not do.
fn parse_destination(
    line: &str,
    mut i: usize,
    stop_at_paren: bool,
    origin: TargetOrigin,
) -> Option<(LinkTargetSpan, usize)> {
    let bytes = line.as_bytes();
    let start = i;

    let (target, angle_bracketed) = if bytes.get(i) == Some(&b'<') {
        i += 1;
        let inner_start = i;
        while i < bytes.len() && bytes[i] != b'>' {
            if bytes[i] == b'\\' {
                i += 1;
            }
            i += 1;
        }
        if bytes.get(i) != Some(&b'>') {
            return None;
        }
        let target = line.get(inner_start..i)?.to_string();
        i += 1;
        (target, true)
    } else {
        let mut parens = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b'\\' => i += 1,
                b' ' | b'\t' => break,
                b'(' if stop_at_paren => parens += 1,
                b')' if stop_at_paren => {
                    if parens == 0 {
                        break;
                    }
                    parens -= 1;
                }
                _ => {}
            }
            i += 1;
        }
        (line.get(start..i)?.to_string(), false)
    };

    Some((
        LinkTargetSpan {
            // The span covers the whole token, `<…>` included: generated
            // paths are percent-encoded, so the replacement never needs the
            // brackets the original may have had.
            start,
            end: i,
            target,
            angle_bracketed,
            origin,
        },
        i,
    ))
}

/// Index of the next run of exactly `run` backticks at or after `from`.
fn find_backtick_run(bytes: &[u8], from: usize, run: usize) -> Option<usize> {
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let len = bytes[i..].iter().take_while(|b| **b == b'`').count();
            if len == run {
                return Some(i);
            }
            i += len;
        } else {
            i += 1;
        }
    }
    None
}

/// Parse the inline image or link whose label starts at `start`, with
/// `marker_len` bytes of prefix before the label's `[` (2 for `![`, 1 for a
/// plain link). Returns its destination span and the index just past the
/// closing `)`.
fn parse_inline_link_at(
    line: &str,
    start: usize,
    marker_len: usize,
) -> Option<(LinkTargetSpan, usize)> {
    let bytes = line.as_bytes();

    // Label: balanced brackets, honouring backslash escapes.
    let mut i = start + marker_len;
    let mut depth = 1usize;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'\\' => i += 1,
            b'[' => depth += 1,
            b']' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    // No `(` after the label means a reference-style or bare bracket, whose
    // destination — if any — lives in a definition line instead.
    if depth != 0 || bytes.get(i) != Some(&b'(') {
        return None;
    }

    i += 1;
    while matches!(bytes.get(i), Some(b' ' | b'\t')) {
        i += 1;
    }

    let origin = if marker_len == 2 {
        TargetOrigin::InlineImage
    } else {
        TargetOrigin::InlineLink
    };
    let (span, i) = parse_destination(line, i, true, origin)?;

    // Optional title, then the closing paren — without which this was never
    // an inline link in the first place.
    let i = skip_optional_title(line, i)?;
    if bytes.get(i) != Some(&b')') {
        return None;
    }

    Some((span, i + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn smoke_classifies_every_target_kind() {
        assert_eq!(classify("/home/kim/a.png"), TargetKind::LocalAbsolute);
        assert_eq!(classify("C:/Users/Kim/a.png"), TargetKind::LocalAbsolute);
        assert_eq!(
            classify("file:///home/kim/a.png"),
            TargetKind::LocalAbsolute
        );
        assert_eq!(classify("../Pictures/a.png"), TargetKind::LocalRelative);
        assert_eq!(classify("a.png"), TargetKind::LocalRelative);
        assert_eq!(classify("https://example.com/a.png"), TargetKind::Remote);
        assert_eq!(classify("//cdn.example.com/a.png"), TargetKind::Remote);
        assert_eq!(classify("data:image/png;base64,AAA"), TargetKind::DataUri);
        assert_eq!(classify("#section"), TargetKind::Fragment);
    }

    #[test]
    fn smoke_resolves_relative_target_against_document_dir() {
        let reference = LinkReference::parse(
            "../Pictures/test.png",
            Some(Path::new("/home/kim/Documents")),
        );
        assert_eq!(
            reference.resolved_path,
            Some(p("/home/kim/Pictures/test.png"))
        );
    }

    #[test]
    #[cfg(unix)]
    fn smoke_absolute_target_resolves_without_document_dir() {
        let reference = LinkReference::parse("/home/kim/Pictures/test.png", None);
        assert_eq!(
            reference.resolved_path,
            Some(p("/home/kim/Pictures/test.png"))
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn smoke_windows_absolute_target_resolves_without_document_dir() {
        let reference = LinkReference::parse(r"C:\Users\Kim\Pictures\test.png", None);
        assert_eq!(
            reference.resolved_path,
            Some(p(r"C:\Users\Kim\Pictures\test.png"))
        );
    }

    #[test]
    fn smoke_relative_target_is_unresolvable_without_document_dir() {
        let reference = LinkReference::parse("../Pictures/test.png", None);
        assert_eq!(reference.resolved_path, None);
        assert!(reference.is_local());
    }

    #[test]
    fn smoke_remote_and_data_targets_never_resolve() {
        assert_eq!(
            LinkReference::parse("https://example.com/a.png", Some(Path::new("/docs")))
                .resolved_path,
            None
        );
        assert_eq!(
            LinkReference::parse("data:image/png;base64,AAA", Some(Path::new("/docs")))
                .resolved_path,
            None
        );
    }

    #[test]
    fn smoke_fragment_is_split_from_the_filesystem_path() {
        let reference =
            LinkReference::parse("../Images/test.png#something", Some(Path::new("/docs/sub")));
        assert_eq!(reference.resolved_path, Some(p("/docs/Images/test.png")));
        assert_eq!(reference.suffix(), "#something");
    }

    #[test]
    fn smoke_percent_encoded_spaces_resolve_to_real_paths() {
        let reference = LinkReference::parse("./My%20Pictures/a.png", Some(Path::new("/docs")));
        assert_eq!(reference.resolved_path, Some(p("/docs/My Pictures/a.png")));
    }

    #[test]
    fn smoke_markdown_path_is_relative_when_document_dir_is_known() {
        let path = markdown_path_for_file(
            Path::new("/home/kim/Pictures/test.png"),
            Some(Path::new("/home/kim/Documents")),
        );
        assert_eq!(path, "../Pictures/test.png");
    }

    #[test]
    fn smoke_markdown_path_is_absolute_for_unsaved_document() {
        let path = markdown_path_for_file(Path::new("/home/kim/Pictures/test.png"), None);
        assert_eq!(path, "/home/kim/Pictures/test.png");
    }

    #[test]
    fn smoke_sibling_file_gets_explicit_dot_prefix() {
        let path = markdown_path_for_file(Path::new("/docs/a.png"), Some(Path::new("/docs")));
        assert_eq!(path, "./a.png");
    }

    #[test]
    fn smoke_markdown_path_encodes_spaces() {
        let path = markdown_path_for_file(
            Path::new("/home/kim/My Pictures/test.png"),
            Some(Path::new("/home/kim")),
        );
        assert_eq!(path, "./My%20Pictures/test.png");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn smoke_different_windows_drives_keep_the_absolute_path() {
        // No relative path spans two drives, so the absolute one is kept.
        assert_eq!(
            relative_path(Path::new(r"D:\Images\a.png"), Path::new(r"C:\Users\Kim")),
            None
        );
        let md = "![t](D:/Images/test.png)\n";
        let edits = plan_path_rebase(md, None, Path::new(r"C:\Users\Kim\Documents\test.md"));
        assert_eq!(apply_edits(md, &edits), "![t](D:/Images/test.png)\n");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn smoke_foreign_platform_absolute_path_is_left_untouched() {
        // A Windows path in a document opened on Linux cannot be resolved
        // here; rewriting it would corrupt a reference this host can't check.
        let md = "![t](C:/Users/Kim/Pictures/test.png)\n";
        let edits = plan_path_rebase(md, None, Path::new("/home/kim/Documents/test.md"));
        assert!(edits.is_empty());
    }

    #[test]
    fn smoke_finds_inline_image_targets() {
        let md = "text ![alt](../a.png) more ![b](<./with space.png> \"title\")";
        let spans = find_link_targets(md);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].target, "../a.png");
        assert_eq!(&md[spans[0].start..spans[0].end], "../a.png");
        assert_eq!(spans[1].target, "./with space.png");
        assert!(spans[1].angle_bracketed);
    }

    #[test]
    fn smoke_skips_images_inside_code_spans_and_fences() {
        let md = "`![x](/a.png)` text\n\n```\n![y](/b.png)\n```\n\n![z](/c.png)\n";
        let spans = find_link_targets(md);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].target, "/c.png");
    }

    #[test]
    fn smoke_finds_inline_link_targets_too() {
        let spans = find_link_targets("see [notes](./notes.md) and ![i](/a.png)");
        assert_eq!(
            spans.iter().map(|s| s.target.as_str()).collect::<Vec<_>>(),
            vec!["./notes.md", "/a.png"]
        );
    }

    #[test]
    fn smoke_finds_reference_definition_targets() {
        let md = "[ref1]: /home/kim/notes.md \"Title\"\n[ref2]: <../a b.png>\n";
        let spans = find_link_targets(md);
        assert_eq!(
            spans.iter().map(|s| s.target.as_str()).collect::<Vec<_>>(),
            vec!["/home/kim/notes.md", "../a b.png"]
        );
    }

    #[test]
    fn smoke_ignores_text_that_only_looks_like_a_definition() {
        // Trailing prose means this is a paragraph, not a definition.
        assert!(find_link_targets("[ref]: /a.png and then some prose").is_empty());
        // An indented code block, not a definition.
        assert!(find_link_targets("    [ref]: /a.png").is_empty());
    }

    #[test]
    fn smoke_footnote_definition_body_is_never_treated_as_a_path() {
        let md = "[^1]: note\n[^2]: a longer note\n";
        let edits = plan_path_rebase(md, Some(Path::new("/a/old.md")), Path::new("/b/new.md"));
        assert!(edits.is_empty());
    }

    #[test]
    fn smoke_reference_style_usage_has_no_destination_to_rewrite() {
        assert!(find_link_targets("![alt][ref] and [text][ref]").is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn smoke_rebase_rewrites_links_images_and_definitions_together() {
        let md = "\
![i](/home/kim/Pictures/a.png)
[notes](/home/kim/Documents/notes.md)
[ref]: /home/kim/Pictures/b.png \"Cap\"
[remote](https://example.com/x)
";
        let edits = plan_path_rebase(md, None, Path::new("/home/kim/Documents/doc.md"));
        assert_eq!(
            apply_edits(md, &edits),
            "\
![i](../Pictures/a.png)
[notes](./notes.md)
[ref]: ../Pictures/b.png \"Cap\"
[remote](https://example.com/x)
"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn smoke_windows_rebase_rewrites_links_images_and_definitions_together() {
        let md = "\
![i](C:/Users/Kim/Pictures/a.png)
[notes](C:/Users/Kim/Documents/notes.md)
[ref]: C:/Users/Kim/Pictures/b.png \"Cap\"
[remote](https://example.com/x)
";
        let edits = plan_path_rebase(md, None, Path::new(r"C:\Users\Kim\Documents\doc.md"));
        assert_eq!(
            apply_edits(md, &edits),
            "\
![i](../Pictures/a.png)
[notes](./notes.md)
[ref]: ../Pictures/b.png \"Cap\"
[remote](https://example.com/x)
"
        );
    }

    #[test]
    fn smoke_rebase_moves_a_reference_definition_on_save_as() {
        let md = "[ref]: ../Pictures/a.png\n";
        let edits = plan_path_rebase(
            md,
            Some(Path::new("/home/kim/Documents/doc.md")),
            Path::new("/home/kim/Projects/deep/doc.md"),
        );
        assert_eq!(apply_edits(md, &edits), "[ref]: ../../Pictures/a.png\n");
    }

    #[test]
    fn smoke_rebase_leaves_in_document_anchors_alone() {
        let md = "[top](#heading) [also](#other)\n";
        let edits = plan_path_rebase(md, None, Path::new("/docs/doc.md"));
        assert!(edits.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn smoke_rebase_converts_absolute_path_on_first_save() {
        let md = "![test](/home/kim/Pictures/test.png)\n";
        let edits = plan_path_rebase(md, None, Path::new("/home/kim/Documents/test.md"));
        assert_eq!(apply_edits(md, &edits), "![test](../Pictures/test.png)\n");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn smoke_windows_rebase_converts_absolute_path_on_first_save() {
        let md = "![test](C:/Users/Kim/Pictures/test.png)\n";
        let edits = plan_path_rebase(md, None, Path::new(r"C:\Users\Kim\Documents\test.md"));
        assert_eq!(apply_edits(md, &edits), "![test](../Pictures/test.png)\n");
    }

    #[test]
    fn smoke_rebase_recomputes_relative_path_on_save_as() {
        let md = "![test](../Pictures/test.png)\n";
        let edits = plan_path_rebase(
            md,
            Some(Path::new("/home/kim/Documents/test.md")),
            Path::new("/home/kim/Projects/deep/test.md"),
        );
        assert_eq!(
            apply_edits(md, &edits),
            "![test](../../Pictures/test.png)\n"
        );
    }

    #[test]
    fn smoke_rebase_leaves_remote_data_and_missing_relative_alone() {
        let md = "![a](https://example.com/a.png) ![b](data:image/png;base64,AAA)\n";
        let edits = plan_path_rebase(md, None, Path::new("/docs/test.md"));
        assert!(edits.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn smoke_rebase_keeps_reference_to_a_deleted_image() {
        let md = "![gone](/home/kim/Pictures/deleted.png)\n";
        let edits = plan_path_rebase(md, None, Path::new("/home/kim/test.md"));
        assert_eq!(apply_edits(md, &edits), "![gone](./Pictures/deleted.png)\n");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn smoke_windows_rebase_keeps_reference_to_a_deleted_image() {
        let md = "![gone](C:/Users/Kim/Pictures/deleted.png)\n";
        let edits = plan_path_rebase(md, None, Path::new(r"C:\Users\Kim\test.md"));
        assert_eq!(apply_edits(md, &edits), "![gone](./Pictures/deleted.png)\n");
    }

    #[test]
    #[cfg(unix)]
    fn smoke_rebase_preserves_the_fragment() {
        let md = "![t](/docs/Images/test.png#something)\n";
        let edits = plan_path_rebase(md, None, Path::new("/docs/test.md"));
        assert_eq!(
            apply_edits(md, &edits),
            "![t](./Images/test.png#something)\n"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn smoke_windows_rebase_preserves_the_fragment() {
        let md = "![t](C:/docs/Images/test.png#something)\n";
        let edits = plan_path_rebase(md, None, Path::new(r"C:\docs\test.md"));
        assert_eq!(
            apply_edits(md, &edits),
            "![t](./Images/test.png#something)\n"
        );
    }

    #[test]
    fn smoke_rebase_is_a_no_op_when_paths_already_match() {
        let md = "![t](./a.png)\n";
        let edits = plan_path_rebase(
            md,
            Some(Path::new("/docs/old.md")),
            Path::new("/docs/new.md"),
        );
        assert!(edits.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn smoke_rebase_rewrites_multiple_images_in_order() {
        let md = "![a](/x/1.png)\n\n![b](/x/y/2.png)\n";
        let edits = plan_path_rebase(md, None, Path::new("/x/doc.md"));
        assert_eq!(edits.len(), 2);
        assert!(edits[0].start < edits[1].start);
        assert_eq!(
            apply_edits(md, &edits),
            "![a](./1.png)\n\n![b](./y/2.png)\n"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn smoke_windows_rebase_rewrites_multiple_images_in_order() {
        let md = "![a](C:/x/1.png)\n\n![b](C:/x/y/2.png)\n";
        let edits = plan_path_rebase(md, None, Path::new(r"C:\x\doc.md"));
        assert_eq!(edits.len(), 2);
        assert!(edits[0].start < edits[1].start);
        assert_eq!(
            apply_edits(md, &edits),
            "![a](./1.png)\n\n![b](./y/2.png)\n"
        );
    }

    #[test]
    #[cfg(unix)]
    fn smoke_rebase_replaces_the_whole_angle_bracketed_destination() {
        let md = "![t](</home/kim/My Pictures/a.png> \"title\")\n";
        let edits = plan_path_rebase(md, None, Path::new("/home/kim/Documents/t.md"));
        assert_eq!(
            apply_edits(md, &edits),
            "![t](../My%20Pictures/a.png \"title\")\n"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn smoke_windows_rebase_replaces_the_whole_angle_bracketed_destination() {
        let md = "![t](<C:/Users/Kim/My Pictures/a.png> \"title\")\n";
        let edits = plan_path_rebase(md, None, Path::new(r"C:\Users\Kim\Documents\t.md"));
        assert_eq!(
            apply_edits(md, &edits),
            "![t](../My%20Pictures/a.png \"title\")\n"
        );
    }

    #[test]
    fn smoke_rebase_leaves_unresolvable_relative_path_alone() {
        // Relative path in a document that was never saved: there is no
        // directory it could have been relative to, so it stays as written.
        let md = "![t](../Pictures/a.png)\n";
        let edits = plan_path_rebase(md, None, Path::new("/home/kim/Documents/t.md"));
        assert!(edits.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn smoke_rebase_skips_images_inside_a_fenced_block() {
        let md = "```\n![x](/home/kim/a.png)\n```\n![y](/home/kim/b.png)\n";
        let edits = plan_path_rebase(md, None, Path::new("/home/kim/t.md"));
        assert_eq!(
            apply_edits(md, &edits),
            "```\n![x](/home/kim/a.png)\n```\n![y](./b.png)\n"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn smoke_windows_rebase_skips_images_inside_a_fenced_block() {
        let md = "```\n![x](C:/Users/Kim/a.png)\n```\n![y](C:/Users/Kim/b.png)\n";
        let edits = plan_path_rebase(md, None, Path::new(r"C:\Users\Kim\t.md"));
        assert_eq!(
            apply_edits(md, &edits),
            "```\n![x](C:/Users/Kim/a.png)\n```\n![y](./b.png)\n"
        );
    }

    #[test]
    fn smoke_missing_targets_are_found_and_existing_ones_are_not() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(dir.path().join("files")).expect("mkdir");
        std::fs::write(dir.path().join("files/there.md"), "# there").expect("write");

        let md = "\
[ok](./files/there.md)
[gone](./files/gone.md)
![img](./files/missing.png)
[def]: ./files/gone-too.md
[remote](https://example.com/x.md)
[anchor](#section)
";
        let missing = find_missing_local_targets(md, Some(dir.path()));
        assert_eq!(
            missing
                .iter()
                .map(|m| m.markdown_path.as_str())
                .collect::<Vec<_>>(),
            vec![
                "./files/gone.md",
                "./files/missing.png",
                "./files/gone-too.md"
            ]
        );
        // The span covers exactly the destination token, ready to underline.
        assert_eq!(&md[missing[0].start..missing[0].end], "./files/gone.md");
        // A broken link and a broken image are reported differently, so the
        // syntax each came from has to survive the scan.
        assert_eq!(
            missing.iter().map(|m| m.origin).collect::<Vec<_>>(),
            vec![
                TargetOrigin::InlineLink,
                TargetOrigin::InlineImage,
                TargetOrigin::ReferenceDefinition
            ]
        );
    }

    #[test]
    fn smoke_missing_target_keeps_the_fragment_out_of_the_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("there.md"), "# there").expect("write");

        // The file exists; only the anchor is unverified, so this is not a
        // missing *target*.
        let missing = find_missing_local_targets("[x](./there.md#sec)\n", Some(dir.path()));
        assert!(missing.is_empty());
    }

    #[test]
    fn smoke_unsaved_document_reports_no_missing_relative_targets() {
        // Nothing to resolve against — judging these would flag every link in
        // a new document.
        let missing = find_missing_local_targets("[x](./nope.md)\n", None);
        assert!(missing.is_empty());
    }

    #[test]
    fn smoke_missing_targets_inside_code_are_ignored() {
        let dir = tempfile::tempdir().expect("tempdir");
        let md = "`[x](./nope.md)`\n\n```\n[y](./nope.md)\n```\n";
        assert!(find_missing_local_targets(md, Some(dir.path())).is_empty());
    }

    #[test]
    fn smoke_line_and_byte_column_is_one_based() {
        let text = "abc\nde✓f\n";
        assert_eq!(line_and_byte_column(text, 0), (1, 1));
        assert_eq!(line_and_byte_column(text, 2), (1, 3));
        assert_eq!(line_and_byte_column(text, 4), (2, 1));
        // `✓` is three bytes, so the column after it is a byte column.
        assert_eq!(line_and_byte_column(text, 9), (2, 6));
    }

    #[test]
    fn smoke_normalize_resolves_dot_segments() {
        assert_eq!(normalize(Path::new("/a/b/../c/./d")), p("/a/c/d"));
    }
}
