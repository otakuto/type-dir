use std::path::Path;

use crate::expr::ConfigExpr;

/// Port for obtaining a `ConfigExpr` from a configuration source (concrete implementations
/// such as YAML are provided by config-impl).
#[allow(dead_code)]
pub trait ConfigSource {
    /// Loads configuration from the specified path and returns a validated `ConfigExpr`.
    ///
    /// Returns `anyhow::Result` to handle both I/O/parse errors and validation errors.
    /// Validation errors are wrapped in `anyhow` as `ConfigErrors`.
    fn load(&self, path: &Path) -> anyhow::Result<ConfigExpr>;
}
