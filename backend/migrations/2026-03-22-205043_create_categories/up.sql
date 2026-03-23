-- Create categories and item_categories tables
CREATE TABLE inventory_categories (
    id TEXT PRIMARY KEY NOT NULL,
    inventory_id TEXT NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT,
    FOREIGN KEY (inventory_id) REFERENCES inventories(id),
    FOREIGN KEY (parent_id) REFERENCES inventory_categories(id)
);

CREATE TABLE item_categories (
    item_id TEXT NOT NULL,
    category_id TEXT NOT NULL,
    PRIMARY KEY (item_id, category_id),
    FOREIGN KEY (item_id) REFERENCES inventory_items(id),
    FOREIGN KEY (category_id) REFERENCES inventory_categories(id)
);

-- Add index for performance
CREATE INDEX idx_inventory_categories_inventory_id ON inventory_categories(inventory_id);
