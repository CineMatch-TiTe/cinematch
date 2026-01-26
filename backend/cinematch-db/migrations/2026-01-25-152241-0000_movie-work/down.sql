-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS user_preferences;

-- Drop triggers and functions for genre include/exclude constraint
DROP TRIGGER IF EXISTS trig_no_genre_in_both_include ON prefs_include_genre;
DROP TRIGGER IF EXISTS trig_no_genre_in_both_exclude ON prefs_exclude_genre;
DROP FUNCTION IF EXISTS prevent_genre_in_both_prefs();
DROP FUNCTION IF EXISTS prevent_genre_in_both_prefs_exclude();

DROP TABLE IF EXISTS prefs_include_genre;
DROP TABLE IF EXISTS prefs_exclude_genre;

ALTER TABLE parties
DROP COLUMN IF EXISTS selected_movie_id;

ALTER TABLE movies
DROP COLUMN IF EXISTS release_year;

DROP INDEX IF EXISTS idx_movies_title;

DROP TRIGGER IF EXISTS trg_set_user_preferences_updated_at ON user_preferences;
DROP FUNCTION IF EXISTS set_user_preferences_updated_at();