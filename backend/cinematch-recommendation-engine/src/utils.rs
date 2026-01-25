use std::collections::HashMap;
use qdrant_client::qdrant::{Filter, Condition};
use cinematch_common::models::FullUserPreferences;
use uuid::Uuid;

/// Generate a Qdrant Filter from FullUserPreferences and a genre name-to-id map
pub fn filter_from_prefs(
    prefs: &FullUserPreferences,
    genre_map: &HashMap<String, Uuid>,
) -> Option<Filter> {
    let mut must_clauses = vec![];
    let mut must_not_clauses = vec![];

    // Included genres (MUST)
    if !prefs.included_genres.is_empty() {
        let include_genre_names: Vec<String> = genre_map
            .iter()
            .filter_map(|(name, &id)| if prefs.included_genres.contains(&id) { Some(name.clone()) } else { None })
            .collect();
        if !include_genre_names.is_empty() {
            must_clauses.push(Condition::matches("genres", include_genre_names));
        }
    }

    // Excluded genres (MUST NOT)
    if !prefs.excluded_genres.is_empty() {
        let exclude_genre_names: Vec<String> = genre_map
            .iter()
            .filter_map(|(name, &id)| if prefs.excluded_genres.contains(&id) { Some(name.clone()) } else { None })
            .collect();
        if !exclude_genre_names.is_empty() {
            must_not_clauses.push(Condition::matches("genres", exclude_genre_names));
        }
    }

    // Release year filter (MUST)
    if let Some(target_year) = prefs.preferred_year {
        let flex = prefs.year_flexibility;
        let min_year = target_year - flex;
        let max_year = target_year + flex;
        must_clauses.push(Condition::range(
            "release_year",
            qdrant_client::qdrant::Range {
                gte: Some(min_year as f64),
                lte: Some(max_year as f64),
                ..Default::default()
            },
        ));
    }

    if must_clauses.is_empty() && must_not_clauses.is_empty() {
        None
    } else {
        // If is_tite, exclude movies with the 'anime' tag
        if prefs.is_tite {
            must_not_clauses.push(Condition::matches("tags", vec!["anime".to_string()]));
        }
        Some(Filter {
            must: must_clauses,
            must_not: must_not_clauses,
            ..Default::default()
        })
    }
}
