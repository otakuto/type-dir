use indexmap::IndexMap;

use crate::expr::ExprRule;
use crate::yaml::{VarName, WithShape, YamlRule};

use super::entry::to_expr_entry;

pub fn to_expr_rule(rule: &YamlRule) -> ExprRule {
    // Convert with params to WithShape (invalid shapes are rejected upfront by check_with_shapes;
    // any that slip through fall back to required scalar).
    let with_params: IndexMap<VarName, WithShape> = rule
        .with_params
        .iter()
        .map(|(name, shape)| {
            let shape = shape.to_shape().unwrap_or(WithShape::Scalar);
            (name.clone(), shape)
        })
        .collect();
    ExprRule {
        with_params,
        note: rule.note.clone(),
        rules: rule
            .body
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let path = format!("rules.{}.rules[{i}]", rule.rule.0);
                to_expr_entry(e, Some(&path))
            })
            .collect(),
    }
}
