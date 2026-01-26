-- Your SQL goes here
-- Table to track which movies were shown to which user in a party
CREATE TABLE shown_movies (
    party_id UUID NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    movie_id BIGINT NOT NULL REFERENCES movies(movie_id) ON DELETE CASCADE,
    shown_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (party_id, user_id, movie_id)
);

-- Table to store votes
CREATE TABLE votes (
    party_id UUID NOT NULL,
    user_id UUID NOT NULL,
    movie_id BIGINT NOT NULL,
    vote_value BOOLEAN NOT NULL, -- TRUE = like, FALSE = dislike
    voted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (party_id, user_id, movie_id),
    FOREIGN KEY (party_id, user_id, movie_id) REFERENCES shown_movies(party_id, user_id, movie_id) ON DELETE CASCADE
);

-- Add can vote to parties
ALTER TABLE parties
ADD COLUMN can_vote BOOLEAN NOT NULL DEFAULT FALSE;

-- Function to auto-update voted_at on votes
CREATE OR REPLACE FUNCTION set_votes_voted_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.voted_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to call the function before update
CREATE TRIGGER trg_set_votes_voted_at
BEFORE UPDATE ON votes
FOR EACH ROW
EXECUTE FUNCTION set_votes_voted_at();