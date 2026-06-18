use crate::expr::{ExprEntry, ExprForSource, ExprMatcher, ExprPattern, ExprSubtree, Quant};
use crate::yaml::{EntryId, RegexPattern, VarName};

/// Builds the YAML-equivalent entry list for testing for-binding scope id construction.
///
/// Structure (pseudo-YAML):
/// ```yaml
/// - for: {id: layer, value: ["aaa", "bbb"]}
///   rules:
///     - dir:
///         regex: '^${value.layer}-pkg-(?<stem>.+)$'
///       id: m          # fields are auto-collected from own captures (stem)
///     - dir: '${value.layer}-docs'
///       rules:
///         - for: {id: n, value: ${m}}   # iterate m records (n is a Record binding)
///           rules:
///             - file: '${value.n.stem}.txt'
/// ```
pub fn make_for_layer_entries() -> Vec<ExprEntry> {
    // (a) dir regex '^${layer}-pkg-(?<stem>.+)$' id: m (fields=capture stem auto-collected)
    let pkg_entry = ExprEntry {
        id: Some(EntryId("m".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Regex(RegexPattern(
                r"^${value.layer}-pkg-(?<stem>.+)$".to_string(),
            )),
            subtree: ExprSubtree::Leaf,
        },
    };

    // (b) dir '${layer}-docs' rules: [ for n in ${m} { file '${n.stem}.txt' } ]
    let inner_for = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("n".to_string()),
            source: ExprForSource::Expr("${dir.m}".to_string()),
            body: vec![ExprEntry {
                id: None,
                source_path: None,
                count: Quant::Default,
                matcher: ExprMatcher::File {
                    pattern: ExprPattern::Exact("${value.n.stem}.txt".to_string()),
                    subtree: ExprSubtree::Leaf,
                },
            }],
        },
    };
    let docs_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Exact("${value.layer}-docs".to_string()),
            subtree: ExprSubtree::Inline(vec![inner_for]),
        },
    };

    // for layer in ["aaa", "bbb"] { pkg_entry, docs_entry }
    vec![ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("layer".to_string()),
            source: ExprForSource::Literal(vec!["aaa".to_string(), "bbb".to_string()]),
            body: vec![pkg_entry, docs_entry],
        },
    }]
}
