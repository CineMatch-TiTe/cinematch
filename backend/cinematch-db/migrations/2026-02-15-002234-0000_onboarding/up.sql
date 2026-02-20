-- Taste cluster centroids from K-Means (K ≈ 15 rows)
CREATE TABLE onboarding_clusters (
    cluster_id    SMALLINT PRIMARY KEY,
    centroid      JSONB NOT NULL,          -- [f64; num_genres] genre-avg-rating vector
    user_count    INTEGER NOT NULL,        -- number of users in this cluster
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Top informative movies per genre for onboarding (≈ 100 per genre)
CREATE TABLE onboarding_movies (
    movie_id      BIGINT NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
    info_gain     REAL NOT NULL,
    rating_dist   JSONB NOT NULL,
    rating_count  INTEGER NOT NULL,
    genre_ids     UUID[] NOT NULL DEFAULT '{}',
    PRIMARY KEY (movie_id)
);

CREATE INDEX idx_onboarding_movies_genre_ids ON onboarding_movies USING GIN (genre_ids);
