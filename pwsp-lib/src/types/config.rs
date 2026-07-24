use crate::{
    types::socket::Request,
    utils::{config::get_config_path, gui::ensure_pwsp_audio_dir},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    pub default_input_name: Option<String>,
    pub default_volume: Option<f32>,
    pub default_volume_multiplier: Option<f32>,
}

impl DaemonConfig {
    pub fn save_to_file(&self) -> Result<()> {
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

    pub fn load_from_file() -> Result<DaemonConfig> {
        let config_path = get_config_path()?.join("daemon.json");
        let bytes = fs::read(config_path)?;
        match serde_json::from_slice::<DaemonConfig>(&bytes) {
            Ok(config) => Ok(config),
            Err(_) => Ok(DaemonConfig::default()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PreferredTheme {
    System,
    Light,
    Dark,
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    AlphabeticalAsc,
    AlphabeticalDesc,
    DateModifiedNewest,
    DateModifiedOldest,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DirSettings {
    pub sort_order: SortOrder,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GuiConfig {
    pub scale_factor: f32,
    pub left_panel_width: f32,

    pub save_volume: bool,
    pub save_volume_multiplier: bool,
    pub save_input: bool,
    pub save_scale_factor: bool,
    pub pause_on_exit: bool,

    pub dirs: Vec<PathBuf>,
    pub dirs_settings: HashMap<PathBuf, DirSettings>,

    pub preferred_theme: PreferredTheme,
}

impl SortOrder {
    pub fn compare(&self, a: &Path, b: &Path) -> Ordering {
        match self {
            SortOrder::AlphabeticalAsc => a.cmp(b),
            SortOrder::AlphabeticalDesc => b.cmp(a),
            SortOrder::DateModifiedNewest => {
                let a_time = fs::metadata(a)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let b_time = fs::metadata(b)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                b_time.cmp(&a_time)
            }
            SortOrder::DateModifiedOldest => {
                let a_time = fs::metadata(a)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                let b_time = fs::metadata(b)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                a_time.cmp(&b_time)
            }
        }
    }
}

impl Default for GuiConfig {
    fn default() -> Self {
        GuiConfig {
            scale_factor: 1.0,
            left_panel_width: 280.0,

            save_volume: false,
            save_volume_multiplier: false,
            save_input: false,
            save_scale_factor: false,
            pause_on_exit: false,

            dirs: vec![ensure_pwsp_audio_dir().unwrap()],

            preferred_theme: PreferredTheme::System,
            dirs_settings: HashMap::new(),
        }
    }
}

impl GuiConfig {
    pub fn get_sort_order(&self, path: &Path) -> SortOrder {
        let mut current = Some(path);
        while let Some(p) = current {
            if let Some(settings) = self.dirs_settings.get(p) {
                return settings.sort_order;
            }
            current = p.parent();
        }
        SortOrder::default()
    }

    pub fn save_to_file(&mut self) -> Result<()> {
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

    pub fn load_from_file() -> Result<GuiConfig> {
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
    pub fn config_path() -> Result<PathBuf> {
        Ok(get_config_path()?.join("hotkeys.json"))
    }

    pub fn load() -> Result<HotkeyConfig> {
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

    pub fn save(&self) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gui_config_default() {
        let config = GuiConfig::default();
        assert_eq!(config.scale_factor, 1.0);
        assert_eq!(config.left_panel_width, 280.0);
        assert!(!config.save_volume);
        assert_eq!(config.preferred_theme, PreferredTheme::System);
    }

    #[test]
    fn test_hotkey_config_operations() {
        let mut config = HotkeyConfig::default();
        assert!(config.slots.is_empty());

        let req = Request::ping();
        config.set_slot("slot1".to_string(), req.clone());
        assert_eq!(config.slots.len(), 1);
        assert_eq!(config.slots[0].slot, "slot1");
        assert_eq!(config.slots[0].action, req);
        assert!(config.slots[0].key_chord.is_none());

        // Test find_slot
        let found = config.find_slot("slot1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().slot, "slot1");

        // Test set_key_chord
        let updated = config.set_key_chord("slot1", Some("Ctrl+A".to_string()));
        assert!(updated);
        assert_eq!(config.slots[0].key_chord.as_deref(), Some("Ctrl+A"));

        // Test set_key_chord for non-existent slot
        let updated_non_existent = config.set_key_chord("slot2", Some("Ctrl+B".to_string()));
        assert!(!updated_non_existent);

        // Test find_slot_mut
        let found_mut = config.find_slot_mut("slot1");
        assert!(found_mut.is_some());
        found_mut.unwrap().key_chord = Some("Ctrl+B".to_string());
        assert_eq!(config.slots[0].key_chord.as_deref(), Some("Ctrl+B"));

        // Test slots_for_chord
        let slots = config.slots_for_chord("Ctrl+B");
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].slot, "slot1");

        let empty_slots = config.slots_for_chord("Ctrl+A");
        assert!(empty_slots.is_empty());

        // Test remove_slot
        let removed = config.remove_slot("slot1");
        assert!(removed);
        assert!(config.slots.is_empty());

        let removed_non_existent = config.remove_slot("slot1");
        assert!(!removed_non_existent);
    }

    #[test]
    fn test_hotkey_config_conflicts() {
        let mut config = HotkeyConfig::default();
        config.set_slot("slot1".to_string(), Request::ping());
        config.set_slot("slot2".to_string(), Request::ping());
        config.set_slot("slot3".to_string(), Request::ping());

        config.set_key_chord("slot1", Some("Ctrl+A".to_string()));
        config.set_key_chord("slot2", Some("Ctrl+A".to_string())); // Conflict with slot1
        config.set_key_chord("slot3", Some("Ctrl+B".to_string()));

        let conflicts = config.find_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts.contains(&("slot1", "slot2")) || conflicts.contains(&("slot2", "slot1")));
    }
}
