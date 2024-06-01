CREATE TABLE users_similarity (
  user1_id INTEGER NOT NULL,
  user2_id INTEGER NOT NULL,
  score INTEGER NOT NULL,
  FOREIGN KEY(user1_id) REFERENCES users(id),
  FOREIGN KEY(user2_id) REFERENCES users(id),
  PRIMARY KEY(user1_id, user2_id)
);

CREATE TABLE user_tags_similarity (
  user_id INTEGER NOT NULL,
  tag_id INTEGER NOT NULL,
  score INTEGER NOT NULL,
  FOREIGN KEY(user_id) REFERENCES users(id),
  FOREIGN KEY(tag_id) REFERENCES tags(id),
  PRIMARY KEY(user_id, tag_id)
);
