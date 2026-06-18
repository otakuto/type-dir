use crate::runtime_impl::env::Scope;

pub(super) fn eval_value(
    var: &crate::yaml::VarName,
    value: &crate::yaml::ValueExpr,
    work_scope: &mut Scope,
) {
    let bound = super::super::expand::eval_value_expr(value, work_scope);
    work_scope.bind_lex(
        crate::runtime_impl::node_id::NodeKind::Value,
        var.0.clone(),
        bound,
    );
}
