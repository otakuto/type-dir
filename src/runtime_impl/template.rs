#[cfg(test)]
#[path = "template_tests/tests.rs"]
mod tests;

use std::collections::HashMap;

use crate::expr::{Hop, parse_ref};

use super::ref_resolve::{ChainValue, head_value, ref_head_parts, resolve_chain};
use super::value::Value;
use crate::runtime_impl::env::Scope;

/// Substitutes `${var}` placeholders in a string with values from the scalar scope.
/// References of the form `${id.var}` (containing `.`) are left as-is without substitution.
pub fn substitute(template: &str, scope: &HashMap<String, String>) -> String {
    substitute_with(template, scope, |value| value.to_string())
}

/// `${var}` substitution for regex context. Literal-escapes variable values with `regex::escape` before embedding.
///
/// When interpolating patterns like `regex: '^myapp-${layer}-...'`, treats the value of `layer` as a literal
/// so that metacharacters (`.`, `*`, etc.) in the value do not silently change the accepted language.
/// Only variable values are escaped; fixed parts of the template (e.g. `^myapp-`) are not escaped.
pub fn substitute_regex_literal(template: &str, scope: &HashMap<String, String>) -> String {
    substitute_with(template, scope, regex::escape)
}

/// Shared implementation for `substitute` variants. Replaces `${var}` (no dot) with scope values,
/// applying `transform` to each replacement value (identity for non-regex contexts, escape for regex).
/// Dotted `${id.var}` references that exist as literal keys in scope are resolved. Other dotted
/// references, unbound variables, and missing `}` are left as-is.
fn substitute_with(
    template: &str,
    scope: &HashMap<String, String>,
    transform: impl Fn(&str) -> String,
) -> String {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("${") {
        let before = &rest[..start];
        result.push_str(before);

        let after_dollar = &rest[start + 2..];
        if let Some(end) = after_dollar.find('}') {
            let var_name = &after_dollar[..end];
            // `${value.<var>}` resolves a value binding from the scalar scope, which holds it under
            // the reserved `value.<var>` key (a scalar value binding flattened by `scope_to_scalar`).
            // Dotted `${id.var}` references are resolved when the full dotted key exists in scope
            // (e.g. `x.regex.n` inserted by record-binding flatten); otherwise left as-is.
            if var_name.contains('.') {
                if let Some(value) = scope.get(var_name) {
                    result.push_str(&transform(value));
                } else {
                    // Dotted reference not found in scope: leave as-is.
                    result.push_str("${");
                    result.push_str(var_name);
                    result.push('}');
                }
            } else if let Some(value) = scope.get(var_name) {
                result.push_str(&transform(value));
            } else {
                // Variables not found in scope are left as-is.
                result.push_str("${");
                result.push_str(var_name);
                result.push('}');
            }
            rest = &after_dollar[end + 1..];
        } else {
            // No closing `}` found; leave as-is.
            result.push_str("${");
            rest = after_dollar;
        }
    }

    result.push_str(rest);
    result
}

/// Resolves a single `${key}` to a scalar value using the new namespace grammar.
///
/// Head dispatch:
/// - `RefHead::Bare(name)` (rejected at compile; should not reach runtime — looked up as an
///   un-namespaced key, which misses):
///   - No hops: look up `name` in lex scope. `Scalar(s)` → `s`; `Record(r)` → `r.whole()`;
///     otherwise literal.
///   - `Hop::Field(f)` (legacy invalid): field lookup on `Record` in lex; otherwise literal.
///   - Qualified hops (Regex/Dir/File): `resolve_chain` on a `Record` in lex.
/// - `RefHead::WithNs { param, tail }`: look up `param` in lex scope; follow `tail` via
///   `resolve_chain` when there are hops. This is the same path as Local but the binding name
///   is `param` and the hops come from `tail`.
/// - `RefHead::RuleNs { rule_id, tail }`: look up `(NodeKind::Rule, rule_id)` in lex; follow `tail`.
/// - `RefHead::ValueNs { var, tail }`: look up the `value:` binding under `(NodeKind::Value, var)`;
///   follow `tail`. A scalar binding resolves to its string.
/// - `RefHead::UseNs { id, tail }`: look up `(NodeKind::Group, id)` in lex; follow `tail` via `resolve_chain`
///   (same path as `RuleNs`). The splice instance wrapper is referenced directly; the desired
///   capture is reached by explicit navigation (e.g. `.dir.<id>.regex.<group>`).
///
/// `Scalars([s])` (exactly one scalar) → `s`; otherwise literal (ambiguous or record set).
fn resolve_key_scalar(key: &str, scope: &Scope) -> String {
    let literal = || format!("${{{}}}", key);
    let r = parse_ref(key);

    // Determine the resolution kind, lookup id, and effective hops from the head namespace.
    // Bare references resolve via the transparent `Scope::get`; qualified references via
    // `(kind, id)` lookups.
    let (ref_kind, lookup_key_cow, effective_hops): (_, _, &[Hop]) =
        ref_head_parts(&r.head, &r.hops);
    let lookup_key: &str = &lookup_key_cow;

    let bound = head_value(scope, ref_kind, lookup_key);

    if effective_hops.is_empty() {
        // Bare reference: no hops.
        return match bound {
            Some(Value::Scalar(s)) => s,
            // Main value: `${x}` evaluates to the full match stored in fields["0"].
            Some(Value::Record(rec)) => rec.whole().to_string(),
            // Set, record list, and unbound variables are kept as literals in patterns.
            _ => literal(),
        };
    }

    if effective_hops.len() == 1
        && let Hop::Field(f) = &effective_hops[0]
    {
        // Legacy single-field hop (invalid in new grammar but kept for graceful degradation).
        if let Some(Value::Record(rec)) = &bound
            && let Some(value) = rec.fields.get(f.as_str())
        {
            return value.clone();
        }
        return literal();
    }

    // One or more qualified hops (Regex/Dir/File): use resolve_chain.
    if let Some(Value::Record(rec)) = &bound {
        return match resolve_chain(rec, effective_hops) {
            Some(ChainValue::Scalars(ss)) if ss.len() == 1 => {
                ss.into_iter().next().unwrap_or_else(literal)
            }
            // Ambiguous (0 or multiple scalars) or record set: not usable in scalar context.
            _ => literal(),
        };
    }
    literal()
}

/// Resolves each `${key}` in a template to a single scalar value and returns a single string.
///
/// Because set iteration is unified into `for` only, no Cartesian expansion is performed.
/// Templates containing no `${...}` are returned as-is.
pub fn resolve_template(template: &str, scope: &Scope) -> String {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("${") {
        let before = &rest[..start];
        result.push_str(before);

        let after_dollar = &rest[start + 2..];
        if let Some(end) = after_dollar.find('}') {
            let key = &after_dollar[..end];
            result.push_str(&resolve_key_scalar(key, scope));
            rest = &after_dollar[end + 1..];
        } else {
            result.push_str("${");
            rest = after_dollar;
        }
    }

    result.push_str(rest);
    result
}
