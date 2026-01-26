-- When the party entered its current phase (Voting, Watching, etc.). Used for timeouts.
ALTER TABLE parties
ADD COLUMN phase_entered_at TIMESTAMPTZ NOT NULL DEFAULT now();

UPDATE parties SET phase_entered_at = updated_at;
