use std::path::Path;

use crate::walk::{DirTree, DirTreeSource};

/// DirTreeSource implementation that walks the real filesystem.
pub struct RealDirTreeSource;

impl DirTreeSource for RealDirTreeSource {
    fn read(&self, base: &Path, ignore: &[String]) -> Result<DirTree, std::io::Error> {
        crate::walk_impl::read_dir_tree::read_dir_tree(base, base, ignore)
    }
}
