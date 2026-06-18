use super::super::check_splice_cycles;
use crate::yaml::RuleName;

use super::fixtures::{for_entry_with, rule, splice_entry};

/// A self-splice guarded by `for` is data-driven and converges, so it must NOT be reported as
/// InfiniteSplice. The `for` iterates a finite captured set and recurses on a strictly smaller
/// subset, so `for` acts as a splice-cycle boundary (like a dir/file node).
#[test]
fn self_splice_under_for_is_not_an_error() {
    // Arrange: body of r is `for x in ${something}: - use: rule.r` (self-splice under a for guard).
    let mut rules = indexmap::IndexMap::new();
    rules.insert(
        RuleName("r".to_string()),
        rule(vec![for_entry_with(
            "x",
            "${something}",
            vec![splice_entry("r")],
        )]),
    );

    // Act
    let errors = check_splice_cycles(&rules);

    // Assert
    assert!(errors.is_empty(), "unexpected: {errors:?}");
}
