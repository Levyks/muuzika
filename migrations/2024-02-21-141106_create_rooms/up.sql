CREATE TYPE room_status AS ENUM ('lobby', 'game', 'results');

CREATE TABLE rooms (
    id BIGINT PRIMARY KEY,
    code VARCHAR(31) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    status room_status NOT NULL DEFAULT  'lobby'
);

CREATE UNIQUE INDEX rooms_code_idx
    ON rooms (code)
    WHERE active;


