use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use rxing::Point;
// ─────────────────────────────────────────────
// Persisted DB models (returned after a save)
// ─────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Shop {
    pub shop_id: String,
    pub shop_name: String,
    pub shop_logo: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ShopReqResult{
    pub shop_id : String,
    pub shop_name: String,
    pub logo_base64 : Option<String>
}

#[derive(Debug, Serialize)]
pub struct Code {
    pub code_id: String,
    pub code_value: String,
    pub code_type: String,
}

#[derive(Debug, Serialize)]
pub struct CouponPayload {
    pub coupon_id: String,
    pub code_id: String,
    pub description: String,
    pub shop_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptEntry {
    pub entry_id: String,
    pub receipt_id: String,
    pub entry_name: String,
    pub entry_quantity: i64,
    pub entry_cost: f64,
    pub entry_discount: f64,
}

#[derive(Debug, Serialize)]
pub struct ReceiptPayload {
    pub receipt_id: String,
    pub shop_name: String,
    pub total_value: f64,
    pub total_discount: f64,
    pub entries: Vec<ReceiptEntry>,
}

// ─────────────────────────────────────────────
// Scan preview types  (never persisted directly)
// Returned to the frontend for user review/edit.
// ─────────────────────────────────────────────

/// One candidate barcode/QR detected in an image.
/// The frontend shows a list of these so the user can pick the right one.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeCandidate {
    /// Zero-based index, used as the selection key
    pub index: usize,
    pub code_value: String,
    pub code_type: String,
    /// Confidence in [0.0, 1.0] – real detector should fill this in
    pub confidence: f64,
    pub bounds: Vec<Point>
}

/// Everything the frontend needs to show a "confirm your coupon" screen.
#[derive(Debug, Serialize)]
pub struct CouponScanPreview {
    /// All candidates detected; user picks one (or edits manually)
    pub candidates: Vec<CodeCandidate>,
    /// Index of the best candidate (pre-selected in the UI)
    pub best_index: usize,
    /// Description pre-filled from EXIF / context if available
    pub suggested_description: String,
}

/// A single editable receipt line item before saving.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptEntryDraft {
    /// Stable client-side id so the frontend can key list items
    pub draft_id: String,
    pub entry_name: String,
    pub entry_quantity: i64,
    pub entry_cost: f64,
    pub entry_discount: f64,
}

/// Everything the frontend needs to show a "confirm your receipt" screen.
#[derive(Debug, Serialize)]
pub struct ReceiptScanPreview {
    /// Shops that matched the receipt header; user picks or creates new
    pub matched_shops: Vec<Shop>,
    /// shop_id pre-selected (may be empty string if no match found)
    pub suggested_shop_id: String,
    /// Raw shop name string parsed from the image, for display / new-shop creation
    pub raw_shop_name: String,
    pub total_value: f64,
    pub total_discount: f64,
    pub entries: Vec<ReceiptEntryDraft>,
}

// ─────────────────────────────────────────────
// Inbound save requests (frontend → backend)
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateShopRequest {
    pub name: String,
    pub logo: String,
}

/// User has reviewed the scan preview, selected a candidate and optionally
/// edited the description, then presses Save.
#[derive(Debug, Deserialize)]
pub struct SaveCouponRequest {
    /// shop_id the coupon belongs to
    pub shop_id: String,
    /// Which candidate index the user chose (from CouponScanPreview)
    pub selected_candidate_index: usize,
    /// All candidates from the preview (passed back so the backend
    /// doesn't need to re-scan; keep it cheap)
    pub candidates: Vec<CodeCandidate>,
    /// Possibly edited by the user
    pub description: String,
}

/// User has reviewed the scan preview and pressed Save.
#[derive(Debug, Deserialize)]
pub struct SaveReceiptRequest {
    /// An existing shop_id, or empty string if the user wants to create a new shop
    pub shop_id: String,
    /// Only used when shop_id is empty — the new shop's name
    pub new_shop_name: Option<String>,
    pub total_value: f64,
    pub total_discount: f64,
    /// Possibly reordered or edited by the user
    pub entries: Vec<ReceiptEntryDraft>,
}

/// Raw image bytes for the scan-only step.
#[derive(Debug, Deserialize)]
pub struct ScanImageRequest {
    pub image_base64: String,
}

// ─────────────────────────────────────────────
// Paginated list payloads
// ─────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

/// Full coupon record joined with its code, for list views
#[derive(Debug, Serialize)]
pub struct CouponView {
    pub coupon_id: String,
    pub description: String,
    pub shop_id: String,
    pub shop_name: String,
    pub code_value: String,
    pub code_type: String,
}

/// Receipt summary row for the list view (no entries)
#[derive(Debug, Serialize)]
pub struct ReceiptSummary {
    pub receipt_id: String,
    pub shop_id: String,
    pub shop_name: String,
    pub total_value: f64,
    pub total_discount: f64,
}

// ─────────────────────────────────────────────
// App state
// ─────────────────────────────────────────────

pub struct AppState {
    pub pool: sqlx::SqlitePool,
}

// ─────────────────────────────────────────────
// Internal pipeline types (not exposed to JS)
// ─────────────────────────────────────────────

pub struct ReceiptPayloadData {
    pub shop_id: String,
    pub raw_shop_name: String,
    pub total_value: f64,
    pub total_discount: f64,
    pub entries: Vec<ReceiptEntryDraft>,
}