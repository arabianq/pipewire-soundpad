use crate::utils::config::get_config_path;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, error::Error, fs, path::PathBuf};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub default_input_id: Option<u32>,
    pub default_volume: Option<f32>,
}

impl DaemonConfig {
    pub fn save_to_file(&self) -> Result<(), Box<dyn Error>> {
        let config_path = get_config_path()?.join("daemon.json");
        let config_dir = config_path.parent().unwrap();

        if !config_path.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let config_json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, config_json.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file() -> Result<DaemonConfig, Box<dyn Error>> {
        let config_path = get_config_path()?.join("daemon.json");
        let bytes = fs::read(config_path)?;
        Ok(serde_json::from_slice::<DaemonConfig>(&bytes)?)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    pub scale_factor: f32,

    pub save_volume: bool,
    pub save_input: bool,
    pub save_scale_factor: bool,

    pub dirs: HashSet<PathBuf>,
}

impl Default for GuiConfig {
    fn default() -> Self {
        GuiConfig {
            scale_factor: 1.0,

            save_volume: false,
            save_input: false,
            save_scale_factor: false,

            dirs: HashSet::default(),
        }
    }
}

impl GuiConfig {
    pub fn save_to_file(&mut self) -> Result<(), Box<dyn Error>> {
        let config_path = get_config_path()?.join("gui.json");
        let config_dir = config_path.parent().unwrap();

        if !config_path.exists() {
            fs::create_dir_all(config_dir)?;
        }

        // Do not save scale factor if user does not want to
        if !self.save_scale_factor {
            self.scale_factor = 1.0;
        }

        let config_json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, config_json.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file() -> Result<GuiConfig, Box<dyn Error>> {
        let config_path = get_config_path()?.join("gui.json");
        let bytes = fs::read(config_path)?;
        Ok(serde_json::from_slice::<GuiConfig>(&bytes)?)
    }
}
