use std::collections::HashMap;

use crate::runtime_impl::template::substitute;

#[test]
fn test_substitute_replaces_known_var() {
    // Arrange
    let mut scope = HashMap::new();
    scope.insert("layer".to_string(), "usecase".to_string());

    // Act
    let result = substitute("myapp-${layer}-foo", &scope);

    // Assert
    assert_eq!(result, "myapp-usecase-foo");
}
