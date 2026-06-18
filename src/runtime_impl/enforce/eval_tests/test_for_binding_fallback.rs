use std::path::Path;

use indexmap::IndexMap;

use crate::expr::{
    ExprEntry, ExprForSource, ExprMatcher, ExprPattern, ExprRule, ExprSubtree, Quant,
};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::env::Scope;
use crate::walk::DirTree;
use crate::yaml::{EntryId, RegexPattern, RuleName, VarName};

/// Out-of-binding fallback union: when `${m}` is referenced from outside the `for` binding (a sibling),
/// the union of all records collected across all bindings is visible (because overlay transparently merges across For).
///
/// Structure (pseudo-YAML):
/// ```yaml
/// - for: {id: layer, value: ["aaa", "bbb"]}
///   rules:
///     - dir:
///         regex: '^${value.layer}-pkg-(?<stem>.+)$'
///       id: m          # fields are auto-collected from own captures (stem)
/// - dir: all-docs
///   rules:
///     - for: {id: n, value: ${m}}       # iterate m records from outside the binding (union)
///       rules:
///         - file: '${value.n.stem}.txt'
/// ```
#[test]
fn test_for_binding_external_reference_sees_union() {
    // Arrange
    let pkg_entry = ExprEntry {
        id: Some(EntryId("m".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Regex(RegexPattern(
                r"^${value.layer}-pkg-(?<stem>.+)$".to_string(),
            )),
            subtree: ExprSubtree::Leaf,
        },
    };
    let for_layer = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("layer".to_string()),
            source: ExprForSource::Literal(vec!["aaa".to_string(), "bbb".to_string()]),
            body: vec![pkg_entry],
        },
    };

    // Sibling (out-of-binding) all-docs: for n in ${m} { file '${n.stem}.txt' }
    let inner_for = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("n".to_string()),
            source: ExprForSource::Expr("${dir.m}".to_string()),
            body: vec![ExprEntry {
                id: None,
                source_path: None,
                count: Quant::Default,
                matcher: ExprMatcher::File {
                    pattern: ExprPattern::Exact("${value.n.stem}.txt".to_string()),
                    subtree: ExprSubtree::Leaf,
                },
            }],
        },
    };
    let all_docs = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Exact("all-docs".to_string()),
            subtree: ExprSubtree::Inline(vec![inner_for]),
        },
    };

    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![
            DirTree {
                name: "aaa-pkg-x".to_string(),
                dirs: vec![],
                files: vec![],
            },
            DirTree {
                name: "bbb-pkg-y".to_string(),
                dirs: vec![],
                files: vec![],
            },
            DirTree {
                name: "all-docs".to_string(),
                dirs: vec![],
                files: vec!["x.txt".to_string(), "y.txt".to_string()],
            },
        ],
        files: vec![],
    };
    let scope = Scope::new();
    let rules: IndexMap<RuleName, ExprRule> = IndexMap::new();
    let path = Path::new("root");

    // Act
    let mut errors = Vec::new();
    eval_node(
        &tree,
        &[for_layer, all_docs],
        &scope,
        &rules,
        path,
        "test_rule",
        &mut errors,
        &mut crate::runtime_impl::record_map::RecordMap::new(),
        &mut TrialMemo::new(),
    );

    // Assert: all-docs sees the union m={x,y}, requires both x.txt and y.txt, no errors
    assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
}
