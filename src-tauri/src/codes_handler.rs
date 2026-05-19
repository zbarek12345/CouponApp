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
}