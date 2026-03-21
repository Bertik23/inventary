-- SQLite version
-- Add unit column and change quantity to REAL
CREATE TABLE inventory_items_new (
    id TEXT PRIMARY KEY NOT NULL,
    inventory_id TEXT NOT NULL,
    barcode TEXT,
    name TEXT NOT NULL,
    quantity REAL NOT NULL DEFAULT 1.0,
    unit TEXT NOT NULL DEFAULT 'pcs',
    product_data TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(inventory_id) REFERENCES inventories(id)
);

INSERT INTO inventory_items_new (id, inventory_id, barcode, name, quantity, unit, product_data, created_at, updated_at)
SELECT id, inventory_id, barcode, name, CAST(quantity AS REAL), 'pcs', product_data, created_at, updated_at FROM inventory_items;

DROP TABLE inventory_items;
ALTER TABLE inventory_items_new RENAME TO inventory_items;
CREATE INDEX idx_barcode ON inventory_items(barcode);
CREATE INDEX idx_inventory_id ON inventory_items(inventory_id);

-- Create custom item templates table
CREATE TABLE IF NOT EXISTS custom_item_templates (
    id TEXT PRIMARY KEY NOT NULL,
    inventory_id TEXT, -- NULL means global default
    name TEXT NOT NULL,
    default_unit TEXT NOT NULL DEFAULT 'pcs',
    UNIQUE(inventory_id, name)
);

-- Insert sensible defaults
INSERT INTO custom_item_templates (id, inventory_id, name, default_unit) VALUES 
('1', NULL, 'Apple', 'pcs'),
('2', NULL, 'Banana', 'pcs'),
('3', NULL, 'Orange', 'pcs'),
('4', NULL, 'Tomato', 'pcs'),
('5', NULL, 'Cucumber', 'pcs'),
('6', NULL, 'Potato', 'kg'),
('7', NULL, 'Onion', 'kg'),
('8', NULL, 'Carrot', 'pcs'),
('9', NULL, 'Flour', 'kg'),
('10', NULL, 'Rice', 'kg'),
('11', NULL, 'Sugar', 'kg'),
('12', NULL, 'Milk', 'l'),
('13', NULL, 'Egg', 'pcs'),
('14', NULL, 'Butter', 'g'),
('15', NULL, 'Pasta', 'g');
