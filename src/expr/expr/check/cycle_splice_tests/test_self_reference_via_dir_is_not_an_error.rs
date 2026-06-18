use super::super::check_splice_cycles;
use crate::yaml::RuleName;

use super::fixtures::{dir_entry_with, rule, splice_entry};

#[test]
fn self_reference_via_dir_is_not_an_error() {
    // Arrange: body of r is a dir entry whose contents splice r (reset by node consumption)
    let mut rules = indexmap::IndexMap::new();
    rules.insert(
        RuleName("r".to_string()),
        rule(vec![dir_entry_with("sub", vec![splice_entry("r")])]),
    );

    // Act
    let errors = check_splice_cycles(&rules);

    // Assert
    assert!(errors.is_empty());
}
