CREATE TABLE proposal
(
    id                  INTEGER   NOT NULL PRIMARY KEY,
    title               TEXT      NOT NULL,
    description         TEXT      NOT NULL,
    metadata            TEXT      NOT NULL,
    content             JSONB     NOT NULL DEFAULT '[]'::JSONB,
    submit_time         TIMESTAMP NOT NULL,
    voting_start_epoch  BIGINT    NOT NULL,
    voting_end_epoch    BIGINT    NOT NULL,
    grace_epoch         BIGINT    NOT NULL,
    proposer_address    TEXT,
    status              TEXT
);
CREATE INDEX proposal_proposer_address_index ON proposal (proposer_address);

CREATE TABLE proposal_vote
(
    proposal_id   INTEGER NOT NULL,
    voter_address TEXT    NOT NULL,
    option        TEXT    NOT NULL,
    height        BIGINT  NOT NULL,
    CONSTRAINT unique_vote UNIQUE (proposal_id, voter_address)
);
CREATE INDEX proposal_vote_proposal_id_index ON proposal_vote (proposal_id);
CREATE INDEX proposal_vote_voter_address_index ON proposal_vote (voter_address);
CREATE INDEX proposal_vote_height_index ON proposal_vote (height);

CREATE TABLE proposal_tally_result
(
    proposal_id  INTEGER    REFERENCES proposal (id) PRIMARY KEY,
    tally_type   TEXT       NOT NULL,
    yes          TEXT       NOT NULL,
    abstain      TEXT       NOT NULL,
    no           TEXT       NOT NULL,
    height       BIGINT     NOT NULL,
    CONSTRAINT unique_tally_result UNIQUE (proposal_id)
);
CREATE INDEX proposal_tally_result_proposal_id_index ON proposal_tally_result (proposal_id);
CREATE INDEX proposal_tally_result_height_index ON proposal_tally_result (height);
