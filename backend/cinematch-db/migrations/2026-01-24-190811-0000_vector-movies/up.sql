CREATE TABLE movies (
    movie_id BIGINT PRIMARY KEY, -- TMDB movie ID, same for qdrant
    title TEXT NOT NULL,
    runtime INTEGER NOT NULL,
    popularity REAL NOT NULL CHECK (popularity >= 0),
    imdb_id TEXT UNIQUE,
    mediawiki_id TEXT UNIQUE,
    rating TEXT,
    release_date TIMESTAMPTZ NOT NULL,
    original_language TEXT,
    poster_url TEXT,
    overview TEXT,
    tagline TEXT
);

-- Table for directors
CREATE TABLE directors (
    director_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL
);

CREATE TABLE genres (
    genre_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE keywords (
    keyword_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE
);

CREATE TABLE cast_members (
    cast_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    profile_url TEXT,
    UNIQUE(name, profile_url)
);

CREATE TABLE production_countries (
    country_code CHAR(3) PRIMARY KEY, -- ISO 3166-1 alpha-2 (3 for safety)
    name VARCHAR(255) NOT NULL UNIQUE
);

-- Table for tracking user movie taste/preferences
CREATE TABLE user_tastes (
    taste_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    party_id UUID,
    movie_id BIGINT NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
    liked BOOLEAN NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    review INTEGER CHECK (review IS NULL OR (review >= 1 AND review <= 10)),
    CONSTRAINT uq_user_movie UNIQUE (user_id, movie_id, party_id)
);

-- Table for storing video keys per movie
CREATE TABLE trailers (
    trailer_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    video_key TEXT NOT NULL UNIQUE
);

-- Join tables for many-to-many relationships
CREATE TABLE movie_genres (
    movie_id BIGINT NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
    genre_id UUID NOT NULL REFERENCES genres(genre_id) ON DELETE CASCADE,
    PRIMARY KEY (movie_id, genre_id)
);

CREATE TABLE movie_trailers (
    movie_id BIGINT NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
    trailer_id UUID NOT NULL REFERENCES trailers(trailer_id) ON DELETE CASCADE,
    PRIMARY KEY (movie_id, trailer_id)
);

CREATE TABLE movie_keywords (
    movie_id BIGINT NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
    keyword_id UUID NOT NULL REFERENCES keywords(keyword_id) ON DELETE CASCADE,
    PRIMARY KEY (movie_id, keyword_id)
);

CREATE TABLE movie_cast (
    movie_id BIGINT NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
    cast_id UUID NOT NULL REFERENCES cast_members(cast_id) ON DELETE CASCADE,
    PRIMARY KEY (movie_id, cast_id)
);

CREATE TABLE movie_production_countries (
    movie_id BIGINT NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
    country_code CHAR(3) NOT NULL REFERENCES production_countries(country_code) ON DELETE CASCADE,
    PRIMARY KEY (movie_id, country_code)
);

-- Join table for movies and directors (many-to-many)
CREATE TABLE movie_directors (
    movie_id BIGINT NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
    director_id UUID NOT NULL REFERENCES directors(director_id) ON DELETE CASCADE,
    PRIMARY KEY (movie_id, director_id)
);

-- Indexes to improve query performance
CREATE INDEX idx_movie_genres_genre_id_movie_id ON movie_genres(genre_id, movie_id);
CREATE INDEX idx_movies_runtime ON movies(runtime);
CREATE INDEX idx_movies_release_date ON movies(release_date);
CREATE INDEX idx_user_tastes_user_id ON user_tastes(user_id);
CREATE INDEX idx_user_tastes_party_id ON user_tastes(party_id);

-- Composite index for user_id and movie_id in user_tastes
CREATE INDEX idx_user_tastes_user_movie ON user_tastes(user_id, movie_id);

-- Composite index for party_id and movie_id in user_tastes
CREATE INDEX idx_user_tastes_party_movie ON user_tastes(party_id, movie_id);

-- Index for movie_trailers movie_id
CREATE INDEX idx_movie_trailers_movie_id ON movie_trailers(movie_id);

-- Index for movie_directors movie_id
CREATE INDEX idx_movie_directors_movie_id ON movie_directors(movie_id);