mod eval_choice;
mod eval_consume;
mod eval_dir_file;
mod eval_fetch;
mod eval_for;
mod eval_group;
mod eval_inner;
mod eval_match;
mod eval_shared;
mod eval_use;
mod eval_value;

#[cfg(test)]
#[path = "eval_tests/tests.rs"]
mod tests;

#[cfg(test)]
pub(crate) use eval_inner::eval_node;
pub(crate) use eval_inner::eval_node_chained;
pub(crate) use eval_inner::eval_node_traced;
pub(crate) use eval_shared::build_effective_chain;
