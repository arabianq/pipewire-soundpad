use std::{error::Error, path::PathBuf};

pub fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    let config_path = dirs::config_dir().expect("Failed to obtain config dir");
    Ok(config_path.join("pwsp"))
}
