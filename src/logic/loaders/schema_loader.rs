//! Markdown schema loader for settings tab
//! Scans src/assets/markdown_schema/ for valid schemas

use std::fs;
use std::path::{Path, PathBuf};
use log::info;

#[derive(Debug, Clone)]
pub struct MarkdownSchema {
    pub name: String, // folder name
    pub path: PathBuf,
    pub ast_path: PathBuf,
    pub syntax_path: PathBuf,
}

/// Returns a list of available schemas (folders with ast.ron and syntax.ron)
pub fn list_available_schemas(root: &Path) -> Vec<MarkdownSchema> {
    let mut schemas = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let ast = path.join("ast.ron");
                let syntax = path.join("syntax.ron");
                if ast.exists() && syntax.exists() {
                    let schema = MarkdownSchema {
                        name: path.file_name().unwrap().to_string_lossy().to_string(),
                        path: path.clone(),
                        ast_path: ast.clone(),
                        syntax_path: syntax.clone(),
                    };
                    // Log the discovered schema so the fields are actually read and recorded.
                    info!(
                        "Found markdown schema: name={} path={:?} ast={:?} syntax={:?}",
                        schema.name,
                        schema.path,
                        schema.ast_path,
                        schema.syntax_path
                    );
                    schemas.push(schema);
                }
            }
        }
    }
    schemas
}
