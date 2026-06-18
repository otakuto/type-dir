use crate::yaml::config::YamlEntry;

/// A `use:` value without the `rule.` prefix is rejected with a descriptive error message.
///
/// The `UseRefValue` deserializer requires the form `rule.<name>`. Any value that does not
/// start with `rule.` must produce a clear parse error so that users are immediately informed
/// of the expected syntax.
#[test]
fn use_without_rule_prefix_is_error() {
    // Arrange: `use:` value is a bare name with no `rule.` prefix.
    let yaml = indoc::indoc! {r#"
        use: crate_src_dir
    "#};

    // Act
    let result = serde_yaml::from_str::<YamlEntry>(yaml);

    // Assert: deserialization fails and the error message mentions `rule.<name>`.
    let err = result.expect_err("expected a parse error for missing rule. prefix");
    let msg = err.to_string();
    assert!(
        msg.contains("rule.<name>"),
        "error message should mention `rule.<name>`, got: {msg}"
    );
}

/// A `use:` value with a non-`rule.` prefix is also rejected.
#[test]
fn use_with_wrong_prefix_is_error() {
    // Arrange: `use:` value starts with `ref.` instead of `rule.`.
    let yaml = indoc::indoc! {r#"
        use: ref.crate_src_dir
    "#};

    // Act
    let result = serde_yaml::from_str::<YamlEntry>(yaml);

    // Assert: deserialization fails and the error message mentions `rule.<name>`.
    let err = result.expect_err("expected a parse error for wrong prefix");
    let msg = err.to_string();
    assert!(
        msg.contains("rule.<name>"),
        "error message should mention `rule.<name>`, got: {msg}"
    );
}

/// A `use:` value of exactly `rule.` (empty name after the prefix) is also rejected.
#[test]
fn use_with_empty_rule_name_is_error() {
    // Arrange: `use:` value is `rule.` with no name following the prefix.
    let yaml = indoc::indoc! {r#"
        use: "rule."
    "#};

    // Act
    let result = serde_yaml::from_str::<YamlEntry>(yaml);

    // Assert: deserialization fails.
    let err = result.expect_err("expected a parse error for empty rule name");
    let msg = err.to_string();
    assert!(
        msg.contains("rule.<name>"),
        "error message should mention `rule.<name>`, got: {msg}"
    );
}
