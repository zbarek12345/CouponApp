mod commands;
mod models;
mod ocr_handler;

#[tokio::main]
async fn main() {
    commands::run().await;
}
