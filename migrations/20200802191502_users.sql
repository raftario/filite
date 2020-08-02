CREATE TABLE users (
    user varchar(32) PRIMARY KEY,
    password varchar(256) NOT NULL,
    role smallint NOT NULL
);
