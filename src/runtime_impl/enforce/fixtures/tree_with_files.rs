use crate::walk::DirTree;

/// Helper that returns a directory tree containing only files.
pub fn tree_with_files(name: &str, files: Vec<&str>) -> DirTree {
    DirTree {
        name: name.to_string(),
        dirs: vec![],
        files: files.into_iter().map(|s| s.to_string()).collect(),
    }
}
