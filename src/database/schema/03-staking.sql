
/* ---- VALIDATORS INFO ---- */

CREATE TABLE validator_info
(
    consensus_address   TEXT   NOT NULL PRIMARY KEY,
    max_change_rate     TEXT   NOT NULL,
    height              BIGINT NOT NULL
);

CREATE TABLE validator_voting_power
(
    validator_address TEXT   NOT NULL PRIMARY KEY,
    voting_power      BIGINT NOT NULL,
    height            BIGINT NOT NULL
);
CREATE INDEX validator_voting_power_height_index ON validator_voting_power (height);

CREATE TABLE validator_commission
(
    validator_address   TEXT    NOT NULL PRIMARY KEY,
    commission          DECIMAL NOT NULL,
    height              BIGINT  NOT NULL
);
CREATE INDEX validator_commission_height_index ON validator_commission (height);

CREATE TABLE validator_status
(
    validator_address TEXT    NOT NULL PRIMARY KEY,
    status            INT     NOT NULL,
    jailed            BOOLEAN NOT NULL,
    height            BIGINT  NOT NULL
);
CREATE INDEX validator_status_height_index ON validator_status (height);


CREATE TABLE validator_description
(
    validator_address TEXT   NOT NULL PRIMARY KEY,
    avatar_url        TEXT,
    website           TEXT,
    details           TEXT,
    height            BIGINT NOT NULL
);
CREATE INDEX validator_description_height_index ON validator_description (height);