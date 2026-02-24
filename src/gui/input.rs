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
            if self.key_pressed(ctx, Key::Enter) && self.app_state.selected_file.is_some() {
                let path = &self.app_state.selected_file.clone().unwrap();
                if modifiers.ctrl {
                    self.play_file(path, true);
                } else if modifiers.shift
                    && let Some(last_track) = self.audio_player_state.tracks.last()
                {
                    self.stop(Some(last_track.id));
                    self.play_file(path, true);
                } else {
                    self.play_file(path, false);
                }
            }

            // Iterate through dirs and files with Ctrl + Up/Down
            let arrow_up_pressed = self.key_pressed(ctx, Key::ArrowUp);
            let arrow_down_pressed = self.key_pressed(ctx, Key::ArrowDown);
            if modifiers.ctrl && (arrow_up_pressed || arrow_down_pressed) {
                if modifiers.shift && !self.app_state.dirs.is_empty() {
                    let mut dirs: Vec<PathBuf> = self.app_state.dirs.iter().cloned().collect();
                    dirs.sort();

                    let current_dir_index: i8;
                    if let Some(current_dir) = &self.app_state.current_dir {
                        if let Some(index) = dirs.iter().position(|x| x == current_dir) {
                            current_dir_index = index as i8;
                        } else {
                            current_dir_index = -1;
                        }
                    } else {
                        current_dir_index = -1;
                    }

                    let mut new_dir_index: i8;

                    new_dir_index =
                        current_dir_index - arrow_up_pressed as i8 + arrow_down_pressed as i8;

                    if new_dir_index < 0 {
                        new_dir_index = (dirs.len() - 1) as i8;
                    } else if new_dir_index >= dirs.len() as i8 {
                        new_dir_index = 0;
                    }

                    self.open_dir(&dirs[new_dir_index as usize]);
                } else if self.app_state.current_dir.is_some() {
                    let files = self.get_filtered_files();

                    if files.is_empty() {
                        return;
                    }

                    let current_files_index = self
                        .app_state
                        .selected_file
                        .as_ref()
                        .and_then(|f| files.iter().position(|x| x == f))
                        .map(|i| i as i64)
                        .unwrap_or(-1);

                    let mut new_files_index =
                        current_files_index - arrow_up_pressed as i64 + arrow_down_pressed as i64;

                    if new_files_index < 0 {
                        new_files_index = (files.len() - 1) as i64;
                    } else if new_files_index >= files.len() as i64 {
                        new_files_index = 0;
                    }

                    self.app_state.selected_file = Some(files[new_files_index as usize].clone());
                }
            }
        }
        // });
    }
}
