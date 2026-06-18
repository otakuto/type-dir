use crate::expr::expr::check::internal::id_shape_derive::derive_rule_id_shape;
use crate::yaml::RuleName;

use super::fixtures::{empty_rule, id_file_entry};

/// For a rule with a single public id bearing a capture, the derived IdShape contains the capture.
#[test]
fn derive_shape_for_rule_with_single_public_id() {
    // Arrange: feature_dir has public id `feat` with capture `stem`
    let feature_dir = {
        let entry = id_file_entry("feat", r"^(?<stem>[a-z_]+)$");
        empty_rule(vec![entry])
    };
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("feature_dir".to_string()), feature_dir);

    // Act
    let shape = derive_rule_id_shape(&RuleName("feature_dir".to_string()), &rules);

    // Assert: shape has capture `stem`
    let shape = shape.expect("shape must be Some for a rule with one public id");
    assert!(
        shape.captures.contains("stem"),
        "expected capture `stem` in shape.captures, got: {:?}",
        shape.captures
    );
}
