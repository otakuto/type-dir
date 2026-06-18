use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::LintError;
use crate::runtime::CheckReport;
use crate::yaml::SpanIndex;

use crate::feature::report::report_to_json;

#[test]
fn missing_required_has_e002_with_parent_name_rule() {
    // Arrange
    let report = CheckReport {
        errors: vec![LintError::MissingRequired {
            parent: PathBuf::from("backend/migrations/0001"),
            name: "down.sql".to_string(),
            is_dir: false,
            rule: "backend".to_string(),
            context: String::new(),
            rule_chain: vec!["backend".to_string()],
            entry_path: None,
        }],
        dirs: vec![],
    };
    let span_index = SpanIndex::default();

    // Act
    let value = report_to_json(&report, &HashMap::new(), &span_index);
    let entry = &value["errors"][0];

    // Assert: uniform schema; parent/name/rule are carried in the message.
    assert_eq!(entry["code"], "LT002");
    let message = entry["message"].as_str().unwrap();
    assert!(
        message.contains("required name not found"),
        "msg: {message}"
    );
    assert!(message.contains("down.sql"), "msg: {message}");
    assert!(
        message.contains("backend/migrations/0001"),
        "msg: {message}"
    );
    assert!(message.contains("backend"), "msg: {message}");
}
