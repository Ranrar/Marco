//! Code fence language normalization.
//!
//! This module exists so both `marco` and `polo` can share:
//! - a small, curated list of common language aliases
//! - consistent display labels (e.g. `rs` -> `Rust`)
//!
//! The core renderer also uses this to populate `data-language` on `<pre>`
//! so CSS themes can show a proper label instead of a generic "Code".

use std::borrow::Cow;

#[derive(Debug, Clone, Copy)]
pub struct CodeLanguage {
    /// Canonical name used for display and (typically) for syntect token lookup.
    pub canonical: &'static str,
    /// Lowercase aliases commonly used in fenced code blocks.
    pub aliases: &'static [&'static str],
}

/// A small, human-curated set of common languages and their aliases.
///
/// Notes:
/// - Matching is ASCII-case-insensitive.
/// - Aliases should be lowercase.
pub const KNOWN_CODE_LANGUAGES: &[CodeLanguage] = &[
    CodeLanguage {
        canonical: "Rust",
        aliases: &["rs", "rust"],
    },
    CodeLanguage {
        canonical: "JavaScript",
        aliases: &["js", "javascript", "jsx", "mjs", "cjs", "node"],
    },
    CodeLanguage {
        canonical: "TypeScript",
        aliases: &["ts", "tsx", "mts", "cts", "typescript"],
    },
    CodeLanguage {
        canonical: "Python",
        aliases: &["py", "python", "python3", "pycon"],
    },
    CodeLanguage {
        canonical: "Bash",
        aliases: &["sh", "bash", "zsh", "shell"],
    },
    CodeLanguage {
        canonical: "HTML",
        aliases: &["html", "htm"],
    },
    CodeLanguage {
        canonical: "CSS",
        aliases: &["css"],
    },
    CodeLanguage {
        canonical: "JSON",
        aliases: &["json", "jsonc", "json5"],
    },
    CodeLanguage {
        canonical: "YAML",
        aliases: &["yaml", "yml"],
    },
    CodeLanguage {
        canonical: "TOML",
        aliases: &["toml"],
    },
    CodeLanguage {
        canonical: "XML",
        aliases: &["xml"],
    },
    CodeLanguage {
        canonical: "Markdown",
        aliases: &["md", "markdown", "mkd", "mkdown"],
    },
    CodeLanguage {
        canonical: "Mermaid",
        aliases: &["mermaid", "mmd"],
    },
    CodeLanguage {
        canonical: "SQL",
        aliases: &["sql"],
    },
    CodeLanguage {
        canonical: "C",
        aliases: &["c", "h"],
    },
    CodeLanguage {
        canonical: "C++",
        aliases: &["cpp", "c++", "cxx", "cc", "hpp", "hh", "h++", "hxx"],
    },
    CodeLanguage {
        canonical: "C#",
        aliases: &["cs", "c#", "csharp"],
    },
    CodeLanguage {
        canonical: "Java",
        aliases: &["java"],
    },
    CodeLanguage {
        canonical: "Dart",
        aliases: &["dart"],
    },
    CodeLanguage {
        canonical: "Go",
        aliases: &["go", "golang"],
    },
    CodeLanguage {
        canonical: "Scala",
        aliases: &["scala"],
    },
    CodeLanguage {
        canonical: "Groovy",
        aliases: &["groovy", "gradle"],
    },
    CodeLanguage {
        canonical: "Clojure",
        aliases: &["clojure", "clj", "edn"],
    },
    CodeLanguage {
        canonical: "Ruby",
        aliases: &["rb", "ruby"],
    },
    CodeLanguage {
        canonical: "Perl",
        aliases: &["perl", "pl", "pm"],
    },
    CodeLanguage {
        canonical: "PHP",
        aliases: &["php"],
    },
    CodeLanguage {
        canonical: "Haskell",
        aliases: &["haskell", "hs"],
    },
    CodeLanguage {
        canonical: "Elixir",
        aliases: &["elixir", "ex", "exs"],
    },
    CodeLanguage {
        canonical: "Erlang",
        aliases: &["erlang", "erl"],
    },
    CodeLanguage {
        canonical: "F#",
        aliases: &["fsharp", "fs", "fsx", "fsi", "fsscript"],
    },
    CodeLanguage {
        canonical: "Kotlin",
        aliases: &["kotlin", "kt"],
    },
    CodeLanguage {
        canonical: "Swift",
        aliases: &["swift"],
    },
    CodeLanguage {
        canonical: "Objective-C",
        aliases: &["objectivec", "objc", "obj-c", "mm"],
    },
    CodeLanguage {
        canonical: "OCaml",
        aliases: &["ocaml", "ml"],
    },
    CodeLanguage {
        canonical: "Lua",
        aliases: &["lua"],
    },
    CodeLanguage {
        canonical: "Nim",
        aliases: &["nim", "nimrod"],
    },
    CodeLanguage {
        canonical: "Nix",
        aliases: &["nix"],
    },
    CodeLanguage {
        canonical: "Zig",
        aliases: &["zig"],
    },
    CodeLanguage {
        canonical: "R",
        aliases: &["r"],
    },
    CodeLanguage {
        canonical: "Matlab",
        aliases: &["matlab"],
    },
    CodeLanguage {
        canonical: "Fortran",
        aliases: &["fortran", "f90", "f95"],
    },
    CodeLanguage {
        canonical: "GraphQL",
        aliases: &["graphql", "gql"],
    },
    CodeLanguage {
        canonical: "Protocol Buffers",
        aliases: &["proto", "protobuf"],
    },
    CodeLanguage {
        canonical: "Solidity",
        aliases: &["solidity", "sol"],
    },
    CodeLanguage {
        canonical: "Terraform (HCL)",
        aliases: &["terraform", "tf", "hcl"],
    },
    CodeLanguage {
        canonical: "INI",
        aliases: &["ini"],
    },
    CodeLanguage {
        canonical: "MathML",
        aliases: &["mathml", "katex"],
    },
    CodeLanguage {
        canonical: "Dockerfile",
        aliases: &["dockerfile", "docker"],
    },
    CodeLanguage {
        canonical: "Makefile",
        aliases: &["makefile", "make", "mk", "mak"],
    },
    CodeLanguage {
        canonical: "PowerShell",
        aliases: &["powershell", "pwsh", "ps", "ps1"],
    },
    CodeLanguage {
        canonical: "Vim Script",
        aliases: &["vim", "vimscript"],
    },
    CodeLanguage {
        canonical: "Assembly",
        aliases: &["asm", "nasm", "x86asm"],
    },
    CodeLanguage {
        canonical: "Plaintext",
        aliases: &["plaintext", "text", "txt"],
    },
    CodeLanguage {
        canonical: "Diff",
        aliases: &["diff", "patch"],
    },
];

/// If `raw` is a known language (by canonical name or alias), return its canonical name.
pub fn canonical_language_name(raw: &str) -> Option<&'static str> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }

    // Fast path: exact canonical match (case-insensitive).
    for lang in KNOWN_CODE_LANGUAGES {
        if raw.eq_ignore_ascii_case(lang.canonical) {
            return Some(lang.canonical);
        }
    }

    let lower = raw.to_ascii_lowercase();
    for lang in KNOWN_CODE_LANGUAGES {
        if lang.aliases.iter().any(|a| *a == lower) {
            return Some(lang.canonical);
        }
    }

    None
}

/// Returns a display label for a fenced code language.
///
/// - Known languages get a canonical, nicely-cased label (`rs` -> `Rust`).
/// - Unknown languages fall back to the trimmed original text (preserving user intent).
pub fn language_display_label<'a>(raw: &'a str) -> Option<Cow<'a, str>> {
    let raw_trimmed = raw.trim();
    if raw_trimmed.is_empty() {
        return None;
    }

    if let Some(canonical) = canonical_language_name(raw_trimmed) {
        return Some(Cow::Borrowed(canonical));
    }

    Some(Cow::Borrowed(raw_trimmed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_canonical_language_name_aliases() {
        assert_eq!(canonical_language_name("rs"), Some("Rust"));
        assert_eq!(canonical_language_name("Rust"), Some("Rust"));
        assert_eq!(canonical_language_name("JS"), Some("JavaScript"));
        assert_eq!(canonical_language_name("c++"), Some("C++"));
        assert_eq!(canonical_language_name("tsx"), Some("TypeScript"));
        assert_eq!(canonical_language_name("gql"), Some("GraphQL"));
        assert_eq!(canonical_language_name("proto"), Some("Protocol Buffers"));
        assert_eq!(canonical_language_name("tf"), Some("Terraform (HCL)"));
        assert_eq!(canonical_language_name("nimrod"), Some("Nim"));
        assert_eq!(canonical_language_name("ps"), Some("PowerShell"));
        assert_eq!(canonical_language_name("mmd"), Some("Mermaid"));
        assert_eq!(canonical_language_name("katex"), Some("MathML"));
    }

    #[test]
    fn smoke_test_language_display_label_unknown_falls_back() {
        assert_eq!(language_display_label("  mylang  ").unwrap(), "mylang");
    }
}
