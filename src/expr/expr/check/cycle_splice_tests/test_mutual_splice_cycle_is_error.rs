use super::super::check_splice_cycles;
use crate::error::SemanticError;
use crate::yaml::RuleName;

use super::fixtures::{rule, splice_entry};

#[test]
fn mutual_splice_cycle_is_error() {
    // Arrange: a splices b and b splices a
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("a".to_string()), rule(vec![splice_entry("b")]));
    rules.insert(RuleName("b".to_string()), rule(vec![splice_entry("a")]));

    // Act
    let errors = check_splice_cycles(&rules);

    // Assert
    assert!(!errors.is_empty());
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, SemanticError::InfiniteSplice { .. }))
    );
}
