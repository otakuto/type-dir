use crate::expr::{ExprEntry, ExprMatcher};

/// Represents a single content-choice Group.
pub struct UseGroup<'a> {
    pub min: usize,
    pub max: Option<usize>,
    pub alternatives: &'a [ExprEntry],
}

/// Returns a [`UseGroup`] if `entries` form a single Group where all alternatives are Use entries.
///
/// Used for detecting content-choice backtracking (splice alternatives of one_of/any_of).
pub fn as_use_group(entries: &[ExprEntry]) -> Option<UseGroup<'_>> {
    let [entry] = entries else {
        return None;
    };
    let ExprMatcher::Choice { min, max, body } = &entry.matcher else {
        return None;
    };
    if body.is_empty() {
        return None;
    }
    let all_splice = body
        .iter()
        .all(|alt| matches!(alt.matcher, ExprMatcher::Use { .. }));
    all_splice.then_some(UseGroup {
        min: *min,
        max: *max,
        alternatives: body.as_slice(),
    })
}
