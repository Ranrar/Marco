//! Cross-platform resource path resolver for Marco
//!
//! Resolves asset, theme, and language paths using the MARCO_DATA_DIR environment variable if set,
//! otherwise falls back to platform-appropriate user data directories.

use std::env;
use std::path::{PathBuf};

/// Resolves a resource path for Marco, checking MARCO_DATA_DIR first, then falling back to platform default.
pub fn resolve_resource_path(subdir: &str, filename: &str) -> PathBuf {
    // 0. Check for resource in the binary directory (e.g., $HOME/.local/bin/marco_language/)
    let is_lang_code = matches!(subdir, "en"|"es"|"fr"|"de");
    if is_lang_code || subdir.starts_with("assets/language") || subdir.starts_with("language") {
        if let Ok(exe_path) = env::current_exe() {
            if let Some(bin_dir) = exe_path.parent() {
                let mut bin_lang_path = bin_dir.to_path_buf();
                bin_lang_path.push("language");
                if is_lang_code {
                    bin_lang_path.push(subdir);
                } else if subdir.ends_with("language") {
                    // e.g. subdir = "language/en"
                    let lang_subdir = subdir.splitn(2, '/').nth(1).unwrap_or("");
                    if !lang_subdir.is_empty() {
                        bin_lang_path.push(lang_subdir);
                    }
                } else if subdir.contains("assets/language/") {
                    // e.g. subdir = "assets/language/en"
                    let lang_subdir = subdir.splitn(3, '/').nth(2).unwrap_or("");
                    if !lang_subdir.is_empty() {
                        bin_lang_path.push(lang_subdir);
                    }
                }
                bin_lang_path.push(filename);
                if bin_lang_path.exists() {
                    eprintln!("[resolve_resource_path] Using binary dir: {}", bin_lang_path.display());
                    return bin_lang_path;
                }
            }
        }
    }
    // 1. Check MARCO_DATA_DIR
    if let Ok(base) = env::var("MARCO_DATA_DIR") {
        let mut path = PathBuf::from(base);
        path.push(subdir);
        path.push(filename);
        if path.exists() || path.parent().map(|p| p.exists()).unwrap_or(false) {
            eprintln!("[resolve_resource_path] Using MARCO_DATA_DIR: {}", path.display());
            return path;
        }
    }

    // 2. Platform fallback
    let base = if cfg!(target_os = "windows") {
        // Use %APPDATA%\marco
        env::var("APPDATA").map(|appdata| {
            let mut p = PathBuf::from(appdata);
            p.push("marco");
            p
        }).unwrap_or_else(|_| {
            // Fallback: current dir
            PathBuf::from(".")
        })
    } else {
        // Use ~/.local/share/marco
        env::var("HOME").map(|home| {
            let mut p = PathBuf::from(home);
            p.push(".local");
            p.push("share");
            p.push("marco");
            p
        }).unwrap_or_else(|_| {
            // Fallback: current dir
            PathBuf::from(".")
        })
    };
    let mut path = base.clone();
    path.push(subdir);
    path.push(filename);
    // Special case: if looking for a directory of .tmTheme files, ensure at least one .tmTheme exists
    if subdir == "ui/ui_theme" && filename.is_empty() {
        if path.exists() && path.is_dir() {
            let has_tmtheme = std::fs::read_dir(&path)
                .map(|mut entries| entries.any(|e| e.ok().and_then(|e| e.path().extension().map(|x| x == "tmTheme")).unwrap_or(false)))
                .unwrap_or(false);
            if has_tmtheme {
                eprintln!("[resolve_resource_path] Using runtime path (with .tmTheme): {}", path.display());
                return path;
            }
        }
    } else if path.exists() || path.parent().map(|p| p.exists()).unwrap_or(false) {
        eprintln!("[resolve_resource_path] Using runtime path: {}", path.display());
        return path;
    }

    // 3. Development fallback: check src/ for any resource files (themes, css, etc.)
    let mut dev_path = PathBuf::from("src");
    dev_path.push(subdir);
    dev_path.push(filename);
    if dev_path.exists() || dev_path.parent().map(|p| p.exists()).unwrap_or(false) {
        eprintln!("[resolve_resource_path] Using dev fallback: {}", dev_path.display());
        return dev_path;
    }

    // 4. Fallback: return the original runtime path (may not exist)
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;


    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_env_var() {
        let temp = TempDir::new().unwrap();
        let marco_data = temp.path();
        let theme_dir = marco_data.join("ui/ui_theme");
        fs::create_dir_all(&theme_dir).unwrap();
        fs::write(theme_dir.join("foo.txt"), b"test").unwrap();
        env::set_var("MARCO_DATA_DIR", marco_data);
        let p = resolve_resource_path("ui/ui_theme", "foo.txt");
        assert!(p.starts_with(marco_data));
        env::remove_var("MARCO_DATA_DIR");
    }

    #[test]
    fn test_linux_fallback() {
        if cfg!(target_os = "windows") {
            return;
        }
        env::remove_var("MARCO_DATA_DIR");
        let home = env::var("HOME").unwrap_or_else(|_| String::from("/home/testuser"));
        let user_css_theme = format!("{}/.local/share/marco/ui/css_theme", home);
        let _ = fs::create_dir_all(&user_css_theme);
        let css_file = format!("{}/bar.css", user_css_theme);
        let _ = fs::write(&css_file, b"test");
        let p = resolve_resource_path("ui/css_theme", "bar.css");
        assert!(p.starts_with(&user_css_theme) || p.starts_with("./ui/css_theme"));
        let _ = fs::remove_file(&css_file);
    }

    #[test]
    fn test_windows_fallback() {
        if !cfg!(target_os = "windows") {
            return;
        }
        env::remove_var("MARCO_DATA_DIR");
        let appdata = env::var("APPDATA").unwrap_or_else(|_| String::from("C:/Users/testuser/AppData/Roaming"));
        let p = resolve_resource_path("language", "baz.yml");
        assert!(p.starts_with(format!("{}/marco/language", appdata)) || p.starts_with("./language"));
    }
}
