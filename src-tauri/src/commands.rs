use crate::models::*;
use crate::ocr_handler::{
    analyze_receipt_layout, OcrBlock, ReceiptFieldCandidate, ReceiptLineCandidate,
};
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use uuid::Uuid;
use base64::Engine;
use tauri::{Manager,path::BaseDirectory};
use std::path::{Path, PathBuf};
#[path = "codes_handler.rs"]
mod codes_handler;

#[derive(Debug, serde::Deserialize)]
pub struct ScanOcrBlocksRequest {
    pub blocks: Vec<OcrBlock>,
}

#[derive(Debug, serde::Serialize)]
pub struct ReceiptScanReview {
    pub matched_shops: Vec<Shop>,
    pub suggested_shop_id: String,
    pub raw_shop_name: String,
    pub total_value: f64,
    pub total_discount: f64,
    pub entries: Vec<ReceiptEntryDraft>,
    pub ocr_blocks: Vec<OcrBlock>,
    pub lines: Vec<ReceiptLineCandidate>,
    pub field_candidates: Vec<ReceiptFieldCandidate>,
}

// ═══════════════════════════════════════════════════════════════
// SHOP
// ═══════════════════════════════════════════════════════════════

/// Create a shop directly (no scan step needed).
#[tauri::command]
pub async fn create_shop(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    request: CreateShopRequest,
) -> Result<Shop, String> {

    let shop_id = Uuid::new_v4().to_string();

    let base_path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("logos");

    // Ensure directory exists
    std::fs::create_dir_all(&base_path)
        .map_err(|e| e.to_string())?;

    let image_name = format!("{shop_id}.png");
    let image_path = base_path.join(&image_name);

    println!("{:?}",image_path);

    // Decode and save image
    codes_handler::codes_handler::base64_to_image(request.logo)
        .await
        .map_err(|e| e.to_string())?
        .save(&image_path)
        .map_err(|e| e.to_string())?;
            
    sqlx::query("INSERT INTO shops (shop_id, shop_name, shop_logo) VALUES (?, ?, ?)")
        .bind(&shop_id)
        .bind(&request.name)
        .bind(&image_name)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Shop {
        shop_id,
        shop_name: request.name,
        shop_logo: Some(image_name),
    })
}

/// Return all shops (simple list, no pagination needed – shops are few).
#[tauri::command]
pub async fn load_shops(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Vec<ShopReqResult>, String> {

    let rows = sqlx::query_as!(
        Shop,
        "SELECT shop_id, shop_name, shop_logo FROM shops ORDER BY shop_name"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let base_path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("logos");

    let mut ret = Vec::with_capacity(rows.len());

    for row in rows {
        let logo_base64 = match &row.shop_logo {
            Some(logo_name) => {
                // Assert PNG extension
                assert!(
                    logo_name.ends_with(".png"),
                    "Non-PNG logo found: {}",
                    logo_name
                );

                let image_path = base_path.join(logo_name);

                // Read PNG bytes directly (fastest)
                let image_bytes =
                    std::fs::read(&image_path)
                        .map_err(|e| {
                            format!(
                                "Failed to read logo '{}': {}",
                                image_path.display(),
                                e
                            )
                        })?;

                Some(base64::engine::general_purpose::STANDARD.encode(image_bytes))
            }
            None => None,
        };

        ret.push(ShopReqResult {
            shop_id: row.shop_id,
            shop_name: row.shop_name,
            logo_base64,
        });
    }

    Ok(ret)
}

#[tauri::command]
pub async fn load_shop(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    shop_id: String,
) -> Result<ShopReqResult, String> {
    let row = sqlx::query_as!(
        Shop,
        "SELECT shop_id, shop_name, shop_logo FROM shops WHERE shop_id = ?",
        shop_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| format!("Shop not found: {e}"))?;

    let base_path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("logos");

    let logo_base64 = match &row.shop_logo {
        Some(logo_name) => {
            let image_path = base_path.join(logo_name);
            let image_bytes = std::fs::read(&image_path)
                .map_err(|e| format!("Failed to read logo '{}': {}", image_path.display(), e))?;
            Some(base64::engine::general_purpose::STANDARD.encode(image_bytes))
        }
        None => None,
    };

    Ok(ShopReqResult {
        shop_id: row.shop_id,
        shop_name: row.shop_name,
        logo_base64,
    })
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
    let candidates = detect_codes(&request.image_base64).await;

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

#[tauri::command]
pub async fn load_coupon(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    coupon_id: String,
) -> Result<CouponDetailView, String> {
    let coupon = sqlx::query_as!(
        CouponView,
        r#"
        SELECT
            c.coupon_id,
            COALESCE(c.description, '')  AS "description!: String",
            c.shop_id,
            COALESCE(s.shop_name, '')  AS "shop_name!: String",
            COALESCE(cd.code_value, '') AS "code_value!: String",
            COALESCE(cd.code_type, '')  AS "code_type!: String"
        FROM coupons c
        JOIN shops  s  ON s.shop_id  = c.shop_id
        JOIN codes  cd ON cd.code_id = c.code_id
        WHERE c.coupon_id = ?
        "#,
        coupon_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| format!("Coupon not found: {e}"))?;

    let shop = load_shop(state, app, coupon.shop_id.clone()).await?;

    Ok(CouponDetailView {
        coupon_id: coupon.coupon_id,
        description: coupon.description,
        shop_id: coupon.shop_id,
        shop_name: coupon.shop_name,
        shop_logo_base64: shop.logo_base64,
        code_value: coupon.code_value,
        code_type: coupon.code_type,
    })
}

#[tauri::command]
pub async fn load_coupons_for_shop(
    state: tauri::State<'_, AppState>,
    shop_id: String,
    offset: i64,
    limit: i64,
) -> Result<Page<CouponView>, String> {
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM coupons WHERE shop_id = ?"
    )
    .bind(&shop_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let items = sqlx::query_as!(
        CouponView,
        r#"
        SELECT
            c.coupon_id,
            COALESCE(c.description, '')  AS description,
            c.shop_id,
            COALESCE(s.shop_name, '')    AS shop_name,
            COALESCE(cd.code_value, '')  AS code_value,
            COALESCE(cd.code_type, '')   AS code_type
        FROM coupons c
        JOIN shops  s  ON s.shop_id  = c.shop_id
        JOIN codes  cd ON cd.code_id = c.code_id
        WHERE c.shop_id = ?
        ORDER BY c.coupon_id
        LIMIT ? OFFSET ?
        "#,
        shop_id,
        limit,
        offset
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Page { items, total, offset, limit })
}

#[tauri::command]
pub async fn load_receipts_for_shop(
    state: tauri::State<'_, AppState>,
    shop_id: String,
    offset: i64,
    limit: i64,
) -> Result<Page<ReceiptSummary>, String> {
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM receipts WHERE shop_id = ?"
    )
    .bind(&shop_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let items = sqlx::query_as!(
        ReceiptSummary,
        r#"
        SELECT
            r.receipt_id,
            r.shop_id,
            COALESCE(s.shop_name, '')    AS shop_name,
            COALESCE(r.total_value, 0.0) AS total_value,
            COALESCE(r.total_discount, 0.0) AS total_discount
        FROM receipts r
        JOIN shops s ON s.shop_id = r.shop_id
        WHERE r.shop_id = ?
        ORDER BY r.receipt_id DESC
        LIMIT ? OFFSET ?
        "#,
        shop_id,
        limit,
        offset
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Page { items, total, offset, limit })
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


#[tauri::command]
pub async fn generate_coupon_code_from_str(state: tauri::State<'_, AppState>,coupon_value: &str, coupon_type : &str)-> Result<String, String>{

    let base64_image = codes_handler::codes_handler::render_code(rxing::BarcodeFormat::from(coupon_type), coupon_value.to_string())
        .await
        .map_err(|e| format!("Failed to generate barcode: {}", e))?;

    return Ok(base64_image);
}

#[tauri::command]
pub async fn generate_coupon_code(
    state: tauri::State<'_, AppState>,
    code_id: uuid::Uuid,
) -> Result<String, String> {
    
    let code_str = code_id.to_string();
    // 1. Fetch the code from database
    let code_record = sqlx::query!(
        r#"
        SELECT c.code_id, c.code_type, c.code_value 
        FROM codes c
        WHERE c.code_id = ?
        "#,
        code_str   // since code_id is TEXT in DB
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    let code_record = match code_record {
        Some(r) => r,
        None => return Err("Code not found".to_string()),
    };

    // 2. Convert string code_type to rxing::BarcodeFormat
    let barcode_format = rxing::BarcodeFormat::from(code_record.code_type.as_str());
    // 3. Generate Base64 image
    let base64_image = codes_handler::codes_handler::render_code(barcode_format, code_record.code_value)
        .await
        .map_err(|e| format!("Failed to generate barcode: {}", e))?;

    Ok(base64_image)
}
// ═══════════════════════════════════════════════════════════════
// RECEIPT  –  two-phase: scan → preview → user edits → save
// ═══════════════════════════════════════════════════════════════

/// Phase 1 – parse OCR word boxes. Nothing written to DB.
/// Returns a preview the user can edit: shop candidates, line items, totals.
#[tauri::command]
pub async fn scan_receipt_image(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    request: ScanImageRequest,
) -> Result<ReceiptScanReview, String> {
    let blocks = if looks_like_ocr_json(&request.image_base64) {
        parse_receipt_ocr_blocks(&request.image_base64)?
    } else {
        scan_image_with_oar_ocr(app, request.image_base64).await?
    };

    build_receipt_scan_review(&state.pool, blocks).await
}

/// Same parser as `scan_receipt_image`, but starts from OCR words that were
/// already produced by a frontend OCR/camera plugin.
#[tauri::command]
pub async fn scan_receipt_ocr_blocks(
    state: tauri::State<'_, AppState>,
    request: ScanOcrBlocksRequest,
) -> Result<ReceiptScanReview, String> {
    build_receipt_scan_review(&state.pool, request.blocks).await
}

async fn build_receipt_scan_review(
    pool: &SqlitePool,
    blocks: Vec<OcrBlock>,
) -> Result<ReceiptScanReview, String> {
    let analysis = analyze_receipt_layout(blocks);
    let matched_shops = find_matching_shops(pool, &analysis.raw_shop_name).await?;
    let suggested_shop_id = matched_shops
        .first()
        .map(|s| s.shop_id.clone())
        .unwrap_or_default();

    Ok(ReceiptScanReview {
        matched_shops,
        suggested_shop_id,
        raw_shop_name: analysis.raw_shop_name,
        total_value: analysis.total_value,
        total_discount: analysis.total_discount,
        entries: analysis.entries,
        ocr_blocks: analysis.ocr_blocks,
        lines: analysis.lines,
        field_candidates: analysis.field_candidates,
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
        shop_id,
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
        SELECT r.receipt_id, r.shop_id, s.shop_name, r.total_value, r.total_discount
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
        shop_id: row.shop_id,
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

fn parse_receipt_ocr_blocks(input: &str) -> Result<Vec<OcrBlock>, String> {
    let trimmed = input.trim();

    if trimmed.starts_with('{') {
        let value: serde_json::Value =
            serde_json::from_str(trimmed).map_err(|e| format!("Invalid OCR JSON object: {e}"))?;
        let blocks = value
            .get("blocks")
            .cloned()
            .ok_or_else(|| "OCR JSON object must contain a blocks array".to_string())?;
        return serde_json::from_value(blocks)
            .map_err(|e| format!("Invalid OCR blocks schema: {e}"));
    }

    if trimmed.starts_with('[') {
        return serde_json::from_str(trimmed)
            .map_err(|e| format!("Invalid OCR JSON array: {e}"));
    }

    Err("Receipt scan now expects OCR word boxes JSON. Use scan_receipt_ocr_blocks, or pass JSON as image_base64 for compatibility.".to_string())
}

fn looks_like_ocr_json(input: &str) -> bool {
    let trimmed = input.trim();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

async fn scan_image_with_oar_ocr(
    app: tauri::AppHandle,
    image_base64: String,
) -> Result<Vec<OcrBlock>, String> {
    let model_paths = resolve_oar_model_paths(&app)?;
    let image_bytes = decode_image_base64(&image_base64)?;

    tauri::async_runtime::spawn_blocking(move || {
        let image = image::load_from_memory(&image_bytes)
            .map_err(|e| format!("Failed to decode receipt image: {e}"))?
            .to_rgb8();

        let mut builder = oar_ocr::oarocr::OAROCRBuilder::new(
            model_paths.text_detection,
            model_paths.text_recognition,
            model_paths.character_dict,
        )
        .return_word_box(true)
        .image_batch_size(1)
        .region_batch_size(16);

        if let Some(path) = model_paths.document_orientation {
            builder = builder.with_document_image_orientation_classification(path);
        }

        if let Some(path) = model_paths.text_line_orientation {
            builder = builder.with_text_line_orientation_classification(path);
        }

        let ocr = builder
            .build()
            .map_err(|e| format!("Failed to initialize local OAR OCR pipeline: {e}"))?;
        let mut results = ocr
            .predict(vec![image])
            .map_err(|e| format!("Local OAR OCR scan failed: {e}"))?;

        let result = results
            .pop()
            .ok_or_else(|| "Local OAR OCR returned no image result".to_string())?;

        Ok(oar_regions_to_blocks(result.text_regions))
    })
    .await
    .map_err(|e| format!("Local OAR OCR worker failed: {e}"))?
}

struct OarModelPaths {
    text_detection: PathBuf,
    text_recognition: PathBuf,
    character_dict: PathBuf,
    document_orientation: Option<PathBuf>,
    text_line_orientation: Option<PathBuf>,
}

fn resolve_oar_model_paths(app: &tauri::AppHandle) -> Result<OarModelPaths, String> {
    let default_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("ocr-models");

    if !default_dir.exists() {
        std::fs::create_dir_all(&default_dir)
            .map_err(|e| format!("Failed to create OCR models directory: {e}"))?;
    }

    let setup_dir = Path::new("./ocr-models");

    let paths = OarModelPaths {
        text_detection: env_path_or_default(
            "OAR_OCR_TEXT_DETECTION_MODEL",
            default_dir.join("det.onnx"),
        ),
        text_recognition: env_path_or_default(
            "OAR_OCR_TEXT_RECOGNITION_MODEL",
            default_dir.join("rec.onnx"),
        ),
        character_dict: env_path_or_default(
            "OAR_OCR_CHARACTER_DICT",
            default_dir.join("dict.txt"),
        ),
        document_orientation: optional_env_path_or_default(
            "OAR_OCR_DOCUMENT_ORIENTATION_MODEL",
            default_dir.join("doc_orient.onnx"),
        ),
        text_line_orientation: optional_env_path_or_default(
            "OAR_OCR_TEXT_LINE_ORIENTATION_MODEL",
            default_dir.join("line_orient.onnx"),
        ),
    };
    
    for (label, path) in [
        ("text detection model", &paths.text_detection),
        ("text recognition model", &paths.text_recognition),
        ("character dictionary", &paths.character_dict),
    ] {
        if !path.exists() {
            let file_name = match label {
                "text detection model" => "det.onnx",
                "text recognition model" => "rec.onnx",
                "character dictionary" => "dict.txt",
                _ => "unknown",
            };

            let setup_path: PathBuf = setup_dir.join(file_name);
            
            if std::fs::exists(&setup_path).is_ok() {
                std::fs::copy(setup_path, path)
                    .map_err(|e| format!("Failed to copy {label} from setup directory: {e}"))?;
                continue;
            }

            return Err(format!(
                "Missing OAR OCR {label}: {}. Put models in the app data ocr-models folder or set OAR_OCR_TEXT_DETECTION_MODEL, OAR_OCR_TEXT_RECOGNITION_MODEL, and OAR_OCR_CHARACTER_DICT.",
                path.display()
            ));
        }
    }

    Ok(paths)
}

fn env_path_or_default(name: &str, default_path: PathBuf) -> PathBuf {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or(default_path)
}

fn optional_env_path_or_default(name: &str, default_path: PathBuf) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| default_path.exists().then_some(default_path))
}

fn decode_image_base64(image_base64: &str) -> Result<Vec<u8>, String> {
    let payload = image_base64
        .split_once(',')
        .map(|(_, payload)| payload)
        .unwrap_or(image_base64)
        .trim();

    base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|e| format!("Invalid receipt image base64: {e}"))
}

fn oar_regions_to_blocks(regions: Vec<oar_ocr::oarocr::TextRegion>) -> Vec<OcrBlock> {
    regions
        .into_iter()
        .flat_map(|region| {
            let Some(text) = region.text else {
                return Vec::new();
            };

            let text = text.to_string();
            if text.trim().is_empty() {
                return Vec::new();
            }

            if let Some(word_boxes) = region.word_boxes {
                let words = text.split_whitespace().collect::<Vec<_>>();
                if !words.is_empty() && words.len() == word_boxes.len() {
                    return words
                        .into_iter()
                        .zip(word_boxes)
                        .map(|(word, bbox)| OcrBlock {
                            text: word.to_string(),
                            bounding_box: oar_box_to_receipt_box(&bbox),
                        })
                        .collect();
                }
            }

            let words = text.split_whitespace().collect::<Vec<_>>();
            if words.len() > 1 {
                return split_region_text_into_blocks(&text, &region.bounding_box);
            }

            vec![OcrBlock {
                text,
                bounding_box: oar_box_to_receipt_box(&region.bounding_box),
            }]
        })
        .collect()
}

fn split_region_text_into_blocks(
    text: &str,
    box_: &oar_ocr::processors::BoundingBox,
) -> Vec<OcrBlock> {
    let x = box_.x_min() as f64;
    let y = box_.y_min() as f64;
    let width = (box_.x_max() as f64 - x).max(1.0);
    let height = (box_.y_max() as f64 - y).max(1.0);
    let text_lines = text.lines().filter(|line| !line.trim().is_empty()).collect::<Vec<_>>();

    if text_lines.len() > 1 {
        let line_height = height / text_lines.len() as f64;

        return text_lines
            .into_iter()
            .enumerate()
            .flat_map(|(line_index, line)| {
                split_text_line_into_blocks(line, x, y + line_height * line_index as f64, width, line_height)
            })
            .collect();
    }

    split_text_line_into_blocks(text, x, y, width, height)
}

fn split_text_line_into_blocks(
    text: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Vec<OcrBlock> {
    let text_len = text.len().max(1) as f64;
    let mut cursor = 0usize;

    text.split_whitespace()
        .map(|word| {
            let leading_bytes = text[cursor..]
                .char_indices()
                .find(|(_, character)| !character.is_whitespace())
                .map(|(index, _)| index)
                .unwrap_or(0);
            cursor += leading_bytes;

            let start = cursor;
            cursor += word.len();

            let start_ratio = start as f64 / text_len;
            let end_ratio = cursor as f64 / text_len;

            OcrBlock {
                text: word.to_string(),
                bounding_box: crate::ocr_handler::BoundingBox {
                    x: x + width * start_ratio,
                    y,
                    width: (width * (end_ratio - start_ratio)).max(6.0),
                    height,
                },
            }
        })
        .collect()
}

fn oar_box_to_receipt_box(box_: &oar_ocr::processors::BoundingBox) -> crate::ocr_handler::BoundingBox {
    let x = box_.x_min() as f64;
    let y = box_.y_min() as f64;
    crate::ocr_handler::BoundingBox {
        x,
        y,
        width: (box_.x_max() as f64 - x).max(0.0),
        height: (box_.y_max() as f64 - y).max(0.0),
    }
}

// ═══════════════════════════════════════════════════════════════
// ── SIMULATIONS ─────────────────────────────────────────────
// Replace these with real rxing / OCR implementations.
// ═══════════════════════════════════════════════════════════════

async fn detect_codes(_image_base64: &str) -> Vec<CodeCandidate>{
    return codes_handler::codes_handler::read_image_code(_image_base64.to_string()).await.unwrap();
}

// ═══════════════════════════════════════════════════════════════
// Tauri app entry point
// ═══════════════════════════════════════════════════════════════

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                // Get app data dir
                let app_data_dir = handle
                    .path()
                    .resolve("", BaseDirectory::AppData)
                    .expect("Failed to resolve app data dir");

                // Ensure folder exists
                std::fs::create_dir_all(&app_data_dir)
                    .expect("Failed to create app data dir");

                // Database path
                let db_path = app_data_dir.join("coupon_app.db");
                
                println!("{:?}", db_path);

                // SQLite connection string
                let db_url = format!(
                    "sqlite://{}",
                    db_path.to_string_lossy()
                );
                println!("{:?}", db_url);

                if !Sqlite::database_exists(&db_url).await.expect("Failed to verify db existance") {
                    sqlx::Sqlite::create_database(&db_url).await.expect("Db creation failed");
                }

                // Create DB automatically if missing
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&db_url)
                    .await
                    .expect("Failed to connect to database");

                // Run migrations
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .expect("Failed to run migrations");

                // Store state
                handle.manage(AppState { pool });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Shops
            create_shop,
            load_shops,
            load_shop,
            load_coupons_for_shop,
            load_receipts_for_shop,
            // Coupons
            scan_coupon_image,
            save_coupon,
            load_coupons,
            load_coupon,
            generate_coupon_code_from_str,
            // Receipts
            scan_receipt_image,
            scan_receipt_ocr_blocks,
            save_receipt,
            load_receipts,
            load_receipt_detail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
