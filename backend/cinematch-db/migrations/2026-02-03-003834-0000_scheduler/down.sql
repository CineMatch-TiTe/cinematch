-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS schedules;
DROP TYPE IF EXISTS timeout_type;

DROP INDEX IF EXISTS idx_schedules_execute_at;
DROP INDEX IF EXISTS idx_schedules_party_id;