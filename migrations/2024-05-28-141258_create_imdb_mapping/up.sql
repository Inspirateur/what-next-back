CREATE TABLE imdb_map (
  oeuvre_id INTEGER NOT NULL UNIQUE,
  imdb_id TEXT NOT NULL UNIQUE,
  PRIMARY KEY (oeuvre_id, imdb_id),
  FOREIGN KEY (oeuvre_id) REFERENCES oeuvres(id)
)
