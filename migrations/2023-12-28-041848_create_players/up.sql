CREATE TABLE players (
    id BIGINT PRIMARY KEY,
    username VARCHAR(63) NOT NULL,
    room_id BIGINT NOT NULL REFERENCES rooms(id),
    last_seen TIMESTAMP NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE UNIQUE INDEX players_username_room_id_idx
    ON players (username, room_id)
    WHERE enabled;

ALTER TABLE rooms
    ADD COLUMN leader_id BIGINT NULL REFERENCES players(id);