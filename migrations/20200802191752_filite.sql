CREATE TABLE filite (
    id varchar(32) PRIMARY KEY,
    ty smallint NOT NULL,
    val text NOT NULL,

    creator varchar(32) NOT NULL REFERENCES users(user),
    created timestamp NOT NULL,

    visibility smallint NOT NULL,
    views int NOT NULL DEFAULT 0
);
