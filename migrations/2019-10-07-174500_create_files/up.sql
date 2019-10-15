CREATE TABLE files (
  id INTEGER NOT NULL PRIMARY KEY,
  filepath TEXT NOT NULL,
  created INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
