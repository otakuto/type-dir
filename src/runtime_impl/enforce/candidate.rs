use crate::expr::ExprEntry;
use crate::yaml::RuleName;

use crate::runtime_impl::env::Scope;

/// Membership key for a group entry.
///
/// - `group_index`: position of the group entry in the expanded list
/// - `alt_index`: position of the alternative within the group
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupKey {
    pub group_index: usize,
    pub alt_index: usize,
}

/// An expanded candidate entry that child nodes are assigned to.
///
/// `group_key` is `None` for a regular entry and `Some(GroupKey)` for a group alternative.
/// `origin` is `Some(N)` if the entry originates from a splice, or `None` for a direct write.
/// `entry_index` is the position of this entry (or, for a group entry, the group entry itself)
/// in the expanded list; it is used as the key for count aggregation.
pub struct Candidate<'a> {
    pub entry: &'a ExprEntry,
    pub scope: &'a Scope,
    pub group_key: Option<GroupKey>,
    pub origin: &'a Option<RuleName>,
    pub entry_index: usize,
}
