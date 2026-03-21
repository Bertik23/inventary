-- SQLite version
DROP TABLE IF EXISTS custom_item_templates;

CREATE TABLE inventory_items_old (
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

INSERT INTO inventory_items_old (id, inventory_id, barcode, name, quantity, product_data, created_at, updated_at)
SELECT id, inventory_id, barcode, name, CAST(quantity AS INTEGER), product_data, created_at, updated_at FROM inventory_items;

DROP TABLE inventory_items;
ALTER TABLE inventory_items_old RENAME TO inventory_items;
CREATE INDEX idx_barcode ON inventory_items(barcode);
