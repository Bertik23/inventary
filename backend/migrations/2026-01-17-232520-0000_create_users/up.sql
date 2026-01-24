-- Drop existing tables to ensure clean slate for the new schema
DROP TABLE IF EXISTS inventory_items;
DROP TABLE IF EXISTS inventory_users;
DROP TABLE IF EXISTS inventories;
DROP TABLE IF EXISTS users;

CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE inventories (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE TABLE inventory_users (
    inventory_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL,
    PRIMARY KEY (inventory_id, user_id),
    FOREIGN KEY (inventory_id) REFERENCES inventories(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE inventory_items (
    id TEXT PRIMARY KEY NOT NULL,
    inventory_id TEXT NOT NULL,
    barcode TEXT,
    name TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    product_data TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (inventory_id) REFERENCES inventories(id)
);

-- Insert default data for development (matches frontend default)
INSERT INTO users (id, username, password_hash) VALUES ('default-user', 'admin', 'HASHED_admin');
INSERT INTO inventories (id, name, owner_id) VALUES ('default-inventory', 'Default Inventory', 'default-user');
INSERT INTO inventory_users (inventory_id, user_id, role) VALUES ('default-inventory', 'default-user', 'owner');