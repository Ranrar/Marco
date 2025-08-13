//! List and label all application themes in src/assets/themes/
use std::path::Path;

/// List all *.css HTML view themes in the given folder, with user-friendly labels
pub fn list_html_view_themes(theme_dir: &Path) -> Vec<ThemeEntry> {
    let mut entries = vec![];
    if let Ok(read_dir) = std::fs::read_dir(theme_dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "css") {
                if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                    let label = fname
                        .replace("-", " ")
                        .replace(".css", "")
                        .split_whitespace()
                        .map(|w| {
                            let mut c = w.chars();
                            match c.next() {
                                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                None => String::new(),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    entries.push(ThemeEntry {
                        filename: fname.to_string(),
                        label,
                    });
                }
            }
        }
    }
    entries
}

/// List all *.xml style scheme files for the editor, with user-friendly labels
pub fn list_editor_style_schemes(theme_dir: &Path) -> Vec<ThemeEntry> {
    let mut entries = vec![];
    if let Ok(read_dir) = std::fs::read_dir(theme_dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "xml") {
                if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                    // Try to parse the XML to get the proper name
                    let label = if let Ok(contents) = std::fs::read_to_string(&path) {
                        extract_style_scheme_name(&contents).unwrap_or_else(|| {
                            // Fallback to filename-based label
                            fname
                                .replace("-", " ")
                                .replace(".xml", "")
                                .split_whitespace()
                                .map(|w| {
                                    let mut c = w.chars();
                                    match c.next() {
                                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                        None => String::new(),
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(" ")
                        })
                    } else {
                        // Fallback to filename-based label
                        fname
                            .replace("-", " ")
                            .replace(".xml", "")
                            .split_whitespace()
                            .map(|w| {
                                let mut c = w.chars();
                                match c.next() {
                                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                    None => String::new(),
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    };
                    
                    entries.push(ThemeEntry {
                        filename: fname.to_string(),
                        label,
                    });
                }
            }
        }
    }
    entries
}

/// Extract the display name from a style scheme XML file
fn extract_style_scheme_name(xml_content: &str) -> Option<String> {
    // Simple XML parsing to extract the name attribute
    for line in xml_content.lines() {
        let line = line.trim();
        if line.starts_with("<style-scheme") {
            // Look for name="..." or _name="..."
            if let Some(start) = line.find("name=\"") {
                let start = start + 6; // Skip 'name="'
                if let Some(end) = line[start..].find('"') {
                    return Some(line[start..start + end].to_string());
                }
            }
            if let Some(start) = line.find("_name=\"") {
                let start = start + 7; // Skip '_name="'
                if let Some(end) = line[start..].find('"') {
                    return Some(line[start..start + end].to_string());
                }
            }
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct ThemeEntry {
    pub filename: String,
    pub label: String,
}