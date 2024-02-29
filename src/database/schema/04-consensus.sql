CREATE TABLE average_block_time_per_hour
(
    one_row_id   BOOL    NOT NULL DEFAULT TRUE PRIMARY KEY,
    average_time DECIMAL NOT NULL,
    height       BIGINT  NOT NULL,
    CHECK (one_row_id)
);
CREATE INDEX average_block_time_per_hour_height_index ON average_block_time_per_hour (height);

CREATE TABLE average_block_time_per_day
(
    one_row_id   BOOL    NOT NULL DEFAULT TRUE PRIMARY KEY,
    average_time DECIMAL NOT NULL,
    height       BIGINT  NOT NULL,
    CHECK (one_row_id)
);
CREATE INDEX average_block_time_per_day_height_index ON average_block_time_per_day (height);
