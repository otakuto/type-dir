use crate::walk::DirTree;

/// Returns true if a child directory with the given name exists in `dirs`.
pub fn has_dir(tree: &DirTree, name: &str) -> bool {
    tree.dirs.iter().any(|d| d.name == name)
}
