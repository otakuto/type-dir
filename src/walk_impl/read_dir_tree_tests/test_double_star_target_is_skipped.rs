use crate::walk_impl::read_dir_tree::read_dir_tree;
use std::fs;

use super::fixtures::*;

// `**/target` causes the target dir itself to be skipped anywhere in the tree.
#[test]
fn test_double_star_target_is_skipped() {
    // Arrange
    let tmp = TempDir::new("target-skip");
    let base = &tmp.path;
    // Create base/crate/target and base/crate/src.
    fs::create_dir_all(base.join("crate/target/debug")).unwrap();
    fs::create_dir_all(base.join("crate/src")).unwrap();

    // `**/target/**` only matches the contents of the directory, leaving the target node itself.
    // Use `**/target` to exclude the target directory itself.
    let ignore_globs = vec!["**/target".to_string()];

    // Act
    let tree = read_dir_tree(&base.join("crate"), base, &ignore_globs).unwrap();

    // Assert — target is skipped and src remains.
    assert!(
        !has_dir(&tree, "target"),
        "target was not skipped: {:?}",
        tree.dirs
    );
    assert!(has_dir(&tree, "src"), "src was removed: {:?}", tree.dirs);
}
