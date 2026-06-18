use std::path::Path;

use crate::expr::compile;
use crate::yaml::YamlConfig;

use super::{MULTI_INSTANCE_YAML, dir, leaf, trace_dump};

/// Dir trace: check_dir records the rule applied to each visited directory in `dirs`.
#[test]
fn test_check_dir_records_dir_trace() {
    // Arrange: crate1/crate2 layout (MULTI_INSTANCE_YAML).
    let crate1 = dir(
        "crate1",
        vec![leaf("queries", &["a.sql"]), leaf("src", &["a_sqlx.rs"])],
        &[],
    );
    let tree = dir("", vec![crate1], &[]);
    let config: YamlConfig = serde_yaml::from_str(MULTI_INSTANCE_YAML).expect("yaml parse failed");
    let config = compile(config).expect("compile failed");

    // Act
    let report = super::super::check_dir(&config, &tree, Path::new(".")).expect("check_dir failed");

    // Assert: the trace contains root (path "", rule root) and crate1 (rule crate_dir).
    let has = |path: &str, rule: &str| {
        report
            .dirs
            .iter()
            .any(|d| d.path.as_path() == Path::new(path) && d.rule.as_str() == rule)
    };
    assert!(
        has("", "root"),
        "root trace not found: {:?}",
        trace_dump(&report)
    );
    assert!(
        has("crate1", "crate_dir"),
        "crate1(crate_dir) trace not found: {:?}",
        trace_dump(&report)
    );
}
