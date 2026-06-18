use regex::Regex;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static REGEX_CACHE: LazyLock<Mutex<HashMap<String, Regex>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn get_or_compile(pattern: &str) -> Option<Regex> {
    let mut cache = REGEX_CACHE.lock().unwrap();
    if let Some(re) = cache.get(pattern) {
        return Some(re.clone());
    }
    let re = Regex::new(pattern).ok()?;
    cache.insert(pattern.to_string(), re.clone());
    Some(re)
}

/// Matches a regex against the given text, returning a map of captures if matched.
///
/// The returned map always contains:
/// - `"0"`: the full match (group 0).
/// - `"1"`, `"2"`, ...: each positional capture group that has a value.
/// - Named groups under their declared name.
pub fn regex_named_captures(pattern: &str, text: &str) -> Option<HashMap<String, String>> {
    let re = get_or_compile(pattern)?;
    let caps = re.captures(text)?;
    let mut map = HashMap::new();
    // Group 0: full match.
    map.insert(
        "0".to_string(),
        caps.get(0).map_or("", |m| m.as_str()).to_string(),
    );
    // Positional groups 1..N.
    for i in 1..re.captures_len() {
        if let Some(m) = caps.get(i) {
            map.insert(i.to_string(), m.as_str().to_string());
        }
    }
    // Named groups.
    for name in re.capture_names().flatten() {
        if let Some(m) = caps.name(name) {
            map.insert(name.to_string(), m.as_str().to_string());
        }
    }
    Some(map)
}
