use anyhow::Context;
use regex::Regex;
use std::path::Path;

use ron::Value;

/// Load a RON value and deserialize into T

pub fn load_ron<T: for<'de> serde::Deserialize<'de>>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    let s = std::fs::read_to_string(&path)
        .with_context(|| format!("reading RON file: {}", path.as_ref().display()))?;
    let v = load_ron_from_str(&s)
        .with_context(|| format!("deserializing RON: {}", path.as_ref().display()))?;
    Ok(v)
}

pub fn load_ron_from_str<T: for<'de> serde::Deserialize<'de>>(s: &str) -> anyhow::Result<T> {
    // Strip common fenced code blocks (```ron ... ``` or ``` ... ```)
    let mut s = s.to_string();
    // remove leading ```...\n block marker
    if let Some(idx) = s.find("```") {
        // remove first fence line
        if let Some(nl) = s[idx..].find('\n') {
            s.replace_range(idx..idx + nl + 1, "");
        }
        // remove trailing ``` if present
        if let Some(tidx) = s.rfind("```") {
            s.truncate(tidx);
        }
    }

    // Try direct deserialize first
    if let Ok(v) = ron::de::from_str::<T>(&s) {
        return Ok(v);
    }

    // Fallback: attempt to find the first brace/paren and prepend the target type name
    if let Some(pos) = s.find('{').or_else(|| s.find('(')) {
        let mut rest = s[pos..].to_string();
        // If rest starts with '{', convert surrounding braces to parentheses for RON struct form
        if rest.starts_with('{') {
            // replace first '{' with '('
            rest.replace_range(0..1, "(");
            // replace last '}' with ')'
            if let Some(last) = rest.rfind('}') {
                rest.replace_range(last..last + 1, ")");
            }
            // Ensure root `type: "x"` becomes `type: Some("x")` to match Option<String>
            // match patterns like: type: "name" and replace with type: Some("name")
            let type_re = Regex::new(r#"(?m)type\s*:\s*"([^"]+)""#)
                .unwrap_or_else(|_| Regex::new(r#"(?m)type\s*:\s*"([^"]+)""#).unwrap());
            if type_re.is_match(&rest) {
                rest = type_re.replace(&rest, r#"type: Some("$1")"#).to_string();
            }
        }
        let type_full = std::any::type_name::<T>();
        let type_name = type_full.split("::").last().unwrap_or(type_full);
        let wrapped = format!("{}{}", type_name, rest);
        eprintln!("DEBUG: attempting wrapped RON:\n{}", wrapped);
        let v2 = ron::de::from_str::<T>(&wrapped)
            .with_context(|| format!("deserializing RON from wrapped str"))?;
        return Ok(v2);
    }

    // final attempt: try direct parse to get a helpful error
    let v = ron::de::from_str::<T>(&s)
        .with_context(|| format!("deserializing RON from str final attempt"))?;
    Ok(v)
}

pub fn load_pest_grammar(path: impl AsRef<Path>) -> anyhow::Result<Vec<String>> {
    let s = std::fs::read_to_string(&path)
        .with_context(|| format!("reading grammar: {}", path.as_ref().display()))?;
    Ok(extract_rule_names_from_str(&s))
}

/// Load raw RON value from a file.
pub fn load_ron_value(path: impl AsRef<Path>) -> anyhow::Result<Value> {
    // First, try a streaming parse from the file which is often more tolerant
    // to top-level wrapper forms and comments.
    let p = path.as_ref();
    if let Ok(f) = std::fs::File::open(&p) {
        if let Ok(v) = ron::de::from_reader::<_, Value>(std::io::BufReader::new(f)) {
            return Ok(v);
        }
    }
    let s = std::fs::read_to_string(&path)
        .with_context(|| format!("reading RON file: {}", path.as_ref().display()))?;
    load_ron_value_from_str(&s)
}

/// Load raw RON value from a string (tries direct parse, or strips a leading wrapper)
pub fn load_ron_value_from_str(s: &str) -> anyhow::Result<Value> {
    if let Ok(v) = ron::de::from_str::<Value>(s) {
        return Ok(v);
    }
    // fallback: try to extract the first balanced `{...}` or `(...)` block and parse that
    fn find_matching(s: &str, open: char, close: char, start: usize) -> Option<usize> {
        let mut depth = 0usize;
        let mut in_string = false;
        let mut escape = false;
        for (i, ch) in s.char_indices().skip(start) {
            if in_string {
                if escape {
                    escape = false;
                    continue;
                }
                if ch == '\\' {
                    escape = true;
                    continue;
                }
                if ch == '"' {
                    in_string = false;
                    continue;
                }
                continue;
            } else {
                if ch == '"' {
                    in_string = true;
                    continue;
                }
                if ch == open {
                    depth += 1;
                } else if ch == close {
                    if depth == 0 {
                        return None;
                    }
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    if let Some(pos) = s.find('{').or_else(|| s.find('(')) {
        let open = s.chars().nth(pos).unwrap();
        let close = if open == '{' { '}' } else { ')' };
        if let Some(end) = find_matching(s, open, close, pos) {
            let snippet = &s[pos..=end];
            // try to capture a wrapper identifier immediately before the opening brace, e.g. `RootNode {`
            let mut wrapper_snippet = snippet.to_string();
            if pos > 0 {
                // scan backwards skipping whitespace
                let mut i = pos;
                while i > 0 && s.as_bytes()[i - 1].is_ascii_whitespace() {
                    i -= 1;
                }
                // now collect identifier chars [A-Za-z0-9_]
                let mut start = i;
                while start > 0
                    && (s.as_bytes()[start - 1].is_ascii_alphanumeric()
                        || s.as_bytes()[start - 1] == b'_')
                {
                    start -= 1;
                }
                if start < i {
                    let name = &s[start..i];
                    wrapper_snippet = format!("{} {}", name, snippet);
                }
            }
            eprintln!("DEBUG: extracted snippet ({}..={}):\n{}", pos, end, snippet);
            // try parsing using wrapper if present, else try raw snippet
            match ron::de::from_str::<Value>(&wrapper_snippet)
                .or_else(|_| ron::de::from_str::<Value>(snippet))
            {
                Ok(v2) => return Ok(v2),
                Err(e) => {
                    eprintln!("RON parse failed: {}", e);
                    // Try stripping C++/C-style single-line comments and retry,
                    // but preserve URLs like http:// or https:// and text inside strings.
                    let cleaned = strip_line_comments_preserve_urls(&wrapper_snippet);
                    // Normalize array children (insert missing commas between top-level elements)
                    let normalized = normalize_children_array(&cleaned);
                    // Preprocess struct-like variants inside children arrays: remove leading identifiers like `Heading {` -> `{`
                    let prepped = preprocess_struct_variants(&normalized);
                    eprintln!("DEBUG: attempting parse after comment-strip and normalization (prepped length={})", prepped.len());
                    // dump normalized snippet for offline inspection
                    let _ = std::fs::write("/tmp/normalized_ast.ron", &prepped);
                    // If preprocessing left a leading identifier (e.g., `RootNode {`), try stripping it
                    let prepped_stripped = if let Some(first_brace) = prepped.find('{') {
                        prepped[first_brace..].to_string()
                    } else {
                        prepped.clone()
                    };
                    match ron::de::from_str::<Value>(&prepped_stripped)
                        .or_else(|_| ron::de::from_str::<Value>(&prepped))
                    {
                        Ok(v3) => return Ok(v3),
                        Err(e2) => {
                            eprintln!("normalized parse failed: {}", e2);
                            // Attempt to parse line/col from error and show context
                            if let Some((ln, col)) = parse_error_line_col(&e2.to_string()) {
                                eprintln!("Error at normalized line {}, col {}. Context:", ln, col);
                                print_context_lines(&prepped_stripped, ln, 5);
                            }
                            return Err(anyhow::anyhow!(
                                "deserializing RON value from cleaned snippet: {}",
                                e2
                            ));
                        }
                    }
                }
            }
        }
    }
    Err(anyhow::anyhow!("unable to parse RON value from input"))
}

fn parse_error_line_col(s: &str) -> Option<(usize, usize)> {
    // look for pattern like "7:17" at start
    let re = Regex::new(r"(\d+):(\d+)").ok()?;
    if let Some(cap) = re.captures(s) {
        let ln = cap.get(1)?.as_str().parse().ok()?;
        let col = cap.get(2)?.as_str().parse().ok()?;
        return Some((ln, col));
    }
    None
}

fn print_context_lines(s: &str, line: usize, radius: usize) {
    let lines: Vec<&str> = s.lines().collect();
    let total = lines.len();
    let idx = if line == 0 { 0 } else { line - 1 };
    let start = if idx >= radius { idx - radius } else { 0 };
    let end = std::cmp::min(total, idx + radius + 1);
    for i in start..end {
        let prefix = if i == idx { ">" } else { " " };
        eprintln!("{} {:4} | {}", prefix, i + 1, lines[i]);
    }
}

fn strip_line_comments_preserve_urls(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut in_string = false;
    let mut string_delim = '\0';
    while let Some(ch) = chars.next() {
        if in_string {
            out.push(ch);
            if ch == '\\' {
                // escape next char if any
                if let Some(n) = chars.next() {
                    out.push(n);
                }
                continue;
            }
            if ch == string_delim {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            string_delim = ch;
            out.push(ch);
            continue;
        }
        if ch == '/' {
            if let Some('/') = chars.peek() {
                // lookbehind: peek last char in out (if any)
                let prev = out.chars().rev().next();
                // if previous char is ':' then this is likely part of a scheme like http://
                if prev == Some(':') {
                    // keep the '//' and continue copying until end of line
                    out.push('/');
                    // consume the second '/'
                    chars.next();
                    out.push('/');
                    continue;
                }
                // Otherwise this is a line comment: skip until end of line
                // consume the second '/'
                chars.next();
                // skip until newline or EOF
                while let Some(nc) = chars.next() {
                    if nc == '\n' {
                        out.push('\n');
                        break;
                    }
                }
                continue;
            } else {
                out.push(ch);
                continue;
            }
        }
        out.push(ch);
    }
    out
}

// Normalize `children: [ ... ]` arrays by inserting commas between adjacent top-level elements
fn normalize_children_array(s: &str) -> String {
    let mut out = s.to_string();
    let needle = "children: [";
    let mut search_start = 0usize;
    while let Some(rel) = out[search_start..].find(needle) {
        let start = search_start + rel;
        // find the '[' position
        if let Some(open_pos_rel) = out[start..].find('[') {
            let open_pos = start + open_pos_rel;
            if let Some(close_pos) = find_matching_bracket(&out, open_pos) {
                let inner = out[open_pos + 1..close_pos].to_string();
                let fixed_inner = insert_commas_between_top_level_elements(&inner);
                out.replace_range(open_pos + 1..close_pos, &fixed_inner);
                search_start = open_pos + 1 + fixed_inner.len();
                continue;
            }
        }
        break;
    }
    out
}

fn find_matching_bracket(s: &str, open_pos: usize) -> Option<usize> {
    let mut chars = s.char_indices().skip_while(|&(i, _)| i < open_pos);
    // consume the opening '['
    let (_, open_ch) = chars.next()?;
    let mut depth: isize = 0;
    let mut in_string = false;
    let mut string_delim = '\0';
    let mut escape = false;
    for (i, ch) in chars {
        if in_string {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == string_delim {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            string_delim = ch;
            continue;
        }
        match ch {
            '[' => depth += 1,
            ']' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

fn insert_commas_between_top_level_elements(inner: &str) -> String {
    let mut out = String::with_capacity(inner.len() + 64);
    let bytes = inner.as_bytes();
    let len = inner.len();
    let mut i = 0usize;
    let mut nesting: isize = 0;
    let mut in_string = false;
    let mut string_delim = 0u8;
    let mut escape = false;

    while i < len {
        let ch = bytes[i] as char;

        // At top-level (nesting==0) and not in string, if we see an identifier followed by '{',
        // treat it as a struct variant and drop the identifier so the inner map remains.
        if nesting == 0 && !in_string && (ch.is_ascii_alphabetic() || ch == '_') {
            // parse identifier
            let mut j = i;
            while j < len {
                let c = bytes[j] as char;
                if c.is_ascii_alphanumeric() || c == '_' {
                    j += 1;
                } else {
                    break;
                }
            }
            // skip whitespace
            let mut k = j;
            while k < len {
                let c = bytes[k] as char;
                if c.is_whitespace() {
                    k += 1;
                    continue;
                }
                break;
            }
            if k < len && bytes[k] as char == '{' {
                // skip identifier and whitespace, continue loop to process '{'
                i = k;
                continue;
            }
            // else fallthrough to normal processing
        }

        // push current char
        out.push(ch);
        i += ch.len_utf8();

        if in_string {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch as u8 == string_delim {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            string_delim = ch as u8;
            continue;
        }
        match ch {
            '{' | '[' | '(' => nesting += 1,
            '}' | ']' | ')' => {
                if nesting > 0 {
                    nesting -= 1;
                }
                if nesting == 0 {
                    // peek ahead skipping whitespace
                    let mut j = i;
                    while j < len {
                        let nc = bytes[j] as char;
                        if nc.is_whitespace() {
                            j += nc.len_utf8();
                            continue;
                        }
                        if nc == ',' || nc == ']' {
                            break;
                        }
                        // insert comma before the next top-level element
                        out.push(',');
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Extract rule names from a pest grammar text by scanning for `<name> =` patterns.
pub fn extract_rule_names_from_str(s: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap();
    let mut set = std::collections::BTreeSet::new();
    for cap in re.captures_iter(s) {
        if let Some(m) = cap.get(1) {
            set.insert(m.as_str().to_string());
        }
    }
    set.into_iter().collect()
}

/// Extract node kind names from an AST RON text using heuristics.
pub fn extract_kinds_from_ast_str(s: &str) -> Vec<String> {
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();

    // 1) explicit `type: "name"` fields
    let re_type = Regex::new(r#"type\s*:\s*\"([^\"]+)\""#).unwrap();
    for cap in re_type.captures_iter(s) {
        if let Some(m) = cap.get(1) {
            set.insert(m.as_str().to_string());
        }
    }

    // 2) scan children arrays and capture variant identifiers like `Heading { ... }` and quoted strings
    let re_children = Regex::new(r"children\s*:\s*\[").unwrap();
    let mut search = 0usize;
    while let Some(pos) = s[search..].find("children") {
        let abs = search + pos;
        // find '[' after this occurrence
        if let Some(br) = s[abs..].find('[') {
            let open = abs + br;
            if let Some(close) = find_matching_bracket(s, open) {
                let inner = &s[open + 1..close];
                // quoted kinds
                let re_q = Regex::new(r#"\"([^\"]+)\""#).unwrap();
                for c in re_q.captures_iter(inner) {
                    if let Some(m) = c.get(1) {
                        set.insert(m.as_str().to_string());
                    }
                }
                // variant identifiers before '{'
                let re_var = Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\{").unwrap();
                for vcap in re_var.captures_iter(inner) {
                    if let Some(vn) = vcap.get(1) {
                        // map `Heading` -> `heading` to match expected type strings
                        set.insert(vn.as_str().to_ascii_lowercase());
                    }
                }
                search = close + 1;
                continue;
            }
        }
        break;
    }

    // 3) fallback: capture any remaining bare words that look like `Heading {` outside arrays
    let re_var_global = Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\{").unwrap();
    for cap in re_var_global.captures_iter(s) {
        if let Some(m) = cap.get(1) {
            set.insert(m.as_str().to_ascii_lowercase());
        }
    }

    set.into_iter().collect()
}

/// Similar extraction for `syntax.ron` files (same heuristics)
pub fn extract_kinds_from_syntax_str(s: &str) -> Vec<String> {
    extract_kinds_from_ast_str(s)
}

/// Build a ron::Value map with a `children: ["kind", ...]` entry from kinds.
pub fn value_from_kinds<I>(kinds: I) -> Value
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    use ron::value::Map;
    let seq = kinds
        .into_iter()
        .map(|k| Value::String(k.into()))
        .collect::<Vec<_>>();
    let mut m = Map::new();
    m.insert(Value::String("children".to_string()), Value::Seq(seq));
    Value::Map(m)
}

// Preprocess children arrays: remove variant names before `{` and ensure commas
fn preprocess_struct_variants(s: &str) -> String {
    let mut out = s.to_string();
    let needle = "children: [";
    let mut search_start = 0usize;
    while let Some(rel) = out[search_start..].find(needle) {
        let start = search_start + rel;
        if let Some(open_pos_rel) = out[start..].find('[') {
            let open_pos = start + open_pos_rel;
            if let Some(close_pos) = find_matching_bracket(&out, open_pos) {
                let inner = &out[open_pos + 1..close_pos];
                let fixed = strip_variants_and_fix_commas(inner);
                out.replace_range(open_pos + 1..close_pos, &fixed);
                search_start = open_pos + 1 + fixed.len();
                continue;
            }
        }
        break;
    }
    out
}

fn strip_variants_and_fix_commas(inner: &str) -> String {
    let mut out = String::with_capacity(inner.len());
    let bytes = inner.as_bytes();
    let mut i = 0usize;
    let len = inner.len();
    let mut nesting: isize = 0;
    let mut in_string = false;
    let mut string_delim = 0u8;
    let mut escape = false;

    while i < len {
        let ch = bytes[i] as char;
        if in_string {
            out.push(ch);
            i += ch.len_utf8();
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch as u8 == string_delim {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            string_delim = ch as u8;
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        if nesting == 0 && (ch.is_ascii_alphabetic() || ch == '_') {
            // possible variant
            let mut j = i;
            while j < len {
                let c = bytes[j] as char;
                if c.is_ascii_alphanumeric() || c == '_' {
                    j += 1;
                } else {
                    break;
                }
            }
            let mut k = j;
            while k < len && (bytes[k] as char).is_whitespace() {
                k += 1;
            }
            if k < len && (bytes[k] as char) == '{' {
                // skip identifier and continue to process '{'
                i = k;
                continue;
            }
        }
        // copy char
        out.push(ch);
        i += ch.len_utf8();
        match ch {
            '{' | '[' | '(' => nesting += 1,
            '}' | ']' | ')' => {
                if nesting > 0 {
                    nesting -= 1;
                }
                if nesting == 0 {
                    // ensure comma between top-level elements
                    let mut j = i;
                    while j < len && (bytes[j] as char).is_whitespace() {
                        j += 1;
                    }
                    if j < len {
                        let nc = bytes[j] as char;
                        if nc != ',' && nc != ']' {
                            out.push(',');
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}
