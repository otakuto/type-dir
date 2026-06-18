use crate::walk_impl::read_dir_tree::read_dir_tree;
use std::fs;

use super::fixtures::*;

// When the parent .gitignore ignores *.log and the child .gitignore re-includes !keep.log,
// keep.log should appear in the DirTree.
#[test]
fn test_gitignore_negation_reinclude() {
    // Arrange
    let tmp = TempDir::new("gitignore-negation-reinclude");
    let base = &tmp.path;
    fs::create_dir_all(base.join("sub")).unwrap();
    fs::write(base.join(".gitignore"), "*.log\n").unwrap();
    fs::write(base.join("sub/.gitignore"), "!keep.log\n").unwrap();
    fs::write(base.join("sub/keep.log"), "").unwrap();
    fs::write(base.join("sub/discard.log"), "").unwrap();

    // Act
    let tree = read_dir_tree(base, base, &[]).unwrap();
    let sub_tree = child(&tree, "sub").expect("sub not found");

    // Assert
    assert!(
        sub_tree.files.contains(&"keep.log".to_string()),
        "keep.log was not reincluded by negation: {:?}",
        sub_tree.files
    );
    assert!(
        !sub_tree.files.contains(&"discard.log".to_string()),
        "discard.log was not ignored: {:?}",
        sub_tree.files
    );
}
