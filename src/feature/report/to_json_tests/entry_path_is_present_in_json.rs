use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::LintError;
use crate::runtime::CheckReport;
use crate::yaml::build_span_index;

use crate::feature::report::report_to_json;

#[test]
fn entry_path_is_present_in_json() {
    // Arrange
    let yaml = "version: 0\nentry: root\nrules:\n  - rule: root\n    ::\n      - file: foo.rs\n";
    let span_index = build_span_index(yaml);
    let report = CheckReport {
        errors: vec![LintError::MissingRequired {
            parent: PathBuf::from("root"),
            name: "foo.rs".to_string(),
            is_dir: false,
            rule: "root".to_string(),
            context: String::new(),
            rule_chain: vec!["root".to_string()],
            entry_path: Some("rules.root.rules[0]".to_string()),
        }],
        dirs: vec![],
    };

    // Act
    let value = report_to_json(&report, &HashMap::new(), &span_index);
    let entry = &value["errors"][0];

    // Assert
    assert_eq!(entry["entry_path"], "rules.root.rules[0]");
    assert!(
        entry["entry_span"]["start"].is_number(),
        "entry_span.start must be present"
    );
    assert!(
        entry["entry_span"]["end"].is_number(),
        "entry_span.end must be present"
    );
}
