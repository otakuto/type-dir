use crate::yaml::{EntryId, RuleName, WithShape};

use super::fixtures::*;

/// A dotted string with two `<kind>.<name>` pairs `rule.a_tree.dir.anode.dir.child` produces
/// `path: [anode, child]`. The kinds are required syntactically; only the names are kept.
#[test]
fn dotted_rule_id_with_two_segments_produces_two_element_path() {
    // Arrange
    let shape = parse("rule.a_tree.dir.anode.dir.child");

    // Act
    let result = shape.to_shape();

    // Assert
    assert_eq!(
        result.unwrap(),
        WithShape::RuleType {
            rule: RuleName("a_tree".to_string()),
            path: vec![EntryId("anode".to_string()), EntryId("child".to_string()),],
        }
    );
}
