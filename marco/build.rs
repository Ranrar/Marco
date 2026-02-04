/// Build script for Marco - embeds Windows icon into the executable
///
/// This script uses embed-resource to properly embed the marco.ico icon
/// into the Windows executable. It handles MSVC, MinGW, and cross-compilation.
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    #[cfg(target_os = "windows")]
    {
        // Rerun if icon or RC file changes
        println!("cargo:rerun-if-changed=../assets/icons/marco.ico");
        println!("cargo:rerun-if-changed=marco.rc");

        // Rerun if target env changes (msvc vs gnu)
        println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ENV");

        let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

        if target_env == "msvc" {
            // Compile the resource file (icon is cosmetic, so manifest_optional)
            embed_resource::compile("marco.rc", embed_resource::NONE)
                .manifest_optional()
                .unwrap();

            println!("cargo:warning=Icon embedded successfully using embed-resource (MSVC)");
        } else if target_env == "gnu" {
            // For MinGW (windows-gnu), embed-resource may emit an MSVC-style .lib.
            // Compile the .rc with windres to a COFF object and pass it directly to the linker.
            let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
            let rc_obj = out_dir.join("marco_rc.o");

            let try_windres = |program: &str| {
                Command::new(program)
                    .args(["-i", "marco.rc", "-O", "coff", "-o"])
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
