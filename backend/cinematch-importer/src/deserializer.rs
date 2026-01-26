use crate::{CastMember, MovieData};
use py_literal::Value as PyValue;
use serde::{Deserialize, Deserializer};

/// Helper: Extract string from PyValue
fn pyvalue_to_string(val: &PyValue) -> Option<String> {
    match val {
        PyValue::String(s) => Some(s.clone()),
        PyValue::Bytes(b) => Some(String::from_utf8_lossy(b).to_string()),
        _ => None,
    }
}

/// Helper: Extract Vec<String> from PyValue (handles lists and tuples)
fn pyvalue_to_string_vec(val: &PyValue) -> Vec<String> {
    match val {
        PyValue::List(items) | PyValue::Tuple(items) => {
            items.iter().filter_map(pyvalue_to_string).collect()
        }
        _ => Vec::new(),
    }
}

/// Deserialize a string that may be quoted, removing surrounding quotes
fn deserialize_quoted_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_string_field(&s).ok_or_else(|| serde::de::Error::custom("Failed to parse string"))
}

/// Deserialize an optional string that may be quoted, removing surrounding quotes
fn deserialize_quoted_option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(parse_string_field(&s))
}

/// Parse a string field - try JSON string first, then Python literal
fn parse_string_field(s: &str) -> Option<String> {
    let trimmed = s.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return None;
    }

    // Try JSON parsing
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed)
        && let Some(st) = val.as_str()
        && !st.is_empty()
    {
        return Some(st.to_string());
    }

    if let Ok(val) = trimmed.parse::<PyValue>()
        && let Some(st) = pyvalue_to_string(&val)
        && !st.is_empty()
    {
        return Some(st);
    }

    // Return as-is if no outer quotes
    let quoted = (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''));
    if !quoted && !trimmed.is_empty() {
        return Some(trimmed.to_string());
    }

    None
}

/// Deserialize string array from JSON or Python format (JSON first, Python literal fallback)
fn deserialize_string_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(parse_string_array(&s))
}

/// Parse string array - try JSON first, then Python literal
fn parse_string_array(s: &str) -> Vec<String> {
    let trimmed = s.trim();

    if trimmed.is_empty() || trimmed == "[]" {
        return Vec::new();
    }

    // Try JSON first
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(trimmed) {
        return arr;
    }

    // Try Python literal parsing
    if let Ok(val) = trimmed.parse::<PyValue>() {
        return pyvalue_to_string_vec(&val);
    }

    // If comma-separated, split and trim
    if trimmed.contains(',') {
        return trimmed
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && *s != "null")
            .map(|s| s.to_string())
            .collect();
    }

    // Otherwise, treat as single string value
    if !trimmed.is_empty() && trimmed != "null" {
        return vec![trimmed.to_string()];
    }

    Vec::new()
}

/// Deserialize cast array from JSON or Python format (JSON first, Python literal fallback)
fn deserialize_cast<'de, D>(deserializer: D) -> Result<Vec<CastMember>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(parse_cast_array(&s))
}

/// Parse cast array - try JSON first, then Python literal
fn parse_cast_array(s: &str) -> Vec<CastMember> {
    let trimmed = s.trim();

    if trimmed.is_empty() || trimmed == "[]" {
        return Vec::new();
    }

    // Try JSON first
    if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(trimmed) {
        return extract_cast_members_from_json(&items);
    }

    // Try Python literal parsing
    if let Ok(val) = trimmed.parse::<PyValue>() {
        return extract_cast_members_from_pyvalue(&val);
    }

    Vec::new()
}

/// Extract cast members from JSON array
fn extract_cast_members_from_json(items: &[serde_json::Value]) -> Vec<CastMember> {
    items
        .iter()
        .filter_map(|item| {
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())?;
            let profile_url = item
                .get("profile_url")
                .and_then(|v| v.as_str())
                .and_then(|s| {
                    if s.is_empty() || s == "null" {
                        None
                    } else {
                        Some(s.to_string())
                    }
                });
            Some(CastMember::with_profile(name, profile_url))
        })
        .collect()
}

/// Extract cast members from PyValue (dict in list)
fn extract_cast_members_from_pyvalue(val: &PyValue) -> Vec<CastMember> {
    match val {
        PyValue::List(items) | PyValue::Tuple(items) => items
            .iter()
            .filter_map(|item| match item {
                PyValue::Dict(pairs) => {
                    let mut name: Option<String> = None;
                    let mut profile_url: Option<String> = None;

                    for (k, v) in pairs {
                        if let Some(key) = pyvalue_to_string(k) {
                            match key.as_str() {
                                "name" => name = pyvalue_to_string(v),
                                "profile_url" => {
                                    if let Some(url) = pyvalue_to_string(v)
                                        && !url.is_empty()
                                        && url != "null"
                                    {
                                        profile_url = Some(url);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    name.map(|n| CastMember::with_profile(n, profile_url))
                }
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Deserialize external_ids object to Vec<(String, String)> (JSON first, Python literal fallback)
fn deserialize_external_ids<'de, D>(deserializer: D) -> Result<Vec<(String, String)>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(parse_external_ids(&s))
}

/// Parse external IDs dict - try JSON first, then Python literal
fn parse_external_ids(s: &str) -> Vec<(String, String)> {
    let trimmed = s.trim();

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
                v.as_str().and_then(|val| {
                    if val.is_empty() || val == "null" {
                        None
                    } else {
                        Some((k.clone(), val.to_string()))
                    }
                })
            })
            .collect();
    }

    // Try Python literal parsing
    if let Ok(val) = trimmed.parse::<PyValue>() {
        return extract_external_ids_from_pyvalue(&val);
    }
    Vec::new()
}

/// Extract external IDs from PyValue dict
fn extract_external_ids_from_pyvalue(val: &PyValue) -> Vec<(String, String)> {
    match val {
        PyValue::Dict(pairs) => pairs
            .iter()
            .filter_map(|(k, v)| {
                if let Some(key) = pyvalue_to_string(k)
                    && let Some(value) = pyvalue_to_string(v)
                    && !value.is_empty()
                    && value != "null"
                {
                    Some((key, value))
                } else {
                    None
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Deserialize release_date - expects YYYY-MM-DD format
fn deserialize_release_date<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(0);
    }

    let date =
        chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)?;
    let datetime = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| serde::de::Error::custom("Invalid date"))?;

    Ok(datetime.and_utc().timestamp())
}

/// Builder for MovieData - deserializes directly from CSV
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct MovieDataBuilder {
    pub movie_id: i64,
    #[serde(default, deserialize_with = "deserialize_quoted_string")]
    pub title: String,
    #[serde(default, deserialize_with = "deserialize_quoted_option_string")]
    pub overview: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_array")]
    pub genres: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_array")]
    pub keywords: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_array")]
    pub director: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_release_date")]
    pub release_date: i64,
    #[serde(default)]
    pub runtime: i64,
    #[serde(default)]
    pub popularity: f32,
    #[serde(default, deserialize_with = "deserialize_quoted_option_string")]
    pub poster_url: Option<String>,
    #[serde(default)]
    pub budget: i64,
    #[serde(default)]
    pub revenue: i64,
    #[serde(default, deserialize_with = "deserialize_quoted_option_string")]
    pub tagline: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_array")]
    pub poster_urls: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_array")]
    pub video_keys: Vec<String>,
    #[serde(
        default,
        rename = "review_texts",
        deserialize_with = "deserialize_string_array"
    )]
    pub reviews: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_external_ids")]
    pub external_ids: Vec<(String, String)>,
    #[serde(
        default,
        rename = "certification",
        deserialize_with = "deserialize_quoted_option_string"
    )]
    pub rating: Option<String>,
    #[serde(default, deserialize_with = "deserialize_cast")]
    pub cast: Vec<CastMember>,
    #[serde(default, deserialize_with = "deserialize_quoted_option_string")]
    pub imdb_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_quoted_option_string")]
    pub mediawiki_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_quoted_option_string")]
    pub original_language: Option<String>,
    #[serde(default)]
    pub production_countries: Vec<String>,
}

#[allow(dead_code)]
impl MovieDataBuilder {
    /// Create a new builder with required fields
    pub fn new(movie_id: i64, title: String, runtime: i64) -> Self {
        Self {
            movie_id,
            title,
            runtime,
            overview: None,
            genres: Vec::new(),
            keywords: Vec::new(),
            director: Vec::new(),
            release_date: 0,
            popularity: 0.0,
            poster_url: None,
            budget: 0,
            revenue: 0,
            tagline: None,
            poster_urls: Vec::new(),
            video_keys: Vec::new(),
            reviews: Vec::new(),
            external_ids: Vec::new(),
            rating: None,
            cast: Vec::new(),
            imdb_id: None,
            mediawiki_id: None,
            original_language: None,
            production_countries: Vec::new(),
        }
    }

    /// Extract IMDb and MediaWiki IDs from external_ids array
    fn extract_external_ids(&mut self) {
        for (key, value) in &self.external_ids {
            match key.as_str() {
                "imdb_id" if self.imdb_id.is_none() && !value.is_empty() && value != "null" => {
                    self.imdb_id = Some(value.clone());
                }
                "wikidata_id"
                    if self.mediawiki_id.is_none() && !value.is_empty() && value != "null" =>
                {
                    self.mediawiki_id = Some(value.clone());
                }
                _ => {}
            }
        }
    }

    /// Build the final MovieData struct
    pub fn build(mut self) -> MovieData {
        // Extract IDs from external_ids
        self.extract_external_ids();

        // Clean up empty/null strings in optional fields
        let clean_string = |s: Option<String>| {
            s.and_then(|val| {
                if val.trim().is_empty() || val == "null" {
                    None
                } else {
                    Some(val)
                }
            })
        };

        MovieData {
            movie_id: self.movie_id,
            title: self.title,
            runtime: self.runtime,
            popularity: self.popularity,
            imdb_id: self.imdb_id,
            mediawiki_id: self.mediawiki_id,
            rating: clean_string(self.rating),
            release_date: self.release_date,
            original_language: clean_string(self.original_language),
            poster_url: clean_string(self.poster_url),
            overview: clean_string(self.overview),
            tagline: clean_string(self.tagline),
            director: self
                .director
                .into_iter()
                .filter(|d| !d.trim().is_empty() && d != "null")
                .collect(),
            genres: self.genres,
            keywords: self.keywords,
            cast: self.cast,
            production_countries: self.production_countries,
            reviews: self.reviews,
            video_keys: self.video_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cast_member_with_profile() {
        let member = CastMember::with_profile(
            "Elijah Wood".to_string(),
            Some("https://image.tmdb.org/t/p/w500/5uYw76OQnQYcVUU01zuM2QCqNzP.jpg".to_string()),
        );

        assert_eq!(member.name, "Elijah Wood");
        assert!(member.profile_url.is_some());
        assert!(!member.get_unique_id().is_empty());
    }

    #[test]
    fn test_extract_external_ids() {
        let mut builder = MovieDataBuilder::new(120, "Test Movie".to_string(), 120);
        builder.external_ids = vec![
            ("imdb_id".to_string(), "tt0120737".to_string()),
            ("wikidata_id".to_string(), "Q127367".to_string()),
        ];

        builder.extract_external_ids();

        assert_eq!(builder.imdb_id, Some("tt0120737".to_string()));
        assert_eq!(builder.mediawiki_id, Some("Q127367".to_string()));
    }

    #[test]
    fn test_build_movie_data() {
        let builder = MovieDataBuilder {
            movie_id: 120,
            title: "The Lord of the Rings: The Fellowship of the Ring".to_string(),
            runtime: 179,
            overview: Some("Young hobbit Frodo Baggins...".to_string()),
            genres: vec![
                "Adventure".to_string(),
                "Fantasy".to_string(),
                "Action".to_string(),
            ],
            keywords: vec!["fantasy".to_string(), "magic".to_string()],
            director: vec!["Peter Jackson".to_string()],
            release_date: 1008633600,
            popularity: 20.9517,
            poster_url: Some(
                "https://image.tmdb.org/t/p/w500/6oom5QYQ2yQTMJIbnvbkBL9cHo6.jpg".to_string(),
            ),
            budget: 93000000,
            revenue: 871368364,
            tagline: Some("One ring to rule them all.".to_string()),
            poster_urls: vec![],
            video_keys: vec![],
            reviews: vec!["A masterpiece.".to_string()],
            external_ids: vec![],
            rating: Some("PG-13".to_string()),
            cast: vec![CastMember::with_profile(
                "Elijah Wood".to_string(),
                Some("https://...".to_string()),
            )],
            imdb_id: Some("tt0120737".to_string()),
            mediawiki_id: Some("Q127367".to_string()),
            original_language: Some("en".to_string()),
            production_countries: vec!["NZ".to_string()],
        };

        let movie = builder.build();

        assert_eq!(movie.movie_id, 120);
        assert_eq!(
            movie.title,
            "The Lord of the Rings: The Fellowship of the Ring"
        );
        assert_eq!(movie.runtime, 179);
        assert_eq!(movie.genres.len(), 3);
        assert_eq!(movie.cast.len(), 1);
    }

    #[test]
    fn test_clean_null_strings() {
        let builder = MovieDataBuilder {
            movie_id: 1,
            title: "Test".to_string(),
            runtime: 100,
            overview: None,
            genres: vec![],
            keywords: vec![],
            director: Vec::new(),
            release_date: 0,
            popularity: 0.0,
            poster_url: None,
            budget: 0,
            revenue: 0,
            tagline: None,
            poster_urls: vec![],
            video_keys: vec![],
            reviews: vec![],
            external_ids: vec![],
            rating: Some("null".to_string()),
            cast: vec![],
            imdb_id: None,
            mediawiki_id: None,
            original_language: Some("".to_string()),
            production_countries: vec![],
        };

        let movie = builder.build();

        // "null" strings should be converted to None
        assert_eq!(movie.rating, None);
        // Empty strings should be converted to None
        assert_eq!(movie.original_language, None);
    }
}
