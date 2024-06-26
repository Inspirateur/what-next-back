CREATE TABLE oeuvres (
  id INTEGER PRIMARY KEY NOT NULL,
  medium INTEGER NOT NULL,
  title TEXT NOT NULL,
  rating INTEGER NOT NULL DEFAULT 0,
  synopsis TEXT NOT NULL DEFAULT '',
  picture TEXT NOT NULL DEFAULT ''
);

CREATE TABLE imdb_map (
  oeuvre_id INTEGER PRIMARY KEY NOT NULL,
  imdb_id TEXT NOT NULL UNIQUE,
  FOREIGN KEY (oeuvre_id) REFERENCES oeuvres(id)
);

CREATE TABLE mal_map (
  oeuvre_id INTEGER PRIMARY KEY NOT NULL,
  mal_id INTEGER NOT NULL UNIQUE,
  FOREIGN KEY (oeuvre_id) REFERENCES oeuvres(id)
);

CREATE TABLE isbn_map (
  oeuvre_id INTEGER PRIMARY KEY NOT NULL,
  isbn10 TEXT NOT NULL UNIQUE,
  FOREIGN KEY (oeuvre_id) REFERENCES oeuvres(id)
);

CREATE TABLE steam_map (
  oeuvre_id INTEGER PRIMARY KEY NOT NULL,
  steam_id INTEGER NOT NULL UNIQUE,
  FOREIGN KEY (oeuvre_id) REFERENCES oeuvres(id)
);

CREATE TABLE tags (
  id INTEGER PRIMARY KEY NOT NULL,
  label TEXT NOT NULL UNIQUE
);

CREATE TABLE oeuvre_tags (
    oeuvre_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (oeuvre_id, tag_id),
    FOREIGN KEY (oeuvre_id) REFERENCES oeuvres(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE TABLE users (
  id INTEGER PRIMARY KEY NOT NULL,
  username TEXT NOT NULL UNIQUE,
  phc TEXT NOT NULL
);

CREATE TABLE user_ratings (
  user_id INTEGER NOT NULL,
  oeuvre_id INTEGER NOT NULL,
  rating INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(oeuvre_id) REFERENCES oeuvres(id),
  PRIMARY KEY(user_id, oeuvre_id)
);

CREATE TABLE users_similarity (
  user1_id INTEGER NOT NULL,
  user2_id INTEGER NOT NULL,
  score INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY(user1_id) REFERENCES users(id),
  FOREIGN KEY(user2_id) REFERENCES users(id),
  PRIMARY KEY(user1_id, user2_id)
);

CREATE TABLE search_tokens (
  oeuvre_id INTEGER NOT NULL,
  token TEXT NOT NULL,
  FOREIGN KEY(oeuvre_id) REFERENCES oeuvres(id),
  PRIMARY KEY(oeuvre_id, token)
);