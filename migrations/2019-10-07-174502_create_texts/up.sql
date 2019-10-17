CREATE TABLE texts (
  id INTEGER NOT NULL PRIMARY KEY,
  contents TEXT NOT NULL,
  highlight INTEGER NOT NULL,
  created INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
