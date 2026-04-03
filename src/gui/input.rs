use crate::gui::SoundpadGui;
use egui::{Context, Id, Key, Modifiers};

use std::path::PathBuf;

/// Convert an egui Key + Modifiers to a normalized chord string like "Ctrl+Shift+A".
fn chord_from_event(modifiers: &Modifiers, key: &Key) -> Option<String> {
    let key_name = match key {
        Key::A => "A",
        Key::B => "B",
        Key::C => "C",
        Key::D => "D",
        Key::E => "E",
        Key::F => "F",
        Key::G => "G",
        Key::H => "H",
        Key::I => "I",
        Key::J => "J",
        Key::K => "K",
        Key::L => "L",
        Key::M => "M",
        Key::N => "N",
        Key::O => "O",
        Key::P => "P",
        Key::Q => "Q",
        Key::R => "R",
        Key::S => "S",
        Key::T => "T",
        Key::U => "U",
        Key::V => "V",
        Key::W => "W",
        Key::X => "X",
        Key::Y => "Y",
        Key::Z => "Z",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        _ => return None,
    };

    // Require at least one modifier for hotkey chords
    if !modifiers.ctrl && !modifiers.alt && !modifiers.shift && !modifiers.command {
        return None;
    }

    let mut parts = vec![];
    if modifiers.ctrl {
        parts.push("Ctrl");
    }
    if modifiers.alt {
        parts.push("Alt");
    }
    if modifiers.shift {
        parts.push("Shift");
    }
    if modifiers.command {
        parts.push("Super");
    }
    parts.push(key_name);

    Some(parts.join("+"))
}

/// Parse a chord string back to (Modifiers, Key) for matching.
pub fn parse_chord(chord: &str) -> Option<(Modifiers, Key)> {
    let parts: Vec<&str> = chord.split('+').collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::NONE;
    for &part in &parts[..parts.len() - 1] {
        match part {
            "Ctrl" => modifiers.ctrl = true,
            "Alt" => modifiers.alt = true,
            "Shift" => modifiers.shift = true,
            "Super" => modifiers.command = true,
            _ => return None,
        }
    }

    let key = match parts[parts.len() - 1] {
        "A" => Key::A,
        "B" => Key::B,
        "C" => Key::C,
        "D" => Key::D,
        "E" => Key::E,
        "F" => Key::F,
        "G" => Key::G,
        "H" => Key::H,
        "I" => Key::I,
        "J" => Key::J,
        "K" => Key::K,
        "L" => Key::L,
        "M" => Key::M,
        "N" => Key::N,
        "O" => Key::O,
        "P" => Key::P,
        "Q" => Key::Q,
        "R" => Key::R,
        "S" => Key::S,
        "T" => Key::T,
        "U" => Key::U,
        "V" => Key::V,
        "W" => Key::W,
        "X" => Key::X,
        "Y" => Key::Y,
        "Z" => Key::Z,
        "0" => Key::Num0,
        "1" => Key::Num1,
        "2" => Key::Num2,
        "3" => Key::Num3,
        "4" => Key::Num4,
        "5" => Key::Num5,
        "6" => Key::Num6,
        "7" => Key::Num7,
        "8" => Key::Num8,
        "9" => Key::Num9,
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        _ => return None,
    };

    Some((modifiers, key))
}

impl SoundpadGui {
    fn key_pressed(&self, ctx: &Context, key: Key) -> bool {
        ctx.input(|i| i.key_pressed(key))
    }

    fn modifiers(&self, ctx: &Context) -> Modifiers {
        ctx.input(|i| i.modifiers)
    }

    fn get_focused(&self, ctx: &Context) -> Option<Id> {
        ctx.memory(|m| m.focused())
    }

    pub fn handle_input(&mut self, ctx: &Context) {
        let modifiers = self.modifiers(ctx);
        let search_focused = {
            if let Some(focused_id) = self.get_focused(ctx)
                && let Some(search_id) = self.app_state.search_field_id
                && focused_id.eq(&search_id)
            {
                true
            } else {
                false
            }
        };

        // Handle hotkey capture mode: listen for a key chord to assign
        if self.app_state.hotkey_capture_active {
            if self.key_pressed(ctx, Key::Escape) {
                self.app_state.hotkey_capture_active = false;
                self.app_state.assigning_hotkey_slot = None;
                self.app_state.assigning_hotkey_for_file = None;
                return;
            }

            // Try to capture a chord from any key press
            let captured = ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers: mods,
                        ..
                    } = event
                        && let Some(chord) = chord_from_event(mods, key)
                    {
                        return Some(chord);
                    }
                }
                None
            });

            if let Some(chord) = captured {
                if let Some(slot) = self.app_state.assigning_hotkey_slot.take() {
                    self.app_state.hotkey_config.set_key_chord(&slot, Some(chord));
                    self.save_hotkey_config();
                } else if let Some(file_path) = self.app_state.assigning_hotkey_for_file.take() {
                    // Auto-create a slot from the file name
                    let slot_name = file_path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    self.app_state
                        .hotkey_config
                        .set_slot(slot_name.clone(), file_path);
                    self.app_state
                        .hotkey_config
                        .set_key_chord(&slot_name, Some(chord));
                    self.save_hotkey_config();
                }
                self.app_state.hotkey_capture_active = false;
            }
            return;
        }

        // Open/close settings
        if !search_focused && self.key_pressed(ctx, Key::I) {
            self.app_state.show_settings = !self.app_state.show_settings;
        }

        // Toggle hotkeys view
        if !search_focused && self.key_pressed(ctx, Key::H) {
            self.app_state.show_hotkeys = !self.app_state.show_hotkeys;
        }

        if !self.app_state.show_settings && !self.app_state.show_hotkeys {
            // Pause / resume audio on space
            if !search_focused && self.key_pressed(ctx, Key::Space) {
                self.play_toggle();
            }

            // Stop all audio tracks on backspace
            if !search_focused && self.key_pressed(ctx, Key::Backspace) {
                self.stop(None);
            }

            // Focus search field
            if self.key_pressed(ctx, Key::Slash) {
                if search_focused {
                    ctx.memory_mut(|m| {
                        m.request_focus(Id::NULL);
                    });
                } else {
                    self.app_state.force_focus_search = true;
                }
            }

            // Play selected file on Enter
            if self.key_pressed(ctx, Key::Enter) {
                if let Some(path) = self.app_state.selected_file.clone() {
                    if modifiers.ctrl {
                        self.play_file(&path, true);
                    } else if modifiers.shift
                        && let Some(last_track) = self.audio_player_state.tracks.last()
                    {
                        self.stop(Some(last_track.id));
                        self.play_file(&path, true);
                    } else {
                        self.play_file(&path, false);
                    }
                }
            }

            // Iterate through dirs and files with Ctrl + Up/Down
            let arrow_up_pressed = self.key_pressed(ctx, Key::ArrowUp);
            let arrow_down_pressed = self.key_pressed(ctx, Key::ArrowDown);
            if modifiers.ctrl && (arrow_up_pressed || arrow_down_pressed) {
                if modifiers.shift && !self.app_state.dirs.is_empty() {
                    let mut dirs: Vec<PathBuf> = self.app_state.dirs.iter().cloned().collect();
                    dirs.sort();

                    let current_dir_index = self
                        .app_state
                        .current_dir
                        .as_ref()
                        .and_then(|cd| dirs.iter().position(|x| x == cd));

                    let new_dir_index =
                        match (current_dir_index, arrow_up_pressed, arrow_down_pressed) {
                            (Some(i), true, false) => (i + dirs.len() - 1) % dirs.len(),
                            (Some(i), false, true) => (i + 1) % dirs.len(),
                            (Some(i), true, true) => i,
                            (None, true, _) => dirs.len() - 1,
                            (None, false, true) => 0,
                            _ => return,
                        };

                    self.open_dir(&dirs[new_dir_index]);
                } else if self.app_state.current_dir.is_some() {
                    let files = self.get_filtered_files();

                    if files.is_empty() {
                        return;
                    }

                    let current_files_index = self
                        .app_state
                        .selected_file
                        .as_ref()
                        .and_then(|f| files.iter().position(|x| x == f));

                    let new_files_index =
                        match (current_files_index, arrow_up_pressed, arrow_down_pressed) {
                            (Some(i), true, false) => (i + files.len() - 1) % files.len(),
                            (Some(i), false, true) => (i + 1) % files.len(),
                            (Some(i), true, true) => i,
                            (None, true, _) => files.len() - 1,
                            (None, false, true) => 0,
                            _ => return,
                        };

                    self.app_state.selected_file = Some(files[new_files_index].clone());
                }
            }

            // Check for hotkey chord triggers
            let slots_to_play: Vec<String> = ctx.input(|i| {
                let mut result = vec![];
                for slot in &self.app_state.hotkey_config.slots {
                    if let Some(chord) = &slot.key_chord
                        && let Some((mods, key)) = parse_chord(chord)
                        && i.modifiers == mods
                        && i.key_pressed(key)
                    {
                        result.push(slot.slot.clone());
                    }
                }
                result
            });

            for slot in slots_to_play {
                self.play_hotkey_slot(&slot);
            }
        }
        // });
    }
}
