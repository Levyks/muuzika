-- Add migration script here (SQLITE)

CREATE TABLE "servers"
(
    "id"          INTEGER PRIMARY KEY AUTOINCREMENT,
    "address"     TEXT    NOT NULL,
    "capacity"    INTEGER NULL,
    "callsign"    TEXT    NULL,
    "description" TEXT    NULL
);