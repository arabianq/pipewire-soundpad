use crate::utils::config::get_config_path;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::PathBuf};
use std::sync::Mutex;

static GUI_CONFIG_SAVE_LOCK: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    pub default_input_name: Option<String>,
    pub default_volume: Option<f32>,
}

impl DaemonConfig {
    pub fn save_to_file(&self) -> Result<(), Box<dyn Error>> {
        let config_path = get_config_path()?.join("daemon.json");

        if let Some(config_dir) = config_path.parent() {
            if !config_path.exists() {
                fs::create_dir_all(config_dir)?;
            }
        }

        let config_json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, config_json.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file() -> Result<DaemonConfig, Box<dyn Error>> {
        let config_path = get_config_path()?.join("daemon.json");
        let bytes = fs::read(config_path)?;
        match serde_json::from_slice::<DaemonConfig>(&bytes) {
            Ok(config) => Ok(config),
            Err(_) => Ok(DaemonConfig::default()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GuiConfig {
    pub scale_factor: f32,
    pub left_panel_width: f32,

    pub save_volume: bool,
    pub save_input: bool,
    pub save_scale_factor: bool,
    pub pause_on_exit: bool,

    pub dirs: Vec<PathBuf>,
}

impl Default for GuiConfig {
    fn default() -> Self {
        GuiConfig {
            scale_factor: 1.0,
            left_panel_width: 280.0,

            save_volume: false,
            save_input: false,
            save_scale_factor: false,
            pause_on_exit: false,

            dirs: vec![],
        }
    }
}

impl GuiConfig {
    pub fn save_to_file(&mut self) {
        // Do not save scale factor if user does not want to
        if !self.save_scale_factor {
            self.scale_factor = 1.0;
        }

        let self_clone = self.clone();

        tokio::task::spawn_blocking(move || {
            let _guard = GUI_CONFIG_SAVE_LOCK
                .get_or_init(|| Mutex::new(()))
                .lock()
                .unwrap_or_else(|e| e.into_inner());

            let config_path = match get_config_path() {
                Ok(path) => path.join("gui.json"),
                Err(e) => {
                    eprintln!("Failed to get config path: {}", e);
                    return;
                }
            };

            if let Some(config_dir) = config_path.parent() {
                if !config_path.exists() {
                    if let Err(e) = fs::create_dir_all(config_dir) {
                        eprintln!("Failed to create config directory: {}", e);
                    }
                }
            }

            match serde_json::to_string_pretty(&self_clone) {
                Ok(config_json) => {
                    if let Err(e) = fs::write(&config_path, config_json.as_bytes()) {
                        eprintln!("Failed to write gui.json: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to serialize gui config: {}", e);
                }
            }
        });
    }

    pub fn load_from_file() -> Result<GuiConfig, Box<dyn Error>> {
        let config_path = get_config_path()?.join("gui.json");
        let bytes = fs::read(config_path)?;
        match serde_json::from_slice::<GuiConfig>(&bytes) {
            Ok(config) => Ok(config),
            Err(_) => Ok(GuiConfig::default()),
        }
    }
}
