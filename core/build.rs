// build.rs
// Automatically copy all TTF fonts from src/assets/fonts/ to src/assets/fonts/ (or another target directory) at build time.

use std::fs;
use std::path::Path;

fn main() {
    // In workspace: assets are at ../assets relative to core/
    let asset_root = Path::new("../assets");
    // Detect build profile (debug/release)
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    // OUT_DIR is something like target/debug/build/core-xxxx/out
    // We want target/debug or target/release
    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .expect("Failed to get target dir");
    let marco_root = target_dir.join("marco_assets");

    // Helper to recursively copy a directory
    fn copy_dir_recursive(src: &Path, dst: &Path) {
        if !dst.exists() {
            fs::create_dir_all(dst).expect("Failed to create target directory");
        }
        if let Ok(entries) = fs::read_dir(src) {
            for entry in entries.flatten() {
                let path = entry.path();
                let dest_path = dst.join(entry.file_name());
                if path.is_dir() {
                    copy_dir_recursive(&path, &dest_path);
                } else {
                    // Only copy if file doesn't exist or is newer
                    let should_copy = !dest_path.exists()
                        || fs::metadata(&path).ok().and_then(|m| m.modified().ok())
                            > fs::metadata(&dest_path)
                                .ok()
                                .and_then(|m| m.modified().ok());

                    if should_copy {
                        fs::copy(&path, &dest_path).expect("Failed to copy file");
                    }
                }
            }
        }
    }

    // Tell cargo to rerun only if asset directories change
    println!("cargo:rerun-if-changed=../assets");

    // Copy fonts
    let fonts_src = asset_root.join("fonts");
    let fonts_dst = marco_root.join("fonts");
    copy_dir_recursive(&fonts_src, &fonts_dst);

    // Copy icons
    let icons_src = asset_root.join("icons");
    let icons_dst = marco_root.join("icons");
    copy_dir_recursive(&icons_src, &icons_dst);

    // Copy documentation
    let doc_src = asset_root.join("documentation");
    let doc_dst = marco_root.join("documentation");
    copy_dir_recursive(&doc_src, &doc_dst);

    // Copy language
    let lang_src = asset_root.join("language");
    let lang_dst = marco_root.join("language");
    copy_dir_recursive(&lang_src, &lang_dst);

    // Copy themes
    let themes_src = asset_root.join("themes");
    let themes_dst = marco_root.join("themes");
    copy_dir_recursive(&themes_src, &themes_dst);

    // Copy settings.ron
    let settings_src = asset_root.join("settings.ron");
    let settings_dst = marco_root.join("settings.ron");
    if settings_src.exists() {
        let should_copy = !settings_dst.exists()
            || fs::metadata(&settings_src)
                .ok()
                .and_then(|m| m.modified().ok())
                > fs::metadata(&settings_dst)
                    .ok()
                    .and_then(|m| m.modified().ok());

        if should_copy {
            fs::copy(&settings_src, &settings_dst).expect("Failed to copy settings.ron");
        }
    }
}
