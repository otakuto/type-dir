use crate::error::SemanticError;
use crate::expr::compile;
use crate::yaml::YamlConfig;

/// Characterization of pass-through removal: if the splice-target rule bare-references a sibling
/// id of the caller node, that id is not self-owned by the splice target, so it is rejected at
/// compile time with E010 (RuleUndeclaredRef).
///
/// In the old implementation (Γ_set pass-through), the caller's id set could reach across the
/// hermetic boundary, but since pass-through was removed and validation (self-owned ids, splice
/// non-transparent) now matches the semantics, the reference is statically rejected.
#[test]
fn test_splice_target_cannot_reference_caller_sibling_id() {
    // Arrange: root declares a sibling (id:s) and splices a separate rule consumer.
    // consumer bare-references ${s}, which it does not own (the caller's id).
    let yaml = indoc::indoc! {r#"
        version: 0
        entry: root
        rules:
          - rule: root
            ::
              - dir: schema
                ::
                  - file:
                      regex: '^(?<name>.+)\.rs$'
                    id: s
              - dir: consumer_dir
                ::
                  - use: rule.consumer
          - rule: consumer
            ::
              - file: '${file.s}.rs'
    "#};
    let config: YamlConfig = serde_yaml::from_str(yaml).expect("yaml parse failed");

    // Act
    let result = compile(config);

    // Assert: consumer does not self-own ${s}, so it becomes a compile error.
    let Err(errors) = result else {
        panic!("reference to caller id did not become a compile error");
    };
    let has_undeclared = errors.0.iter().any(|e| {
        matches!(
            e,
            SemanticError::RuleUndeclaredRef { rule, reference }
                if rule == "consumer" && reference == "file.s"
        )
    });
    assert!(
        has_undeclared,
        "RuleUndeclaredRef(E010) for consumer's ${{file.s}} reference not found: {errors:?}"
    );
}
