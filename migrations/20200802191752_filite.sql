CREATE TABLE filite (
    id varchar(32) NOT NULL PRIMARY KEY,
    ty int NOT NULL,
    val text NOT NULL,

    creator varchar(32) NOT NULL REFERENCES users(id),
    created timestamp NOT NULL,

    visibility int NOT NULL,
    views int NOT NULL
);
