CREATE TABLE users (
  id INTEGER PRIMARY KEY NOT NULL,
  username TEXT NOT NULL UNIQUE,
  pwd_hash BLOB NOT NULL,
  pwd_salt BLOB NOT NULL
);

CREATE TABLE user_ratings (
  user_id INTEGER NOT NULL,
  oeuvre_id INTEGER NOT NULL,
  rating INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(oeuvre_id) REFERENCES oeuvres(id),
  PRIMARY KEY(user_id, oeuvre_id)
);