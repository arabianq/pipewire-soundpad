mod gui;

use anyhow::{Context, Result, anyhow};
use pwsp_lib::utils::gui::ensure_pwsp_audio_dir;
use rust_i18n::i18n;
use std::{
    env,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

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
        .ok_or_else(|| anyhow!("URI does not containt an expected prefix: {}", prefix))?;

    let file_name_encoded = match target_url.split('/').next_back() {
        Some(path) => path.to_string(),
        None => {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went back")
                .as_nanos();
            format!("downloaded_audio_{}.mp3", id)
        }
    };

    let file_name = percent_encoding::percent_decode_str(&file_name_encoded.clone())
        .decode_utf8()
        .unwrap_or_else(|_| file_name_encoded.into())
        .into_owned();

    let normalized_file_name = file_name.replace('\\', "/");
    let sanitized_file_name = Path::new(&normalized_file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("downloaded_audio.mp3");

    let save_path = ensure_pwsp_audio_dir().join(sanitized_file_name);

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
