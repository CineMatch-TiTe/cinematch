//! Lightweight parser for Python literal formats found in CSV data.
//!
//! Handles three patterns:
//! - `['Action', 'Comedy']` → Vec<String>
//! - `{'key': 'value', ...}` → Vec<(String, String)>
//! - `[{'name': 'Tom', 'url': '...'}, ...]` → Vec<HashMap<String, String>>
//!
//! Replaces the `py_literal` crate dependency.

use std::collections::HashMap;

/// Parse a Python-style string list: `['foo', 'bar']` → Vec<String>
/// Also handles JSON-style `["foo", "bar"]` as a fallback.
pub fn parse_string_list(input: &str) -> Vec<String> {
    let trimmed = input.trim();

    if trimmed.is_empty() || trimmed == "[]" {
        return Vec::new();
    }

    // Try JSON first (double-quoted)
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(trimmed) {
        return arr;
    }

    // Parse Python-style single-quoted list
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return extract_quoted_strings(&trimmed[1..trimmed.len() - 1]);
    }

    // Comma-separated plain values
    if trimmed.contains(',') {
        return trimmed
            .split(',')
            .map(|s| s.trim().trim_matches('\'').trim_matches('"').to_string())
            .filter(|s| !s.is_empty() && s != "null" && s != "None")
            .collect();
    }

    // Single value
    let val = trimmed.trim_matches('\'').trim_matches('"');
    if !val.is_empty() && val != "null" && val != "None" {
        vec![val.to_string()]
    } else {
        Vec::new()
    }
}

/// Parse a Python-style dict: `{'key': 'value', ...}` → Vec<(String, String)>
/// Also handles JSON-style `{"key": "value"}`.
pub fn parse_dict(input: &str) -> Vec<(String, String)> {
    let trimmed = input.trim();

    if trimmed.is_empty() || trimmed == "{}" {
        return Vec::new();
    }

    // Try JSON first
    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(trimmed)
        && let Some(map) = obj.as_object()
    {
        return map
            .iter()
            .filter_map(|(k, v)| {
                let val = match v {
                    serde_json::Value::String(s) if !s.is_empty() && s != "null" => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => return None,
                };
                Some((k.clone(), val))
            })
            .collect();
    }

    // Parse Python-style dict with single quotes
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = &trimmed[1..trimmed.len() - 1];
        return parse_py_key_value_pairs(inner);
    }

    Vec::new()
}

/// Parse a Python-style list of dicts: `[{'name': 'Tom', ...}, ...]` → Vec<HashMap<String, String>>
/// Also handles JSON-style `[{"name": "Tom"}]`.
pub fn parse_dict_list(input: &str) -> Vec<HashMap<String, String>> {
    let trimmed = input.trim();

    if trimmed.is_empty() || trimmed == "[]" {
        return Vec::new();
    }

    // Try JSON first
    if let Ok(arr) = serde_json::from_str::<Vec<HashMap<String, String>>>(trimmed) {
        return arr;
    }

    // Parse Python-style list of dicts
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let inner = &trimmed[1..trimmed.len() - 1];
        return split_dicts(inner)
            .into_iter()
            .map(|dict_str| {
                let pairs = parse_py_key_value_pairs(&dict_str);
                pairs.into_iter().collect()
            })
            .filter(|m: &HashMap<String, String>| !m.is_empty())
            .collect();
    }

    Vec::new()
}

/// Extract quoted strings from inside a list (already stripped of outer brackets).
/// Handles both `'single'` and `"double"` quotes, with escaped quotes inside.
fn extract_quoted_strings(input: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch == '\'' || ch == '"' {
            let quote = ch;
            chars.next(); // consume opening quote
            let mut value = String::new();
            let mut escaped = false;

            for c in chars.by_ref() {
                if escaped {
                    value.push(c);
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == quote {
                    break;
                } else {
                    value.push(c);
                }
            }

            if !value.is_empty() && value != "null" && value != "None" {
                results.push(value);
            }
        } else {
            chars.next();
        }
    }

    results
}

/// Parse Python key-value pairs from inside a dict (already stripped of outer braces).
/// e.g. `'imdb_id': 'tt0114709', 'wikidata_id': 'Q171048'`
fn parse_py_key_value_pairs(input: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut chars = input.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace and commas
        skip_ws_and_commas(&mut chars);

        // Parse key
        let key = match parse_py_quoted_string(&mut chars) {
            Some(k) => k,
            None => break,
        };

        // Skip colon and whitespace
        skip_until_value(&mut chars);

        // Parse value — could be quoted string, None, number, or nested structure
        let value = parse_py_value(&mut chars);

        if let Some(val) = value
            && !val.is_empty()
            && val != "null"
            && val != "None"
        {
            results.push((key, val));
        }
    }

    results
}

/// Parse a single Python quoted string from the char iterator.
fn parse_py_quoted_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
    skip_ws_and_commas(chars);

    let &quote = chars.peek()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    chars.next(); // consume opening quote

    let mut value = String::new();
    let mut escaped = false;

    for c in chars.by_ref() {
        if escaped {
            value.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == quote {
            return Some(value);
        } else {
            value.push(c);
        }
    }

    Some(value)
}

/// Parse a Python value — string, None, True, False, number, or skip nested structure.
fn parse_py_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
    skip_ws_and_commas(chars);

    let &ch = chars.peek()?;

    if ch == '\'' || ch == '"' {
        return parse_py_quoted_string(chars);
    }

    // Unquoted value (None, True, False, number)
    let mut value = String::new();
    while let Some(&c) = chars.peek() {
        if c == ',' || c == '}' || c == ']' {
            break;
        }
        value.push(c);
        chars.next();
    }

    let val = value.trim().to_string();
    if val == "None" || val == "null" || val.is_empty() {
        None
    } else {
        Some(val)
    }
}

fn skip_ws_and_commas(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if c == ' ' || c == ',' || c == '\n' || c == '\r' || c == '\t' {
            chars.next();
        } else {
            break;
        }
    }
}

fn skip_until_value(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if c == ':' || c == ' ' || c == '\t' {
            chars.next();
        } else {
            break;
        }
    }
}

/// Split a string containing multiple `{...}` dicts into individual dict bodies.
fn split_dicts(input: &str) -> Vec<String> {
    let mut dicts = Vec::new();
    let mut depth = 0;
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '{' => {
                depth += 1;
                if depth == 1 {
                    current.clear();
                    continue;
                }
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    dicts.push(current.clone());
                    continue;
                }
            }
            _ => {}
        }
        if depth > 0 {
            current.push(ch);
        }
    }

    dicts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_py_string_list() {
        assert_eq!(
            parse_string_list("['Action', 'Comedy', 'Drama']"),
            vec!["Action", "Comedy", "Drama"]
        );
    }

    #[test]
    fn test_parse_json_string_list() {
        assert_eq!(
            parse_string_list(r#"["Action", "Comedy"]"#),
            vec!["Action", "Comedy"]
        );
    }

    #[test]
    fn test_parse_empty_list() {
        assert!(parse_string_list("[]").is_empty());
        assert!(parse_string_list("").is_empty());
    }

    #[test]
    fn test_parse_py_dict() {
        let result = parse_dict("{'imdb_id': 'tt0114709', 'wikidata_id': 'Q171048'}");
        assert_eq!(result.len(), 2);
        assert!(result.contains(&("imdb_id".to_string(), "tt0114709".to_string())));
        assert!(result.contains(&("wikidata_id".to_string(), "Q171048".to_string())));
    }

    #[test]
    fn test_parse_py_dict_with_none() {
        let result = parse_dict("{'id': 862, 'imdb_id': 'tt0114709', 'facebook_id': None}");
        // None values should be filtered out
        assert!(!result.iter().any(|(k, _)| k == "facebook_id"));
        assert!(
            result
                .iter()
                .any(|(k, v)| k == "imdb_id" && v == "tt0114709")
        );
        // Numeric value should be kept as string
        assert!(result.iter().any(|(k, v)| k == "id" && v == "862"));
    }

    #[test]
    fn test_parse_dict_list() {
        let input = "[{'name': 'Tom Hanks', 'profile_url': 'https://example.com/tom.jpg'}, {'name': 'Tim Allen', 'profile_url': 'https://example.com/tim.jpg'}]";
        let result = parse_dict_list(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].get("name").unwrap(), "Tom Hanks");
        assert_eq!(result[1].get("name").unwrap(), "Tim Allen");
    }

    #[test]
    fn test_parse_json_dict() {
        let result = parse_dict(r#"{"imdb_id": "tt0114709", "wikidata_id": "Q171048"}"#);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_single_value() {
        assert_eq!(parse_string_list("John Lasseter"), vec!["John Lasseter"]);
    }
}
