-- Create user_ratings table for global, long-term user relationship with movies
CREATE TABLE user_ratings (
    rating_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    movie_id BIGINT NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
    liked BOOLEAN, -- true=like, false=dislike, null=skip/none
    rating INTEGER, -- 0-10 star rating
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, movie_id)
);

-- Migrate global tastes (where party_id is null) to user_ratings
-- We use DISTINCT just in case, though app logic should have prevented duplicates
INSERT INTO user_ratings (user_id, movie_id, liked, rating, updated_at)
SELECT DISTINCT ON (user_id, movie_id) user_id, movie_id, liked, review, updated_at
FROM user_tastes
WHERE party_id IS NULL
ORDER BY user_id, movie_id, updated_at DESC;

-- Rename user_tastes to party_picks to reflect its new session-only purpose
ALTER TABLE user_tastes RENAME TO party_picks;

-- Remove global entries from party_picks as they are now in user_ratings
DELETE FROM party_picks WHERE party_id IS NULL;

-- Make party_id NOT NULL in party_picks since it is now strictly for party sessions
ALTER TABLE party_picks ALTER COLUMN party_id SET NOT NULL;

-- Remove review column from party_picks as reviews are now global in user_ratings
ALTER TABLE party_picks DROP COLUMN review;
