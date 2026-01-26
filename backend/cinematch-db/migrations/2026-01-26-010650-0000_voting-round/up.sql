-- Your SQL goes here
-- voting_round: 1 = first vote, 2 = second round (top 3). NULL when not in Voting.
ALTER TABLE parties
ADD COLUMN voting_round SMALLINT;
