
# Project Specification: Receipt & Coupon Manager (Tauri + Vue.js + Rust)

## 1. Project Overview
This project is a desktop application built using the **Tauri** framework, featuring a **Rust** backend and a **Vue.js** frontend managed exclusively via **Bun**. The application serves as a centralized dashboard for managing shops, coupons, and receipts. It leverages local **SQLite** for data persistence and integrates **OCR + LLM** capabilities for intelligent receipt processing.

---

## 2. Tech Stack & Environment Setup

### Core Stack
* **Backend:** Rust (Stable)
* **Frontend Framework:** Vue.js 3 (Composition API, Pinia for state management)
* **Package Manager & Bundler:** Bun + Vite
* **Database:** SQLite (managed via `sqlx` in Rust for compile-time checked queries)
* **Communication:** Tauri IPC (Commands & Events)

### System Requirements & Tooling
To initialize the project using Bun and Tauri, the agent must execute:
```bash
# Initialize the Tauri app with Vue and Bun
bun create tauri-app --template vue-ts

```

In `src-tauri/tauri.conf.json`, ensure the package manager and scripts point explicitly to Bun:

```json
{
  "build": {
    "beforeDevCommand": "bun run dev",
    "beforeBuildCommand": "bun run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  }
}

```

---

## 3. Database Schema (SQLite)

The database must enforce relational integrity with foreign keys. Below is the relational schema implemented via `sqlx` migrations:

```sql
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

```

---

## 4. Backend Dependencies (`Cargo.toml`)

To ensure premium parsing speed, type safety, and seamless LLM communication, utilize the following tip-top crates:

```toml
[dependencies]
tauri = { version = "2.0", features = [] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
sqlx = { version = "0.8", features = [ "runtime-tokio-native-tls", "sqlite", "uuid" ] }
uuid = { version = "1.0", features = ["v4", "serde"] }
reqwest = { version = "0.12", features = ["json", "multipart"] } # For LLM/OCR API requests
rxing = "0.5" # Multi-format barcode/QR code scanner library in pure Rust

```

---

## 5. Tauri Command Specifications & Logic Flow

### `create_company` (or `create_shop`)

* **Purpose:** Inserts a new merchant into the database.
* **Signature:**
```rust
#[tauri::command]
async fn create_shop(state: tauri::State<'_, AppState>, name: String, logo: Option<String>) -> Result<Shop, String>;

```



### `create_coupon` (with Code Scanner Recognition)

* **Purpose:** Processes a coupon image or raw stream, decodes the barcode/QR metadata using `rxing`, registers the Code entity, and links it to a Coupon.
* **Logic Flow:**
1. Receives a Base64 image string or raw image bytes from the frontend camera wrapper.
2. Passes the image matrix to `rxing::helpers::detect_multiple_in_luma`.
3. Extracts the payload string (`code_value`) and symbology type (`code_type`).
4. Persists the data across `codes` and `coupons` tables within an SQL transaction.


* **Signature:**
```rust
#[tauri::command]
async fn create_coupon(state: tauri::State<'_, AppState>, shop_id: String, description: String, image_base64: String) -> Result<CouponPayload, String>;

```



### `create_receipt` (with OCR + Multimodal LLM Parsing)

* **Purpose:** Parses visual or textual receipt structures into completely structured database items using modern AI workflows.
* **Logic Flow:**
> **Tip-Top Architecture Note:** Rather than running clunky local OCR engines followed by regular expression parsing, use a Multimodal LLM API request (e.g., Anthropic Claude Sonnet or OpenAI GPT-4o) via `reqwest`. This passes the receipt image directly with a strict JSON-Schema instruction prompt. It extracts the vendor, line items, itemized quantities, single-unit costs, and applied item/total discounts natively in a single pass.


* **Signature:**
```rust
#[tauri::command]
async fn create_receipt(state: tauri::State<'_, AppState>, image_base64: String) -> Result<ReceiptPayload, String>;

```



---

## 6. Test Datasets

Use these verified payloads for backend unit tests and frontend mocking:

### Test Input 1: `create_shop`

```json
{
  "name": "Cyberdyne Systems Cafe",
  "logo": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
}

```

### Test Input 2: `create_coupon` (Simulated Scan Event Result)

```json
{
  "shop_id": "shop_uuid_12345",
  "description": "20% Off any Espresso-based Drink",
  "scanned_code": {
    "code_value": "SUMMER-CAFFEINE-2026",
    "code_type": "QR_CODE"
  }
}

```

### Test Input 3: Structured Output Expected from OCR + LLM (`create_receipt`)

```json
{
  "shop_name": "Cyberdyne Systems Cafe",
  "total_value": 18.50,
  "total_discount": 2.00,
  "entries": [
    {
      "entry_name": "Tactical Espresso Double Shot",
      "entry_quantity": 2,
      "entry_cost": 5.00,
      "entry_discount": 0.50
    },
    {
      "entry_name": "Cyber-Croissant V2",
      "entry_quantity": 1,
      "entry_cost": 10.50,
      "entry_discount": 1.00
    }
  ]
}

```

---

## 7. Implementation Execution Checklist

* [ ] Initialize frontend directory layout via Bun and Vue TypeScript template.
* [ ] Establish `src-tauri/src/schema.sql` and integrate `sqlx` database pool setup in `main.rs`.
* [ ] Implement `rxing` decoding routines inside coupon domain services.
* [ ] Configure `reqwest` client with bearer token headers to point to a structured-output LLM inference endpoint.
* [ ] Hook frontend components to Tauri's global `invoke` client module using clean async/await structures.

```

```