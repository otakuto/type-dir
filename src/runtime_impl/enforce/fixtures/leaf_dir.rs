use crate::walk::DirTree;

/// Helper that builds an empty leaf directory tree.
pub fn leaf_dir(name: &str) -> DirTree {
    DirTree {
        name: name.to_string(),
        dirs: vec![],
        files: vec![],
    }
}
