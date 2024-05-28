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
