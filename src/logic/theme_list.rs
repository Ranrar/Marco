//! List and label all application themes in src/assets/themes/gtk4
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

#[derive(Debug, Clone)]
pub struct ThemeEntry {
    pub filename: String,
    pub label: String,
}