#[cfg(test)]
#[path = "read_dir_tree_tests/tests.rs"]
mod tests;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::walk::{DirTree, build_ignore_matcher};
use ignore::WalkBuilder;
use ignore::gitignore::Gitignore;

/// (list of directory names, list of file names) directly under a single directory.
type Children = (Vec<String>, Vec<String>);

/// Map from directory path (relative to root) to its direct `Children`.
type ChildMap = HashMap<PathBuf, Children>;

/// Reads the directory tree rooted at the specified root.
/// Symbolic links are not followed.
///
/// `base` is the anchor directory for ignore globs (e.g., repository root),
/// and `root` is the directory to read (normally under `base`).
///
/// `.gitignore`, global gitignore, `.git/info/exclude`, and parent-directory gitignore rules are
/// delegated to `ignore::WalkBuilder`. The config `ignore:` globs are evaluated separately as the
/// least-specific matcher. Dotfiles are kept in the lint target set via `hidden(false)`; instead,
/// only the `.git` directory is excluded manually (the `ignore` crate does not exclude `.git` on
/// its own).
pub fn read_dir_tree(
    root: &Path,
    base: &Path,
    ignore_globs: &[String],
) -> Result<DirTree, std::io::Error> {
    let config_matcher = build_ignore_matcher(base, ignore_globs)?;

    let mut builder = WalkBuilder::new(root);
    builder
        // Enable .gitignore / global gitignore / .git/info/exclude / parent gitignore.
        .standard_filters(true)
        // Do not exclude dotfiles (keep .dir-lint.yaml and .github/ in the lint target set).
        .hidden(false)
        // Apply .gitignore even outside a git repository (test fixtures run in temp directories).
        .require_git(false)
        // Do not follow symlinked directories.
        .follow_links(false);

    builder.filter_entry({
        let m = config_matcher.clone();
        move |dent| !should_skip(&m, dent)
    });

    let mut children = collect_children(root, &builder, &config_matcher)?;
    Ok(build_tree(root, &mut children))
}

/// Excludes entries that are the `.git` directory or match config globs from the walk.
/// Returning `false` prevents the entry from being yielded and skips descent into directories.
fn should_skip(config_matcher: &Gitignore, dent: &ignore::DirEntry) -> bool {
    let is_dir = dent.file_type().is_some_and(|ft| ft.is_dir());

    // Exclude the .git directory (the only manual exclusion not handled by the ignore crate).
    if is_dir && dent.file_name() == ".git" {
        return true;
    }

    config_matcher.matched(dent.path(), is_dir).is_ignore()
}

/// Collects the list of directory names and file names directly under each directory (relative to
/// root). Empty directories are also registered as keys so that they become nodes in the next
/// stage.
fn collect_children(
    root: &Path,
    builder: &WalkBuilder,
    config_matcher: &Gitignore,
) -> Result<ChildMap, std::io::Error> {
    let mut children: ChildMap = HashMap::new();
    // Pre-register root itself as an empty entry so a node is created even if it has no children.
    children.insert(root.to_path_buf(), (Vec::new(), Vec::new()));

    for result in builder.build() {
        let dent = result.map_err(ignore_error_to_io)?;

        // Skip root itself because it is handled separately as the tree root.
        if dent.depth() == 0 {
            continue;
        }

        let path = dent.path();
        let name = dent.file_name().to_string_lossy().into_owned();
        let parent = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| root.to_path_buf());

        let file_type = match dent.file_type() {
            Some(ft) => ft,
            // Entries with no file_type (only root) never reach here.
            None => continue,
        };

        if file_type.is_symlink() {
            // symlink: follow the target and include it if it resolves to a file; ignore if a directory.
            if let Ok(target_meta) = std::fs::metadata(path)
                && target_meta.is_file()
                && !config_matcher.matched(path, false).is_ignore()
            {
                children.entry(parent).or_default().1.push(name);
            }
            continue;
        }

        if file_type.is_dir() {
            // Register the directory under its parent and also create its own entry (even if empty).
            children.entry(parent).or_default().0.push(name);
            children.entry(path.to_path_buf()).or_default();
        } else if file_type.is_file() {
            children.entry(parent).or_default().1.push(name);
        }
    }

    Ok(children)
}

/// Recursively builds a DirTree from the collected children map, starting from root.
/// Directory names and file names are sorted to produce a stable order.
fn build_tree(path: &Path, children: &mut ChildMap) -> DirTree {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let (mut dir_names, mut files) = children.remove(path).unwrap_or_default();
    dir_names.sort();
    files.sort();

    let dirs = dir_names
        .iter()
        .map(|dir_name| build_tree(&path.join(dir_name), children))
        .collect();

    DirTree { name, dirs, files }
}

/// Converts `ignore::Error` into `std::io::Error`.
/// Since `Display` includes the path, all information needed for diagnosis is preserved.
fn ignore_error_to_io(err: ignore::Error) -> std::io::Error {
    let message = err.to_string();
    match err.into_io_error() {
        Some(io_err) => std::io::Error::new(io_err.kind(), message),
        None => std::io::Error::other(message),
    }
}
