use crate::walk_impl::read_dir_tree::read_dir_tree;
use std::fs;

use super::fixtures::*;

// An anchored path (`sub/skip_me`) skips only that dir;
// a same-named dir at a different location is not skipped.
#[test]
fn test_anchored_path_skips_only_that_path() {
    // Arrange
    let tmp = TempDir::new("anchored-path");
    let base = &tmp.path;
    // base/sub/skip_me (target to skip) and base/other/skip_me (same name, different path).
    fs::create_dir_all(base.join("sub/skip_me")).unwrap();
    fs::create_dir_all(base.join("other/skip_me")).unwrap();

    // Anchor with a leading slash to target only sub/skip_me.
    let ignore_globs = vec!["/sub/skip_me".to_string()];

    // Act
    let tree = read_dir_tree(base, base, &ignore_globs).unwrap();
    let sub = child(&tree, "sub").expect("sub not found");
    let other = child(&tree, "other").expect("other not found");

    // Assert — sub/skip_me is skipped; other/skip_me remains.
    assert!(
        !has_dir(sub, "skip_me"),
        "sub/skip_me was not skipped: {:?}",
        sub.dirs
    );
    assert!(
        has_dir(other, "skip_me"),
        "other/skip_me was incorrectly skipped: {:?}",
        other.dirs
    );
}
