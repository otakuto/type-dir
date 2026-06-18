use std::fs;
use std::path::PathBuf;

/// Guard that creates a temporary directory and cleans it up on Drop.
pub struct TempDir {
    pub path: PathBuf,
}

impl TempDir {
    pub fn new(label: &str) -> TempDir {
        // Create a unique name using the process ID and label.
        let unique = format!("dir-lint-fs-walk-{}-{}", std::process::id(), label);
        let path = std::env::temp_dir().join(unique);
        // Remove any existing directory, then recreate it.
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("failed to create temporary directory");
        TempDir { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
