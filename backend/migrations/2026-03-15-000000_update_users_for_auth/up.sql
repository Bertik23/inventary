ALTER TABLE users ADD COLUMN email TEXT;
ALTER TABLE users ADD COLUMN reset_token TEXT;
ALTER TABLE users ADD COLUMN reset_token_expiry TIMESTAMP;

UPDATE users SET email = 'admin@example.com' WHERE id = 'default-user';

CREATE UNIQUE INDEX idx_users_email ON users(email);
