-- Party state enum for Diesel
CREATE TYPE party_state AS ENUM (
    'created',
    'picking',
    'voting',
    'watching',
    'review',
    'disbanded'
);

-- External auth provider enum
CREATE TYPE auth_provider AS ENUM (
    'google',
    'github',
    'discord'
);

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(32) NOT NULL,
    oneshot BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- External accounts table (links OAuth providers to users)
-- A user can have multiple providers (google, github, discord)
CREATE TABLE external_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider auth_provider NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    display_name VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Each provider account can only be linked once globally
    CONSTRAINT unique_provider_account UNIQUE (provider, provider_user_id),
    
    -- A user can only link one account per provider
    CONSTRAINT unique_user_provider UNIQUE (user_id, provider)
);

-- Party table
CREATE TABLE parties (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    party_leader_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    state party_state NOT NULL DEFAULT 'created',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    disbanded_at TIMESTAMPTZ -- should be set when party is disbanded
);

-- Party codes table (4-char alphanumeric codes for joining)
CREATE TABLE party_codes (
    code CHAR(4) PRIMARY KEY UNIQUE, -- Not unique since rust side will ensure uniqueness, this is just a failsafe check if this exists
    party_id UUID NOT NULL UNIQUE REFERENCES parties(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Junction table for party members
CREATE TABLE party_members (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    party_id UUID NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_ready BOOLEAN NOT NULL DEFAULT false,
    
    PRIMARY KEY (user_id, party_id)
);

-- Indexes for common queries
CREATE INDEX idx_parties_leader ON parties(party_leader_id);
CREATE INDEX idx_party_codes_party ON party_codes(party_id);
CREATE INDEX idx_party_members_party ON party_members(party_id);
CREATE INDEX idx_party_members_ready ON party_members(is_ready);
CREATE INDEX idx_party_members_user ON party_members(user_id);
CREATE INDEX idx_external_accounts_user ON external_accounts(user_id);
CREATE INDEX idx_external_accounts_provider ON external_accounts(provider, provider_user_id);

-- Trigger: Persistent users (oneshot=false) must have at least one external account
CREATE OR REPLACE FUNCTION check_persistent_user_has_external_account()
RETURNS TRIGGER AS $$
BEGIN
    -- Only check if user is being set to persistent (oneshot=false)
    IF NEW.oneshot = false THEN
        IF NOT EXISTS (SELECT 1 FROM external_accounts WHERE user_id = NEW.id) THEN
            RAISE EXCEPTION 'Persistent user (oneshot=false) must have at least one external account linked';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_persistent_user_needs_external_account
    BEFORE INSERT OR UPDATE OF oneshot ON users
    FOR EACH ROW
    WHEN (NEW.oneshot = false)
    EXECUTE FUNCTION check_persistent_user_has_external_account();

-- Trigger: Prevent deleting last external account for persistent user
CREATE OR REPLACE FUNCTION check_external_account_deletion()
RETURNS TRIGGER AS $$
DECLARE
    user_is_oneshot BOOLEAN;
    account_count INTEGER;
BEGIN
    SELECT oneshot INTO user_is_oneshot FROM users WHERE id = OLD.user_id;
    
    -- If user is persistent, ensure they keep at least one account
    IF user_is_oneshot = false THEN
        SELECT COUNT(*) INTO account_count FROM external_accounts WHERE user_id = OLD.user_id;
        IF account_count <= 1 THEN
            RAISE EXCEPTION 'Cannot delete last external account for persistent user';
        END IF;
    END IF;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_last_external_account_deletion
    BEFORE DELETE ON external_accounts
    FOR EACH ROW
    EXECUTE FUNCTION check_external_account_deletion();
