use glib_build_tools::compile_resources;

fn main() {
    // Compile resources if they exist
    if std::path::Path::new("resources").exists() {
        compile_resources(
            &["resources"],  // Changed to array slice
            "resources/resources.gresource.xml",
            "resources.gresource",
        );
    }
}