CREATE TABLE users (
    id varchar(32) NOT NULL PRIMARY KEY,
    password varchar(256) NOT NULL,
    role int NOT NULL
);
