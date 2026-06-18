use crate::expr::Hop;
use crate::runtime_impl::ref_resolve::{ChainValue, resolve_chain};
use crate::runtime_impl::value::Record;

/// `.regex.0` in a chain returns the full match stored in `fields["0"]`.
///
/// This is the canonical way to access the full matched name after `Record.name` was removed:
/// `${x.regex.0}` yields the same string that the matcher captured for group 0.
#[test]
fn resolve_chain_regex_zero_returns_whole_match() {
    // Arrange
    let mut rec = Record::default();
    rec.fields.insert("0".to_string(), "my-module".to_string());
    rec.fields
        .insert("stem".to_string(), "my-module".to_string());
    let hops = vec![Hop::Regex("0".to_string())];

    // Act
    let result = resolve_chain(&rec, &hops);

    // Assert
    assert_eq!(
        result,
        Some(ChainValue::Scalars(vec!["my-module".to_string()])),
        "expected regex.0 to return the full match"
    );
}
