use crate::gui::SoundpadGui;
use egui::{
    Button, Color32, ComboBox, FontFamily, Label, RichText, ScrollArea, Slider, TextEdit, Ui, Vec2,
};
use egui_material_icons::icons;
use pwsp::types::audio_player::PlayerState;
use pwsp::utils::gui::format_time_pair;
use std::{error::Error, path::PathBuf};

impl SoundpadGui {
    pub fn draw_waiting_for_daemon(&mut self, ui: &mut Ui) {
        ui.centered_and_justified(|ui| {
            ui.label(
                RichText::new("Waiting for PWSP daemon to start...")
                    .size(34.0)
                    .monospace(),
            );
        });
    }

    pub fn draw_settings(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 5.0;
            // --------- Back Button and Title ----------
            ui.horizontal_top(|ui| {
                let back_button = Button::new(icons::ICON_ARROW_BACK).frame(false);
                let back_button_response = ui.add(back_button);
                if back_button_response.clicked() {
                    self.app_state.show_settings = false;
                }

                ui.add_space(ui.available_width() / 2.0 - 40.0);

                ui.label(RichText::new("Settings").color(Color32::WHITE).monospace());
            });
            // --------------------------------

            ui.separator();
            ui.add_space(20.0);

            // --------- Checkboxes ----------
            let save_volume_response =
                ui.checkbox(&mut self.config.save_volume, "Always remember volume");
            let save_input_response =
                ui.checkbox(&mut self.config.save_input, "Always remember microphone");
            let save_scale_response = ui.checkbox(
                &mut self.config.save_scale_factor,
                "Always remember UI scale factor",
            );

            if save_volume_response.changed()
                || save_input_response.changed()
                || save_scale_response.changed()
            {
                self.config.save_to_file().ok();
            }
            // --------------------------------
        });
    }

    pub fn draw(&mut self, ui: &mut Ui) -> Result<(), Box<dyn Error>> {
        self.draw_header(ui);
        self.draw_body(ui);
        ui.separator();
        self.draw_footer(ui);
        Ok(())
    }

    fn draw_header(&mut self, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            // Current file name
            ui.label(
                RichText::new(
                    self.audio_player_state
                        .current_file_path
                        .to_string_lossy()
                        .to_string(),
                )
                .color(Color32::WHITE)
                .family(FontFamily::Monospace),
            );
            // Media controls
            self.draw_controls(ui);
            ui.separator();
        });
    }

    fn draw_controls(&mut self, ui: &mut Ui) {
        ui.horizontal_top(|ui| {
            // ---------- Play Button ----------
            let play_button = Button::new(match self.audio_player_state.state {
                PlayerState::Playing => icons::ICON_PAUSE,
                PlayerState::Paused | PlayerState::Stopped => icons::ICON_PLAY_ARROW,
            })
            .corner_radius(15.0);

            let play_button_response = ui.add_sized([30.0, 30.0], play_button);
            if play_button_response.clicked() {
                self.play_toggle();
            }
            // --------------------------------

            // ---------- Position Slider ----------
            let position_slider = Slider::new(
                &mut self.app_state.position_slider_value,
                0.0..=self.audio_player_state.duration,
            )
            .show_value(false)
            .step_by(1.0);

            let default_slider_width = ui.spacing().slider_width;
            let position_slider_width = ui.available_width()
                - (30.0 * 3.0)
                - default_slider_width
                - (ui.spacing().item_spacing.x * 5.0);
            ui.spacing_mut().slider_width = position_slider_width;
            let position_slider_response = ui.add_sized([30.0, 30.0], position_slider);
            if position_slider_response.drag_stopped() {
                self.app_state.position_dragged = true;
            }
            // --------------------------------

            // ---------- Time Label ----------
            let time_label = Label::new(
                RichText::new(format_time_pair(
                    self.audio_player_state.position,
                    self.audio_player_state.duration,
                ))
                .monospace(),
            );
            ui.add_sized([30.0, 30.0], time_label);
            // --------------------------------

            // ---------- Volume Icon ----------
            let volume_icon = if self.audio_player_state.volume > 0.7 {
                icons::ICON_VOLUME_UP
            } else if self.audio_player_state.volume == 0.0 {
                icons::ICON_VOLUME_OFF
            } else if self.audio_player_state.volume < 0.3 {
                icons::ICON_VOLUME_MUTE
            } else {
                icons::ICON_VOLUME_DOWN
            };
            let volume_icon = Label::new(RichText::new(volume_icon).size(18.0));
            ui.add_sized([30.0, 25.0], volume_icon);
            // --------------------------------

            // ---------- Volume Slider ----------
            let volume_slider = Slider::new(&mut self.app_state.volume_slider_value, 0.0..=1.0)
                .show_value(false)
                .step_by(0.01);

            ui.spacing_mut().slider_width = default_slider_width;
            ui.spacing_mut().item_spacing.x = 0.0;

            let volume_slider_response = ui.add_sized([30.0, 30.0], volume_slider);
            if volume_slider_response.drag_stopped() {
                self.app_state.volume_dragged = true;
            }
            // --------------------------------
        });
    }

    fn draw_body(&mut self, ui: &mut Ui) {
        let dirs_size = Vec2::new(ui.available_width() / 4.0, ui.available_height() - 40.0);

        ui.horizontal(|ui| {
            self.draw_dirs(ui, dirs_size);
            ui.separator();

            let files_size = Vec2::new(ui.available_width(), ui.available_height() - 40.0);
            self.draw_files(ui, files_size);
        });
    }

    fn draw_dirs(&mut self, ui: &mut Ui, area_size: Vec2) {
        ui.vertical(|ui| {
            ui.set_min_width(area_size.x);
            ui.set_min_height(area_size.y);

            ScrollArea::vertical().id_salt(0).show(ui, |ui| {
                let mut dirs: Vec<PathBuf> = self.app_state.dirs.iter().cloned().collect();
                dirs.sort();
                for path in dirs.iter() {
                    ui.horizontal(|ui| {
                        let name = path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.to_string_lossy().to_string());

                        let dir_button = Button::new(name).frame(false);
                        let dir_button_response = ui.add(dir_button);
                        if dir_button_response.clicked() {
                            self.app_state.current_dir = Some(path.clone());
                        }

                        let delete_dir_button = Button::new(icons::ICON_DELETE).frame(false);
                        let delete_dir_button_response =
                            ui.add_sized([18.0, 18.0], delete_dir_button);
                        if delete_dir_button_response.clicked() {
                            self.remove_dir(path.clone());
                        }
                    });
                }

                ui.horizontal(|ui| {
                    let add_dir_button = egui::Button::new(icons::ICON_ADD).frame(false);
                    let add_dir_button_response = ui.add_sized([18.0, 18.0], add_dir_button);
                    if add_dir_button_response.clicked() {
                        self.add_dir();
                    }
                });
            });
        });
    }

    fn draw_files(&mut self, ui: &mut Ui, area_size: Vec2) {
        let extensions = [
            "mp3", "wav", "ogg", "flac", "mp4", "m4a", "aac", "mov", "mkv", "webm", "avi",
        ];

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [ui.available_width(), 22.0],
                    TextEdit::singleline(&mut self.app_state.search_query).hint_text("Search..."),
                );
            });

            ui.separator();

            ScrollArea::vertical().id_salt(1).show(ui, |ui| {
                ui.set_min_width(area_size.x);
                ui.set_min_height(area_size.y);

                ui.vertical(|ui| {
                    if let Some(path) = self.app_state.current_dir.clone() {
                        for entry in path.read_dir().unwrap() {
                            let entry = entry.unwrap();
                            let entry_path = entry.path();

                            if entry_path.is_dir() {
                                continue;
                            }

                            if !extensions.contains(
                                &entry_path.extension().unwrap_or_default().to_str().unwrap(),
                            ) {
                                continue;
                            }

                            let file_name = entry_path
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string();

                            let search_query = self
                                .app_state
                                .search_query
                                .to_lowercase()
                                .trim()
                                .to_string();

                            if !file_name.to_lowercase().contains(search_query.as_str()) {
                                continue;
                            }

                            let file_button = Button::new(file_name).frame(false);
                            let file_button_response = ui.add(file_button);
                            if file_button_response.clicked() {
                                self.play_file(entry_path);
                            }
                        }
                    }
                });
            });
        });
    }

    fn draw_footer(&mut self, ui: &mut Ui) {
        ui.add_space(5.0);
        ui.horizontal_top(|ui| {
            // ---------- Microphone selection ----------
            let mut mics: Vec<(&u32, &String)> =
                self.audio_player_state.all_inputs.iter().collect();
            mics.sort_by_key(|(k, _)| *k);

            let mut selected_input = self.audio_player_state.current_input.to_owned();
            let prev_input = selected_input.to_owned();
            ComboBox::from_label("Choose microphone")
                .selected_text(
                    self.audio_player_state
                        .all_inputs
                        .get(&selected_input)
                        .unwrap_or(&String::new()),
                )
                .show_ui(ui, |ui| {
                    for (index, device) in mics {
                        ui.selectable_value(&mut selected_input, index.to_owned(), device);
                    }
                });

            if selected_input != prev_input {
                self.set_input(selected_input);
            }
            // --------------------------------

            ui.add_space(ui.available_width() - 18.0 - ui.spacing().item_spacing.x);

            // ---------- Settings button ----------
            let settings_button = Button::new(icons::ICON_SETTINGS).frame(false);
            let settings_button_response = ui.add_sized([18.0, 18.0], settings_button);
            if settings_button_response.clicked() {
                self.app_state.show_settings = true;
            }
            // --------------------------------
        });
    }
}
