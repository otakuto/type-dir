use std::path::Path;

use crate::expr::{ConfigExpr, ConfigSource};

/// `ConfigSource` implementation that loads `ConfigExpr` from a YAML file.
#[allow(dead_code)]
pub struct YamlConfigSource;

impl ConfigSource for YamlConfigSource {
    fn load(&self, path: &Path) -> anyhow::Result<ConfigExpr> {
        let yaml = crate::yaml::load_yaml_config(path)?;
        // ConfigErrors implements Error, so it can be wrapped in anyhow directly.
        crate::expr::compile(yaml).map_err(anyhow::Error::new)
    }
}
