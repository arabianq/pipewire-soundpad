use crate::utils::config::get_config_path;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fs, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HotkeySlot {
    pub slot: String,
    pub sound_path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
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
            Err(_) => Ok(HotkeyConfig::default()),
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

    pub fn set_slot(&mut self, slot: String, sound_path: PathBuf) {
        if let Some(existing) = self.find_slot_mut(&slot) {
            existing.sound_path = sound_path;
        } else {
            self.slots.push(HotkeySlot {
                slot,
                sound_path,
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
    pub fn find_conflicts(&self) -> Vec<(String, String)> {
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
                        conflicts.push((slots[i].to_string(), slots[j].to_string()));
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
    fn test_set_and_find_slot() {
        let mut config = HotkeyConfig::default();
        config.set_slot("1".into(), PathBuf::from("/sounds/airhorn.mp3"));
        assert!(config.find_slot("1").is_some());
        assert_eq!(
            config.find_slot("1").unwrap().sound_path,
            PathBuf::from("/sounds/airhorn.mp3")
        );
    }

    #[test]
    fn test_overwrite_slot() {
        let mut config = HotkeyConfig::default();
        config.set_slot("1".into(), PathBuf::from("/sounds/a.mp3"));
        config.set_slot("1".into(), PathBuf::from("/sounds/b.mp3"));
        assert_eq!(config.slots.len(), 1);
        assert_eq!(
            config.find_slot("1").unwrap().sound_path,
            PathBuf::from("/sounds/b.mp3")
        );
    }

    #[test]
    fn test_remove_slot() {
        let mut config = HotkeyConfig::default();
        config.set_slot("1".into(), PathBuf::from("/sounds/a.mp3"));
        assert!(config.remove_slot("1"));
        assert!(!config.remove_slot("1"));
        assert!(config.slots.is_empty());
    }

    #[test]
    fn test_conflict_detection() {
        let mut config = HotkeyConfig::default();
        config.set_slot("1".into(), PathBuf::from("/sounds/a.mp3"));
        config.set_slot("2".into(), PathBuf::from("/sounds/b.mp3"));
        config.set_key_chord("1", Some("Ctrl+1".into()));
        config.set_key_chord("2", Some("Ctrl+1".into()));

        let conflicts = config.find_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], ("1".to_string(), "2".to_string()));
    }

    #[test]
    fn test_no_conflicts() {
        let mut config = HotkeyConfig::default();
        config.set_slot("1".into(), PathBuf::from("/sounds/a.mp3"));
        config.set_slot("2".into(), PathBuf::from("/sounds/b.mp3"));
        config.set_key_chord("1", Some("Ctrl+1".into()));
        config.set_key_chord("2", Some("Ctrl+2".into()));

        assert!(config.find_conflicts().is_empty());
    }

    #[test]
    fn test_slots_for_chord() {
        let mut config = HotkeyConfig::default();
        config.set_slot("1".into(), PathBuf::from("/sounds/a.mp3"));
        config.set_key_chord("1", Some("Ctrl+1".into()));

        let slots = config.slots_for_chord("Ctrl+1");
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].slot, "1");

        assert!(config.slots_for_chord("Ctrl+2").is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut config = HotkeyConfig::default();
        config.set_slot("airhorn".into(), PathBuf::from("/sounds/airhorn.mp3"));
        config.set_key_chord("airhorn", Some("Ctrl+Alt+A".into()));

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HotkeyConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.slots.len(), 1);
        assert_eq!(deserialized.slots[0].slot, "airhorn");
        assert_eq!(
            deserialized.slots[0].key_chord,
            Some("Ctrl+Alt+A".to_string())
        );
    }
}
