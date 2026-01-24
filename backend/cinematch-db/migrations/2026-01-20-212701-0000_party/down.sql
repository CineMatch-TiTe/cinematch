-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS trg_prevent_last_external_account_deletion ON external_accounts;
DROP TRIGGER IF EXISTS trg_persistent_user_needs_external_account ON users;
DROP FUNCTION IF EXISTS check_external_account_deletion();
DROP FUNCTION IF EXISTS check_persistent_user_has_external_account();
DROP TABLE IF EXISTS party_members;
DROP TABLE IF EXISTS party_codes;
DROP TABLE IF EXISTS parties;
DROP TABLE IF EXISTS external_accounts;
DROP TABLE IF EXISTS users;
DROP TYPE IF EXISTS party_state;
DROP TYPE IF EXISTS auth_provider;