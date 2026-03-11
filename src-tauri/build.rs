fn main() {
    // Build without windows resources (no icon for now)
    #[cfg(not(target_os = "windows"))]
    tauri_build::build();

    #[cfg(target_os = "windows")]
    {
        // On Windows, skip icon compilation by using attributes without icon
        let attrs = tauri_build::Attributes::new();
        if let Err(e) = tauri_build::try_build(attrs) {
            // If icon is required, just use basic build
            println!("cargo:warning=Skipping Windows resource compilation: {}", e);
        }
    }
}
