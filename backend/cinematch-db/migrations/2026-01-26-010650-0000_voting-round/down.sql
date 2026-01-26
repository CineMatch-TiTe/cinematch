-- This file should undo anything in `up.sql`
ALTER TABLE parties
DROP COLUMN IF EXISTS voting_round;
