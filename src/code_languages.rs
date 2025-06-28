use regex::Regex;
use std::collections::HashMap;

/// Represents a programming language with syntax highlighting rules
#[derive(Debug, Clone)]
pub struct CodeLanguage {
    pub name: String,
    pub aliases: Vec<String>,
    pub file_extensions: Vec<String>,
    pub keywords: Vec<String>,
    pub comment_patterns: Vec<String>,
    pub string_patterns: Vec<String>,
    pub number_pattern: String,
    pub function_pattern: Option<String>,
    pub class_pattern: Option<String>,
    pub color_scheme: LanguageColorScheme,
}

/// Color scheme for syntax highlighting
#[derive(Debug, Clone)]
pub struct LanguageColorScheme {
    pub keyword_color: String,
    pub comment_color: String,
    pub string_color: String,
    pub number_color: String,
    pub function_color: String,
    pub class_color: String,
    pub background_color: String,
    pub text_color: String,
}

impl Default for LanguageColorScheme {
    fn default() -> Self {
        Self {
            keyword_color: "#d73a49".to_string(),    // Red
            comment_color: "#6a737d".to_string(),    // Gray
            string_color: "#032f62".to_string(),     // Dark blue
            number_color: "#005cc5".to_string(),     // Blue
            function_color: "#6f42c1".to_string(),   // Purple
            class_color: "#e36209".to_string(),      // Orange
            background_color: "#f6f8fa".to_string(), // Light gray
            text_color: "#24292e".to_string(),       // Dark gray
        }
    }
}

/// Manager for code languages and syntax highlighting
pub struct CodeLanguageManager {
    languages: HashMap<String, CodeLanguage>,
    language_patterns: HashMap<String, Vec<Regex>>,
}

impl CodeLanguageManager {
    pub fn new() -> Self {
        let mut manager = Self {
            languages: HashMap::new(),
            language_patterns: HashMap::new(),
        };
        
        // Initialize with top 10 most popular programming languages
        manager.initialize_default_languages();
        
        manager
    }
    
    /// Initialize the top 10 most popular programming languages
    fn initialize_default_languages(&mut self) {
        // 1. JavaScript
        self.add_language(CodeLanguage {
            name: "JavaScript".to_string(),
            aliases: vec!["js".to_string(), "javascript".to_string(), "node".to_string()],
            file_extensions: vec![".js".to_string(), ".mjs".to_string(), ".jsx".to_string()],
            keywords: vec![
                "async", "await", "break", "case", "catch", "class", "const", "continue",
                "debugger", "default", "delete", "do", "else", "export", "extends", "finally",
                "for", "function", "if", "import", "in", "instanceof", "let", "new", "return",
                "super", "switch", "this", "throw", "try", "typeof", "var", "void", "while",
                "with", "yield", "true", "false", "null", "undefined"
            ].iter().map(|s| s.to_string()).collect(),
            comment_patterns: vec!["//.*$".to_string(), "/\\*[\\s\\S]*?\\*/".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string(), "`[^`\\\\]*(?:\\\\.[^`\\\\]*)*`".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[eE][+-]?\\d+)?\\b".to_string(),
            function_pattern: Some("\\b[a-zA-Z_$][a-zA-Z0-9_$]*\\s*(?=\\()".to_string()),
            class_pattern: Some("\\bclass\\s+([a-zA-Z_$][a-zA-Z0-9_$]*)".to_string()),
            color_scheme: LanguageColorScheme::default(),
        });
        
        // 2. Python
        self.add_language(CodeLanguage {
            name: "Python".to_string(),
            aliases: vec!["py".to_string(), "python".to_string(), "python3".to_string()],
            file_extensions: vec![".py".to_string(), ".pyw".to_string(), ".pyi".to_string()],
            keywords: vec![
                "and", "as", "assert", "break", "class", "continue", "def", "del", "elif",
                "else", "except", "exec", "finally", "for", "from", "global", "if", "import",
                "in", "is", "lambda", "not", "or", "pass", "print", "raise", "return", "try",
                "while", "with", "yield", "True", "False", "None", "async", "await"
            ].iter().map(|s| s.to_string()).collect(),
            comment_patterns: vec!["#.*$".to_string(), "\"\"\"[\\s\\S]*?\"\"\"".to_string(), "'''[\\s\\S]*?'''".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[eE][+-]?\\d+)?\\b".to_string(),
            function_pattern: Some("\\bdef\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            class_pattern: Some("\\bclass\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            color_scheme: LanguageColorScheme::default(),
        });
        
        // 3. Java
        self.add_language(CodeLanguage {
            name: "Java".to_string(),
            aliases: vec!["java".to_string()],
            file_extensions: vec![".java".to_string()],
            keywords: vec![
                "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char",
                "class", "const", "continue", "default", "do", "double", "else", "enum",
                "extends", "final", "finally", "float", "for", "goto", "if", "implements",
                "import", "instanceof", "int", "interface", "long", "native", "new", "package",
                "private", "protected", "public", "return", "short", "static", "strictfp",
                "super", "switch", "synchronized", "this", "throw", "throws", "transient",
                "try", "void", "volatile", "while", "true", "false", "null"
            ].iter().map(|s| s.to_string()).collect(),
            comment_patterns: vec!["//.*$".to_string(), "/\\*[\\s\\S]*?\\*/".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[fFdDlL])?\\b".to_string(),
            function_pattern: Some("\\b[a-zA-Z_][a-zA-Z0-9_]*\\s*(?=\\()".to_string()),
            class_pattern: Some("\\bclass\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            color_scheme: LanguageColorScheme::default(),
        });
        
        // 4. TypeScript
        self.add_language(CodeLanguage {
            name: "TypeScript".to_string(),
            aliases: vec!["ts".to_string(), "typescript".to_string()],
            file_extensions: vec![".ts".to_string(), ".tsx".to_string()],
            keywords: vec![
                "abstract", "any", "as", "async", "await", "boolean", "break", "case", "catch",
                "class", "const", "continue", "debugger", "declare", "default", "delete", "do",
                "else", "enum", "export", "extends", "false", "finally", "for", "from", "function",
                "get", "if", "implements", "import", "in", "instanceof", "interface", "is", "let",
                "module", "namespace", "never", "new", "null", "number", "object", "of", "package",
                "private", "protected", "public", "readonly", "return", "set", "static", "string",
                "super", "switch", "symbol", "this", "throw", "true", "try", "type", "typeof",
                "undefined", "unique", "unknown", "var", "void", "while", "with", "yield"
            ].iter().map(|s| s.to_string()).collect(),
            comment_patterns: vec!["//.*$".to_string(), "/\\*[\\s\\S]*?\\*/".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string(), "`[^`\\\\]*(?:\\\\.[^`\\\\]*)*`".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[eE][+-]?\\d+)?\\b".to_string(),
            function_pattern: Some("\\b[a-zA-Z_$][a-zA-Z0-9_$]*\\s*(?=\\()".to_string()),
            class_pattern: Some("\\bclass\\s+([a-zA-Z_$][a-zA-Z0-9_$]*)".to_string()),
            color_scheme: LanguageColorScheme::default(),
        });
        
        // 5. C#
        self.add_language(CodeLanguage {
            name: "C#".to_string(),
            aliases: vec!["cs".to_string(), "csharp".to_string(), "c#".to_string()],
            file_extensions: vec![".cs".to_string()],
            keywords: vec![
                "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char",
                "checked", "class", "const", "continue", "decimal", "default", "delegate",
                "do", "double", "else", "enum", "event", "explicit", "extern", "false",
                "finally", "fixed", "float", "for", "foreach", "goto", "if", "implicit",
                "in", "int", "interface", "internal", "is", "lock", "long", "namespace",
                "new", "null", "object", "operator", "out", "override", "params", "private",
                "protected", "public", "readonly", "ref", "return", "sbyte", "sealed",
                "short", "sizeof", "stackalloc", "static", "string", "struct", "switch",
                "this", "throw", "true", "try", "typeof", "uint", "ulong", "unchecked",
                "unsafe", "ushort", "using", "virtual", "void", "volatile", "while"
            ].iter().map(|s| s.to_string()).collect(),
            comment_patterns: vec!["//.*$".to_string(), "/\\*[\\s\\S]*?\\*/".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[fFdDmM])?\\b".to_string(),
            function_pattern: Some("\\b[a-zA-Z_][a-zA-Z0-9_]*\\s*(?=\\()".to_string()),
            class_pattern: Some("\\bclass\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            color_scheme: LanguageColorScheme::default(),
        });
        
        // 6. PHP
        self.add_language(CodeLanguage {
            name: "PHP".to_string(),
            aliases: vec!["php".to_string()],
            file_extensions: vec![".php".to_string(), ".phtml".to_string(), ".php3".to_string(), ".php4".to_string(), ".php5".to_string()],
            keywords: vec![
                "abstract", "and", "array", "as", "break", "callable", "case", "catch", "class",
                "clone", "const", "continue", "declare", "default", "die", "do", "echo", "else",
                "elseif", "empty", "enddeclare", "endfor", "endforeach", "endif", "endswitch",
                "endwhile", "eval", "exit", "extends", "final", "finally", "for", "foreach",
                "function", "global", "goto", "if", "implements", "include", "include_once",
                "instanceof", "insteadof", "interface", "isset", "list", "namespace", "new",
                "or", "print", "private", "protected", "public", "require", "require_once",
                "return", "static", "switch", "throw", "trait", "try", "unset", "use", "var",
                "while", "xor", "yield", "true", "false", "null"
            ].iter().map(|s| s.to_string()).collect(),
            comment_patterns: vec!["//.*$".to_string(), "#.*$".to_string(), "/\\*[\\s\\S]*?\\*/".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[eE][+-]?\\d+)?\\b".to_string(),
            function_pattern: Some("\\bfunction\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            class_pattern: Some("\\bclass\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            color_scheme: LanguageColorScheme::default(),
        });
        
        // 7. C++
        self.add_language(CodeLanguage {
            name: "C++".to_string(),
            aliases: vec!["cpp".to_string(), "c++".to_string(), "cxx".to_string()],
            file_extensions: vec![".cpp".to_string(), ".cxx".to_string(), ".cc".to_string(), ".hpp".to_string(), ".h++".to_string()],
            keywords: vec![
                "alignas", "alignof", "and", "and_eq", "asm", "auto", "bitand", "bitor",
                "bool", "break", "case", "catch", "char", "char16_t", "char32_t", "class",
                "compl", "const", "constexpr", "const_cast", "continue", "decltype", "default",
                "delete", "do", "double", "dynamic_cast", "else", "enum", "explicit", "export",
                "extern", "false", "float", "for", "friend", "goto", "if", "inline", "int",
                "long", "mutable", "namespace", "new", "noexcept", "not", "not_eq", "nullptr",
                "operator", "or", "or_eq", "private", "protected", "public", "register",
                "reinterpret_cast", "return", "short", "signed", "sizeof", "static",
                "static_assert", "static_cast", "struct", "switch", "template", "this",
                "thread_local", "throw", "true", "try", "typedef", "typeid", "typename",
                "union", "unsigned", "using", "virtual", "void", "volatile", "wchar_t",
                "while", "xor", "xor_eq"
            ].iter().map(|s| s.to_string()).collect(),
            comment_patterns: vec!["//.*$".to_string(), "/\\*[\\s\\S]*?\\*/".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[fFlLuU]*)?\\b".to_string(),
            function_pattern: Some("\\b[a-zA-Z_][a-zA-Z0-9_]*\\s*(?=\\()".to_string()),
            class_pattern: Some("\\bclass\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            color_scheme: LanguageColorScheme::default(),
        });
        
        // 8. C
        self.add_language(CodeLanguage {
            name: "C".to_string(),
            aliases: vec!["c".to_string()],
            file_extensions: vec![".c".to_string(), ".h".to_string()],
            keywords: vec![
                "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
                "else", "enum", "extern", "float", "for", "goto", "if", "inline", "int", "long",
                "register", "restrict", "return", "short", "signed", "sizeof", "static", "struct",
                "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "_Bool",
                "_Complex", "_Imaginary"
            ].iter().map(|s| s.to_string()).collect(),
            comment_patterns: vec!["//.*$".to_string(), "/\\*[\\s\\S]*?\\*/".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[fFlLuU]*)?\\b".to_string(),
            function_pattern: Some("\\b[a-zA-Z_][a-zA-Z0-9_]*\\s*(?=\\()".to_string()),
            class_pattern: None,
            color_scheme: LanguageColorScheme::default(),
        });
        
        // 9. Go
        self.add_language(CodeLanguage {
            name: "Go".to_string(),
            aliases: vec!["go".to_string(), "golang".to_string()],
            file_extensions: vec![".go".to_string()],
            keywords: vec![
                "break", "case", "chan", "const", "continue", "default", "defer", "else",
                "fallthrough", "for", "func", "go", "goto", "if", "import", "interface",
                "map", "package", "range", "return", "select", "struct", "switch", "type",
                "var", "true", "false", "iota", "nil"
            ].iter().map(|s| s.to_string()).collect(),
            comment_patterns: vec!["//.*$".to_string(), "/\\*[\\s\\S]*?\\*/".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "`[^`]*`".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[eE][+-]?\\d+)?\\b".to_string(),
            function_pattern: Some("\\bfunc\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            class_pattern: Some("\\btype\\s+([a-zA-Z_][a-zA-Z0-9_]*)\\s+struct".to_string()),
            color_scheme: LanguageColorScheme::default(),
        });
        
        // 10. Rust
        self.add_language(CodeLanguage {
            name: "Rust".to_string(),
            aliases: vec!["rust".to_string(), "rs".to_string()],
            file_extensions: vec![".rs".to_string()],
            keywords: vec![
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
                "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static",
                "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while",
                "abstract", "become", "box", "do", "final", "macro", "override", "priv",
                "typeof", "unsized", "virtual", "yield"
            ].iter().map(|s| s.to_string()).collect(),
            comment_patterns: vec!["//.*$".to_string(), "/\\*[\\s\\S]*?\\*/".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?(?:[fFiIuU]\\d*)?\\b".to_string(),
            function_pattern: Some("\\bfn\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            class_pattern: Some("\\bstruct\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            color_scheme: LanguageColorScheme::default(),
        });
    }
    
    /// Add a new language to the manager
    pub fn add_language(&mut self, language: CodeLanguage) {
        let mut patterns = Vec::new();
        
        // Compile regex patterns for this language
        if let Ok(keyword_regex) = Regex::new(&format!("\\b({})\\b", language.keywords.join("|"))) {
            patterns.push(keyword_regex);
        }
        
        for comment_pattern in &language.comment_patterns {
            if let Ok(regex) = Regex::new(comment_pattern) {
                patterns.push(regex);
            }
        }
        
        for string_pattern in &language.string_patterns {
            if let Ok(regex) = Regex::new(string_pattern) {
                patterns.push(regex);
            }
        }
        
        if let Ok(number_regex) = Regex::new(&language.number_pattern) {
            patterns.push(number_regex);
        }
        
        if let Some(function_pattern) = &language.function_pattern {
            if let Ok(regex) = Regex::new(function_pattern) {
                patterns.push(regex);
            }
        }
        
        if let Some(class_pattern) = &language.class_pattern {
            if let Ok(regex) = Regex::new(class_pattern) {
                patterns.push(regex);
            }
        }
        
        // Store language with all its aliases
        let language_name = language.name.clone();
        for alias in &language.aliases {
            self.languages.insert(alias.clone(), language.clone());
        }
        self.languages.insert(language_name.clone(), language.clone());
        
        self.language_patterns.insert(language_name, patterns);
    }
    
    /// Get a language by name or alias
    pub fn get_language(&self, name: &str) -> Option<&CodeLanguage> {
        self.languages.get(&name.to_lowercase())
    }
    
    /// Get all available language names
    pub fn get_language_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.languages.keys()
            .filter(|name| {
                // Only return primary names, not aliases
                if let Some(lang) = self.languages.get(*name) {
                    &lang.name.to_lowercase() == *name
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        names.sort();
        names
    }
    
    /// Get language suggestions based on partial input
    pub fn get_language_suggestions(&self, partial: &str) -> Vec<String> {
        let partial_lower = partial.to_lowercase();
        let mut suggestions: Vec<String> = self.languages.keys()
            .filter(|name| name.starts_with(&partial_lower))
            .cloned()
            .collect();
        suggestions.sort();
        suggestions.truncate(10); // Limit to 10 suggestions
        suggestions
    }
    
    /// Detect language from file extension
    pub fn detect_language_from_extension(&self, filename: &str) -> Option<&CodeLanguage> {
        let extension = std::path::Path::new(filename)
            .extension()?
            .to_str()?;
        let ext_with_dot = format!(".{}", extension);
        
        for language in self.languages.values() {
            if language.file_extensions.contains(&ext_with_dot) {
                return Some(language);
            }
        }
        None
    }
    
    /// Generate CSS for syntax highlighting
    pub fn generate_css_for_language(&self, language_name: &str) -> String {
        if let Some(language) = self.get_language(language_name) {
            format!(
                r#"
                .code-block-{} {{
                    background-color: {};
                    color: {};
                    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
                    font-size: 0.9em;
                    line-height: 1.4;
                    padding: 12px;
                    border-radius: 6px;
                    border: 1px solid #e1e4e8;
                    overflow-x: auto;
                }}
                
                .code-block-{} .keyword {{
                    color: {};
                    font-weight: bold;
                }}
                
                .code-block-{} .comment {{
                    color: {};
                    font-style: italic;
                }}
                
                .code-block-{} .string {{
                    color: {};
                }}
                
                .code-block-{} .number {{
                    color: {};
                }}
                
                .code-block-{} .function {{
                    color: {};
                    font-weight: bold;
                }}
                
                .code-block-{} .class {{
                    color: {};
                    font-weight: bold;
                }}
                "#,
                language_name.to_lowercase(),
                language.color_scheme.background_color,
                language.color_scheme.text_color,
                language_name.to_lowercase(),
                language.color_scheme.keyword_color,
                language_name.to_lowercase(),
                language.color_scheme.comment_color,
                language_name.to_lowercase(),
                language.color_scheme.string_color,
                language_name.to_lowercase(),
                language.color_scheme.number_color,
                language_name.to_lowercase(),
                language.color_scheme.function_color,
                language_name.to_lowercase(),
                language.color_scheme.class_color
            )
        } else {
            String::new()
        }
    }
    
    /// Apply syntax highlighting to code text
    pub fn highlight_code(&self, code: &str, language_name: &str) -> String {
        if let Some(language) = self.get_language(language_name) {
            let mut highlighted_code = Self::html_escape(code);
            
            // Apply keyword highlighting
            if !language.keywords.is_empty() {
                let keyword_pattern = format!(r"\b({})\b", language.keywords.join("|"));
                if let Ok(keyword_regex) = Regex::new(&keyword_pattern) {
                    highlighted_code = keyword_regex.replace_all(&highlighted_code, 
                        r#"<span class="keyword">$1</span>"#).to_string();
                }
            }
            
            // Apply comment highlighting
            for comment_pattern in &language.comment_patterns {
                if let Ok(comment_regex) = Regex::new(comment_pattern) {
                    highlighted_code = comment_regex.replace_all(&highlighted_code, 
                        r#"<span class="comment">$0</span>"#).to_string();
                }
            }
            
            // Apply string highlighting
            for string_pattern in &language.string_patterns {
                if let Ok(string_regex) = Regex::new(string_pattern) {
                    highlighted_code = string_regex.replace_all(&highlighted_code, 
                        r#"<span class="string">$0</span>"#).to_string();
                }
            }
            
            // Apply number highlighting
            if let Ok(number_regex) = Regex::new(&language.number_pattern) {
                highlighted_code = number_regex.replace_all(&highlighted_code, 
                    r#"<span class="number">$0</span>"#).to_string();
            }
            
            // Apply function highlighting
            if let Some(function_pattern) = &language.function_pattern {
                if let Ok(function_regex) = Regex::new(function_pattern) {
                    highlighted_code = function_regex.replace_all(&highlighted_code, 
                        r#"<span class="function">$0</span>"#).to_string();
                }
            }
            
            // Apply class highlighting
            if let Some(class_pattern) = &language.class_pattern {
                if let Ok(class_regex) = Regex::new(class_pattern) {
                    highlighted_code = class_regex.replace_all(&highlighted_code, 
                        r#"<span class="class">$0</span>"#).to_string();
                }
            }
            
            format!(
                r#"<div class="code-block code-block-{}">{}</div>"#,
                language_name.to_lowercase(),
                highlighted_code
            )
        } else {
            format!(r#"<div class="code-block code-block-plain">{}</div>"#, Self::html_escape(code))
        }
    }
    
    /// Create a new custom language with basic defaults
    pub fn create_custom_language(
        name: String,
        aliases: Vec<String>,
        file_extensions: Vec<String>,
        keywords: Vec<String>,
    ) -> CodeLanguage {
        CodeLanguage {
            name,
            aliases,
            file_extensions,
            keywords,
            comment_patterns: vec!["//.*$".to_string(), "/\\*[\\s\\S]*?\\*/".to_string()],
            string_patterns: vec!["\"[^\"\\\\]*(?:\\\\.[^\"\\\\]*)*\"".to_string(), "'[^'\\\\]*(?:\\\\.[^'\\\\]*)*'".to_string()],
            number_pattern: "\\b\\d+(?:\\.\\d+)?\\b".to_string(),
            function_pattern: Some("\\b[a-zA-Z_][a-zA-Z0-9_]*\\s*(?=\\()".to_string()),
            class_pattern: Some("\\bclass\\s+([a-zA-Z_][a-zA-Z0-9_]*)".to_string()),
            color_scheme: LanguageColorScheme::default(),
        }
    }
    
    /// Validate that a language definition has proper regex patterns
    pub fn validate_language(&self, language: &CodeLanguage) -> Result<(), String> {
        // Validate comment patterns
        for pattern in &language.comment_patterns {
            if Regex::new(pattern).is_err() {
                return Err(format!("Invalid comment pattern: {}", pattern));
            }
        }
        
        // Validate string patterns
        for pattern in &language.string_patterns {
            if Regex::new(pattern).is_err() {
                return Err(format!("Invalid string pattern: {}", pattern));
            }
        }
        
        // Validate number pattern
        if Regex::new(&language.number_pattern).is_err() {
            return Err(format!("Invalid number pattern: {}", language.number_pattern));
        }
        
        // Validate function pattern if present
        if let Some(pattern) = &language.function_pattern {
            if Regex::new(pattern).is_err() {
                return Err(format!("Invalid function pattern: {}", pattern));
            }
        }
        
        // Validate class pattern if present
        if let Some(pattern) = &language.class_pattern {
            if Regex::new(pattern).is_err() {
                return Err(format!("Invalid class pattern: {}", pattern));
            }
        }
        
        Ok(())
    }
    
    /// Check if a language exists by name or alias
    pub fn has_language(&self, name: &str) -> bool {
        self.get_language(name).is_some()
    }
    
    /// Get the total number of supported languages
    pub fn language_count(&self) -> usize {
        // Count unique languages (not aliases)
        self.languages.values()
            .map(|lang| &lang.name)
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
    
    /// Get language by file extension
    pub fn get_language_by_extension(&self, extension: &str) -> Option<&CodeLanguage> {
        let ext_with_dot = if extension.starts_with('.') {
            extension.to_string()
        } else {
            format!(".{}", extension)
        };
        
        for language in self.languages.values() {
            if language.file_extensions.contains(&ext_with_dot) {
                return Some(language);
            }
        }
        
        None
    }

    /// Simple HTML escape function
    #[allow(dead_code)]
    fn html_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
}
