/// Build script for Marco - embeds Windows icon into the executable
///
/// This script uses embed-resource to properly embed the marco.ico icon
/// into the Windows executable. It handles MSVC, MinGW, and cross-compilation.

#[cfg(target_os = "windows")]
fn main() {
    // Rerun if icon or RC file changes
    println!("cargo:rerun-if-changed=../assets/icons/marco.ico");
    println!("cargo:rerun-if-changed=marco.rc");
    
    // Compile the resource file (icon is cosmetic, so manifest_optional)
    embed_resource::compile("marco.rc", embed_resource::NONE)
        .manifest_optional()
        .unwrap();
    
    println!("cargo:warning=Icon embedded successfully using embed-resource");
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // No-op on non-Windows platforms
}
