mod gui;

use anyhow::{Context, Result};
use pwsp::utils::gui::ensure_pwsp_audio_dir;
use rust_i18n::i18n;
use std::{env, path::PathBuf};

i18n!("locales", fallback = "en");

#[tokio::main]
async fn main() -> Result<()> {
    let locale = sys_locale::get_locale().unwrap_or(String::from("en-US"));
    rust_i18n::set_locale(&locale);

    let args = env::args().skip(1).collect::<Vec<String>>();

    if let Some(uri) = args.first() {
        match download_audio_from_url(uri).await {
            Ok(path) => println!("Successfully downloaded to: {:?}", path),
            Err(e) => eprintln!("Error downloading file: {}", e),
        }
    } else {
        gui::run().await?;
    }

    Ok(())
}

async fn download_audio_from_url(uri: &str) -> Result<PathBuf> {
    let prefix = "soundpad://sound/url/";

    let target_url = uri
        .strip_prefix(prefix)
        .ok_or_else(|| anyhow::anyhow!("URI does not containt an expected prefix: {}", prefix))?;

    let file_name_encoded = target_url
        .split('/')
        .next_back()
        .unwrap_or("downloaded_audio.mp3");

    let file_name = percent_encoding::percent_decode_str(file_name_encoded)
        .decode_utf8()
        .unwrap_or_else(|_| file_name_encoded.into())
        .into_owned();

    let save_path = ensure_pwsp_audio_dir().join(file_name);

    let response = reqwest::get(target_url)
        .await?
        .error_for_status()
        .context("Failed to fetch file")?;

    let bytes = response.bytes().await?;

    tokio::fs::write(&save_path, bytes)
        .await
        .context("Failed to save file to disk")?;

    Ok(save_path)
}
