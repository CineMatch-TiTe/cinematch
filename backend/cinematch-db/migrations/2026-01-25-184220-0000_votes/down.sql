-- This file should undo anything in `up.sql`

ALTER TABLE parties
DROP COLUMN IF EXISTS can_vote;

DROP TABLE IF EXISTS votes;
DROP TABLE IF EXISTS shown_movies;

-- Drop trigger and function for auto-updating voted_at
DROP TRIGGER IF EXISTS trg_set_votes_voted_at ON votes;
DROP FUNCTION IF EXISTS set_votes_voted_at;