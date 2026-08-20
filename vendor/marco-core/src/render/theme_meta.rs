/// Theme metadata: parses descriptive `--theme-*` custom properties
/// out of a theme token CSS file.
///
/// Theme files may declare these alongside their colour/font tokens in `:root`,
/// e.g.:
///
/// ```css
/// :root {
///   --theme-name: 'GitHub';
///   --theme-author: 'Kim Skov Rasmussen';
///   --theme-license: 'MIT';
///   --theme-version: '0.24.0';
///   --theme-description: 'GitHub-inspired theme (Primer palette)';
///   /* ...regular colour/font tokens... */
/// }
/// ```
///
/// Because these are ordinary CSS custom properties, they need no support from
/// [`base_css()`](super::base_css::base_css) — they simply carry data through
/// the same mechanism as every other token. Consumers (e.g. a theme picker UI)
/// call [`parse_theme_metadata`] on the raw theme CSS to read them back out,
/// instead of deriving a display name from the filename.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThemeMetadata {
    pub name: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

/// Extract [`ThemeMetadata`] from a theme's raw CSS text.
///
/// Looks for declarations of the form `--theme-<field>: '<value>';`
/// (single or double quotes). Missing or malformed declarations are left as
/// `None` rather than erroring — metadata is optional, descriptive data.
pub fn parse_theme_metadata(css: &str) -> ThemeMetadata {
    ThemeMetadata {
        name: extract_token(css, "--theme-name"),
        author: extract_token(css, "--theme-author"),
        license: extract_token(css, "--theme-license"),
        version: extract_token(css, "--theme-version"),
        description: extract_token(css, "--theme-description"),
    }
}

fn extract_token(css: &str, property: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(offset) = css[search_from..].find(property) {
        let start = search_from + offset;
        let after_name = &css[start + property.len()..];
        // Require the match to end the property name: next non-space char must
        // be ':' , not another identifier char (e.g. `-extra`) that would mean
        // this is actually a different, longer custom property name.
        let next_non_space = after_name.trim_start();
        if let Some(value) = next_non_space.strip_prefix(':') {
            let semicolon = value.find(';')?;
            let raw = value[..semicolon].trim();
            let quote = raw.chars().next()?;
            if quote == '\'' || quote == '"' {
                if let Some(rest) = raw.strip_prefix(quote) {
                    if let Some(unquoted) = rest.strip_suffix(quote) {
                        return Some(unquoted.trim().to_string());
                    }
                }
            }
            return None;
        }
        search_from = start + property.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_fields_single_quotes() {
        let css = r#"
:root {
  --theme-name: 'GitHub';
  --theme-author: 'Kim Skov Rasmussen';
  --theme-license: 'MIT';
  --theme-version: '0.24.0';
  --theme-description: 'GitHub-inspired theme';
  --text-color: #1f2328;
}
"#;
        let meta = parse_theme_metadata(css);
        assert_eq!(meta.name.as_deref(), Some("GitHub"));
        assert_eq!(meta.author.as_deref(), Some("Kim Skov Rasmussen"));
        assert_eq!(meta.license.as_deref(), Some("MIT"));
        assert_eq!(meta.version.as_deref(), Some("0.24.0"));
        assert_eq!(meta.description.as_deref(), Some("GitHub-inspired theme"));
    }

    #[test]
    fn parses_double_quotes() {
        let css = r#"--theme-name: "Nord";"#;
        assert_eq!(parse_theme_metadata(css).name.as_deref(), Some("Nord"));
    }

    #[test]
    fn missing_fields_are_none() {
        let css = ":root { --text-color: #333; }";
        let meta = parse_theme_metadata(css);
        assert_eq!(meta, ThemeMetadata::default());
    }

    #[test]
    fn does_not_match_longer_property_names_with_same_prefix() {
        let css = "--theme-name-extra: 'not-a-name'; --theme-name: 'Real';";
        let meta = parse_theme_metadata(css);
        assert_eq!(meta.name.as_deref(), Some("Real"));
    }
}
