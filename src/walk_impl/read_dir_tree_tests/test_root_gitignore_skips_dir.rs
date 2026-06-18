use crate::walk_impl::read_dir_tree::read_dir_tree;
use std::fs;

use super::fixtures::*;

// A directory listed in the root .gitignore should not appear in the DirTree.
#[test]
fn test_root_gitignore_skips_dir() {
    // Arrange
    let tmp = TempDir::new("root-gitignore-skips-dir");
    let base = &tmp.path;
    fs::create_dir_all(base.join("cache")).unwrap();
    fs::create_dir_all(base.join("src")).unwrap();
    fs::write(base.join(".gitignore"), "cache/\n").unwrap();

    // Act
    let tree = read_dir_tree(base, base, &[]).unwrap();

    // Assert
    assert!(
        !has_dir(&tree, "cache"),
        "cache was not skipped by .gitignore: {:?}",
        tree.dirs
    );
    assert!(
        has_dir(&tree, "src"),
        "src was incorrectly removed: {:?}",
        tree.dirs
    );
}
