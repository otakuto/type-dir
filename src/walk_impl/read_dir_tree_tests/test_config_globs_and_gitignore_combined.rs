use crate::walk_impl::read_dir_tree::read_dir_tree;
use std::fs;

use super::fixtures::*;

// Both config ignore_globs and .gitignore should take effect.
#[test]
fn test_config_globs_and_gitignore_combined() {
    // Arrange
    let tmp = TempDir::new("config-globs-and-gitignore-combined");
    let base = &tmp.path;
    fs::create_dir_all(base.join("by_glob")).unwrap();
    fs::create_dir_all(base.join("by_gitignore")).unwrap();
    fs::create_dir_all(base.join("kept")).unwrap();
    fs::write(base.join(".gitignore"), "by_gitignore/\n").unwrap();

    let ignore_globs = vec!["by_glob/".to_string()];

    // Act
    let tree = read_dir_tree(base, base, &ignore_globs).unwrap();

    // Assert
    assert!(
        !has_dir(&tree, "by_glob"),
        "by_glob was not skipped by config globs: {:?}",
        tree.dirs
    );
    assert!(
        !has_dir(&tree, "by_gitignore"),
        "by_gitignore was not skipped by .gitignore: {:?}",
        tree.dirs
    );
    assert!(
        has_dir(&tree, "kept"),
        "kept was incorrectly removed: {:?}",
        tree.dirs
    );
}
