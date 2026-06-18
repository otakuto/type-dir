use crate::expr::expr::check::internal::id_shape_derive::derive_rule_id_shape;

use super::fixtures::empty_rule;

/// For a recursive rule (e.g. feature_dir that splices itself via a child), the derivation
/// terminates (no infinite loop) due to lazy RuleRef and visited set.
#[test]
fn derive_shape_terminates_for_self_referential_rule() {
    use crate::yaml::{EntryId, RuleName as RN, YamlEntry, YamlEntryKind, YamlPattern};
    use indexmap::IndexMap;

    // Arrange: feature_dir has id `feat`, and inside feat's rules it splices feature_dir again
    // feat_entry: dir with id `feat`
    //   rules:
    //     - use: rule.feature_dir  (self-splice — recursive)
    let self_splice = YamlEntry {
        id: None,
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Use {
            rule: RN("feature_dir".to_string()),
            with_args: IndexMap::new(),
            colocated_rules: None,
        },
    };
    let feat_entry = YamlEntry {
        id: Some(EntryId("feat".to_string())),
        optional: None,
        min: None,
        max: None,
        count: None,
        kind: YamlEntryKind::Dir {
            pattern: YamlPattern::Exact("feat".to_string()),
            body: Some(vec![self_splice]),
            colocated_use_ref: None,
        },
    };
    let feature_dir = empty_rule(vec![feat_entry]);
    let mut rules = IndexMap::new();
    rules.insert(RN("feature_dir".to_string()), feature_dir);

    // Act: must terminate
    let shape = derive_rule_id_shape(&RN("feature_dir".to_string()), &rules);

    // Assert: shape is Some (public id `feat` is found), and child_ids contains `feat` as RuleRef
    let shape = shape.expect("shape must be Some");
    assert!(
        shape.child_ids.contains_key("feat"),
        "expected child_id `feat` in shape, got: {:?}",
        shape.child_ids
    );
}
