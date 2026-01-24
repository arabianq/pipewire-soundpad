use crate::gui::SoundpadGui;
use egui::{Context, Key};

use std::path::PathBuf;

impl SoundpadGui {
    pub fn handle_input(&mut self, ctx: &Context) {
        if ctx.memory(|reader| { reader.focused() }.is_some()) {
            return;
        }

        ctx.input(|i| {
            // Close app on espace
            if i.key_pressed(Key::Escape) {
                std::process::exit(0);
            }

            // Open/close settings
            if i.key_pressed(Key::I) {
                self.app_state.show_settings = !self.app_state.show_settings;
            }

            if i.key_pressed(Key::Enter) && self.app_state.selected_file.is_some() {
                self.play_file(
                    &self.app_state.selected_file.clone().unwrap(),
                    i.modifiers.ctrl,
                );
            }

            if !self.app_state.show_settings {
                // Pause / resume audio on space
                if i.key_pressed(Key::Space) {
                    self.play_toggle();
                }

                // Stop all audio tracks on backspace
                if i.key_pressed(Key::Backspace) {
                    self.stop(None);
                }

                // Focus search field
                if i.key_pressed(Key::Slash) && self.app_state.search_field_id.is_some() {
                    self.app_state.force_focus_id = self.app_state.search_field_id;
                }

                // Iterate through dirs if there are some
                if i.modifiers.ctrl {
                    let arrow_up_pressed = i.key_pressed(Key::ArrowUp);
                    let arrow_down_pressed = i.key_pressed(Key::ArrowDown);

                    if arrow_up_pressed || arrow_down_pressed {
                        if i.modifiers.shift && !self.app_state.dirs.is_empty() {
                            let mut dirs: Vec<PathBuf> =
                                self.app_state.dirs.iter().cloned().collect();
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

                            new_dir_index = current_dir_index - arrow_up_pressed as i8
                                + arrow_down_pressed as i8;

                            if new_dir_index < 0 {
                                new_dir_index = (dirs.len() - 1) as i8;
                            } else if new_dir_index >= dirs.len() as i8 {
                                new_dir_index = 0;
                            }

                            self.open_dir(&dirs[new_dir_index as usize]);
                        } else if self.app_state.current_dir.is_some() {
                            let mut files: Vec<PathBuf> =
                                self.app_state.files.iter().cloned().collect();
                            files.sort();

                            let current_files_index: i64;
                            if let Some(selected_file) = &self.app_state.selected_file {
                                if let Some(index) = files.iter().position(|x| x == selected_file) {
                                    current_files_index = index as i64;
                                } else {
                                    current_files_index = -1;
                                }
                            } else {
                                current_files_index = -1;
                            }

                            let mut new_files_index: i64;

                            new_files_index = current_files_index - arrow_up_pressed as i64
                                + arrow_down_pressed as i64;

                            if new_files_index < 0 {
                                new_files_index = (files.len() - 1) as i64;
                            } else if new_files_index >= files.len() as i64 {
                                new_files_index = 0;
                            }

                            self.app_state.selected_file =
                                Some(files[new_files_index as usize].clone());
                        }
                    }
                }
            }
        });
    }
}
