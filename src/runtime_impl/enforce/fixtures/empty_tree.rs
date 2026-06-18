use crate::walk::DirTree;

/// Helper that returns an empty directory tree.
pub fn empty_tree(name: &str) -> DirTree {
    DirTree {
        name: name.to_string(),
        dirs: vec![],
        files: vec![],
    }
}
