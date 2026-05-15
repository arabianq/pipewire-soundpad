mod gui;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    gui::run().await
}
