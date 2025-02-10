use std::error::Error;
use std::process::Command;
use std::collections::HashMap;


pub struct AudioDevice {
    pub nick: String,
    pub name: String
}


pub struct InputDevice {
    pub audio_device: AudioDevice,
    pub input_fl: String,
    pub input_fr: String
}


pub struct OutputDevice {
    pub audio_device: AudioDevice,
    pub output_fl: String,
    pub output_fr: String
}


impl OutputDevice {
    pub fn link(&self, input_device: &InputDevice) {
        Command::new("pw-link")
            .arg(&self.output_fl)
            .arg(&input_device.input_fl)
            .status().ok();

        Command::new("pw-link")
            .arg(&self.output_fr)
            .arg(&input_device.input_fr)
            .status().ok();
    }

    pub fn unlink(&self, input_device: &InputDevice) {
        Command::new("pw-link")
            .arg("--disconnect")
            .arg(&self.output_fl)
            .arg(&input_device.input_fl)
            .status().ok();

        Command::new("pw-link")
            .arg("--disconnect")
            .arg(&self.output_fr)
            .arg(&input_device.input_fr)
            .status().ok();
    }
}


fn get_pw_entries() -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
    let output = Command::new("pw-cli")
        .args(&["ls", "Node"])
        .output()
        .expect("Failed to execute pw-cli ls Node");

    let output_str = String::from_utf8_lossy(&output.stdout);

    let mut entries = Vec::new();
    let mut current_entry = HashMap::new();

    for line in output_str.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue
        }

        if line.starts_with("id ") {
            if !current_entry.is_empty() {
                entries.push(current_entry);
                current_entry = HashMap::new();
            }
        } else {
            let mut parts = line.splitn(2, " = ");

            let key = match parts.next() {
                Some(k) => k.trim().to_string(),
                None => continue
            };

            let value = match parts.next() {
                Some(k) => k.trim().strip_prefix("\"").unwrap().strip_suffix("\"").unwrap().to_string(),
                None => continue
            };

            current_entry.insert(key, value);
        }
    }

    if !current_entry.is_empty() {
        entries.push(current_entry);
    }

    Ok(entries)
}

pub fn get_input_devices() -> Result<Vec<InputDevice>, Box<dyn Error>> {
    let entries = get_pw_entries()?;

    let mut input_devices = Vec::new();

    for entry in entries.iter() {
        let media_class = entry.get("media.class").map(String::as_str).unwrap_or("");
        let nick = entry.get("node.nick").map(String::as_str)
            .unwrap_or(entry.get("node.description").map(String::as_str)
                .unwrap_or(entry.get("node.name").map(String::as_str).unwrap_or("")));
        let name = entry.get("node.name").map(String::as_str).unwrap_or("");

        if media_class.is_empty() {
            continue
        }

        if !media_class.starts_with(&"Audio/Source") {
            continue
        }

        if nick.is_empty() || name.is_empty() {
            continue
        }

        let audio_device = AudioDevice {
            nick: nick.to_string(),
            name: name.to_string(),
        };

        let device = InputDevice {
            audio_device,
            input_fl: String::new(),
            input_fr: String::new()
        };

        input_devices.push(device);
    }

    let output = Command::new("pw-link")
        .arg("-i")
        .output()
        .expect("Failed to execute pw-link -i");

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        let line = line.trim();

        for device in input_devices.iter_mut() {
            if line.starts_with(device.audio_device.name.as_str()) {
                if line.ends_with("input_MONO") {
                    device.input_fl = line.to_string();
                    device.input_fr = line.to_string();
                } else if line.ends_with("input_FL") {
                    device.input_fl = line.to_string();
                } else if line.ends_with("input_FR") {
                    device.input_fr = line.to_string();
                }
            }
        }
    }

    Ok(input_devices)
}

pub fn get_output_devices() -> Result<Vec<OutputDevice>, Box<dyn Error>> {
    let entries = get_pw_entries()?;

    let mut output_devices = Vec::new();

    for entry in entries.iter() {
        let media_class = entry.get("media.class").map(String::as_str).unwrap_or("");
        let nick = entry.get("node.nick").map(String::as_str)
            .unwrap_or(entry.get("node.description").map(String::as_str)
                .unwrap_or(entry.get("node.name").map(String::as_str).unwrap_or("")));
        let name = entry.get("node.name").map(String::as_str).unwrap_or("");

        if media_class.is_empty() {
            continue
        }

        if !media_class.starts_with(&"Stream/Output/Audio") {
            continue
        }

        if nick.is_empty() || name.is_empty() {
            continue
        }

        let audio_device = AudioDevice {
            nick: nick.to_string(),
            name: name.to_string(),
        };

        let device = OutputDevice {
            audio_device,
            output_fl: String::new(),
            output_fr: String::new()
        };

        output_devices.push(device);
    }

    let output = Command::new("pw-link")
        .arg("-o")
        .output()
        .expect("Failed to execute pw-link -o");

    let output_str = String::from_utf8_lossy(&output.stdout);

    for line in output_str.lines() {
        let line = line.trim();

        for device in output_devices.iter_mut() {
            if line.starts_with(device.audio_device.name.as_str()) {
                if line.ends_with("capture_MONO") {
                    device.output_fl = line.to_string();
                    device.output_fr = line.to_string();
                } else if line.ends_with("output_FL") {
                    device.output_fl = line.to_string();
                } else if line.ends_with("output_FR") {
                    device.output_fr = line.to_string();
                }
            }
        }
    }

    Ok(output_devices)
}