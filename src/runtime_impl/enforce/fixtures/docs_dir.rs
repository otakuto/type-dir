use crate::walk::DirTree;

/// Helper that builds a directory tree with files (used for docs-type dirs in tests).
pub fn docs_dir(name: &str, files: &[&str]) -> DirTree {
    DirTree {
        name: name.to_string(),
        dirs: vec![],
        files: files.iter().map(|s| s.to_string()).collect(),
    }
}
