use serde::{Deserialize, Serialize};

use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Settings {
    pub saved_dirs: Vec<PathBuf>,
    pub saved_mic: String,
    pub saved_volume: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            saved_dirs: Vec::new(),
            saved_mic: String::new(),
            saved_volume: 1.0,
        }
    }
}

impl Clone for Settings {
    fn clone(&self) -> Self {
        Settings {
            saved_dirs: self.saved_dirs.clone(),
            saved_mic: self.saved_mic.clone(),
            saved_volume: self.saved_volume,
        }
    }
}

impl Settings {
    pub fn save_to_file(&self, file_path: &PathBuf) {
        let mut file = fs::File::create(file_path).unwrap();
        let buf = serde_json::to_vec(&self).unwrap();
        file.write_all(&buf[..]).ok();
    }
}

pub fn load_from_file(file_path: &PathBuf) -> Settings {
    let mut file = fs::File::open(file_path).unwrap();
    let mut buf: Vec<u8> = vec![];
    file.read_to_end(&mut buf).ok();

    let mut settings: Settings = serde_json::from_slice(&buf[..]).unwrap();
    settings.saved_volume = settings.saved_volume.clamp(0.0, 1.0);

    settings
}
