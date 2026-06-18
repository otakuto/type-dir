use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::LintError;
use crate::runtime::CheckReport;
use crate::yaml::SpanIndex;

use crate::feature::report::report_to_json;

#[test]
fn report_has_errors_and_dir() {
    // Arrange
    let report = CheckReport {
        errors: vec![LintError::Undeclared {
            path: PathBuf::from("extra.txt"),
            is_dir: false,
            rule: "root".to_string(),
            rule_chain: vec!["root".to_string()],
        }],
        dirs: vec![crate::runtime::DirTrace {
            path: PathBuf::from("backend"),
            rule: "crate_dir".to_string(),
        }],
    };
    let span_index = SpanIndex::default();

    // Act
    let value = report_to_json(&report, &HashMap::new(), &span_index);

    // Assert
    assert_eq!(value["errors"][0]["code"], "LT001");
    assert_eq!(value["dir"][0]["path"], "backend");
    assert_eq!(value["dir"][0]["rule"], "crate_dir");
}
