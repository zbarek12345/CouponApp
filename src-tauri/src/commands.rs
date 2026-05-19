use crate::models::*;
use sqlx::SqlitePool;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════
// SHOP
// ═══════════════════════════════════════════════════════════════

/// Create a shop directly (no scan step needed).
#[tauri::command]
pub async fn create_shop(
    state: tauri::State<'_, AppState>,
    request: CreateShopRequest,
) -> Result<Shop, String> {
    let shop_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO shops (shop_id, shop_name, shop_logo) VALUES (?, ?, ?)")
        .bind(&shop_id)
        .bind(&request.name)
        .bind(&request.logo)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Shop {
        shop_id,
        shop_name: request.name,
        shop_logo: request.logo,
    })
}

/// Return all shops (simple list, no pagination needed – shops are few).
#[tauri::command]
pub async fn load_shops(state: tauri::State<'_, AppState>) -> Result<Vec<Shop>, String> {
    let rows = sqlx::query_as!(
        Shop,
        "SELECT shop_id, shop_name, shop_logo FROM shops ORDER BY shop_name"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════
// COUPON  –  two-phase: scan → preview → user edits → save
// ═══════════════════════════════════════════════════════════════

/// Phase 1 – scan an image and return all detected code candidates.
/// Nothing is written to the DB. The frontend shows the candidates so the
/// user can pick the correct one (or type a value manually).
#[tauri::command]
pub async fn scan_coupon_image(
    _state: tauri::State<'_, AppState>,
    request: ScanImageRequest,
) -> Result<CouponScanPreview, String> {
    // 🔍 Replace `simulate_rxing_detection` with real rxing / bardecoder call.
    let candidates = simulate_rxing_detection(&request.image_base64);

    // Pre-select the candidate with the highest confidence.
    let best_index = candidates
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.confidence.partial_cmp(&b.confidence).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);

    Ok(CouponScanPreview {
        candidates,
        best_index,
        suggested_description: String::new(),
    })
}

/// Phase 2 – user confirmed (and possibly edited) the preview; persist it.
#[tauri::command]
pub async fn save_coupon(
    state: tauri::State<'_, AppState>,
    request: SaveCouponRequest,
) -> Result<CouponPayload, String> {
    let candidate = request
        .candidates
        .get(request.selected_candidate_index)
        .ok_or_else(|| "Invalid candidate index".to_string())?;

    let code_id = Uuid::new_v4().to_string();
    let coupon_id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO codes (code_id, code_value, code_type) VALUES (?, ?, ?)")
        .bind(&code_id)
        .bind(&candidate.code_value)
        .bind(&candidate.code_type)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO coupons (coupon_id, code_id, description, shop_id) VALUES (?, ?, ?, ?)",
    )
    .bind(&coupon_id)
    .bind(&code_id)
    .bind(&request.description)
    .bind(&request.shop_id)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(CouponPayload {
        coupon_id,
        code_id,
        description: request.description,
        shop_id: request.shop_id,
    })
}

/// Paginated list of coupons joined with their shop and code.
/// `offset` and `limit` are passed from the frontend (e.g. 0 / 20).
#[tauri::command]
pub async fn load_coupons(
    state: tauri::State<'_, AppState>,
    offset: i64,
    limit: i64,
) -> Result<Page<CouponView>, String> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM coupons")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    let items = sqlx::query_as!(
        CouponView,
        r#"
        SELECT
            c.coupon_id,
            COALESCE(c.description, '') AS description,
            c.shop_id,
            COALESCE(s.shop_name, '') AS shop_name,
            COALESCE(cd.code_value, '') AS code_value,
            COALESCE(cd.code_type, '') AS code_type
        FROM coupons c
        JOIN shops  s  ON s.shop_id  = c.shop_id
        JOIN codes  cd ON cd.code_id = c.code_id
        ORDER BY c.coupon_id
        LIMIT ? OFFSET ?
        "#,
        limit,
        offset
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Page { items, total, offset, limit })
}

// ═══════════════════════════════════════════════════════════════
// RECEIPT  –  two-phase: scan → preview → user edits → save
// ═══════════════════════════════════════════════════════════════

/// Phase 1 – OCR + LLM parse.  Nothing written to DB.
/// Returns a preview the user can edit: shop candidates, line items, totals.
#[tauri::command]
pub async fn scan_receipt_image(
    state: tauri::State<'_, AppState>,
    request: ScanImageRequest,
) -> Result<ReceiptScanPreview, String> {
    // 📄 Replace with real OCR → Claude API pipeline.
    let parsed = simulate_ocr_llm_parsing(&request.image_base64).await;

    // Find shops whose name is similar to what was parsed from the receipt.
    let matched_shops = find_matching_shops(&state.pool, &parsed.raw_shop_name).await?;

    // Use the first match as the suggestion, or empty string.
    let suggested_shop_id = matched_shops
        .first()
        .map(|s| s.shop_id.clone())
        .unwrap_or_default();

    Ok(ReceiptScanPreview {
        matched_shops,
        suggested_shop_id,
        raw_shop_name: parsed.raw_shop_name,
        total_value: parsed.total_value,
        total_discount: parsed.total_discount,
        entries: parsed.entries,
    })
}

/// Phase 2 – user confirmed (and possibly edited) the preview; persist it.
/// If `shop_id` is empty the backend creates a new shop from `new_shop_name`.
#[tauri::command]
pub async fn save_receipt(
    state: tauri::State<'_, AppState>,
    request: SaveReceiptRequest,
) -> Result<ReceiptPayload, String> {
    // Resolve (or create) the shop.
    let (shop_id, shop_name) = if request.shop_id.is_empty() {
        let new_name = request
            .new_shop_name
            .filter(|s| !s.is_empty())
            .ok_or_else(|| "Provide either an existing shop_id or a new_shop_name".to_string())?;

        let new_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO shops (shop_id, shop_name) VALUES (?, ?)")
            .bind(&new_id)
            .bind(&new_name)
            .execute(&state.pool)
            .await
            .map_err(|e| e.to_string())?;

        (new_id, new_name)
    } else {
        let name = sqlx::query_scalar::<_, String>(
            "SELECT shop_name FROM shops WHERE shop_id = ?",
        )
        .bind(&request.shop_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| format!("Shop not found: {e}"))?;

        (request.shop_id.clone(), name)
    };

    let receipt_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO receipts (receipt_id, shop_id, total_value, total_discount) VALUES (?, ?, ?, ?)",
    )
    .bind(&receipt_id)
    .bind(&shop_id)
    .bind(request.total_value)
    .bind(request.total_discount)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut persisted_entries: Vec<ReceiptEntry> = Vec::new();

    for draft in &request.entries {
        let entry_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO receipt_entries \
             (entry_id, receipt_id, entry_name, entry_quantity, entry_cost, entry_discount) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&entry_id)
        .bind(&receipt_id)
        .bind(&draft.entry_name)
        .bind(draft.entry_quantity)
        .bind(draft.entry_cost)
        .bind(draft.entry_discount)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        persisted_entries.push(ReceiptEntry {
            entry_id,
            receipt_id: receipt_id.clone(),
            entry_name: draft.entry_name.clone(),
            entry_quantity: draft.entry_quantity,
            entry_cost: draft.entry_cost,
            entry_discount: draft.entry_discount,
        });
    }

    Ok(ReceiptPayload {
        receipt_id,
        shop_name,
        total_value: request.total_value,
        total_discount: request.total_discount,
        entries: persisted_entries,
    })
}

/// Paginated receipt summaries (no entries – load those separately per receipt).
#[tauri::command]
pub async fn load_receipts(
    state: tauri::State<'_, AppState>,
    offset: i64,
    limit: i64,
) -> Result<Page<ReceiptSummary>, String> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM receipts")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    let items = sqlx::query_as!(
        ReceiptSummary,
        r#"
        SELECT
            r.receipt_id,
            r.shop_id,
            COALESCE(s.shop_name, '') AS shop_name,
            COALESCE(r.total_value, 0.0) AS total_value,
            COALESCE(r.total_discount, 0.0) AS total_discount
        FROM receipts r
        JOIN shops s ON s.shop_id = r.shop_id
        ORDER BY r.receipt_id DESC
        LIMIT ? OFFSET ?
        "#,
        limit,
        offset
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Page { items, total, offset, limit })
}

/// Fetch the full detail for one receipt including its entries.
#[tauri::command]
pub async fn load_receipt_detail(
    state: tauri::State<'_, AppState>,
    receipt_id: String,
) -> Result<ReceiptPayload, String> {
    let row = sqlx::query!(
        r#"
        SELECT r.receipt_id, s.shop_name, r.total_value, r.total_discount
        FROM receipts r
        JOIN shops s ON s.shop_id = r.shop_id
        WHERE r.receipt_id = ?
        "#,
        receipt_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| format!("Receipt not found: {e}"))?;

    let entries = sqlx::query_as!(
        ReceiptEntry,
        r#"
        SELECT entry_id, receipt_id, COALESCE(entry_name, '') AS entry_name, COALESCE(entry_quantity, 0.0) AS entry_quantity, COALESCE(entry_cost, 0.0) AS entry_cost,
        COALESCE(entry_discount, 0.0) AS entry_discount
        FROM receipt_entries
        WHERE receipt_id = ?
        ORDER BY entry_id
        "#,
        receipt_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ReceiptPayload {
        receipt_id: row.receipt_id,
        shop_name: row.shop_name,
        total_value: row.total_value,
        total_discount: row.total_discount.unwrap_or(0.0),
        entries,
    })
}

// ═══════════════════════════════════════════════════════════════
// Internal helpers
// ═══════════════════════════════════════════════════════════════

async fn find_matching_shops(pool: &SqlitePool, raw_name: &str) -> Result<Vec<Shop>, String> {
    // Simple LIKE search; replace with FTS or fuzzy matching as needed.
    let pattern = format!("%{}%", raw_name.trim());
    sqlx::query_as!(
        Shop,
        "SELECT shop_id, shop_name, shop_logo FROM shops WHERE shop_name LIKE ? ORDER BY shop_name LIMIT 5",
        pattern
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())
}

// ═══════════════════════════════════════════════════════════════
// ── SIMULATIONS ─────────────────────────────────────────────
// Replace these with real rxing / OCR+LLM implementations.
// ═══════════════════════════════════════════════════════════════

fn simulate_rxing_detection(_image_base64: &str) -> Vec<CodeCandidate> {
    vec![
        CodeCandidate {
            index: 0,
            code_value: "SUMMER-CAFFEINE-2026".to_string(),
            code_type: "QR_CODE".to_string(),
            confidence: 0.97,
        },
        CodeCandidate {
            index: 1,
            code_value: "8712345678906".to_string(),
            code_type: "EAN-13".to_string(),
            confidence: 0.61,
        },
    ]
}

async fn simulate_ocr_llm_parsing(_image_base64: &str) -> ReceiptPayloadData {
    let draft_receipt_id = Uuid::new_v4().to_string();

    ReceiptPayloadData {
        shop_id: String::new(), // not known yet – matched in scan_receipt_image
        raw_shop_name: "Cyberdyne Coffee".to_string(),
        total_value: 18.50,
        total_discount: 2.00,
        entries: vec![
            ReceiptEntryDraft {
                draft_id: Uuid::new_v4().to_string(),
                entry_name: "Tactical Espresso Double Shot".to_string(),
                entry_quantity: 2,
                entry_cost: 5.00,
                entry_discount: 0.50,
            },
            ReceiptEntryDraft {
                draft_id: Uuid::new_v4().to_string(),
                entry_name: "Cyber-Croissant V2".to_string(),
                entry_quantity: 1,
                entry_cost: 10.50,
                entry_discount: 1.00,
            },
        ],
    }
}

// ═══════════════════════════════════════════════════════════════
// Tauri app entry point
// ═══════════════════════════════════════════════════════════════

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let pool = sqlx::SqlitePool::connect("sqlite://./coupon_app.db").await.unwrap();
    
    //let mig = sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        
    tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
        .manage(AppState { pool })
        .invoke_handler(tauri::generate_handler![
            // Shops
            create_shop,
            load_shops,
            // Coupons
            scan_coupon_image,
            save_coupon,
            load_coupons,
            // Receipts
            scan_receipt_image,
            save_receipt,
            load_receipts,
            load_receipt_detail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}