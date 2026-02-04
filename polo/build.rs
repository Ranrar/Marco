/// Build script for Polo - embeds Windows icon into the executable
///
/// This script only runs on Windows builds and uses winres to embed
/// the polo.ico icon into the compiled executable.
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    #[cfg(target_os = "windows")]
    {
        // Only rerun if icon file changes
        println!("cargo:rerun-if-changed=../assets/icons/polo.ico");
        println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENV");

        let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

        if target_env == "msvc" {
            let mut res = winres::WindowsResource::new();
            res.set_icon("../assets/icons/polo.ico");

            if let Err(e) = res.compile() {
                eprintln!("Warning: Failed to compile Windows resources: {}", e);
            }
        } else if target_env == "gnu" {
            // On windows-gnu, compile a tiny .rc with windres and pass the COFF object to the linker.
            let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
            let rc_path = out_dir.join("polo.rc");
            let rc_obj = out_dir.join("polo_rc.o");

            // The icon path is relative to the crate root (where build.rs runs).
            let rc_contents = "1 ICON \"../assets/icons/polo.ico\"\n";
            if let Err(e) = std::fs::write(&rc_path, rc_contents) {
                println!(
                    "cargo:warning=Failed writing polo.rc ({}); continuing without icon embedding",
                    e
                );
                return;
            }

            let try_windres = |program: &str| {
                Command::new(program)
                    .args(["-i"])
                    .arg(&rc_path)
                    .args(["-O", "coff", "-o"])
                    .arg(&rc_obj)
                    .status()
            };

            let status = try_windres("windres").or_else(|_| try_windres("x86_64-w64-mingw32-windres"));

            match status {
                Ok(s) if s.success() => {
                    println!("cargo:rustc-link-arg={}", rc_obj.display());
                    println!("cargo:warning=Icon embedded successfully using windres (GNU)");
                }
                Ok(s) => {
                    println!(
                        "cargo:warning=windres failed (status: {:?}); continuing without icon embedding",
                        s
                    );
                }
                Err(e) => {
                    println!(
                        "cargo:warning=windres not available ({}); continuing without icon embedding",
                        e
                    );
                }
            }
        } else {
            println!(
                "cargo:warning=Unknown CARGO_CFG_TARGET_ENV='{}'; continuing without icon embedding",
                target_env
            );
        }
    }
}
