use std::collections::HashMap;
use std::fs;

/// Write the grammar tree to multiple file formats with color coding
pub fn show_grammar_tree() {
    println!("🌳 Marco Grammar Dependency Tree");
    println!("=================================\n");

    // Read the grammar file
    let grammar_path = "../../src/components/marco_engine/marco_grammar.pest";
    let grammar_content = match fs::read_to_string(grammar_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read grammar file {}: {}", grammar_path, e);
            std::process::exit(1);
        }
    };

    println!("📄 Grammar file: {}", grammar_path);
    println!("📏 File size: {} bytes", grammar_content.len());
    println!("📝 Lines: {}", grammar_content.lines().count());

    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all("src/results") {
        eprintln!("❌ Failed to create results directory: {}", e);
        std::process::exit(1);
    }

    // Parse rules and their dependencies
    let mut rules = HashMap::new();
    let mut rule_count = 0;

    println!("\n🔍 Parsing rule dependencies...");

    for line in grammar_content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        // Detect rule definitions
        if let Some(eq_pos) = line.find(" = ") {
            let rule_name = line[..eq_pos].trim();
            let rule_definition = &line[eq_pos + 3..];

            // Skip built-in rules starting with //
            if rule_name.starts_with("//") {
                continue;
            }

            rule_count += 1;

            // Extract dependencies from rule definition
            let dependencies = extract_rule_dependencies(rule_definition);
            rules.insert(
                rule_name.to_string(),
                (rule_definition.to_string(), dependencies),
            );
        }
    }

    println!("✅ Parsed {} rules", rule_count);
    println!("\n🌳 Building Grammar Dependency Structure...");

    // Build dependency tree starting from key entry points
    let entry_points = vec!["file", "document", "block", "inline"];
    let mut tree_nodes = Vec::new();
    let mut visited = std::collections::HashSet::new();

    for entry_point in entry_points {
        if rules.contains_key(entry_point) {
            if let Some(tree_node) = build_dependency_tree(entry_point, &rules, &mut visited, 0) {
                tree_nodes.push(tree_node);
            }
        }
    }

    // Add any unvisited rules as separate branches
    let mut unvisited_rules = Vec::new();
    for rule_name in rules.keys() {
        if !visited.contains(rule_name) {
            if let Some(tree_node) =
                build_dependency_tree(rule_name, &rules, &mut std::collections::HashSet::new(), 0)
            {
                unvisited_rules.push(tree_node);
            }
        }
    }

    if !unvisited_rules.is_empty() {
        let unvisited_tree = ascii_tree::Tree::Node("Other Rules".to_string(), unvisited_rules);
        tree_nodes.push(unvisited_tree);
    }

    // Create and write the complete grammar tree to files
    if !tree_nodes.is_empty() {
        let grammar_tree = ascii_tree::Tree::Node(
            format!("Marco Grammar Dependencies ({} total rules)", rule_count),
            tree_nodes,
        );

        let mut output = String::new();
        match ascii_tree::write_tree(&mut output, &grammar_tree) {
            Ok(_) => {
                // Write to files with different formats
                write_grammar_tree_to_files(&output, rule_count);

                println!("✅ Grammar dependency tree visualization complete!");
                println!("📊 Summary:");
                println!("  - Total rules analyzed: {}", rule_count);
                println!("  - Entry points: file → document → block/inline");
                println!("  - Tree shows actual rule dependencies and relationships");
                println!("📝 File generated:");
                println!("  - src/results/grammar_tree.html (interactive color-coded HTML)");
            }
            Err(e) => {
                eprintln!("❌ Failed to generate grammar tree: {}", e);
            }
        }
    } else {
        println!("⚠️  No grammar rules found in the file.");
    }
}

/// Enhanced grammar analysis with detailed categorization
pub fn analyze_grammar_structure() {
    println!("🔬 Detailed Grammar Analysis");
    println!("============================\n");

    let grammar_path = "../../src/components/marco_engine/marco_grammar.pest";
    let grammar_content = match fs::read_to_string(grammar_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read grammar file: {}", e);
            return;
        }
    };

    let mut categories = HashMap::new();
    let mut rule_types = HashMap::new();
    let mut complexity_analysis = Vec::new();

    for line in grammar_content.lines() {
        let line = line.trim();

        if let Some(eq_pos) = line.find(" = ") {
            let rule_name = line[..eq_pos].trim();
            let rule_definition = &line[eq_pos + 3..];

            // Analyze rule type
            let rule_type = if rule_definition.starts_with("@{") {
                "Atomic"
            } else if rule_definition.starts_with("_{") {
                "Silent"
            } else if rule_definition.starts_with("${") {
                "Compound-Atomic"
            } else {
                "Normal"
            };

            // Count complexity indicators
            let complexity = rule_definition.matches('|').count()
                + rule_definition.matches('*').count()
                + rule_definition.matches('+').count()
                + rule_definition.matches('?').count();

            // Categorize by function
            let category = categorize_rule(rule_name);

            categories
                .entry(category)
                .or_insert_with(Vec::new)
                .push(rule_name);
            *rule_types.entry(rule_type).or_insert(0) += 1;
            complexity_analysis.push((rule_name, complexity));
        }
    }

    // Display analysis
    println!("📊 Rule Categories:");
    for (category, rules) in &categories {
        println!("  • {}: {} rules", category, rules.len());
    }

    println!("\n🏗️  Rule Types:");
    for (rule_type, count) in &rule_types {
        println!("  • {}: {} rules", rule_type, count);
    }

    println!("\n🔥 Most Complex Rules:");
    complexity_analysis.sort_by(|a, b| b.1.cmp(&a.1));
    for (rule_name, complexity) in complexity_analysis.iter().take(10) {
        println!("  • {}: {} complexity points", rule_name, complexity);
    }
}

// Helper functions below...

fn categorize_rule(rule_name: &str) -> String {
    match rule_name {
        name if name.starts_with("H") || name.contains("heading") => "Headings".to_string(),
        name if name.contains("bold") || name.contains("italic") || name.contains("emphasis") => {
            "Formatting".to_string()
        }
        name if name.contains("link") || name.contains("url") || name.contains("autolink") => {
            "Links".to_string()
        }
        name if name.contains("list") || name.contains("task") => "Lists".to_string(),
        name if name.contains("code") || name.contains("math") => "Code & Math".to_string(),
        name if name.contains("table") => "Tables".to_string(),
        name if name.contains("image") => "Images".to_string(),
        name if name.contains("admonition") || name.contains("tab") => {
            "Marco Extensions".to_string()
        }
        name if name.contains("text") || name.contains("word") || name.contains("unicode") => {
            "Text Processing".to_string()
        }
        name if name.starts_with("KW_") => "Keywords".to_string(),
        name if name == "WHITESPACE" || name == "NEWLINE" || name.contains("INDENT") => {
            "Whitespace".to_string()
        }
        _ => "Other".to_string(),
    }
}

/// Safe string truncation that respects Unicode character boundaries
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Find the last valid character boundary before max_len - 3 (for "...")
        let target_len = if max_len > 3 { max_len - 3 } else { 0 };
        let mut truncate_at = target_len.min(s.len());

        while truncate_at > 0 && !s.is_char_boundary(truncate_at) {
            truncate_at -= 1;
        }

        if truncate_at > 0 {
            format!("{}...", &s[..truncate_at])
        } else {
            "...".to_string()
        }
    }
}

/// Extract rule dependencies from a rule definition
fn extract_rule_dependencies(rule_definition: &str) -> Vec<String> {
    let mut dependencies = Vec::new();

    // Remove operators and extract identifiers
    let cleaned = rule_definition
        .replace("@{", " ")
        .replace("_{", " ")
        .replace("${", " ")
        .replace("!{", " ")
        .replace("}", " ")
        .replace("(", " ")
        .replace(")", " ")
        .replace("[", " ")
        .replace("]", " ")
        .replace("\"", " ")
        .replace("'", " ")
        .replace("~", " ")
        .replace("|", " ")
        .replace("*", " ")
        .replace("+", " ")
        .replace("?", " ")
        .replace("&", " ")
        .replace("!", " ")
        .replace("{", " ")
        .replace("^", " ");

    // Extract potential rule names (exclude literals and built-ins)
    for token in cleaned.split_whitespace() {
        if is_valid_rule_name(token) {
            dependencies.push(token.to_string());
        }
    }

    // Remove duplicates while preserving order
    let mut unique_deps = Vec::new();
    for dep in dependencies {
        if !unique_deps.contains(&dep) {
            unique_deps.push(dep);
        }
    }

    unique_deps
}

/// Check if a token is a valid rule name (not a literal or built-in)
fn is_valid_rule_name(token: &str) -> bool {
    // Skip empty strings
    if token.is_empty() {
        return false;
    }

    // Skip literals (quoted strings)
    if token.starts_with('"') || token.starts_with('\'') {
        return false;
    }

    // Skip numbers and single characters
    if token.len() == 1 || token.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Skip pest built-ins
    let builtins = [
        "SOI",
        "EOI",
        "NEWLINE",
        "ANY",
        "ASCII_DIGIT",
        "ASCII_ALPHA",
        "ASCII_ALPHANUMERIC",
        "LETTER",
        "PUNCTUATION",
        "WHITESPACE",
    ];
    if builtins.contains(&token) {
        return false;
    }

    // Skip common operators and keywords
    let operators = ["=", ":", ".", "-", "+", "0", "1", "2", "3", "4", "5", "6"];
    if operators.contains(&token) {
        return false;
    }

    // Must start with letter or underscore
    if let Some(first_char) = token.chars().next() {
        first_char.is_ascii_alphabetic() || first_char == '_'
    } else {
        false
    }
}

/// Build a dependency tree for a rule recursively
fn build_dependency_tree(
    rule_name: &str,
    rules: &HashMap<String, (String, Vec<String>)>,
    visited: &mut std::collections::HashSet<String>,
    depth: usize,
) -> Option<ascii_tree::Tree> {
    // Skip TOC entirely as it creates circular references and isn't essential for visualization
    if rule_name == "toc" {
        return None;
    }

    // Prevent infinite recursion
    if depth > 10 || visited.contains(rule_name) {
        return Some(ascii_tree::Tree::Leaf(vec![format!(
            "{} [CIRCULAR]",
            rule_name
        )]));
    }

    visited.insert(rule_name.to_string());

    if let Some((rule_def, dependencies)) = rules.get(rule_name) {
        if dependencies.is_empty() {
            // Leaf rule - show definition
            let display_def = truncate_string(rule_def, 40);
            Some(ascii_tree::Tree::Leaf(vec![format!(
                "{} = {}",
                rule_name, display_def
            )]))
        } else {
            // Branch rule - show dependencies
            let mut child_nodes = Vec::new();

            for dep in dependencies {
                // Skip TOC dependencies as they create circular references
                if dep == "toc" {
                    continue;
                }

                // Only include dependencies that exist in our rules
                if rules.contains_key(dep) {
                    if let Some(child_tree) =
                        build_dependency_tree(dep, rules, &mut visited.clone(), depth + 1)
                    {
                        child_nodes.push(child_tree);
                    }
                }
            }

            if child_nodes.is_empty() {
                // No valid child dependencies, show as leaf
                let display_def = truncate_string(rule_def, 40);
                Some(ascii_tree::Tree::Leaf(vec![format!(
                    "{} = {}",
                    rule_name, display_def
                )]))
            } else {
                Some(ascii_tree::Tree::Node(rule_name.to_string(), child_nodes))
            }
        }
    } else {
        Some(ascii_tree::Tree::Leaf(vec![format!(
            "{} (not found)",
            rule_name
        )]))
    }
}

/// Write the grammar tree to multiple file formats
fn write_grammar_tree_to_files(tree_output: &str, rule_count: usize) {
    // Only write HTML version with color coding
    write_html_grammar_tree(tree_output, rule_count);
}

/// Write HTML version of the grammar tree with syntax highlighting
fn write_html_grammar_tree(tree_output: &str, rule_count: usize) {
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Marco Grammar Dependency Tree</title>
    <style>
        body {{
            font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
            background-color: #1e1e1e;
            color: #d4d4d4;
            margin: 0;
            padding: 20px;
            line-height: 1.4;
            height: 100vh;
            display: flex;
            flex-direction: column;
            box-sizing: border-box;
        }}
        
        .header-table {{
            width: 100%;
            background-color: #252525;
            border-radius: 8px;
            border-collapse: collapse;
            margin-bottom: 20px;
            border: 1px solid #3c3c3c;
            flex-shrink: 0;
        }}
        
        .header-table td {{
            padding: 15px;
            vertical-align: top;
            border-right: 1px solid #3c3c3c;
        }}
        
        .header-table td:last-child {{
            border-right: none;
        }}
        
        .title-cell {{
            width: 30%;
            border-left: 4px solid #007acc;
        }}
        
        .title-cell h1 {{
            font-size: 18px;
            margin: 0 0 8px 0;
            color: #4ec9b0;
        }}
        
        .title-cell p {{
            font-size: 11px;
            margin: 2px 0;
            line-height: 1.3;
        }}
        
        .legend-cell {{
            width: 40%;
            border-left: 4px solid #4ec9b0;
        }}
        
        .legend-cell h3 {{
            font-size: 12px;
            margin: 0 0 10px 0;
            color: #4ec9b0;
        }}
        
        .legend-table {{
            width: 100%;
            border-collapse: collapse;
            font-size: 10px;
        }}
        
        .legend-table td {{
            padding: 2px 6px;
            border: none;
            vertical-align: middle;
        }}
        
        .legend-table td:first-child {{
            width: 80px;
            font-weight: bold;
        }}
        
        .search-cell {{
            width: 30%;
            border-left: 4px solid #c586c0;
        }}
        
        .search-cell h3 {{
            font-size: 12px;
            margin: 0 0 10px 0;
            color: #c586c0;
        }}
        
        .search-controls {{
            display: flex;
            flex-direction: column;
            gap: 8px;
        }}
        
        .search-input-row {{
            display: flex;
            gap: 5px;
        }}
        
        #search-input {{
            flex: 1;
            padding: 6px;
            background-color: #3c3c3c;
            border: 1px solid #555;
            color: #d4d4d4;
            border-radius: 4px;
            font-size: 11px;
        }}
        
        .search-buttons {{
            display: flex;
            gap: 3px;
        }}
        
        .search-btn {{
            padding: 6px 8px;
            background-color: #3c3c3c;
            border: 1px solid #555;
            color: #d4d4d4;
            border-radius: 3px;
            cursor: pointer;
            font-size: 10px;
        }}
        
        .search-btn:hover {{
            background-color: #4c4c4c;
        }}
        
        .search-info {{
            font-size: 10px;
            color: #9cdcfe;
        }}
        
        .tree-container {{
            background-color: #2d2d30;
            padding: 20px;
            border-radius: 8px;
            overflow-y: auto;
            white-space: pre;
            font-size: 14px;
            flex: 1;
            min-height: 0;
            border: 1px solid #3c3c3c;
        }}
        
        .entry-point {{ color: #4ec9b0; font-weight: bold; }}
        .block-rule {{ color: #569cd6; }}
        .inline-rule {{ color: #9cdcfe; }}
        .text-rule {{ color: #ce9178; }}
        .formatting-rule {{ color: #c586c0; }}
        .macro-rule {{ color: #dcdcaa; }}
        .keyword-rule {{ color: #d4d4d4; background-color: #3c3c3c; padding: 2px; }}
        .circular {{ color: #f44747; font-style: italic; }}
        
        .highlight {{
            background-color: #ffd700;
            color: #000;
            font-weight: bold;
        }}
        
        .current-match {{
            background-color: #ff6b6b;
            color: #fff;
            font-weight: bold;
            border-radius: 2px;
        }}
    </style>
    <script>
        let currentMatches = [];
        let currentMatchIndex = -1;
        let originalContent = '';
        
        function initializeSearch() {{
            const treeContainer = document.querySelector('.tree-container');
            originalContent = treeContainer.innerHTML;
        }}
        
        function searchTree() {{
            const searchTerm = document.getElementById('search-input').value.toLowerCase().trim();
            const treeContainer = document.querySelector('.tree-container');
            
            if (searchTerm === '') {{
                treeContainer.innerHTML = originalContent;
                currentMatches = [];
                currentMatchIndex = -1;
                updateSearchInfo();
                return;
            }}
            
            // Reset to original content
            treeContainer.innerHTML = originalContent;
            
            // Find all matches
            const text = treeContainer.textContent;
            const regex = new RegExp(searchTerm.replace(/[.*+?^${{}}()|[\\]\\\\]/g, '\\\\$&'), 'gi');
            const matches = [...text.matchAll(regex)];
            
            if (matches.length === 0) {{
                updateSearchInfo(0, 0);
                return;
            }}
            
            // Highlight all matches
            let highlightedHTML = originalContent;
            const searchRegex = new RegExp(`(${{searchTerm.replace(/[.*+?^${{}}()|[\\]\\\\]/g, '\\\\$&')}})`, 'gi');
            highlightedHTML = highlightedHTML.replace(searchRegex, '<span class="highlight" data-match-id="$1">$1</span>');
            
            treeContainer.innerHTML = highlightedHTML;
            
            // Update match tracking
            currentMatches = document.querySelectorAll('.highlight');
            currentMatchIndex = -1;
            updateSearchInfo(matches.length, currentMatchIndex + 1);
        }}
        
        function jumpToNext() {{
            if (currentMatches.length === 0) return;
            
            // Remove current highlight
            if (currentMatchIndex >= 0) {{
                currentMatches[currentMatchIndex].classList.remove('current-match');
            }}
            
            // Move to next match
            currentMatchIndex = (currentMatchIndex + 1) % currentMatches.length;
            const currentMatch = currentMatches[currentMatchIndex];
            
            // Highlight current match
            currentMatch.classList.add('current-match');
            
            // Scroll to match
            currentMatch.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
            
            updateSearchInfo(currentMatches.length, currentMatchIndex + 1);
        }}
        
        function jumpToPrev() {{
            if (currentMatches.length === 0) return;
            
            // Remove current highlight
            if (currentMatchIndex >= 0) {{
                currentMatches[currentMatchIndex].classList.remove('current-match');
            }}
            
            // Move to previous match
            currentMatchIndex = currentMatchIndex <= 0 ? currentMatches.length - 1 : currentMatchIndex - 1;
            const currentMatch = currentMatches[currentMatchIndex];
            
            // Highlight current match
            currentMatch.classList.add('current-match');
            
            // Scroll to match
            currentMatch.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
            
            updateSearchInfo(currentMatches.length, currentMatchIndex + 1);
        }}
        
        function updateSearchInfo(total = 0, current = 0) {{
            const infoElement = document.querySelector('.search-info');
            if (total === 0) {{
                infoElement.textContent = 'No matches';
            }} else {{
                infoElement.textContent = `${{current}} of ${{total}} matches`;
            }}
        }}
        
        // Initialize when page loads
        document.addEventListener('DOMContentLoaded', function() {{
            initializeSearch();
            
            // Add keyboard shortcuts
            document.getElementById('search-input').addEventListener('keydown', function(e) {{
                if (e.key === 'Enter') {{
                    e.preventDefault();
                    if (e.shiftKey) {{
                        jumpToPrev();
                    }} else {{
                        jumpToNext();
                    }}
                }}
            }});
        }});
    </script>
</head>
<body>
    <table class="header-table">
        <tr>
            <td class="title-cell">
                <h1>🌳 Marco Grammar Tree</h1>
                <p>Generated from marco.pest</p>
                <p><strong>Rules:</strong> {rule_count} | <strong>Date:</strong> {timestamp}</p>
            </td>
            <td class="legend-cell">
                <h3>📊 Color Legend</h3>
                <table class="legend-table">
                    <tr><td><span class="entry-point">Entry Points</span></td><td>Main grammar rules</td></tr>
                    <tr><td><span class="block-rule">Block Rules</span></td><td>Block elements</td></tr>
                    <tr><td><span class="inline-rule">Inline Rules</span></td><td>Inline elements</td></tr>
                    <tr><td><span class="formatting-rule">Formatting</span></td><td>Text formatting</td></tr>
                    <tr><td><span class="macro-rule">Marco Ext</span></td><td>Custom features</td></tr>
                    <tr><td><span class="keyword-rule">Keywords</span></td><td>Reserved words</td></tr>
                    <tr><td><span class="circular">Circular</span></td><td>Circular refs</td></tr>
                </table>
            </td>
            <td class="search-cell">
                <h3>🔍 Search & Navigate</h3>
                <div class="search-controls">
                    <div class="search-input-row">
                        <input type="text" id="search-input" placeholder="Search grammar rules..." 
                               onkeyup="searchTree()" autocomplete="off">
                    </div>
                    <div class="search-buttons">
                        <button class="search-btn" onclick="jumpToPrev()">↑ Prev</button>
                        <button class="search-btn" onclick="jumpToNext()">↓ Next</button>
                    </div>
                    <div class="search-info">Enter search term</div>
                </div>
            </td>
        </tr>
    </table>
    
    <div class="tree-container">
{colored_tree}
    </div>
</body>
</html>"#,
        rule_count = rule_count,
        timestamp = get_current_timestamp(),
        colored_tree = colorize_tree_for_html(tree_output)
    );

    match fs::write("src/results/grammar_tree.html", html_content) {
        Ok(_) => println!("📄 HTML version written to src/results/grammar_tree.html"),
        Err(e) => eprintln!("❌ Failed to write HTML file: {}", e),
    }
}

/// Apply HTML color coding to the tree output
fn colorize_tree_for_html(tree_output: &str) -> String {
    let mut result = String::new();

    for line in tree_output.lines() {
        let colored_line = if line.contains("[CIRCULAR]") {
            // Remove [CIRCULAR] marker and apply red styling to the whole rule name
            let clean_line = line.replace(" [CIRCULAR]", "");
            apply_html_class(&clean_line, "circular")
        } else if contains_rule_type(line, &["file", "document"]) {
            apply_html_class(line, "entry-point")
        } else if contains_rule_type(
            line,
            &[
                "block",
                "paragraph",
                "list",
                "table",
                "blockquote",
                "hr",
                "admonition",
            ],
        ) {
            apply_html_class(line, "block-rule")
        } else if contains_rule_type(line, &["inline", "link", "image", "footnote"]) {
            apply_html_class(line, "inline-rule")
        } else if contains_rule_type(
            line,
            &[
                "bold",
                "italic",
                "emphasis",
                "highlight",
                "strikethrough",
                "superscript",
                "subscript",
            ],
        ) {
            apply_html_class(line, "formatting-rule")
        } else if contains_rule_type(
            line,
            &[
                "admonition",
                "tab",
                "user_mention",
                "bookmark",
                "toc",
                "run_",
                "macro",
            ],
        ) {
            apply_html_class(line, "macro-rule")
        } else if line.contains("KW_") {
            apply_html_class(line, "keyword-rule")
        } else if contains_rule_type(line, &["text", "word", "unicode", "math_symbol"]) {
            apply_html_class(line, "text-rule")
        } else {
            escape_html(line)
        };

        result.push_str(&colored_line);
        result.push('\n');
    }

    result
}

/// Check if a line contains any of the specified rule types
fn contains_rule_type(line: &str, rule_types: &[&str]) -> bool {
    let line_lower = line.to_lowercase();
    rule_types
        .iter()
        .any(|rule_type| line_lower.contains(rule_type))
}

/// Apply HTML class to a line while preserving tree structure
fn apply_html_class(line: &str, class: &str) -> String {
    // Find the rule name after tree symbols and wrap it with HTML class
    // Look for patterns like "├─ rule_name" or "└─ rule_name"
    if let Some(start) = line.find(" ") {
        // Find the first word after whitespace/tree symbols
        let after_symbols = &line[start..];
        if let Some(word_start) = after_symbols.find(|c: char| c.is_alphabetic() || c == '_') {
            let word_start_absolute = start + word_start;
            let word_part = &line[word_start_absolute..];

            // Find the end of the rule name
            if let Some(word_end) = word_part.find(' ') {
                let word_end_absolute = word_start_absolute + word_end;
                let prefix = &line[..word_start_absolute];
                let rule_name = &line[word_start_absolute..word_end_absolute];
                let suffix = &line[word_end_absolute..];

                format!(
                    r#"{}<span class="{}">{}</span>{}"#,
                    escape_html(prefix),
                    class,
                    escape_html(rule_name),
                    escape_html(suffix)
                )
            } else {
                // Rule name goes to end of line
                let prefix = &line[..word_start_absolute];
                let rule_name = word_part;

                format!(
                    r#"{}<span class="{}">{}</span>"#,
                    escape_html(prefix),
                    class,
                    escape_html(rule_name)
                )
            }
        } else {
            escape_html(line)
        }
    } else {
        escape_html(line)
    }
}

/// Escape HTML characters
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Get current timestamp (using chrono for accurate date)
fn get_current_timestamp() -> String {
    use chrono::Utc;

    Utc::now().format("%Y-%m-%d").to_string()
}
