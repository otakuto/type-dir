use std::collections::HashMap;

use crate::runtime_impl::template::substitute;

#[test]
fn test_substitute_no_placeholders() {
    // Arrange
    let scope = HashMap::new();

    // Act
    let result = substitute("literal-string", &scope);

    // Assert
    assert_eq!(result, "literal-string");
}
