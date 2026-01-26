-- Revert: make liked NOT NULL again
-- First, delete any rows with NULL liked values (skip records) to make it safe
DELETE FROM user_tastes WHERE liked IS NULL;

-- Then set the column back to NOT NULL
ALTER TABLE user_tastes ALTER COLUMN liked SET NOT NULL;
