mod commands;
mod models;

#[tokio::main]
async fn main() {
    commands::run().await;
}