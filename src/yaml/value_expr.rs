/// A `value:` binding expression. The bound type is restricted to a string scalar or a list of
/// strings. Both the scalar and each list element are template strings that are interpolated
/// (`resolve_template`) against the current scope at enforcement/collection time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueExpr {
    /// `value: 'abc'` — a single (template) string. Binds a scalar in scope.
    Scalar(String),
    /// `value: ['a', 'b']` — a list of (template) strings. Binds a set in scope, usable as a
    /// `for ... in ${value.<var>}` source.
    List(Vec<String>),
}
