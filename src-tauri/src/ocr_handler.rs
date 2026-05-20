use serde::{Deserialize, Serialize};

/// Struktura reprezentująca współrzędne pojedynczego wyrazu z OCR
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Definicja pojedynczego słowa przechwyconego przez wtyczkę AI OCR na froncie
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OcrBlock {
    pub text: String,
    pub bounding_box: BoundingBox,
}

/// Struktura zwrotna, z gotowym już "posklejanym" użytecznym obiektem paragonu
#[derive(Debug, Serialize)]
pub struct ParsedReceipt {
    pub shop_name: String,
    pub total_value: f64,
    pub total_discount: f64,
    pub entries: Vec<ParsedEntry>,
}

#[derive(Debug, Serialize)]
pub struct ParsedEntry {
    pub entry_name: String,
    pub entry_quantity: i64,
    pub entry_cost: f64,
    pub entry_discount: f64,
}

/// Główna funkcja analityczna (Heurystyka) 
/// Otrzymuje same słowa z wymiarami z frontu i buduje z tego użyteczny paragon
pub fn parse_receipt_layout(mut blocks: Vec<OcrBlock>) -> ParsedReceipt {
    if blocks.is_empty() {
        return ParsedReceipt {
            shop_name: "Nieznany Sklep".to_string(),
            total_value: 0.0,
            total_discount: 0.0,
            entries: vec![],
        };
    }

    // 1. Obliczenie środków wys. (oś Y) każdego bloku i sortowanie ich z góry na dół
    blocks.sort_by(|a, b| {
        let center_a = a.bounding_box.y + (a.bounding_box.height / 2.0);
        let center_b = b.bounding_box.y + (b.bounding_box.height / 2.0);
        center_a.partial_cmp(&center_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // 2. Grupowanie słów w równe "linijki" tekstowe wg ich wysokości
    let mut lines: Vec<Vec<OcrBlock>> = Vec::new();
    let mut current_line: Vec<OcrBlock> = vec![blocks[0].clone()];
    
    // Przewidujemy 60% wysokości pierwszego słowa jako tolerancję "że to wciąż ta sama linijka"
    let tolerance = blocks[0].bounding_box.height * 0.6; 

    for block in blocks.into_iter().skip(1) {
        let center = block.bounding_box.y + (block.bounding_box.height / 2.0);
        let current_line_center = current_line[0].bounding_box.y + (current_line[0].bounding_box.height / 2.0);
        
        if (center - current_line_center).abs() < tolerance {
            current_line.push(block);
        } else {
            // Skoro to już inna, nowa linijka, posortujmy w tej starej słowa od lewej do prawej
            current_line.sort_by(|a, b| a.bounding_box.x.partial_cmp(&b.bounding_box.x).unwrap());
            lines.push(current_line);
            current_line = vec![block];
        }
    }
    current_line.sort_by(|a, b| a.bounding_box.x.partial_cmp(&b.bounding_box.x).unwrap());
    lines.push(current_line);

    // 3. Budowa logiki
    let mut parsed = ParsedReceipt {
        shop_name: String::new(),
        total_value: 0.0,
        total_discount: 0.0,
        entries: Vec::new(),
    };

    // Roboczo uznajmy, że pierwsza czytelna górna linijka to zawsze tytuł sklepu
    if let Some(first_line) = lines.first() {
        parsed.shop_name = first_line.iter().map(|b| b.text.clone()).collect::<Vec<_>>().join(" ");
    } else {
        parsed.shop_name = "Nieznany Sklep".to_string();
    }

    // Sprawdzamy resztę wierszy pod kątem towarów
    for i in 1..lines.len() {
        let line_words: Vec<String> = lines[i].iter().map(|b| b.text.clone()).collect();
        if line_words.is_empty() { continue; }
        
        let full_text = line_words.join(" ").to_uppercase();

        // Wykrywamy podsumowanie wiersza
        if full_text.contains("SUMA") || full_text.contains("TOTAL") || full_text.contains("RAZEM") {
            if let Some(last_word) = line_words.last() {
                if let Some(price) = parse_price(last_word) {
                    parsed.total_value = price;
                }
            }
            continue;
        }

        // KWALIFIKACJA PRODUKTU:
        // Jeśli w linijce jest więcej niż 1 wyraz, a ostatnim z nich jest kwota, uznajemy to za produkt z ceną.
        if line_words.len() > 1 {
            let last_word = line_words.last().unwrap();
            if let Some(price) = parse_price(last_word) {
                let item_name = line_words[..line_words.len() - 1].join(" ");
                
                // Ignorujemy śmieci fiskalne (ew. dodasz tu resztę wyjątków np. "PTU", "KASA")
                if !item_name.to_uppercase().contains("NIP") && !item_name.to_uppercase().contains("PODATEK") {
                    parsed.entries.push(ParsedEntry {
                        entry_name: item_name,
                        entry_quantity: 1, // Algorytm do rozpoznawania "2 x 5,00" będzie kolejnym krokiem
                        entry_cost: price,
                        entry_discount: 0.0, // Tak samo system rabatów
                    });
                }
            }
        }
    }

    // Bezpieczny upadek (Fallback), jeśli system fiskalny miał na dole napis np. "DO ZAPŁATY" którego nie wychwyciliśmy - sumujemy sami po kosztach zebranych produktów
    if parsed.total_value == 0.0 {
        parsed.total_value = parsed.entries.iter().map(|e| e.entry_cost).sum();
    }

    parsed
}

/// Szybki detektor wyciągający pieniądze z napisu (np. "12,99", "12.99" albo wadliwe "5,OO")
fn parse_price(text: &str) -> Option<f64> {
    let normalized = text
        .replace(',', ".")  // Zamiana przecinka na kropkę floatową
        .replace('O', "0")  // Naprawa najpopularniejszych
        .replace('o', "0"); // pomyłek OCR z zerami
    
    // Ignoruj litery typu PLN pod koniec
    let stripped: String = normalized.chars().filter(|c| c.is_numeric() || *c == '.').collect();
    
    if !stripped.is_empty() && stripped.contains('.') {
        stripped.parse::<f64>().ok()
    } else {
        None
    }
}