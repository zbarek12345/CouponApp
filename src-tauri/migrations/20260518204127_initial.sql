-- Enable foreign key support
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS shops (
    shop_id TEXT PRIMARY KEY NOT NULL,
    shop_name TEXT NOT NULL,
    shop_logo TEXT -- Base64 encoded or local asset URI
);

CREATE TABLE IF NOT EXISTS codes (
    code_id TEXT PRIMARY KEY NOT NULL,
    code_value TEXT NOT NULL,
    code_type TEXT NOT NULL -- e.g., 'QR', 'UPC-A', 'EAN-13', 'Code128', 'AlphaNumeric'
);

CREATE TABLE IF NOT EXISTS coupons (
    coupon_id TEXT PRIMARY KEY NOT NULL,
    code_id TEXT NOT NULL,
    description TEXT,
    shop_id TEXT NOT NULL,
    FOREIGN KEY (code_id) REFERENCES codes(code_id) ON DELETE CASCADE,
    FOREIGN KEY (shop_id) REFERENCES shops(shop_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS receipts (
    receipt_id TEXT PRIMARY KEY NOT NULL,
    shop_id TEXT NOT NULL, -- Maps to Company ID / Shop ID
    total_value REAL NOT NULL,
    total_discount REAL DEFAULT 0.0,
    FOREIGN KEY (shop_id) REFERENCES shops(shop_id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS receipt_entries (
    entry_id TEXT PRIMARY KEY NOT NULL,
    receipt_id TEXT NOT NULL,
    entry_name TEXT NOT NULL,
    entry_quantity INTEGER NOT NULL,
    entry_cost REAL NOT NULL, -- Price per unit before entry discount
    entry_discount REAL DEFAULT 0.0, -- Discount applied per single unit
    FOREIGN KEY (receipt_id) REFERENCES receipts(receipt_id) ON DELETE CASCADE
);
