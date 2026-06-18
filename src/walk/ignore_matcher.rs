use std::io;
use std::path::Path;

use ignore::gitignore::{Gitignore, GitignoreBuilder};

/// Builds a `Gitignore` matcher from `ignore_globs` anchored at `base`.
/// Each glob is interpreted as a gitignore-format line with `base` as the anchor.
///
/// A malformed glob is surfaced as an `io::Error` (the caller maps it to the appropriate layer
/// error, e.g. `RuntimeError::InvalidIgnoreGlob`).
pub fn build_ignore_matcher(base: &Path, ignore_globs: &[String]) -> Result<Gitignore, io::Error> {
    let mut builder = GitignoreBuilder::new(base);
    for glob in ignore_globs {
        builder.add_line(None, glob).map_err(io::Error::other)?;
    }
    let matcher = builder.build().map_err(io::Error::other)?;
    Ok(matcher)
}
