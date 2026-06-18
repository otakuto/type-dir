use super::super::check_splice_cycles;
use crate::error::SemanticError;
use crate::yaml::RuleName;

use super::fixtures::{rule, splice_entry};

#[test]
fn direct_self_splice_is_infinite_recursion_error() {
    // Arrange: body of r splices r itself
    let mut rules = indexmap::IndexMap::new();
    rules.insert(RuleName("r".to_string()), rule(vec![splice_entry("r")]));

    // Act
    let errors = check_splice_cycles(&rules);

    // Assert
    assert_eq!(errors.len(), 1);
    assert!(matches!(&errors[0], SemanticError::InfiniteSplice { .. }));
}
