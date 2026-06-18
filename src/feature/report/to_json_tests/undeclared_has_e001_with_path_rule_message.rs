use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::LintError;
use crate::runtime::CheckReport;
use crate::yaml::SpanIndex;

use crate::feature::report::report_to_json;

#[test]
fn undeclared_has_e001_with_path_rule_message() {
    // Arrange
    let report = CheckReport {
        errors: vec![LintError::Undeclared {
            path: PathBuf::from("backend/foo"),
            is_dir: true,
            rule: "crate_dir".to_string(),
            rule_chain: vec!["crate_dir".to_string()],
        }],
        dirs: vec![],
    };
    let span_index = SpanIndex::default();

    // Act
    let value = report_to_json(&report, &HashMap::new(), &span_index);
    let entry = &value["errors"][0];

    // Assert: uniform schema (code/message), with path & rule carried in the message.
    assert_eq!(entry["code"], "LT001");
    assert!(
        entry["note"].is_null(),
        "note should be absent when not set"
    );
    let message = entry["message"].as_str().unwrap();
    assert!(message.contains("undeclared path"), "msg: {message}");
    assert!(message.contains("backend/foo"), "msg: {message}");
    assert!(message.contains("crate_dir"), "msg: {message}");
}
