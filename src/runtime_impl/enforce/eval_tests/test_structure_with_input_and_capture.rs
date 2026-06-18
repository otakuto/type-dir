use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprMatcher, ExprPattern, ExprRule, ExprSubtree, Interval, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, make_file_entry, splice_entry};
use crate::walk::DirTree;
use crate::yaml::{RegexPattern, RuleName, VarName, WithShape};

#[test]
fn test_structure_with_input_and_capture() {
    // Arrange: layer_crate is a content-model rule that describes dir names
    // (regex takes layer as input and domain as a capture). The parent splices it with count{0,1} (optional).
    let mut layer_crate_with_params = IndexMap::new();
    layer_crate_with_params.insert(VarName("layer".to_string()), WithShape::Scalar);

    let crate_dir_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Explicit(Interval {
            min: 0,
            max: Some(1),
        }),
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Regex(RegexPattern(
                r"^myapp-${layer}-(?<domain>[a-z][a-z0-9-]*)$".to_string(),
            )),
            subtree: ExprSubtree::Inline(vec![make_file_entry(
                ExprPattern::Exact("Cargo.toml".to_string()),
                Some((0, Some(1))),
            )]),
        },
    };
    let layer_crate_structure = ExprRule {
        with_params: layer_crate_with_params,
        note: None,
        rules: vec![crate_dir_entry],
    };
    let rule_name = RuleName("layer_crate".to_string());
    let mut rules = IndexMap::new();
    rules.insert(rule_name.clone(), layer_crate_structure);

    let foo_dir = DirTree {
        name: "myapp-usecase-foo".to_string(),
        dirs: vec![],
        files: vec![],
    };
    let other_dir = DirTree {
        name: "myapp-other-foo".to_string(),
        dirs: vec![],
        files: vec![],
    };
    let tree = DirTree {
        name: "parent".to_string(),
        dirs: vec![foo_dir, other_dir],
        files: vec![],
    };

    let mut input_map = IndexMap::new();
    input_map.insert(VarName("layer".to_string()), "usecase".to_string());
    let entries = vec![splice_entry("layer_crate", input_map)];
    let scope = empty_scope();
    let path = Path::new("parent");

    // Act
    let mut errors = Vec::new();
    eval_node(
        &tree,
        &entries,
        &scope,
        &rules,
        path,
        "test_rule",
        &mut errors,
        &mut crate::runtime_impl::record_map::RecordMap::new(),
        &mut TrialMemo::new(),
    );

    // Assert: with layer=usecase, myapp-usecase-foo matches; myapp-other-foo is Undeclared
    assert_eq!(errors.len(), 1);
    let LintError::Undeclared { path: err_path, .. } = &errors[0] else {
        panic!("expected Undeclared, got {:?}", errors[0]);
    };
    assert_eq!(err_path, &PathBuf::from("parent/myapp-other-foo"));
}
