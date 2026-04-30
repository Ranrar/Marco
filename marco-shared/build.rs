// build.rs
// Copies runtime assets from marco-shared/src/assets/ to target/{debug|release}/marco_assets/
// at build time. Runs as part of the marco-shared crate so assets are available for both
// `marco` and `polo` binaries (which both depend on marco-shared).

use std::fs;
use std::path::Path;

fn main() {
    // Assets are next to this build.rs, under src/assets.
    let asset_root = Path::new("src/assets");

    // Detect target directory. OUT_DIR looks like target/debug/build/<crate>-<hash>/out;
    // we want target/debug or target/release.
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .expect("Failed to get target dir");
    let marco_root = target_dir.join("marco_assets");

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

    println!("cargo:rerun-if-changed=src/assets");

    for sub in ["fonts", "icons", "documentation", "language", "themes"] {
        copy_dir_recursive(&asset_root.join(sub), &marco_root.join(sub));
    }

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
