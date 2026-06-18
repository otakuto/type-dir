use crate::runtime_impl::env::{Scope, ScopeRef};
use crate::runtime_impl::node_id::NodeKind;
use crate::runtime_impl::value::{Record, Value};

/// Even when the iteration variable name and the for id share the same name (as in `for i in ... / id: i`),
/// lex `(Value, "i")` and env `(For, "i")` coexist without interfering.
///
/// - `${value.i}` (= lookup_lex(Value, "i")) returns the iteration variable.
/// - `${for.i}` (= lookup_env(For, "i")) returns the accumulated record set.
/// - bare `${i}` (= transparent get) returns the iteration variable (lex-first).
#[test]
fn test_same_name_value_and_for_coexist() {
    // Arrange: register the iteration variable (Value,"i") as a scalar and the for accumulation (For,"i") as a record set.
    let mut scope = Scope::new();
    scope.bind_lex(NodeKind::Value, "i", Value::Scalar("iter".to_string()));
    let mut rec = Record::default();
    rec.fields.insert("0".to_string(), "rec0".to_string());
    scope.bind_env(NodeKind::For, "i", vec![rec.clone()]);

    // Act
    let lex_side = scope.lookup_lex(NodeKind::Value, "i").cloned();
    let env_side = scope.lookup_env(NodeKind::For, "i").map(<[Record]>::to_vec);
    let bare = scope.get("i");

    // Assert: both sides are held independently; bare resolves to the iteration variable (lex-first).
    assert_eq!(lex_side, Some(Value::Scalar("iter".to_string())));
    assert_eq!(env_side, Some(vec![rec]));
    assert_eq!(
        bare,
        Some(ScopeRef::Lex(&Value::Scalar("iter".to_string())))
    );
    // Pulling the env side does not corrupt the lex side (Value,"i").
    assert!(scope.lookup_lex(NodeKind::Value, "i").is_some());
    // Pulling the lex side does not corrupt the env side (For,"i").
    assert!(scope.lookup_env(NodeKind::For, "i").is_some());
}

/// When the same `(kind, id)` exists in both lex and env maps, the transparent `get` resolves
/// with lex priority.
///
/// lex and env are separated by value type; kind does not determine which side a binding goes into.
/// Even when the same `(Dir, "x")` exists in both sides, they coexist without overwriting each other,
/// and `get` checks lex first and returns the lex side.
#[test]
fn test_get_prefers_lex_over_env_on_identical_kind_and_id() {
    // Arrange: place the same (Dir, "x") in both the lex side (Record) and env side (record set).
    let mut scope = Scope::new();
    let mut lex_rec = Record::default();
    lex_rec.fields.insert("0".to_string(), "lex".to_string());
    scope.bind_lex(NodeKind::Dir, "x", Value::Record(lex_rec.clone()));
    let mut env_rec = Record::default();
    env_rec.fields.insert("0".to_string(), "env".to_string());
    scope.bind_env(NodeKind::Dir, "x", vec![env_rec.clone()]);

    // Act
    let bare = scope.get("x");
    let lex_side = scope.lookup_lex(NodeKind::Dir, "x").cloned();
    let env_side = scope.lookup_env(NodeKind::Dir, "x").map(<[Record]>::to_vec);

    // Assert: get returns the lex side Record (lex-first). Both maps are held independently.
    assert_eq!(bare, Some(ScopeRef::Lex(&Value::Record(lex_rec.clone()))));
    assert_eq!(lex_side, Some(Value::Record(lex_rec)));
    assert_eq!(env_side, Some(vec![env_rec]));
}

/// Pre-declare (empty declaration / replace) and extend (merge) in env work as expected for the same `(kind, id)`.
#[test]
fn test_env_declare_then_extend_accumulates() {
    // Arrange: pre-declare (Dir,"d") with an empty set, then accumulate via two separate merges.
    let mut scope = Scope::new();
    let r1 = {
        let mut r = Record::default();
        r.fields.insert("0".to_string(), "one".to_string());
        r
    };
    let r2 = {
        let mut r = Record::default();
        r.fields.insert("0".to_string(), "two".to_string());
        r
    };
    scope.declare_env(NodeKind::Dir, "d", vec![]);
    scope.bind_env(NodeKind::Dir, "d", vec![r1.clone()]);
    scope.bind_env(NodeKind::Dir, "d", vec![r2.clone()]);

    // Act
    let acc = scope.lookup_env(NodeKind::Dir, "d").map(<[Record]>::to_vec);

    // Assert: both records are accumulated via merge (not replaced).
    assert_eq!(acc, Some(vec![r1, r2]));
}

/// When a value: binding and a for-iteration variable share the same name in lex, the binding in
/// the top frame shadows the one below (sequential-let / lexical shadowing).
#[test]
fn test_lex_shadowing_across_frames() {
    // Arrange: bind (Value,"v")=outer in the lower frame, then push and bind (Value,"v")=inner in the upper frame.
    let mut scope = Scope::new();
    scope.bind_lex(NodeKind::Value, "v", Value::Scalar("outer".to_string()));
    scope.push();
    scope.bind_lex(NodeKind::Value, "v", Value::Scalar("inner".to_string()));

    // Act: while the upper frame is visible, inner is returned; after pop, outer is restored.
    let shadowed = scope.lookup_lex(NodeKind::Value, "v").cloned();
    scope.pop();
    let restored = scope.lookup_lex(NodeKind::Value, "v").cloned();

    // Assert
    assert_eq!(shadowed, Some(Value::Scalar("inner".to_string())));
    assert_eq!(restored, Some(Value::Scalar("outer".to_string())));
}
