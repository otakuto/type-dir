use crate::walk_impl::read_dir_tree::read_dir_tree;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use super::fixtures::*;

// When a subdirectory is not readable, the error message contains the directory path.
#[test]
fn test_permission_denied_includes_path() {
    // Arrange
    let tmp = TempDir::new("permission-denied");
    let base = &tmp.path;
    let locked = base.join("locked");
    fs::create_dir_all(&locked).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

    // Skip when running as root, where permission bits are not enforced and the
    // directory stays readable, which would otherwise make the assertion fail.
    if fs::read_dir(&locked).is_ok() {
        let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o755));
        return;
    }

    // Act
    let result = read_dir_tree(base, base, &[]);

    // Restore permissions so TempDir::drop can clean up.
    let _ = fs::set_permissions(&locked, fs::Permissions::from_mode(0o755));

    // Assert
    let err = result.expect_err("expected error for unreadable directory");
    let msg = err.to_string();
    assert!(
        msg.contains("locked"),
        "error message does not contain the path: {msg}"
    );
}
