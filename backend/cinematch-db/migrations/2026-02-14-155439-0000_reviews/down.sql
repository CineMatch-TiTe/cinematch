-- Restore reviews to party_picks (temporarily renamed back to user_tastes later)
ALTER TABLE party_picks ADD COLUMN review INTEGER;

-- Make party_id nullable again
ALTER TABLE party_picks ALTER COLUMN party_id DROP NOT NULL;

-- Rename back to the original table name
ALTER TABLE party_picks RENAME TO user_tastes;

-- Restore global data from user_ratings back into user_tastes
INSERT INTO user_tastes (user_id, movie_id, liked, review, updated_at)
SELECT user_id, movie_id, liked, rating, updated_at
FROM user_ratings;

-- Delete the new global table
DROP TABLE user_ratings;
