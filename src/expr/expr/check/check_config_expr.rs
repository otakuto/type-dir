use indexmap::IndexMap;

use crate::error::SemanticError;
use crate::yaml::{RuleName, YamlRule};

use super::internal::id_shape_derive::build_id_shapes;
use super::internal::{
    check_capture_requires_id, check_duplicate_id, check_entry_combination, check_entry_refs,
    check_fetch, check_match_exhaustive, check_pattern, check_rule_var_scope, check_undefined_rule,
    check_with_compat, check_with_keys, check_with_shapes,
};

/// Checks the static consistency of a `YamlConfig` and returns all found errors.
///
/// Checks performed:
/// 1. Whether referenced rules in rules are defined (check_undefined_rule)
/// 2. Field combinations in entries (prohibiting coexistence of dir/file and rule, splice uniqueness) (check_entry_combination)
/// 3. Whether with keys are within the declared range of the referenced rule (check_with_keys)
/// 4. Whether the rule body references only variables from with params/captures/self-owned ids (check_rule_var_scope)
/// 5. Whether ids are globally unique across all rules (check_duplicate_id)
/// 6. Whether entry points to a defined rule (check_entry_refs)
/// 7. Whether shape declarations in with are parseable and field references are consistent (check_with_shapes)
/// 8. Whether caller-side with declarations match the static shape of the passed id (check_with_compat / E018, E021)
/// 9. Whether each `match` exhaustively covers its scrutinee Sum's tags and the scrutinee is a Sum (check_match_exhaustive / E024, E025)
/// 10. Whether named captures without `id` exist on dir/file entries (check_capture_requires_id / E026)
/// 11. Whether all `fetch` alternatives share the same named capture set (check_fetch / E027)
pub fn check_config_expr_yaml(
    rules: &IndexMap<RuleName, YamlRule>,
    entry: &RuleName,
) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    errors.extend(check_undefined_rule(rules));
    errors.extend(check_entry_combination(rules));
    errors.extend(check_with_keys(rules));
    errors.extend(check_rule_var_scope(rules));
    errors.extend(check_duplicate_id(rules));
    errors.extend(check_entry_refs(entry, rules));
    errors.extend(check_pattern(rules));
    // Build the global id→shape map once and share it between the two checks that need it,
    // avoiding a redundant full-AST traversal.
    let id_shapes = build_id_shapes(rules);
    errors.extend(check_with_shapes(rules, &id_shapes));
    errors.extend(check_with_compat(rules, &id_shapes));
    errors.extend(check_match_exhaustive(rules));
    errors.extend(check_capture_requires_id(rules));
    errors.extend(check_fetch(rules));
    errors
}
