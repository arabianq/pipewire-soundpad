use crate::gui::{SUPPORTED_EXTENSIONS, SoundpadGui};
use egui::{
    Align, AtomExt, Button, CollapsingHeader, Color32, ComboBox, CursorIcon, FontFamily, Label,
    Layout, RichText, ScrollArea, Sense, Slider, TextEdit, Ui, Vec2,
};
use egui_material_icons::icons;
use pwsp::types::audio_player::TrackInfo;
use pwsp::utils::gui::format_time_pair;
use std::{error::Error, path::PathBuf, time::Instant};

use pwsp::types::gui::AppState;

enum TrackAction {
    Pause(u32),
    Resume(u32),
    ToggleLoop(u32),
    Stop(u32),
}

impl SoundpadGui {
    fn get_volume_icon(volume: f32) -> &'static str {
        if volume > 0.7 {
            icons::ICON_VOLUME_UP
        } else if volume <= 0.0 {
            icons::ICON_VOLUME_OFF
        } else if volume < 0.3 {
            icons::ICON_VOLUME_MUTE
        } else {
            icons::ICON_VOLUME_DOWN
        }
    }

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
            let pause_on_exit_response = ui.checkbox(
                &mut self.config.pause_on_exit,
                "Pause audio playback when the window is closed",
            );

            if save_volume_response.changed()
                || save_input_response.changed()
                || save_scale_response.changed()
                || pause_on_exit_response.changed()
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
            if self.audio_player_state.tracks.is_empty() {
                ui.label("No tracks playing");
                return;
            }

            let tracks = self.audio_player_state.tracks.clone();
            let mut action = None;

            for track in tracks {
                CollapsingHeader::new(
                    RichText::new(
                        track
                            .path
                            .file_stem()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    )
                    .color(Color32::WHITE)
                    .family(FontFamily::Monospace),
                )
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(act) = Self::draw_track_control(ui, &mut self.app_state, &track) {
                        action = Some(act);
                    }
                });
                ui.separator();
            }

            if let Some(action) = action {
                match action {
                    TrackAction::Pause(id) => self.pause(Some(id)),
                    TrackAction::Resume(id) => self.resume(Some(id)),
                    TrackAction::ToggleLoop(id) => self.toggle_loop(Some(id)),
                    TrackAction::Stop(id) => self.stop(Some(id)),
                }
            }
        });
    }

    fn draw_track_control(
        ui: &mut Ui,
        app_state: &mut AppState,
        track: &TrackInfo,
    ) -> Option<TrackAction> {
        let ui_state = app_state.track_ui_states.entry(track.id).or_default();

        let should_update_position = !ui_state.position_dragged
            && ui_state
                .ignore_position_update_until
                .map(|t| Instant::now() > t)
                .unwrap_or(true);

        if should_update_position {
            ui_state.position_slider_value = track.position;
        }

        let should_update_volume = !ui_state.volume_dragged
            && ui_state
                .ignore_volume_update_until
                .map(|t| Instant::now() > t)
                .unwrap_or(true);

        if should_update_volume {
            ui_state.volume_slider_value = track.volume;
        }

        let mut action = None;

        ui.horizontal_top(|ui| {
            // ---------- Play Button ----------
            let play_button = Button::new(if track.paused {
                icons::ICON_PLAY_ARROW
            } else {
                icons::ICON_PAUSE
            })
            .corner_radius(15.0);

            let play_button_response = ui.add_sized([30.0, 30.0], play_button);
            if play_button_response.clicked() {
                if track.paused {
                    action = Some(TrackAction::Resume(track.id));
                } else {
                    action = Some(TrackAction::Pause(track.id));
                }
            }
            // --------------------------------

            // ---------- Loop Button ----------
            let loop_button = Button::new(
                RichText::new(if track.looped {
                    icons::ICON_REPEAT_ONE
                } else {
                    icons::ICON_REPEAT
                })
                .size(18.0),
            )
            .frame(false);

            let loop_button_response = ui.add_sized([15.0, 30.0], loop_button);
            if loop_button_response.clicked() {
                action = Some(TrackAction::ToggleLoop(track.id));
            }
            // --------------------------------

            // ---------- Position Slider ----------
            let duration = track.duration.unwrap_or(1.0);
            let position_slider = Slider::new(&mut ui_state.position_slider_value, 0.0..=duration)
                .show_value(false)
                .step_by(0.01);

            let default_slider_width = ui.spacing().slider_width;
            let position_slider_width = ui.available_width()
                - (30.0 * 3.0)
                - default_slider_width
                - (ui.spacing().item_spacing.x * 6.0);
            ui.spacing_mut().slider_width = position_slider_width;
            let position_slider_response = ui.add_sized([30.0, 30.0], position_slider);
            if position_slider_response.drag_stopped() {
                ui_state.position_dragged = true;
            }
            // --------------------------------

            // ---------- Time Label ----------
            let time_label =
                Label::new(RichText::new(format_time_pair(track.position, duration)).monospace());
            ui.add_sized([30.0, 30.0], time_label);
            // --------------------------------

            // ---------- Volume Icon ----------
            let volume_icon = Self::get_volume_icon(track.volume);
            let volume_label = Label::new(RichText::new(volume_icon).size(18.0));
            ui.add_sized([30.0, 30.0], volume_label)
                .on_hover_text(format!("Volume: {:.0}%", track.volume * 100.0));
            // --------------------------------

            // ---------- Volume Slider ----------
            let volume_slider = Slider::new(&mut ui_state.volume_slider_value, 0.0..=1.0)
                .show_value(false)
                .step_by(0.01);

            ui.spacing_mut().slider_width = default_slider_width - 30.0;
            ui.spacing_mut().item_spacing.x = 0.0;

            let volume_slider_response = ui.add_sized([30.0, 30.0], volume_slider);
            if volume_slider_response.drag_stopped() {
                ui_state.volume_dragged = true;
            }
            // --------------------------------

            // ---------- Stop Button ---------
            let stop_button = Button::new(icons::ICON_CLOSE).frame(false);
            let stop_button_response = ui.add_sized([30.0, 30.0], stop_button);
            if stop_button_response.clicked() {
                action = Some(TrackAction::Stop(track.id));
            }
            // --------------------------------
        });

        action
    }

    fn draw_body(&mut self, ui: &mut Ui) {
        let left_panel_width = (ui.available_width() / 4.0 + self.config.vertical_separator_width)
            .max(100.0)
            .min(ui.available_width() - 100.0);
        let dirs_size = Vec2::new(left_panel_width, ui.available_height() - 40.0);

        ui.horizontal(|ui| {
            self.draw_dirs(ui, dirs_size);

            let (rect, response) = ui.allocate_at_least(
                Vec2::new(ui.spacing().item_spacing.x, ui.available_height()),
                Sense::click_and_drag(),
            );

            if ui.is_rect_visible(rect) {
                let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
                ui.painter().vline(rect.center().x, rect.y_range(), stroke);
            }

            let vertical_separator_response =
                response.on_hover_and_drag_cursor(CursorIcon::ResizeHorizontal);

            if vertical_separator_response.dragged() {
                self.config.vertical_separator_width += vertical_separator_response.drag_delta().x;
            }

            if vertical_separator_response.drag_stopped() {
                self.config.save_to_file().ok();
            }

            let files_size = Vec2::new(ui.available_width(), ui.available_height() - 40.0);
            self.draw_files(ui, files_size);
        });
    }

    fn draw_dirs(&mut self, ui: &mut Ui, area_size: Vec2) {
        ui.vertical(|ui| {
            ui.set_min_width(area_size.x);
            ui.set_min_height(area_size.y);

            ScrollArea::vertical().id_salt(0).show(ui, |ui| {
                ui.set_min_width(area_size.x);

                let mut dirs: Vec<PathBuf> = self.app_state.dirs.iter().cloned().collect();
                dirs.sort();
                for path in dirs.iter() {
                    ui.horizontal(|ui| {
                        let name = path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.to_string_lossy().to_string());

                        let mut dir_button_text = RichText::new(name.clone());
                        if let Some(current_dir) = &self.app_state.current_dir {
                            if current_dir.eq(path) {
                                dir_button_text = dir_button_text.color(Color32::WHITE);
                            }
                        }

                        let dir_button =
                            Button::new(dir_button_text.atom_max_width(area_size.x)).frame(false);

                        let dir_button_response = ui.add(dir_button);
                        if dir_button_response.clicked() {
                            self.open_dir(path);
                        }

                        let delete_dir_button = Button::new(icons::ICON_DELETE).frame(false);
                        let delete_dir_button_response =
                            ui.add_sized([18.0, 18.0], delete_dir_button);
                        if delete_dir_button_response.clicked() {
                            self.remove_dir(&path.clone());
                        }
                    });
                }

                ui.horizontal(|ui| {
                    let add_dirs_button = Button::new(icons::ICON_ADD).frame(false);
                    let add_dirs_button_response = ui.add_sized([18.0, 18.0], add_dirs_button);
                    if add_dirs_button_response.clicked() {
                        self.add_dirs();
                    }
                });

                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    let play_file_button = Button::new("Play file");
                    let play_file_button_response = ui.add(play_file_button);
                    if play_file_button_response.clicked() {
                        self.open_file();
                    }
                });
            });
        });
    }

    fn draw_files(&mut self, ui: &mut Ui, area_size: Vec2) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let search_field = ui.add_sized(
                    [ui.available_width(), 22.0],
                    TextEdit::singleline(&mut self.app_state.search_query).hint_text("Search..."),
                );

                self.app_state.search_field_id = Some(search_field.id);
            });

            ui.separator();

            ScrollArea::vertical().id_salt(1).show(ui, |ui| {
                ui.set_min_width(area_size.x);
                ui.set_min_height(area_size.y);

                ui.vertical(|ui| {
                    let mut files: Vec<PathBuf> = self.app_state.files.iter().cloned().collect();
                    files.sort();

                    for entry_path in files {
                        if entry_path.is_dir() {
                            continue;
                        }

                        if !SUPPORTED_EXTENSIONS
                            .contains(&entry_path.extension().unwrap_or_default().to_str().unwrap())
                        {
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

                        let mut file_button_text = RichText::new(file_name);
                        if let Some(current_file) = &self.app_state.selected_file {
                            if current_file.eq(&entry_path) {
                                file_button_text = file_button_text.color(Color32::WHITE);
                            }
                        }

                        let file_button = Button::new(file_button_text).frame(false);
                        let file_button_response = ui.add(file_button);
                        if file_button_response.clicked() {
                            ui.input(|i| {
                                if i.modifiers.ctrl {
                                    self.play_file(&entry_path, true);
                                } else if i.modifiers.shift
                                    && let Some(last_track) = self.audio_player_state.tracks.last()
                                {
                                    self.stop(Some(last_track.id));
                                    self.play_file(&entry_path, true);
                                } else {
                                    self.play_file(&entry_path, false);
                                }
                            });
                            self.app_state.selected_file = Some(entry_path);
                        }
                    }
                });
            });
        });
    }

    fn draw_footer(&mut self, ui: &mut Ui) {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            // ---------- Microphone selection ----------
            let mut mics: Vec<(&String, &String)> =
                self.audio_player_state.all_inputs.iter().collect();
            mics.sort_by_key(|(k, _)| *k);

            let mut selected_input = self.audio_player_state.current_input.to_owned();
            let prev_input = selected_input.to_owned();
            ComboBox::from_label("Choose microphone")
                .height(30.0)
                .selected_text(
                    self.audio_player_state
                        .all_inputs
                        .get(&selected_input)
                        .unwrap_or(&String::new()),
                )
                .show_ui(ui, |ui| {
                    for (name, nick) in mics {
                        ui.selectable_value(&mut selected_input, name.to_owned(), nick);
                    }
                });

            if selected_input != prev_input {
                self.set_input(selected_input);
            }
            // --------------------------------

            // ---------- Master Volume Slider ----------
            let volume_icon = Self::get_volume_icon(self.audio_player_state.volume);
            let volume_label = Label::new(RichText::new(volume_icon).size(18.0));
            ui.add_sized([18.0, 18.0], volume_label)
                .on_hover_text(format!(
                    "Master Volume: {:.0}%",
                    self.audio_player_state.volume * 100.0
                ));

            let should_update_volume = !self.app_state.volume_dragged
                && self
                    .app_state
                    .ignore_volume_update_until
                    .map(|t| Instant::now() > t)
                    .unwrap_or(true);

            if should_update_volume {
                self.app_state.volume_slider_value = self.audio_player_state.volume;
            }

            let volume_slider = Slider::new(&mut self.app_state.volume_slider_value, 0.0..=1.0)
                .show_value(false)
                .step_by(0.01);
            let volume_slider_response = ui.add_sized([150.0, 18.0], volume_slider);
            if volume_slider_response.drag_stopped() {
                self.app_state.volume_dragged = true;
            }
            // ------------------------------------------

            ui.add_space(ui.available_width() - 18.0 - ui.spacing().item_spacing.x);

            // ---------- Settings button ----------
            let settings_button =
                Button::new(icons::ICON_SETTINGS.atom_size(Vec2::new(18.0, 18.0))).frame(false);
            let settings_button_response = ui.add_sized([18.0, 18.0], settings_button);
            if settings_button_response.clicked() {
                self.app_state.show_settings = true;
            }
            // --------------------------------
        });
    }
}
