use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum VectorCollection {
    #[serde(rename = "movie_plot")]
    MoviePlot,
    #[serde(rename = "movie_cast_crew")]
    MovieCastCrew,
    #[serde(rename = "movie_reviews")]
    MovieReviews,
    #[serde(rename = "movie_combined")]
    #[default]
    MovieCombined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieData {
    // ===== REQUIRED FIELDS (always present) =====
    pub movie_id: i64, // TMDB movie ID - primary identifier, same for qdrant
    pub title: String, // Movie title
    pub runtime: i64,  // Duration in minutes

    // ===== GUARANTEED METADATA =====
    pub popularity: f32, // Popularity score (TMDB)

    // ===== OPTIONAL IDENTIFIERS =====
    pub imdb_id: Option<String>,      // IMDb ID if available
    pub mediawiki_id: Option<String>, // MediaWiki ID if available

    // ===== OPTIONAL METADATA =====
    pub rating: Option<String>, // MPAA rating (e.g., PG, R, etc.)
    pub release_date: i64,      // Release date as unix timestamp (0 if unknown)
    pub original_language: Option<String>, // Language code (e.g., "en", "fr")
    pub poster_url: Option<String>, // Primary poster image URL

    // ===== OPTIONAL CONTENT (for semantic search) =====
    pub overview: Option<String>, // Short plot summary
    pub tagline: Option<String>,  // Movie tagline/motto
    pub director: Vec<String>,    // Director name(s)

    // ===== VECTOR FIELDS (may be empty) =====
    pub genres: Vec<String>,               // List of genre labels
    pub keywords: Vec<String>,             // List of thematic keywords
    pub cast: Vec<CastMember>,             // Cast members with name and profile URL
    pub production_countries: Vec<String>, // List of production country codes
    pub reviews: Vec<String>,              // User review texts
    pub video_keys: Vec<String>,           // List of video keys (YouTube, etc)
}

/// Cast member with name and optional profile URL for unique identification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CastMember {
    pub name: String,
    pub profile_url: Option<String>,
}

impl CastMember {
    pub fn new(name: String) -> Self {
        Self {
            name,
            profile_url: None,
        }
    }

    pub fn with_profile(name: String, profile_url: Option<String>) -> Self {
        Self { name, profile_url }
    }

    pub fn get_unique_id(&self) -> String {
        match &self.profile_url {
            Some(url) => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                url.hash(&mut hasher);
                let hash = hasher.finish();
                format!("{} ({:x})", self.name, hash)
            }
            None => self.name.clone(),
        }
    }
}

impl MovieData {
    /// Extract text for plot/theme semantic search
    /// Combines title, genres, overview, keywords, and tagline (light embedding focus)
    pub fn get_plot_text(&self) -> String {
        let mut parts = Vec::new();

        parts.push(self.title.trim().to_string());
        parts.push(format!("Runtime: {} minutes", self.runtime));

        if !self.genres.is_empty() {
            parts.push(format!("Genres: {}", self.genres.join(", ")));
        }

        if let Some(tagline) = &self.tagline && !tagline.trim().is_empty() {
            parts.push(tagline.trim().to_string());
        }

        if let Some(overview) = &self.overview && !overview.trim().is_empty() {
            parts.push(overview.trim().to_string());
        }

        if !self.keywords.is_empty() {
            parts.push(format!("Themes: {}", self.keywords.join(", ")));
        }

        parts.join(". ")
    }

    /// Extract cast and director information for actor/crew search
    /// Focuses on title, genres, director, and cast for people-centric search
    /// Includes profile URL identifiers for actors with duplicate names
    pub fn get_cast_crew_text(&self) -> String {
        let mut parts = Vec::new();

        parts.push(self.title.trim().to_string());

        if !self.genres.is_empty() {
            parts.push(format!("Genres: {}", self.genres.join(", ")));
        }

        if !self.director.is_empty() {
            let director_str = self.director.join(", ");
            if !director_str.trim().is_empty() {
                parts.push(format!("Directed by: {}", director_str.trim()));
            }
        }

        if !self.cast.is_empty() {
            // Build cast text with smart differentiation
            let cast_text = self.build_cast_text_with_differentiation();
            let cast_preview = if cast_text.len() > 300 {
                cast_text.chars().take(300).collect::<String>()
            } else {
                cast_text
            };
            parts.push(format!("Cast: {}", cast_preview.trim()));
        }

        parts.join(". ")
    }

    /// Build cast text with differentiation for same-named actors
    /// Uses profile URL info to distinguish actors with identical names
    fn build_cast_text_with_differentiation(&self) -> String {
        let mut cast_names = Vec::new();

        // Check for duplicate names
        let mut name_counts = std::collections::HashMap::new();
        for member in &self.cast {
            *name_counts.entry(member.name.clone()).or_insert(0) += 1;
        }

        // Build cast list with differentiation where needed
        for member in &self.cast {
            let entry = if name_counts[&member.name] > 1 && member.profile_url.is_some() {
                // Duplicate name with profile - include a hash identifier
                member.get_unique_id()
            } else {
                // Unique name or no profile - just use the name
                member.name.clone()
            };
            cast_names.push(entry);
        }

        cast_names.join(", ")
    }

    /// Extract review sentiment and opinions
    /// Focuses on title, genres, and user reviews for sentiment-based search
    pub fn get_reviews_text(&self) -> String {
        let mut parts = vec![self.title.trim().to_string()];

        if !self.genres.is_empty() {
            parts.push(format!("Genres: {}", self.genres.join(", ")));
        }

        if self.reviews.is_empty() {
            parts.push("No reviews available".to_string());
            return parts.join(". ");
        }

        parts.push("Reviews:".to_string());

        // Combine reviews with summaries, limiting total length
        let reviews_text = self
            .reviews
            .iter()
            .map(|r| r.trim())
            .filter(|r| !r.is_empty())
            .take(5) // Limit to first 5 reviews for embedding
            .map(|r| {
                if r.len() > 200 {
                    format!("{}...", r.chars().take(200).collect::<String>())
                } else {
                    r.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" | ");

        if !reviews_text.is_empty() {
            parts.push(reviews_text);
        }

        parts.join(". ")
    }

    /// Comprehensive text combining all semantic aspects
    /// Includes title, genres, and everything for full-text embedding (largest/richest vector)
    pub fn get_combined_text(&self) -> String {
        let mut parts = Vec::new();

        parts.push(self.title.trim().to_string());
        parts.push(format!("Runtime: {} minutes", self.runtime));

        if !self.genres.is_empty() {
            parts.push(format!("Genres: {}", self.genres.join(", ")));
        }

        if let Some(tagline) = &self.tagline && !tagline.trim().is_empty() {
            parts.push(tagline.trim().to_string());
        }

        if !self.director.is_empty() {
            let director_str = self.director.join(", ");
            if !director_str.trim().is_empty() {
                parts.push(format!("Director: {}", director_str.trim()));
            }
        }

        if !self.cast.is_empty() {
            let cast_text = self.build_cast_text_with_differentiation();
            let cast_preview = if cast_text.len() > 200 {
                cast_text.chars().take(200).collect::<String>()
            } else {
                cast_text
            };
            parts.push(format!("Cast: {}", cast_preview.trim()));
        }

        if let Some(overview) = &self.overview && !overview.trim().is_empty() {
            parts.push(overview.trim().to_string());
        }

        if !self.keywords.is_empty() {
            parts.push(format!("Keywords: {}", self.keywords.join(", ")));
        }

        // Include reviews in comprehensive embedding
        if !self.reviews.is_empty() {
            let reviews_summary = self
                .reviews
                .iter()
                .take(3)
                .map(|r| r.trim())
                .filter(|r| !r.is_empty())
                .map(|r| {
                    if r.len() > 150 {
                        format!("{}...", r.chars().take(150).collect::<String>())
                    } else {
                        r.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            if !reviews_summary.is_empty() {
                parts.push(format!("Reviews: {}", reviews_summary));
            }
        }

        parts.join(". ")
    }
}
