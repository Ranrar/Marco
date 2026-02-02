/// Build script for Polo - embeds Windows icon into the executable
///
/// This script only runs on Windows builds and uses winres to embed
/// the polo.ico icon into the compiled executable.

#[cfg(target_os = "windows")]
fn main() {
    // Only rerun if icon file changes
    println!("cargo:rerun-if-changed=../assets/icons/polo.ico");
    
    let mut res = winres::WindowsResource::new();
    res.set_icon("../assets/icons/polo.ico");
    
    if let Err(e) = res.compile() {
        eprintln!("Warning: Failed to compile Windows resources: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // No-op on non-Windows platforms
}
