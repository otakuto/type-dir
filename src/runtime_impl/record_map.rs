/// Map from id to record list. Shared between the collect phase and the enforce phase.
pub type RecordMap = indexmap::IndexMap<String, Vec<crate::runtime_impl::value::Record>>;
