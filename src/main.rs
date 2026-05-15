mod gui;

use anyhow::Result;
use rust_i18n::i18n;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() -> Result<()> {
    let locale = sys_locale::get_locale().unwrap_or(String::from("en-US"));
    rust_i18n::set_locale(&locale);

    gui::run().await
}
