DROP TABLE IF EXISTS users;

CREATE TABLE
    IF NOT EXISTS users (
        id integer PRIMARY KEY AUTOINCREMENT,
        username text NOT NULL,
        password_hash text NOT NULL
    );

DROP TABLE IF EXISTS user_tickets;

CREATE TABLE
    IF NOT EXISTS user_tickets (
        id integer PRIMARY KEY AUTOINCREMENT,
        def integer NOT NULL,
        user integer NOT NULL,
        qr text NOT NULL,
        usages integer DEFAULT 0
    );

DROP TABLE IF EXISTS ticket_defs;

CREATE TABLE
    IF NOT EXISTS ticket_defs (
        id integer PRIMARY KEY AUTOINCREMENT,
        title text NOT NULL,
        price integer NOT NULL,
        start text NOT NULL,
        expiry text NOT NULL
    );

INSERT INTO
    ticket_defs (title, price, start, expiry)
VALUES
    (
        'Term 2 Bee Bus Student',
        10500,
        '2025-01-01T04:00:00.000000000',
        '2025-04-01T03:59:00.000000000'
    ),
    (
        'Term 3 Bee Bus Student',
        10500,
        '2025-04-01T04:00:00.000000000',
        '2025-06-30T03:59:00.000000000'
    );