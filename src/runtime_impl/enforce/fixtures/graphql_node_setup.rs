use indexmap::IndexMap;

use crate::expr::{
    ExprEntry, ExprForSource, ExprMatcher, ExprPattern, ExprRule, ExprSubtree, Quant,
};
use crate::yaml::{EntryId, RegexPattern, RuleName, VarName};

use super::make_file_entry::make_file_entry;

/// Helper that constructs a node equivalent to server_graphql_dir (new format).
///
/// Structure (pseudo-YAML):
/// ```yaml
/// - dir: schema
///   rules:
///     - file:
///         regex: '^(?<stem>[a-z_]+_(mutation|query))\.rs$'
///       id: handler          # fields = capture stem auto-collected
/// - dir: handler
///   rules:
///     - for: {id: h, value: ${handler}}
///       rules:
///         - dir: '${value.h.stem}_handler'
///         - file: '${value.h.stem}_handler.rs'
/// ```
pub fn graphql_node_setup() -> (IndexMap<RuleName, ExprRule>, Vec<ExprEntry>) {
    // schema dir: inner file has id: handler (capture stem is collected automatically)
    let handler_file = ExprEntry {
        id: Some(EntryId("handler".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::File {
            pattern: ExprPattern::Regex(RegexPattern(
                r"^(?<stem>[a-z_]+_(mutation|query))\.rs$".to_string(),
            )),
            subtree: ExprSubtree::Leaf,
        },
    };
    // The schema dir carries `id: schema` so its captured `handler` records nest under its record;
    // the sibling handler dir references them by the path `${dir.schema.file.handler}` (encapsulation:
    // inner ids no longer bubble out of an id-less dir).
    let schema_entry = ExprEntry {
        id: Some(EntryId("schema".to_string())),
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Exact("schema".to_string()),
            subtree: ExprSubtree::Inline(vec![handler_file]),
        },
    };

    // handler dir: for h in ${dir.schema.file.handler} { dir '${h.stem}_handler', file '${h.stem}_handler.rs' }
    let handler_for = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::For {
            var: VarName("h".to_string()),
            source: ExprForSource::Expr("${dir.schema.file.handler}".to_string()),
            body: vec![
                ExprEntry {
                    id: None,
                    source_path: None,
                    count: Quant::Default,
                    matcher: ExprMatcher::Dir {
                        pattern: ExprPattern::Exact("${value.h.stem}_handler".to_string()),
                        subtree: ExprSubtree::Leaf,
                    },
                },
                make_file_entry(
                    ExprPattern::Exact("${value.h.stem}_handler.rs".to_string()),
                    None,
                ),
            ],
        },
    };
    let handler_entry = ExprEntry {
        id: None,
        source_path: None,
        count: Quant::Default,
        matcher: ExprMatcher::Dir {
            pattern: ExprPattern::Exact("handler".to_string()),
            subtree: ExprSubtree::Inline(vec![handler_for]),
        },
    };

    // This setup has no separate rules (the for body is inline).
    let rules = IndexMap::new();
    (rules, vec![schema_entry, handler_entry])
}
