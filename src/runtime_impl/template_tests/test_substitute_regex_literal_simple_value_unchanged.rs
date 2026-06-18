use std::collections::HashMap;

use crate::runtime_impl::template::substitute_regex_literal;

/// Simple values without metacharacters look the same after escaping and expand as usual.
#[test]
fn test_substitute_regex_literal_simple_value_unchanged() {
    // Arrange: layer=usecase (no metacharacters).
    let mut scope = HashMap::new();
    scope.insert("layer".to_string(), "usecase".to_string());

    // Act
    let result = substitute_regex_literal("^myapp-${layer}-", &scope);

    // Assert: no metacharacters, so the result is identical after escaping.
    assert_eq!(result, "^myapp-usecase-");
}
