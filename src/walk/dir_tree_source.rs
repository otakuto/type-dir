use std::path::Path;

use crate::walk::DirTree;

/// Port for obtaining a DirTree from a base directory (concrete implementations such as real FS are provided by the walk_impl module).
pub trait DirTreeSource {
    /// Reads the target directory tree rooted at `base`, using `base` as the anchor for ignore globs
    /// and applying the given `ignore` globs.
    fn read(&self, base: &Path, ignore: &[String]) -> Result<DirTree, std::io::Error>;
}
