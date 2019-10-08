CREATE TABLE texts (
  id INTEGER NOT NULL PRIMARY KEY,
  contents TEXT NOT NULL,
  created INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
