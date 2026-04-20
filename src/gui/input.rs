use crate::gui::SoundpadGui;
use egui::{Context, Id, Key, Modifiers};
use pwsp::types::socket::Request;
use pwsp::utils::gui::make_request_async;

use std::path::PathBuf;

/// Convert an egui Key + Modifiers to a normalized chord string like "Ctrl+Shift+A".
fn chord_from_event(modifiers: &Modifiers, key: &Key) -> Option<String> {
    let key_name = key.name();
    let is_valid = (key_name.len() == 1
        && key_name.chars().next().unwrap().is_ascii_alphanumeric())
        || (key_name.starts_with('F')
            && key_name.len() > 1
            && key_name[1..].chars().all(|c| c.is_ascii_digit()));
    if !is_valid {
        return None;
    }

    // Require at least one modifier for hotkey chords (ignoring command/Super due to Wayland/Niri bug)
    if !modifiers.ctrl && !modifiers.alt && !modifiers.shift {
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
    // We intentionally ignore modifiers.command (Super) here to bypass a Wayland/Niri bug
    // where the Super key modifier is constantly active.

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

    let key_name = parts[parts.len() - 1];
    let is_valid = (key_name.len() == 1
        && key_name.chars().next().unwrap().is_ascii_alphanumeric())
        || (key_name.starts_with('F')
            && key_name.len() > 1
            && key_name[1..].chars().all(|c| c.is_ascii_digit()));

    if !is_valid {
        return None;
    }

    let key = Key::from_name(key_name)?;

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
                    make_request_async(Request::set_hotkey_key(&slot, &chord));
                    self.app_state
                        .hotkey_config
                        .set_key_chord(&slot, Some(chord));
                } else if let Some(file_path) = self.app_state.assigning_hotkey_for_file.take() {
                    // Auto-create a slot from the file name
                    let slot_name = file_path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let action = Request::play(&file_path.to_string_lossy(), false);

                    make_request_async(Request::set_hotkey_action_and_key(
                        &slot_name, &action, &chord,
                    ));

                    self.app_state
                        .hotkey_config
                        .set_slot(slot_name.clone(), action);
                    self.app_state
                        .hotkey_config
                        .set_key_chord(&slot_name, Some(chord.clone()));
                }
                self.app_state.hotkey_capture_active = false;
                self.app_state.assigning_hotkey_slot = None;
                self.app_state.assigning_hotkey_for_file = None;
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
            if self.key_pressed(ctx, Key::Enter)
                && let Some(path) = self.app_state.selected_file.clone()
            {
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

            // Iterate through dirs and files with Ctrl + Up/Down
            let arrow_up_pressed = self.key_pressed(ctx, Key::ArrowUp);
            let arrow_down_pressed = self.key_pressed(ctx, Key::ArrowDown);
            if modifiers.ctrl && (arrow_up_pressed || arrow_down_pressed) {
                if modifiers.shift && !self.app_state.dirs.is_empty() {
                    let mut dirs: Vec<PathBuf> = self.app_state.dirs.to_vec();
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
