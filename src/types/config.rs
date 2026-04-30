use crate::{types::socket::Request, utils::config::get_config_path};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fs, path::PathBuf};

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    pub default_input_name: Option<String>,
    pub default_volume: Option<f32>,
}

impl DaemonConfig {
    pub fn save_to_file(&self) -> Result<(), Box<dyn Error>> {
        let config_path = get_config_path()?.join("daemon.json");

        if let Some(config_dir) = config_path.parent()
            && !config_path.exists()
        {
            fs::create_dir_all(config_dir)?;
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

    pub show_index_column: bool,
    pub show_hotkey_column: bool,
    pub show_modified_column: bool,

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

            show_index_column: true,
            show_hotkey_column: true,
            show_modified_column: true,

            dirs: vec![],
        }
    }
}

impl GuiConfig {
    pub fn save_to_file(&mut self) -> Result<(), Box<dyn Error>> {
        let config_path = get_config_path()?.join("gui.json");

        if let Some(config_dir) = config_path.parent()
            && !config_path.exists()
        {
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
        match serde_json::from_slice::<GuiConfig>(&bytes) {
            Ok(config) => Ok(config),
            Err(_) => Ok(GuiConfig::default()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HotkeySlot {
    pub slot: String,
    pub action: Request,
    pub key_chord: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct HotkeyConfig {
    #[serde(default)]
    pub slots: Vec<HotkeySlot>,
}

impl HotkeyConfig {
    pub fn config_path() -> Result<PathBuf, Box<dyn Error>> {
        Ok(get_config_path()?.join("hotkeys.json"))
    }

    pub fn load() -> Result<HotkeyConfig, Box<dyn Error>> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(HotkeyConfig::default());
        }
        let bytes = fs::read(&path)?;
        match serde_json::from_slice::<HotkeyConfig>(&bytes) {
            Ok(config) => Ok(config),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let path = Self::config_path()?;
        if let Some(dir) = path.parent()
            && !dir.exists()
        {
            fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json.as_bytes())?;
        Ok(())
    }

    pub fn find_slot(&self, slot: &str) -> Option<&HotkeySlot> {
        self.slots.iter().find(|s| s.slot == slot)
    }

    pub fn find_slot_mut(&mut self, slot: &str) -> Option<&mut HotkeySlot> {
        self.slots.iter_mut().find(|s| s.slot == slot)
    }

    pub fn set_slot(&mut self, slot: String, action: Request) {
        if let Some(existing) = self.find_slot_mut(&slot) {
            existing.action = action;
        } else {
            self.slots.push(HotkeySlot {
                slot,
                action,
                key_chord: None,
            });
        }
    }

    pub fn set_key_chord(&mut self, slot: &str, key_chord: Option<String>) -> bool {
        if let Some(existing) = self.find_slot_mut(slot) {
            existing.key_chord = key_chord;
            true
        } else {
            false
        }
    }

    pub fn remove_slot(&mut self, slot: &str) -> bool {
        let len = self.slots.len();
        self.slots.retain(|s| s.slot != slot);
        self.slots.len() != len
    }

    /// Returns pairs of slot names that share the same key chord.
    pub fn find_conflicts(&self) -> Vec<(&str, &str)> {
        let mut conflicts = vec![];
        let mut chord_map: HashMap<&str, Vec<&str>> = HashMap::new();

        for s in &self.slots {
            if let Some(chord) = &s.key_chord {
                chord_map.entry(chord.as_str()).or_default().push(&s.slot);
            }
        }

        for slots in chord_map.values() {
            if slots.len() > 1 {
                for i in 0..slots.len() {
                    for j in (i + 1)..slots.len() {
                        conflicts.push((slots[i], slots[j]));
                    }
                }
            }
        }

        conflicts
    }

    /// Find which slot(s) have the given key chord.
    pub fn slots_for_chord(&self, chord: &str) -> Vec<&HotkeySlot> {
        self.slots
            .iter()
            .filter(|s| s.key_chord.as_deref() == Some(chord))
            .collect()
    }
}
