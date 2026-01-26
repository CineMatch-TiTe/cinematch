-- Make liked nullable to support skip (true=like, false=dislike, null=skip)
-- This is safe as we're only removing a constraint, not changing data
ALTER TABLE user_tastes ALTER COLUMN liked DROP NOT NULL;
