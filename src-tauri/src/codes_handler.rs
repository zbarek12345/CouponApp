pub mod codes_handler {

    use std::result;

use base64::Engine; // <-- required to bring .decode() into scope
    use rxing::{
        common::HybridBinarizer,
        BinaryBitmap,
        MultiFormatReader,
        BarcodeFormat,
        DecodeHints,
        Reader, // <-- needed to call .decode() on MultiFormatReader
    };
    use image::DynamicImage;

    use crate::models::CodeCandidate;

    /// Decodes a base64-encoded image string into an in-memory image buffer
    async fn base64_to_image(
        image_str: String,
    ) -> Result<DynamicImage, Box<dyn std::error::Error + Send + Sync>> {
        let image_bytes = base64::engine::general_purpose::STANDARD
            .decode(image_str) // decode_vec is gone; .decode() returns Vec<u8> directly
            .map_err(|e| format!("Error decoding base64: {}", e))?;

        let img = image::load_from_memory(&image_bytes)
            .map_err(|e| format!("Error loading image: {}", e))?;

        Ok(img)
    }

    pub async fn read_image_code(
        image_str: String,
    ) -> Result<Vec<CodeCandidate>, Box<dyn std::error::Error + Send + Sync>> {
        let dynamic_img = base64_to_image(image_str).await?;

        let result_vec   = rxing::helpers::detect_multiple_in_image(dynamic_img);
        
        println!("Running scan of image\n");
        match result_vec {
            Ok(rxing_result ) => {
                let mut candidates :Vec<CodeCandidate> = Vec::new();
                
                println!("Scan successful, codes identified");

                let mut id : usize = 0;
                for res in rxing_result{
                    candidates.push(
                        CodeCandidate{
                            index : id,
                            code_value : res.getText().to_string(),
                            code_type : res.getBarcodeFormat().to_string(),
                            confidence : 1.0,
                            bounds : res.getPoints().to_vec()
                        }
                    );
                    id+=1;

                    let print_points = res.getPoints();
                    println!("points for code : {}", id);
                    for point in print_points {
                        println!("({}, {})",point.x,point.y)
                    }
                }

                Ok(candidates)
            }
            Err(_) => Ok(vec![]),
        }
    }

    use rxing::{EncodeHints, MultiFormatWriter, Writer};
    use rxing::common::BitMatrix;
    use image::{ImageBuffer, Luma, ImageEncoder, ImageFormat};
    use base64::{engine::general_purpose};

    use std::io::Cursor;

    pub async fn render_code(
    code_type: BarcodeFormat,
    code_value: String,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        
        let width = 400u32;
        let height = match code_type {
            BarcodeFormat::QR_CODE | BarcodeFormat::DATA_MATRIX | BarcodeFormat::AZTEC => 400,
            _ => 200, // 1D barcodes look better taller
        };

        // Optional encoding hints
        let mut hints = EncodeHints::default();
        // hints.set_margin(10);        // uncomment if you want margin control

        let writer = MultiFormatWriter::default();

        let bit_matrix: BitMatrix = writer.encode_with_hints(
            &code_value,
            &code_type,
            width as i32,
            height as i32,
            &hints,
        )?;

        // Convert BitMatrix to image
        let img = bit_matrix_to_image(&bit_matrix);

        // Encode image to PNG bytes
        let mut png_bytes = Vec::new();
        {
            let mut cursor = Cursor::new(&mut png_bytes);
            img.write_to(&mut cursor, ImageFormat::Png)?;
        }

        // Convert to Base64
        let base64_str = general_purpose::STANDARD.encode(&png_bytes);

        Ok(base64_str)
    }

// Helper function: BitMatrix → ImageBuffer
    fn bit_matrix_to_image(matrix: &BitMatrix) -> ImageBuffer<Luma<u8>, Vec<u8>> {
        let width = matrix.getWidth() as u32;
        let height = matrix.getHeight() as u32;

        ImageBuffer::from_fn(width, height, |x, y| {
            if matrix.get(x , y) {
                Luma([0u8])      // Black (barcode)
            } else {
                Luma([255u8])    // White (background)
            }
        })
    }
}