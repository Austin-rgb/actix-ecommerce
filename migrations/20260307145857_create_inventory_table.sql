CREATE TABLE IF NOT EXISTS inventory (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    sku TEXT NOT NULL,
    total_quantity INTEGER NOT NULL,
    reserved_quantity INTEGER NOT NULL DEFAULT 0
);
