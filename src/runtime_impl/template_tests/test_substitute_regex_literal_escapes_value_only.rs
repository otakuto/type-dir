use std::collections::HashMap;

use crate::runtime_impl::template::substitute_regex_literal;

/// Regex-context interpolation literal-escapes variable values with `regex::escape`; fixed parts are not escaped.
#[test]
fn test_substitute_regex_literal_escapes_value_only() {
    // Arrange: v=a.b (contains metacharacter `.`). Fixed parts `^` and `$` are not escaped.
    let mut scope = HashMap::new();
    scope.insert("v".to_string(), "a.b".to_string());

    // Act
    let result = substitute_regex_literal("^${v}$", &scope);

    // Assert: only the `.` in the value is escaped to `\.`.
    assert_eq!(result, r"^a\.b$");
}
