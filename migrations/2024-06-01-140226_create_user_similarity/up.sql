CREATE TABLE users_similarity (
  user1_id INTEGER NOT NULL,
  user2_id INTEGER NOT NULL,
  score INTEGER NOT NULL,
  FOREIGN KEY(user1_id) REFERENCES users(id),
  FOREIGN KEY(user2_id) REFERENCES users(id),
  PRIMARY KEY(user1_id, user2_id)
);