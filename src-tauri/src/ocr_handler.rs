use crate::models::ReceiptEntryDraft;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OcrBlock {
    pub text: String,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptLineCandidate {
    pub line_id: String,
    pub index: usize,
    pub text: String,
    pub role: String,
    pub confidence: f64,
    pub bounding_box: BoundingBox,
    pub words: Vec<OcrBlock>,
    pub selected: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptFieldCandidate {
    pub field_id: String,
    pub role: String,
    pub label: String,
    pub value_text: String,
    pub numeric_value: Option<f64>,
    pub confidence: f64,
    pub line_index: Option<usize>,
    pub bounding_box: BoundingBox,
    pub selected: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptOcrAnalysis {
    pub raw_shop_name: String,
    pub total_value: f64,
    pub total_discount: f64,
    pub entries: Vec<ReceiptEntryDraft>,
    pub ocr_blocks: Vec<OcrBlock>,
    pub lines: Vec<ReceiptLineCandidate>,
    pub field_candidates: Vec<ReceiptFieldCandidate>,
}

pub fn analyze_receipt_layout(blocks: Vec<OcrBlock>) -> ReceiptOcrAnalysis {
    let mut cleaned_blocks: Vec<OcrBlock> = blocks
        .into_iter()
        .filter(|block| !block.text.trim().is_empty())
        .map(|mut block| {
            block.text = normalize_ocr_token(block.text.trim());
            block
        })
        .collect();

    if cleaned_blocks.is_empty() {
        return ReceiptOcrAnalysis {
            raw_shop_name: String::new(),
            total_value: 0.0,
            total_discount: 0.0,
            entries: Vec::new(),
            ocr_blocks: Vec::new(),
            lines: Vec::new(),
            field_candidates: Vec::new(),
        };
    }

    cleaned_blocks.sort_by(|a, b| {
        center_y(&a.bounding_box)
            .partial_cmp(&center_y(&b.bounding_box))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.bounding_box
                    .x
                    .partial_cmp(&b.bounding_box.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let median_height = median(
        cleaned_blocks
            .iter()
            .map(|block| block.bounding_box.height.max(1.0))
            .collect(),
    );
    let tolerance = (median_height * 0.7).max(6.0);
    let grouped = group_into_lines(cleaned_blocks.clone(), tolerance);
    let mut lines: Vec<ReceiptLineCandidate> = grouped
        .into_iter()
        .enumerate()
        .map(|(index, mut words)| {
            words.sort_by(|a, b| {
                a.bounding_box
                    .x
                    .partial_cmp(&b.bounding_box.x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let text = words
                .iter()
                .map(|word| word.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            ReceiptLineCandidate {
                line_id: Uuid::new_v4().to_string(),
                index,
                role: "unknown".to_string(),
                confidence: 0.2,
                bounding_box: union_boxes(words.iter().map(|word| &word.bounding_box)),
                words,
                text,
                selected: false,
            }
        })
        .collect();

    let mut analysis = ReceiptOcrAnalysis {
        raw_shop_name: detect_shop_name(&lines),
        total_value: 0.0,
        total_discount: 0.0,
        entries: Vec::new(),
        ocr_blocks: cleaned_blocks,
        lines: Vec::new(),
        field_candidates: Vec::new(),
    };

    let price_re = Regex::new(r"(?i)-?\d+[,.]\d{2}").expect("valid price regex");
    let qty_re =
        Regex::new(r"(?i)(?:^|\s)(\d{1,3})\s*(?:x|\*|szt\.?|pcs|@)(?:\s|$)")
            .expect("valid quantity regex");
    let unit_price_re =
        Regex::new(r"(?i)(\d{1,3})\s*(?:szt\.?|pcs)?\s*(?:x|\*)\s*(-?\d+[,.]\d{2})")
            .expect("valid unit regex");
    let merged_item_re = Regex::new(
        r"(?i)(\d{1,3})\s*(?:szt\.?|pcs)?\s*(?:x|\*)\s*(-?\d+[,.]\d{2})\s*(?:zł|zl|pln)?\.?\s+(-?\d+[,.]\d{2})\s*(?:zł|zl|pln)?\.?\s*[A-Z]?",
    )
    .expect("valid merged item regex");

    let mut last_entry_index: Option<usize> = None;

    for index in 0..lines.len() {
        let normalized = normalize_text(&lines[index].text);
        let prices = extract_prices(&price_re, &lines[index].text);
        let role = if merged_item_re.is_match(&lines[index].text) {
            "item"
        } else {
            classify_line(&normalized, &prices)
        };

        lines[index].role = role.to_string();
        lines[index].confidence = match role {
            "shop_name" => 0.75,
            "total" | "discount" | "item" => 0.85,
            _ => 0.25,
        };
        lines[index].selected = matches!(role, "shop_name" | "total" | "discount" | "item");

        match role {
            "shop_name" => {
                add_field(
                    &mut analysis.field_candidates,
                    "shop_name",
                    "Shop name",
                    &lines[index].text,
                    None,
                    0.75,
                    Some(index),
                    lines[index].bounding_box.clone(),
                    true,
                );
            }
            "total" => {
                if let Some(total) = prices.last() {
                    analysis.total_value = total.value.abs();
                    let total_box = box_for_text(&lines[index], &total.raw)
                        .unwrap_or_else(|| lines[index].bounding_box.clone());
                    add_field(
                        &mut analysis.field_candidates,
                        "total",
                        "Receipt total",
                        &total.raw,
                        Some(total.value.abs()),
                        0.9,
                        Some(index),
                        total_box,
                        true,
                    );
                }
            }
            "discount" => {
                if let Some(discount) = prices.last() {
                    let value = discount.value.abs();
                    analysis.total_discount += value;
                    let discount_box = box_for_text(&lines[index], &discount.raw)
                        .unwrap_or_else(|| lines[index].bounding_box.clone());
                    add_field(
                        &mut analysis.field_candidates,
                        "total_discount",
                        "Receipt discount",
                        &discount.raw,
                        Some(value),
                        0.82,
                        Some(index),
                        discount_box,
                        true,
                    );

                    if let Some(entry_index) = last_entry_index {
                        if normalized.contains("RABAT")
                            || normalized.contains("DISCOUNT")
                            || normalized.contains("PROMO")
                        {
                            analysis.entries[entry_index].entry_discount += value;
                        }
                    }
                }
            }
            "item" => {
                if merged_item_re.is_match(&lines[index].text) {
                    if let Some(entry_index) =
                        parse_merged_item_line(&lines[index], index, &merged_item_re, &mut analysis)
                    {
                        last_entry_index = Some(entry_index);
                        continue;
                    }
                }

                if let Some(price) = prices.last() {
                    let price_box = box_for_text(&lines[index], &price.raw)
                        .unwrap_or_else(|| lines[index].bounding_box.clone());
                    let quantity = qty_re
                        .captures(&lines[index].text)
                        .and_then(|captures| captures.get(1))
                        .and_then(|qty| qty.as_str().parse::<i64>().ok())
                        .unwrap_or(1);
                    let entry_cost = unit_price_re
                        .captures(&lines[index].text)
                        .and_then(|captures| captures.get(2))
                        .and_then(|unit| parse_price(unit.as_str()))
                        .unwrap_or(price.value.abs());
                    let entry_name = detect_item_name(&lines[index], &price.raw);

                    let draft = ReceiptEntryDraft {
                        draft_id: Uuid::new_v4().to_string(),
                        entry_name: entry_name.clone(),
                        entry_quantity: quantity,
                        entry_cost,
                        entry_discount: 0.0,
                    };
                    analysis.entries.push(draft);
                    last_entry_index = Some(analysis.entries.len() - 1);

                    add_field(
                        &mut analysis.field_candidates,
                        "item_name",
                        "Product name",
                        &entry_name,
                        None,
                        0.78,
                        Some(index),
                        name_box(&lines[index], &price.raw),
                        true,
                    );
                    add_field(
                        &mut analysis.field_candidates,
                        "quantity",
                        "Quantity",
                        &quantity.to_string(),
                        Some(quantity as f64),
                        0.7,
                        Some(index),
                        lines[index].bounding_box.clone(),
                        true,
                    );
                    add_field(
                        &mut analysis.field_candidates,
                        "item_price",
                        "Product price",
                        &price.raw,
                        Some(entry_cost),
                        0.86,
                        Some(index),
                        price_box,
                        true,
                    );
                }
            }
            _ => {}
        }
    }

    if analysis.total_value == 0.0 {
        analysis.total_value = analysis
            .entries
            .iter()
            .map(|entry| entry.entry_cost * entry.entry_quantity as f64 - entry.entry_discount)
            .sum::<f64>()
            .max(0.0);
    }

    analysis.lines = lines;
    analysis
}

fn group_into_lines(blocks: Vec<OcrBlock>, tolerance: f64) -> Vec<Vec<OcrBlock>> {
    let mut lines: Vec<Vec<OcrBlock>> = Vec::new();

    for block in blocks {
        let block_center = center_y(&block.bounding_box);
        if let Some(line) = lines.iter_mut().find(|line| {
            let line_center = median(line.iter().map(|word| center_y(&word.bounding_box)).collect());
            (block_center - line_center).abs() <= tolerance
        }) {
            line.push(block);
        } else {
            lines.push(vec![block]);
        }
    }

    lines
}

fn parse_merged_item_line(
    line: &ReceiptLineCandidate,
    line_index: usize,
    merged_item_re: &Regex,
    analysis: &mut ReceiptOcrAnalysis,
) -> Option<usize> {
    let formulas = merged_item_re
        .captures_iter(&line.text)
        .filter_map(|captures| {
            let full = captures.get(0)?;
            let quantity = captures
                .get(1)
                .and_then(|value| value.as_str().parse::<i64>().ok())
                .unwrap_or(1);
            let unit_cost = captures
                .get(2)
                .and_then(|value| parse_price(value.as_str()))
                .unwrap_or(0.0)
                .abs();
            let line_total = captures
                .get(3)
                .and_then(|value| parse_price(value.as_str()))
                .unwrap_or(unit_cost * quantity as f64)
                .abs();

            Some(MergedItemFormula {
                start: full.start(),
                raw: full.as_str().trim().to_string(),
                quantity,
                unit_cost,
                line_total,
            })
        })
        .collect::<Vec<_>>();

    if formulas.is_empty() {
        return None;
    }

    let names_text = line.text[..formulas[0].start].trim();
    let names = split_merged_names(names_text);
    let offset = names.len().saturating_sub(formulas.len());
    let mut last_entry_index = None;

    for (index, formula) in formulas.iter().enumerate() {
        let name = names
            .get(offset + index)
            .or_else(|| names.get(index))
            .cloned()
            .unwrap_or_else(|| "Receipt item".to_string());

        analysis.entries.push(ReceiptEntryDraft {
            draft_id: Uuid::new_v4().to_string(),
            entry_name: name.clone(),
            entry_quantity: formula.quantity,
            entry_cost: formula.unit_cost,
            entry_discount: 0.0,
        });
        last_entry_index = Some(analysis.entries.len() - 1);

        add_field(
            &mut analysis.field_candidates,
            "item_name",
            "Product name",
            &name,
            None,
            0.62,
            Some(line_index),
            line.bounding_box.clone(),
            true,
        );
        add_field(
            &mut analysis.field_candidates,
            "quantity",
            "Quantity",
            &formula.quantity.to_string(),
            Some(formula.quantity as f64),
            0.74,
            Some(line_index),
            line.bounding_box.clone(),
            true,
        );
        add_field(
            &mut analysis.field_candidates,
            "item_price",
            "Product price",
            &formula.raw,
            Some(formula.unit_cost),
            0.72,
            Some(line_index),
            line.bounding_box.clone(),
            true,
        );
        add_field(
            &mut analysis.field_candidates,
            "line_total",
            "Line total",
            &format!("{:.2}", formula.line_total),
            Some(formula.line_total),
            0.72,
            Some(line_index),
            line.bounding_box.clone(),
            true,
        );
    }

    last_entry_index
}

#[derive(Debug)]
struct MergedItemFormula {
    start: usize,
    raw: String,
    quantity: i64,
    unit_cost: f64,
    line_total: f64,
}

fn split_merged_names(text: &str) -> Vec<String> {
    let tax_marker_re = Regex::new(r"(?i)\s*[-–]\s*[A-Z]\b\s*").expect("valid tax marker regex");

    tax_marker_re
        .split(text)
        .map(clean_item_name)
        .filter(|name| !name.is_empty() && !is_noise_item_name(name))
        .collect()
}

fn is_noise_item_name(name: &str) -> bool {
    let normalized = normalize_text(name);
    normalized.starts_with("PARAGON")
        || normalized.contains("FISKALNY")
        || normalized.starts_with("NIP")
        || normalized.starts_with("NR ")
        || normalized.starts_with("OPUST")
        || normalized.starts_with("RABAT")
        || normalized.starts_with("DISCOUNT")
        || normalized.chars().filter(|character| character.is_alphabetic()).count() < 3
}

fn clean_item_name(text: &str) -> String {
    let cleaned = text
        .trim()
        .trim_matches('-')
        .trim()
        .to_string();
    let tax_suffix_re = Regex::new(r"(?i)\s+[-–]?\s*[A-Z]\s*$").expect("valid tax suffix regex");

    tax_suffix_re.replace(&cleaned, "").trim().to_string()
}

fn classify_line(normalized: &str, prices: &[DetectedPrice]) -> &'static str {
    if is_total_line(normalized) {
        "total"
    } else if is_discount_line(normalized) {
        "discount"
    } else if prices.last().is_some()
        && !contains_any(normalized, &["NIP", "VAT", "PTU", "TAX", "KASA", "PARAGON", "RECEIPT"])
    {
        "item"
    } else {
        "unknown"
    }
}

fn detect_shop_name(lines: &[ReceiptLineCandidate]) -> String {
    lines
        .iter()
        .take(6)
        .find(|line| {
            let normalized = normalize_text(&line.text);
            line.text.len() >= 3
                && !is_total_line(&normalized)
                && !is_discount_line(&normalized)
                && !contains_any(
                    &normalized,
                    &["NIP", "VAT", "PTU", "TAX", "KASA", "TEL", "DATA", "DATE"],
                )
        })
        .map(|line| line.text.clone())
        .unwrap_or_default()
}

fn detect_item_name(line: &ReceiptLineCandidate, price_text: &str) -> String {
    let mut words = line
        .words
        .iter()
        .filter(|word| word.text != price_text && parse_price(&word.text).is_none())
        .map(|word| word.text.as_str())
        .collect::<Vec<_>>();

    while words
        .first()
        .map(|word| word.eq_ignore_ascii_case("x") || word.parse::<i64>().is_ok())
        .unwrap_or(false)
    {
        words.remove(0);
    }

    let name = words.join(" ").trim().to_string();
    if name.is_empty() {
        line.text.replace(price_text, "").trim().to_string()
    } else {
        name
    }
}

fn is_total_line(text: &str) -> bool {
    contains_any(
        text,
        &[
            "TOTAL",
            "SUMA",
            "RAZEM",
            "DO ZAPLATY",
            "DO ZAPL",
            "AMOUNT DUE",
            "BALANCE",
        ],
    )
}

fn is_discount_line(text: &str) -> bool {
    contains_any(
        text,
        &[
            "RABAT",
            "DISCOUNT",
            "PROMO",
            "PROMOCJA",
            "SAVINGS",
            "OSZCZ",
            "OPUST",
            "OBNIZ",
            "OBNI",
        ],
    ) || text.trim_start().starts_with('-')
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

#[derive(Debug, Clone)]
struct DetectedPrice {
    raw: String,
    value: f64,
}

fn extract_prices(price_re: &Regex, text: &str) -> Vec<DetectedPrice> {
    price_re
        .find_iter(text)
        .filter_map(|matched| {
            parse_price(matched.as_str()).map(|value| DetectedPrice {
                raw: matched.as_str().to_string(),
                value,
            })
        })
        .collect()
}

fn parse_price(text: &str) -> Option<f64> {
    let normalized = text
        .replace(',', ".")
        .replace('O', "0")
        .replace('o', "0")
        .replace(' ', "");
    let stripped: String = normalized
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == '.' || *character == '-')
        .collect();

    if stripped.contains('.') {
        stripped.parse::<f64>().ok()
    } else {
        None
    }
}

fn normalize_ocr_token(text: &str) -> String {
    text.replace("×", "x")
        .replace('；', ";")
        .replace(" ,", ",")
        .replace(" .", ".")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn add_field(
    fields: &mut Vec<ReceiptFieldCandidate>,
    role: &str,
    label: &str,
    value_text: &str,
    numeric_value: Option<f64>,
    confidence: f64,
    line_index: Option<usize>,
    bounding_box: BoundingBox,
    selected: bool,
) {
    fields.push(ReceiptFieldCandidate {
        field_id: Uuid::new_v4().to_string(),
        role: role.to_string(),
        label: label.to_string(),
        value_text: value_text.to_string(),
        numeric_value,
        confidence,
        line_index,
        bounding_box,
        selected,
    });
}

fn normalize_text(text: &str) -> String {
    text.to_uppercase()
        .replace('Ą', "A")
        .replace('Ć', "C")
        .replace('Ę', "E")
        .replace('Ł', "L")
        .replace('Ń', "N")
        .replace('Ó', "O")
        .replace('Ś', "S")
        .replace('Ź', "Z")
        .replace('Ż', "Z")
}

fn center_y(box_: &BoundingBox) -> f64 {
    box_.y + box_.height / 2.0
}

fn union_boxes<'a>(boxes: impl Iterator<Item = &'a BoundingBox>) -> BoundingBox {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for box_ in boxes {
        min_x = min_x.min(box_.x);
        min_y = min_y.min(box_.y);
        max_x = max_x.max(box_.x + box_.width);
        max_y = max_y.max(box_.y + box_.height);
    }

    if min_x == f64::MAX {
        BoundingBox::default()
    } else {
        BoundingBox {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

fn name_box(line: &ReceiptLineCandidate, price_text: &str) -> BoundingBox {
    let boxes = line
        .words
        .iter()
        .filter(|word| word.text != price_text && parse_price(&word.text).is_none())
        .map(|word| &word.bounding_box);
    union_boxes(boxes)
}

fn box_for_text(line: &ReceiptLineCandidate, text: &str) -> Option<BoundingBox> {
    line.words
        .iter()
        .find(|word| word.text == text)
        .map(|word| word.bounding_box.clone())
}

fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values[values.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_merged_receipt_item_columns() {
        let analysis = analyze_receipt_layout(vec![OcrBlock {
            text: "NAP07 COCA 850ML-A CHRUPKI CURLY 120g-C BATON KINDER BUENO-C 2szt. x6,50 13,00C 1szt. x5,40 5,40C".to_string(),
            bounding_box: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 600.0,
                height: 24.0,
            },
        }]);

        assert_eq!(analysis.entries.len(), 2);
        assert_eq!(analysis.entries[0].entry_name, "CHRUPKI CURLY 120g");
        assert_eq!(analysis.entries[0].entry_quantity, 2);
        assert_eq!(analysis.entries[0].entry_cost, 6.50);
        assert_eq!(analysis.entries[1].entry_name, "BATON KINDER BUENO");
        assert_eq!(analysis.entries[1].entry_quantity, 1);
        assert_eq!(analysis.entries[1].entry_cost, 5.40);
    }

    #[test]
    fn parses_zabka_multi_item_blob_with_discount_name() {
        let analysis = analyze_receipt_layout(vec![OcrBlock {
            text: "BATON KINDER BUENO-C CHRUPKI CURLY 120g-C NAPOJ COCA 850ml-A OPUST NAPOJ COCA 850ml-A 1szt. x5,40 5,40C 2szt. x6,50 13,00C 2szt. x7,99 15,98A -2,78A SUMA PLN 31,60".to_string(),
            bounding_box: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 900.0,
                height: 28.0,
            },
        }]);

        assert_eq!(analysis.entries.len(), 3);
        assert_eq!(analysis.entries[0].entry_name, "BATON KINDER BUENO");
        assert_eq!(analysis.entries[0].entry_quantity, 1);
        assert_eq!(analysis.entries[0].entry_cost, 5.40);
        assert_eq!(analysis.entries[1].entry_name, "CHRUPKI CURLY 120g");
        assert_eq!(analysis.entries[1].entry_quantity, 2);
        assert_eq!(analysis.entries[1].entry_cost, 6.50);
        assert_eq!(analysis.entries[2].entry_name, "NAPOJ COCA 850ml");
        assert_eq!(analysis.entries[2].entry_quantity, 2);
        assert_eq!(analysis.entries[2].entry_cost, 7.99);
    }

    #[test]
    fn parses_zabka_single_item_star_price_line() {
        let analysis = analyze_receipt_layout(vec![OcrBlock {
            text: "NAPOJ BURN 250ML A 1 * 3,99 zł. 3,99 A".to_string(),
            bounding_box: BoundingBox {
                x: 0.0,
                y: 0.0,
                width: 450.0,
                height: 22.0,
            },
        }]);

        assert_eq!(analysis.entries.len(), 1);
        assert_eq!(analysis.entries[0].entry_name, "NAPOJ BURN 250ML");
        assert_eq!(analysis.entries[0].entry_quantity, 1);
        assert_eq!(analysis.entries[0].entry_cost, 3.99);
    }
}
