use std::collections::HashMap;

use crate::runtime_impl::template::substitute;

#[test]
fn test_substitute_leaves_id_dot_var() {
    // Arrange
    let mut scope = HashMap::new();
    scope.insert("domain".to_string(), "application".to_string());

    // Act
    // dotted ${id.var} references are left as-is.
    let result = substitute("myapp-${layer}-${crate.domain}", &scope);

    // Assert
    assert_eq!(result, "myapp-${layer}-${crate.domain}");
}
