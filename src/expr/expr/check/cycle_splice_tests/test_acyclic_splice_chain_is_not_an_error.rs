use super::super::check_splice_cycles;
use crate::yaml::RuleName;

use super::fixtures::{rule, splice_entry};

#[test]
fn acyclic_splice_chain_is_not_an_error() {
    // Arrange: a -> b -> c (no cycle)
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("a".to_string()), rule(vec![splice_entry("b")]));
    rules.insert(RuleName("b".to_string()), rule(vec![splice_entry("c")]));
    rules.insert(RuleName("c".to_string()), rule(vec![]));

    // Act
    let errors = check_splice_cycles(&rules);

    // Assert
    assert!(errors.is_empty());
}
