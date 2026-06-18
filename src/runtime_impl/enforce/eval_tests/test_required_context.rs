use std::path::Path;

use indexmap::IndexMap;

use crate::error::LintError;
use crate::expr::{ExprEntry, ExprForSource, ExprMatcher, ExprPattern, ExprRule, Quant};
use crate::runtime_impl::enforce::TrialMemo;
use crate::runtime_impl::enforce::eval::eval_node;
use crate::runtime_impl::enforce::fixtures::{empty_scope, make_file_entry};
use crate::runtime_impl::value::Record;
use crate::walk::DirTree;
use crate::yaml::VarName;

/// Helper that constructs a `for` entry iterating over an expression source.
fn make_for_entry(var: &str, expr: &str, rules: Vec<ExprEntry>) -> ExprEntry {
    ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName(var.to_string()),
            source: ExprForSource::Expr(expr.to_string()),
            body: rules,
        },
    }
}

/// When a required file is missing inside a `for` binding, `MissingRequired.context`
/// contains the bound variable name and the full match (e.g. `"f=z"`).
#[test]
fn missing_required_inside_for_binding_includes_context() {
    // Arrange: sets["src"] = [Record { fields{"0":"z"} }].
    // for {id: f, value: ${src}} { file: '${value.f.regex.0}.rs' } requires z.rs, but the tree is empty.
    let file_entry = make_file_entry(
        ExprPattern::Exact("${value.f.regex.0}.rs".to_string()),
        None,
    );
    let for_entry = make_for_entry("f", "${src}", vec![file_entry]);
    let tree = DirTree {
        name: "root".to_string(),
        dirs: vec![],
        files: vec![],
    };
    let entries = vec![for_entry];
    let rules: IndexMap<_, ExprRule> = IndexMap::new();
    let path = Path::new("root");

    let mut rec_z = Record::default();
    rec_z.fields.insert("0".to_string(), "z".to_string());
    let mut scope = empty_scope();
    // Place the record set producer on the env side (Dir). The for source `${src}` resolves via transparent get.
    scope.bind_env(
        crate::runtime_impl::node_id::NodeKind::Dir,
        "src",
        vec![rec_z],
    );

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

    // Assert: one MissingRequired error whose context contains "f=z".
    assert_eq!(errors.len(), 1, "unexpected errors: {:?}", errors);
    let LintError::MissingRequired { context, .. } = &errors[0] else {
        panic!("expected MissingRequired, got {:?}", errors[0]);
    };
    assert!(
        context.contains("f=z"),
        "expected context to contain 'f=z', got: {:?}",
        context
    );
}
