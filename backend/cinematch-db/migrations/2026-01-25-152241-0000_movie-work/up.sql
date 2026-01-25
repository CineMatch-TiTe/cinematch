-- Your SQL goes here
CREATE TABLE user_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    target_release_year INTEGER, -- Preferred release year
    release_year_flex INTEGER NOT NULL DEFAULT 0, -- +/- years from target_release_year
    is_tite BOOLEAN NOT NULL DEFAULT FALSE, -- Whether the user prefers TITE movies
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);


CREATE TABLE prefs_include_genre (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    genre_id UUID NOT NULL REFERENCES genres(genre_id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, genre_id)
);

CREATE TABLE prefs_exclude_genre (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    genre_id UUID NOT NULL REFERENCES genres(genre_id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, genre_id)
);

-- Create default user_preferences for all users who do not have preferences
INSERT INTO user_preferences (user_id, target_release_year, release_year_flex, created_at, updated_at)
SELECT u.id, NULL, 0, now(), now()
FROM users u
LEFT JOIN user_preferences p ON u.id = p.user_id
WHERE p.user_id IS NULL;

-- add the selected movie to party as a field
ALTER TABLE parties
ADD COLUMN selected_movie_id BIGINT REFERENCES movies(movie_id) ON DELETE CASCADE;

-- add the selected movie to party as a field
ALTER TABLE movies
ADD COLUMN release_year INTEGER;

-- Index for name to movie
CREATE INDEX idx_movies_title ON movies(title);
CREATE INDEX idx_movies_release_year ON movies(release_year);

-- Migrate script to populate release_year from release_date
UPDATE movies
SET release_year = EXTRACT(YEAR FROM release_date)::INTEGER
WHERE release_date IS NOT NULL;

-- Prevent a genre from being both included and excluded for the same user
CREATE OR REPLACE FUNCTION prevent_genre_in_both_prefs()
RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM prefs_exclude_genre
        WHERE user_id = NEW.user_id AND genre_id = NEW.genre_id
    ) THEN
        RAISE EXCEPTION 'Genre cannot be both included and excluded for the same user';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trig_no_genre_in_both_include
BEFORE INSERT ON prefs_include_genre
FOR EACH ROW EXECUTE FUNCTION prevent_genre_in_both_prefs();

CREATE OR REPLACE FUNCTION prevent_genre_in_both_prefs_exclude()
RETURNS TRIGGER AS $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM prefs_include_genre
        WHERE user_id = NEW.user_id AND genre_id = NEW.genre_id
    ) THEN
        RAISE EXCEPTION 'Genre cannot be both included and excluded for the same user';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trig_no_genre_in_both_exclude
BEFORE INSERT ON prefs_exclude_genre
FOR EACH ROW EXECUTE FUNCTION prevent_genre_in_both_prefs_exclude();

-- Function to auto-update updated_at on user_preferences
CREATE OR REPLACE FUNCTION set_user_preferences_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to call the function before update
CREATE TRIGGER trg_set_user_preferences_updated_at
BEFORE UPDATE ON user_preferences
FOR EACH ROW
EXECUTE FUNCTION set_user_preferences_updated_at();