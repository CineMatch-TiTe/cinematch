//! Simplified CSV deserializer using the custom `pylit` parser module.
//!
//! Replaces the old 558-line deserializer that depended on `py_literal`.
//! All Python-literal parsing is now delegated to `super::pylit`.

use super::pylit;
use cinematch_db::conn::qdrant::models::{CastMember, MovieData};
use serde::{Deserialize, Deserializer};

// ── String field helpers ──────────────────────────────────────────────

/// Deserialize a string, stripping surrounding quotes.
fn deserialize_quoted_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_string_field(&s).ok_or_else(|| serde::de::Error::custom("Failed to parse string"))
}

/// Deserialize an optional string, stripping surrounding quotes.
fn deserialize_quoted_option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(parse_string_field(&s))
}

/// Clean and extract a string field value.
fn parse_string_field(s: &str) -> Option<String> {
    let trimmed = s.trim();

    if trimmed.is_empty() || trimmed == "null" || trimmed == "None" {
        return None;
    }

    // Strip outer quotes if present
    let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    if unquoted.is_empty() || unquoted == "null" || unquoted == "None" {
        None
    } else {
        Some(unquoted.to_string())
    }
}

// ── Array/Dict field helpers ──────────────────────────────────────────

/// Deserialize a string array (Python list or JSON array).
fn deserialize_string_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(pylit::parse_string_list(&s))
}

/// Deserialize cast list from Python/JSON format.
fn deserialize_cast<'de, D>(deserializer: D) -> Result<Vec<CastMember>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(parse_cast_array(&s))
}

/// Parse cast array using pylit dict-list parser.
fn parse_cast_array(s: &str) -> Vec<CastMember> {
    pylit::parse_dict_list(s)
        .into_iter()
        .filter_map(|dict| {
            let name = dict.get("name")?.clone();
            if name.is_empty() || name == "null" {
                return None;
            }
            let profile_url = dict
                .get("profile_url")
                .cloned()
                .filter(|u| !u.is_empty() && u != "null" && u != "None");
            Some(CastMember::with_profile(name, profile_url))
        })
        .collect()
}

/// Deserialize external_ids dict from Python/JSON format.
fn deserialize_external_ids<'de, D>(deserializer: D) -> Result<Vec<(String, String)>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(pylit::parse_dict(&s))
}

/// Deserialize release_date from YYYY-MM-DD format to Unix timestamp.
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

// ── MovieDataBuilder ──────────────────────────────────────────────────

/// Builder for MovieData — deserializes directly from CSV rows.
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
    /// Create a new builder with required fields.
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

    /// Extract IMDb and MediaWiki IDs from external_ids array.
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

    /// Build the final MovieData struct.
    pub fn build(mut self) -> MovieData {
        self.extract_external_ids();

        let clean_string = |s: Option<String>| {
            s.and_then(|val| {
                if val.trim().is_empty() || val == "null" || val == "None" {
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

    #[test]
    fn test_parse_cast_from_python_literal() {
        let input = "[{'name': 'Tom Hanks', 'profile_url': 'https://example.com/tom.jpg'}, {'name': 'Tim Allen', 'profile_url': None}]";
        let cast = parse_cast_array(input);
        assert_eq!(cast.len(), 2);
        assert_eq!(cast[0].name, "Tom Hanks");
        assert!(cast[0].profile_url.is_some());
        assert_eq!(cast[1].name, "Tim Allen");
        assert!(cast[1].profile_url.is_none());
    }
}
