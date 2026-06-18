use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::LintError;
use crate::yaml::SpanIndex;

use crate::feature::report::to_json::error_to_json;

#[test]
fn note_field_is_present_when_rule_has_note() {
    // Arrange: provide a notes map with a note for rule "backend_data_dir"
    let error = LintError::MissingRequired {
        parent: PathBuf::from("backend/data"),
        name: "x".to_string(),
        is_dir: false,
        rule: "backend_data_dir".to_string(),
        context: String::new(),
        rule_chain: vec!["backend_data_dir".to_string()],
        entry_path: None,
    };
    let mut notes = HashMap::new();
    notes.insert(
        "backend_data_dir".to_string(),
        "do not place files directly under data/".to_string(),
    );
    let span_index = SpanIndex::default();

    // Act
    let value = error_to_json(&error, &notes, &span_index);

    // Assert
    assert_eq!(value["note"], "do not place files directly under data/");
}
