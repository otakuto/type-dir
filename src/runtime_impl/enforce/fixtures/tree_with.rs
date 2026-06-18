use crate::walk::DirTree;

/// Helper that constructs a root DirTree containing only files.
pub fn tree_with(files: &[&str]) -> DirTree {
    DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: files.iter().map(|s| s.to_string()).collect(),
    }
}
