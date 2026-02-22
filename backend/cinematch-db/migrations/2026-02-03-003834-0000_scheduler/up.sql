-- Create timeout_type enum type
CREATE TYPE timeout_type AS ENUM ('VotingStarting', 'VotingEnding', 'WatchingEnding', 'ReadyTimeout');

-- Create scheduler table to track scheduled tasks
-- Note: entries are deleted when executed or cancelled, so we only store pending schedules
CREATE TABLE schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    party_id UUID REFERENCES parties(id) ON DELETE CASCADE,
    timeout_type timeout_type NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    execute_at TIMESTAMPTZ NOT NULL
);

-- Index for finding due schedules (execute_at <= now) for batch execution
CREATE INDEX idx_schedules_execute_at ON schedules(execute_at);
-- Index for finding schedules by party (for cleanup when party disbanded)
CREATE INDEX idx_schedules_party_id ON schedules(party_id);
