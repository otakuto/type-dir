use crate::walk_impl::read_dir_tree::read_dir_tree;
use std::fs;

use super::fixtures::*;

// A .gitignore in a child directory should cause its grandchild directory to be ignored.
#[test]
fn test_nested_gitignore_skips_grandchild() {
    // Arrange
    let tmp = TempDir::new("nested-gitignore-skips-grandchild");
    let base = &tmp.path;
    fs::create_dir_all(base.join("child/ignored_grandchild")).unwrap();
    fs::create_dir_all(base.join("child/kept_grandchild")).unwrap();
    fs::write(base.join("child/.gitignore"), "ignored_grandchild/\n").unwrap();

    // Act
    let tree = read_dir_tree(base, base, &[]).unwrap();
    let child_tree = child(&tree, "child").expect("child not found");

    // Assert
    assert!(
        !has_dir(child_tree, "ignored_grandchild"),
        "ignored_grandchild was not skipped by child/.gitignore: {:?}",
        child_tree.dirs
    );
    assert!(
        has_dir(child_tree, "kept_grandchild"),
        "kept_grandchild was incorrectly removed: {:?}",
        child_tree.dirs
    );
}
