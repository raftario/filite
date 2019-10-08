CREATE TABLE links (
  id INTEGER NOT NULL PRIMARY KEY,
  forward TEXT NOT NULL,
  created INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
  updated INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
)
