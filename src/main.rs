mod gui;

use rust_i18n::i18n;
use std::error::Error;

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let locale = sys_locale::get_locale().unwrap_or(String::from("en-US"));
    rust_i18n::set_locale(&locale);

    gui::run().await
}
