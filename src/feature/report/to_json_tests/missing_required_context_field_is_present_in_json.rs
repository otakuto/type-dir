use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::LintError;
use crate::runtime::CheckReport;
use crate::yaml::SpanIndex;

use crate::feature::report::report_to_json;

#[test]
fn missing_required_context_field_is_present_in_json() {
    // Arrange
    let report = CheckReport {
        errors: vec![LintError::MissingRequired {
            parent: PathBuf::from("root"),
            name: "z.rs".to_string(),
            is_dir: false,
            rule: "test_rule".to_string(),
            context: "f=z".to_string(),
            rule_chain: vec!["test_rule".to_string()],
            entry_path: None,
        }],
        dirs: vec![],
    };
    let span_index = SpanIndex::default();

    // Act
    let value = report_to_json(&report, &HashMap::new(), &span_index);

    // Assert: the for-binding provenance is carried in the message as `(context: f=z)`.
    let message = value["errors"][0]["message"].as_str().unwrap();
    assert!(message.contains("context: f=z"), "msg: {message}");
}
