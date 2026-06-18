use crate::walk::DirTree;

/// Returns the child subtree with the given name, if it exists.
pub fn child<'a>(tree: &'a DirTree, name: &str) -> Option<&'a DirTree> {
    tree.dirs.iter().find(|d| d.name == name)
}
