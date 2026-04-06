use crate::{types::config::HotkeyConfig, utils::commands::parse_command};
use evdev::{Device, EventStream, EventSummary, KeyCode};

struct ModifierState {
    ctrl: bool,
    alt: bool,
    shift: bool,
    meta: bool,
}

impl ModifierState {
    fn new() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    fn update(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL => self.ctrl = pressed,
            KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT => self.alt = pressed,
            KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT => self.shift = pressed,
            KeyCode::KEY_LEFTMETA | KeyCode::KEY_RIGHTMETA => self.meta = pressed,
            _ => {}
        }
    }

    fn any_active(&self) -> bool {
        self.ctrl || self.alt || self.shift || self.meta
    }

    fn is_modifier(key: KeyCode) -> bool {
        matches!(
            key,
            KeyCode::KEY_LEFTCTRL
                | KeyCode::KEY_RIGHTCTRL
                | KeyCode::KEY_LEFTALT
                | KeyCode::KEY_RIGHTALT
                | KeyCode::KEY_LEFTSHIFT
                | KeyCode::KEY_RIGHTSHIFT
                | KeyCode::KEY_LEFTMETA
                | KeyCode::KEY_RIGHTMETA
        )
    }
}

fn evdev_key_name(key: KeyCode) -> Option<&'static str> {
    match key {
        KeyCode::KEY_A => Some("A"),
        KeyCode::KEY_B => Some("B"),
        KeyCode::KEY_C => Some("C"),
        KeyCode::KEY_D => Some("D"),
        KeyCode::KEY_E => Some("E"),
        KeyCode::KEY_F => Some("F"),
        KeyCode::KEY_G => Some("G"),
        KeyCode::KEY_H => Some("H"),
        KeyCode::KEY_I => Some("I"),
        KeyCode::KEY_J => Some("J"),
        KeyCode::KEY_K => Some("K"),
        KeyCode::KEY_L => Some("L"),
        KeyCode::KEY_M => Some("M"),
        KeyCode::KEY_N => Some("N"),
        KeyCode::KEY_O => Some("O"),
        KeyCode::KEY_P => Some("P"),
        KeyCode::KEY_Q => Some("Q"),
        KeyCode::KEY_R => Some("R"),
        KeyCode::KEY_S => Some("S"),
        KeyCode::KEY_T => Some("T"),
        KeyCode::KEY_U => Some("U"),
        KeyCode::KEY_V => Some("V"),
        KeyCode::KEY_W => Some("W"),
        KeyCode::KEY_X => Some("X"),
        KeyCode::KEY_Y => Some("Y"),
        KeyCode::KEY_Z => Some("Z"),
        KeyCode::KEY_1 => Some("1"),
        KeyCode::KEY_2 => Some("2"),
        KeyCode::KEY_3 => Some("3"),
        KeyCode::KEY_4 => Some("4"),
        KeyCode::KEY_5 => Some("5"),
        KeyCode::KEY_6 => Some("6"),
        KeyCode::KEY_7 => Some("7"),
        KeyCode::KEY_8 => Some("8"),
        KeyCode::KEY_9 => Some("9"),
        KeyCode::KEY_0 => Some("0"),
        KeyCode::KEY_F1 => Some("F1"),
        KeyCode::KEY_F2 => Some("F2"),
        KeyCode::KEY_F3 => Some("F3"),
        KeyCode::KEY_F4 => Some("F4"),
        KeyCode::KEY_F5 => Some("F5"),
        KeyCode::KEY_F6 => Some("F6"),
        KeyCode::KEY_F7 => Some("F7"),
        KeyCode::KEY_F8 => Some("F8"),
        KeyCode::KEY_F9 => Some("F9"),
        KeyCode::KEY_F10 => Some("F10"),
        KeyCode::KEY_F11 => Some("F11"),
        KeyCode::KEY_F12 => Some("F12"),
        _ => None,
    }
}

fn build_chord(modifiers: &ModifierState, key_name: &str) -> String {
    let mut parts = Vec::with_capacity(5);
    if modifiers.ctrl {
        parts.push("Ctrl");
    }
    if modifiers.alt {
        parts.push("Alt");
    }
    if modifiers.shift {
        parts.push("Shift");
    }
    if modifiers.meta {
        parts.push("Super");
    }
    parts.push(key_name);
    parts.join("+")
}

fn is_keyboard(device: &Device) -> bool {
    device
        .supported_keys()
        .is_some_and(|keys| keys.contains(KeyCode::KEY_A) && keys.contains(KeyCode::KEY_Z))
}

async fn handle_device_events(mut stream: EventStream) {
    let mut modifiers = ModifierState::new();

    loop {
        match stream.next_event().await {
            Ok(event) => {
                if let EventSummary::Key(_, key, value) = event.destructure() {
                    // 0 = released, 1 = pressed, 2 = repeat
                    if value == 0 || value == 1 {
                        modifiers.update(key, value == 1);
                    }

                    // Only trigger on press, skip modifiers and bare keys
                    if value != 1 || ModifierState::is_modifier(key) || !modifiers.any_active() {
                        continue;
                    }

                    let Some(key_name) = evdev_key_name(key) else {
                        continue;
                    };

                    let chord = build_chord(&modifiers, key_name);

                    let config = match HotkeyConfig::load() {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    let slots = config.slots_for_chord(&chord);
                    for slot in slots {
                        if let Some(cmd) = parse_command(&slot.action) {
                            cmd.execute().await;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Global hotkeys: device read error: {e}");
                break;
            }
        }
    }
}

pub async fn start_global_hotkey_listener() {
    let keyboards: Vec<_> = evdev::enumerate()
        .filter(|(_, dev)| is_keyboard(dev))
        .collect();

    if keyboards.is_empty() {
        eprintln!(
            "Global hotkeys: no keyboard devices found. \
             Make sure your user is in the 'input' group."
        );
        return;
    }

    println!(
        "Global hotkeys: found {} keyboard device(s)",
        keyboards.len()
    );

    for (path, device) in keyboards {
        match device.into_event_stream() {
            Ok(stream) => {
                println!("Global hotkeys: listening on {}", path.display());
                tokio::spawn(handle_device_events(stream));
            }
            Err(e) => {
                eprintln!("Global hotkeys: failed to open {}: {}", path.display(), e);
            }
        }
    }
}
