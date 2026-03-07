use crate::gui::SoundpadGui;
use egui::{Context, Id, Key, Modifiers};

use std::path::PathBuf;

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

        // Open/close settings
        if !search_focused && self.key_pressed(ctx, Key::I) {
            self.app_state.show_settings = !self.app_state.show_settings;
        }

        if !self.app_state.show_settings {
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

                    let new_dir_index = match (current_dir_index, arrow_up_pressed, arrow_down_pressed) {
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

                    let new_files_index = match (current_files_index, arrow_up_pressed, arrow_down_pressed) {
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
        }
        // });
    }
}
