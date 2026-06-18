use crate::yaml::YamlPattern;

/// Extracts the pattern string from a `YamlPattern`.
/// For `Spec`, returns the regex string if present; otherwise returns an empty string.
pub fn pattern_str(pattern: &YamlPattern) -> &str {
    match pattern {
        YamlPattern::Exact(s) => s.as_str(),
        YamlPattern::Spec(spec) => spec.regex.as_ref().map(|r| r.0.as_str()).unwrap_or(""),
    }
}

/// Extracts named capture names (`(?<name>` / `(?P<name>`) from a regex pattern.
pub fn named_captures(pattern: &YamlPattern) -> Vec<String> {
    let regex_str = match pattern {
        YamlPattern::Exact(_) => return Vec::new(),
        YamlPattern::Spec(spec) => match &spec.regex {
            Some(r) => r.0.as_str(),
            None => return Vec::new(),
        },
    };
    let mut names = Vec::new();
    for marker in ["(?<", "(?P<"] {
        let mut rest = regex_str;
        while let Some(pos) = rest.find(marker) {
            let after = &rest[pos + marker.len()..];
            if let Some(gt) = after.find('>') {
                let name = &after[..gt];
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    names.push(name.to_string());
                }
                rest = &after[gt + 1..];
            } else {
                break;
            }
        }
    }
    names
}
