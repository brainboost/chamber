use std::path::PathBuf;

pub fn get_default_workspace_path() -> String {
    if let Some(home) = dirs::home_dir() {
        home.join(".chamber")
            .join("workspace")
            .to_string_lossy()
            .to_string()
    } else {
        ".chamber/workspace".to_string()
    }
}

pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return home.join(path.trim_start_matches("~/"));
        }
    }

    PathBuf::from(path)
}
