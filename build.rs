use glib_build_tools::compile_resources;
use std::process::Command;

fn main() {
    // Compile GSettings schema
    println!("cargo:rerun-if-changed=org.marco.editor.gschema.xml");
    
    let output = Command::new("glib-compile-schemas")
        .arg(".")
        .arg("--strict")
        .output();
        
    match output {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Failed to compile GSettings schema: {}", stderr);
                std::process::exit(1);
            } else {
                println!("GSettings schema compiled successfully");
            }
        }
        Err(e) => {
            eprintln!("Failed to run glib-compile-schemas: {}", e);
            std::process::exit(1);
        }
    }
    
    // Set environment variable for runtime
    println!("cargo:rustc-env=GSETTINGS_SCHEMA_DIR={}", std::env::current_dir().unwrap().display());
    
    // Compile resources if they exist
    if std::path::Path::new("resources").exists() {
        compile_resources(
            &["resources"],  // Changed to array slice
            "resources/resources.gresource.xml",
            "resources.gresource",
        );
    }
}