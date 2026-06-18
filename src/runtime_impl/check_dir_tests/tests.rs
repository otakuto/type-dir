mod test_bare_dir_requires_empty;
mod test_check_dir_records_dir_trace;
mod test_dir_skip_marker_allows_contents;
mod test_for_source_splice_id_qualified_hop;
mod test_multi_instance_no_bleed_detects_foreign;
mod test_multi_instance_no_bleed_ok;
mod test_optional_splice_relax_does_not_leak_into_child;
mod test_sibling_forward_reference_ok;
mod test_splice_supplies_own_id_on_site;
mod test_splice_target_cannot_reference_caller_sibling_id;
mod test_use_ns_explicit_nav;
mod test_value_binding_interpolates_with_param;
mod test_value_list_drives_for_iteration;
mod test_value_scalar_binding_resolves_in_sibling;

use std::path::Path;

use crate::error::LintError;
use crate::expr::compile;
use crate::walk::DirTree;
use crate::yaml::YamlConfig;

/// YAML for a configuration in which id sets do not bleed across multiple instances.
/// For each `crate_dir`, sql files under `queries` (id:q, self-captured name) are collected,
/// and consumed by `for {id: x, value: ${q}}` under `src` of the same `crate_dir` via `${value.x.regex.name}`.
pub(crate) const MULTI_INSTANCE_YAML: &str = indoc::indoc! {r#"
    version: 0
    entry: root
    rules:
      - rule: root
        ::
          - dir:
              regex: '^crate[0-9]+$'
            ::
              - use: rule.crate_dir
      - rule: crate_dir
        ::
          - dir: queries
            id: queries
            ::
              - file:
                  regex: '^(?<name>.+)\.sql$'
                id: q
          - dir: src
            ::
              - for:
                  id: x
                  value: ${dir.queries.file.q}
                ::
                  - file: '${value.x.regex.name}_sqlx.rs'
"#};

/// Helper that creates a `DirTree` node with child directories and files.
pub(crate) fn dir(name: &str, dirs: Vec<DirTree>, files: &[&str]) -> DirTree {
    DirTree {
        name: name.to_string(),
        dirs,
        files: files.iter().map(|s| s.to_string()).collect(),
    }
}

/// Helper that creates a leaf directory containing only files.
pub(crate) fn leaf(name: &str, files: &[&str]) -> DirTree {
    dir(name, vec![], files)
}

/// Builds a `ConfigExpr` from a YAML string, checks `tree` with base=".", and returns only the diagnostics.
pub(crate) fn run(yaml: &str, tree: &DirTree) -> Vec<LintError> {
    let config: YamlConfig = serde_yaml::from_str(yaml).expect("yaml parse failed");
    let config = compile(config).expect("compile failed");
    super::check_dir(&config, tree, Path::new("."))
        .expect("check_dir failed")
        .errors
}

/// Dumps the trace as a list of (path, rule) pairs (used for assertion failure messages).
pub(crate) fn trace_dump(report: &crate::runtime::CheckReport) -> Vec<(String, String)> {
    report
        .dirs
        .iter()
        .map(|d| (d.path.display().to_string(), d.rule.clone()))
        .collect()
}
