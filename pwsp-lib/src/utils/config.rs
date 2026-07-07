use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn get_config_path() -> Result<PathBuf> {
    let config_path = dirs::config_dir().context("Failed to obtain config dir")?;
    Ok(config_path.join("pwsp"))
}
