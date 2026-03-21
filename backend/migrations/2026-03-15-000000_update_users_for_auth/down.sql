DROP INDEX IF EXISTS idx_users_email;
-- SQLite doesn't support DROP COLUMN in older versions, but for migration consistency:
ALTER TABLE users DROP COLUMN email;
ALTER TABLE users DROP COLUMN reset_token;
ALTER TABLE users DROP COLUMN reset_token_expiry;
