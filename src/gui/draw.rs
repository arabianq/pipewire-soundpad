use crate::gui::SoundpadGui;
use egui::{
    Align, AtomExt, Button, CollapsingHeader, Color32, ComboBox, CursorIcon, FontFamily, Label,
    Layout, RichText, ScrollArea, Sense, Slider, TextEdit, Ui, Vec2,
};
use egui_dnd::dnd;
use egui_extras::{Column, TableBuilder};
use egui_material_icons::icons::*;
use pwsp::types::socket::Request;
use pwsp::types::{audio_player::TrackInfo, gui::{AppState, SortColumn, SortDir}};
use pwsp::utils::gui::{
    cycle_sort, format_mtime_opt, format_time_pair, make_request_async,
    refresh_mtime_cache, slot_index_map, sort_files,
};
use std::{
    path::Path,
    time::Instant,
};

enum TrackAction {
    Pause(u32),
    Resume(u32),
    ToggleLoop(u32),
    Stop(u32),
}

enum HotkeyAction {
    Remove(String),
    Capture(String),
    ClearChord(String),
    Play(String),
}

impl SoundpadGui {
    fn get_volume_icon(volume: f32) -> &'static str {
        if volume > 0.7 {
            ICON_VOLUME_UP.codepoint
        } else if volume <= 0.0 {
            ICON_VOLUME_OFF.codepoint
        } else if volume < 0.3 {
            ICON_VOLUME_MUTE.codepoint
        } else {
            ICON_VOLUME_DOWN.codepoint
        }
    }

    pub fn draw(&mut self, ui: &mut Ui) {
        self.draw_header(ui);
        self.draw_body(ui);
        ui.separator();
        self.draw_footer(ui);
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

    pub fn draw_hotkey_capture(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.label(
                RichText::new("Press a key combination (e.g. Ctrl+Alt+1)")
                    .size(18.0)
                    .color(Color32::YELLOW)
                    .monospace(),
            );
            ui.add_space(10.0);
            let target = if let Some(slot) = &self.app_state.assigning_hotkey_slot {
                format!("for slot '{}'", slot)
            } else if let Some(path) = &self.app_state.assigning_hotkey_for_file {
                format!(
                    "for '{}'",
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            } else {
                String::new()
            };
            ui.label(RichText::new(target).size(16.0));
            ui.add_space(10.0);
            ui.label("Press Escape to cancel");
        });
    }

    pub fn draw_settings(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 5.0;
            // --------- Back Button and Title ----------
            ui.horizontal_top(|ui| {
                let back_button = Button::new(ICON_ARROW_BACK).frame(false);
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

            ui.add_space(10.0);
            ui.separator();
            ui.label(RichText::new("Columns").monospace());

            let show_index_response =
                ui.checkbox(&mut self.config.show_index_column, "Show # column");
            let show_hotkey_response =
                ui.checkbox(&mut self.config.show_hotkey_column, "Show Hotkey column");
            let show_modified_response = ui.checkbox(
                &mut self.config.show_modified_column,
                "Show Last Modified column",
            );

            if save_volume_response.changed()
                || save_input_response.changed()
                || save_scale_response.changed()
                || pause_on_exit_response.changed()
                || show_index_response.changed()
                || show_hotkey_response.changed()
                || show_modified_response.changed()
            {
                self.config.save_to_file().ok();
            }
            // --------------------------------

            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.label(format!("GUI version: {}", env!("CARGO_PKG_VERSION")));
            });
        });
    }

    pub fn draw_hotkeys(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 5.0;

            self.draw_hotkeys_header(ui);
            ui.separator();

            self.draw_hotkeys_search(ui);
            ui.separator();
            ui.add_space(5.0);

            let action = self.draw_hotkeys_table(ui);

            if let Some(action) = action {
                self.handle_hotkey_action(action);
            }
        });
    }

    fn draw_hotkeys_header(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let back_button = Button::new(ICON_ARROW_BACK).frame(false);
            if ui.add(back_button).clicked() {
                self.app_state.show_hotkeys = false;
            }

            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Hotkeys").color(Color32::WHITE).monospace());
            });
        });
    }

    fn draw_hotkeys_search(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.menu_button(format!("{} Add Command", ICON_ADD.codepoint), |ui| {
                let mut selected_cmd = None;
                if ui.button("Toggle Pause").clicked() {
                    selected_cmd = Some(("cmd_toggle_pause", Request::toggle_pause(None)));
                }
                if ui.button("Stop Playback").clicked() {
                    selected_cmd = Some(("cmd_stop", Request::stop(None)));
                }
                if ui.button("Pause Playback").clicked() {
                    selected_cmd = Some(("cmd_pause", Request::pause(None)));
                }
                if ui.button("Resume Playback").clicked() {
                    selected_cmd = Some(("cmd_resume", Request::resume(None)));
                }
                if ui.button("Toggle Loop").clicked() {
                    selected_cmd = Some(("cmd_toggle_loop", Request::toggle_loop(None)));
                }

                if let Some((slot_name, req)) = selected_cmd {
                    make_request_async(Request::set_hotkey_action(slot_name, &req));
                    self.app_state
                        .hotkey_config
                        .set_slot(slot_name.to_string(), req);
                    self.app_state.assigning_hotkey_slot = Some(slot_name.to_string());
                    self.app_state.hotkey_capture_active = true;
                    ui.close();
                }
            });

            ui.add_space(10.0);

            ui.add(
                TextEdit::singleline(&mut self.app_state.hotkey_search_query)
                    .hint_text("Search hotkeys...")
                    .desired_width(f32::INFINITY),
            );
        });
    }

    fn draw_hotkeys_table(&mut self, ui: &mut Ui) -> Option<HotkeyAction> {
        let conflicts = self.app_state.hotkey_config.find_conflicts();
        let conflict_slots: std::collections::HashSet<&str> =
            conflicts.into_iter().flat_map(|(a, b)| [a, b]).collect();

        let search = self.app_state.hotkey_search_query.to_lowercase();
        let mut action: Option<HotkeyAction> = None;

        let slots: Vec<_> = self
            .app_state
            .hotkey_config
            .slots
            .iter()
            .filter(|s| {
                if search.is_empty() {
                    return true;
                }
                s.slot.to_lowercase().contains(&search)
                    || format!("{:?}", s.action).to_lowercase().contains(&search)
                    || s.key_chord
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&search)
            })
            .cloned()
            .collect();

        let available_width = ui.available_width();
        let col_width = (available_width / 4.0).max(80.0);

        TableBuilder::new(ui)
            .striped(true)
            .column(Column::exact(col_width).clip(true)) // Slot
            .column(Column::exact(col_width).clip(true)) // Sound / Action name
            .column(Column::exact(col_width).clip(true)) // Key Chord
            .column(Column::exact(col_width).clip(true)) // Actions
            .header(30.0, |mut header| {
                header.col(|ui| {
                    ui.label(
                        RichText::new("Slot")
                            .strong()
                            .monospace()
                            .color(Color32::LIGHT_GRAY),
                    );
                });
                header.col(|ui| {
                    ui.label(
                        RichText::new("Sound")
                            .strong()
                            .monospace()
                            .color(Color32::LIGHT_GRAY),
                    );
                });
                header.col(|ui| {
                    ui.label(
                        RichText::new("Key Chord")
                            .strong()
                            .monospace()
                            .color(Color32::LIGHT_GRAY),
                    );
                });
                header.col(|ui| {
                    ui.label(
                        RichText::new("Actions")
                            .strong()
                            .monospace()
                            .color(Color32::LIGHT_GRAY),
                    );
                });
            })
            .body(|mut body| {
                if slots.is_empty() {
                    body.row(30.0, |mut row| {
                        row.col(|_| {});
                        row.col(|ui| {
                            ui.label(
                                RichText::new("No hotkey slots configured.")
                                    .color(Color32::GRAY),
                            );
                        });
                        row.col(|_| {});
                        row.col(|_| {});
                    });
                    return;
                }

                for slot in &slots {
                    body.row(30.0, |mut row| {
                        // Column 1: Slot
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if conflict_slots.contains(slot.slot.as_str()) {
                                    ui.label(
                                        RichText::new(ICON_WARNING.codepoint)
                                            .color(Color32::from_rgb(255, 165, 0)),
                                    )
                                    .on_hover_text("Key chord conflict");
                                }
                                ui.add(
                                    Label::new(RichText::new(&slot.slot).monospace())
                                        .truncate(),
                                );
                            });
                        });

                        // Column 2: Sound / Action name
                        row.col(|ui| {
                            let action_name = match slot.action.name.as_str() {
                                "play" => {
                                    if let Some(file_path_str) =
                                        slot.action.args.get("file_path")
                                    {
                                        Path::new(file_path_str)
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .to_string()
                                    } else {
                                        "Play".to_string()
                                    }
                                }
                                "toggle_pause" => "Toggle Pause".to_string(),
                                "pause" => "Pause Playback".to_string(),
                                "resume" => "Resume Playback".to_string(),
                                "stop" => "Stop Playback".to_string(),
                                "toggle_loop" => "Toggle Loop".to_string(),
                                other => other.to_string(),
                            };
                            ui.add(
                                Label::new(RichText::new(action_name).monospace()).truncate(),
                            );
                        });

                        // Column 3: Key Chord
                        row.col(|ui| {
                            let chord_text = slot.key_chord.as_deref().unwrap_or("(none)");
                            ui.add(
                                Label::new(RichText::new(chord_text).monospace().color(
                                    if slot.key_chord.is_some() {
                                        Color32::from_rgb(100, 200, 100)
                                    } else {
                                        Color32::GRAY
                                    },
                                ))
                                .truncate(),
                            );
                        });

                        // Column 4: Actions
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if ui
                                    .add(Button::new(ICON_DELETE).frame(false))
                                    .on_hover_text("Remove slot")
                                    .clicked()
                                {
                                    action = Some(HotkeyAction::Remove(slot.slot.clone()));
                                }
                                if ui
                                    .add(Button::new(ICON_KEYBOARD).frame(false))
                                    .on_hover_text("Set key chord")
                                    .clicked()
                                {
                                    action = Some(HotkeyAction::Capture(slot.slot.clone()));
                                }
                                if slot.key_chord.is_some()
                                    && ui
                                        .add(Button::new(ICON_BACKSPACE).frame(false))
                                        .on_hover_text("Clear key chord")
                                        .clicked()
                                {
                                    action = Some(HotkeyAction::ClearChord(slot.slot.clone()));
                                }
                                if ui
                                    .add(Button::new(ICON_PLAY_ARROW).frame(false))
                                    .on_hover_text("Play")
                                    .clicked()
                                {
                                    action = Some(HotkeyAction::Play(slot.slot.clone()));
                                }
                            });
                        });
                    });
                }
            });

        action
    }

    fn handle_hotkey_action(&mut self, action: HotkeyAction) {
        match action {
            HotkeyAction::Remove(slot) => {
                make_request_async(Request::clear_hotkey(&slot));
                self.app_state.hotkey_config.remove_slot(&slot);
            }
            HotkeyAction::Capture(slot) => {
                self.app_state.assigning_hotkey_slot = Some(slot);
                self.app_state.hotkey_capture_active = true;
            }
            HotkeyAction::ClearChord(slot) => {
                make_request_async(Request::clear_hotkey_key(&slot));
                self.app_state.hotkey_config.set_key_chord(&slot, None);
            }
            HotkeyAction::Play(slot) => {
                self.play_hotkey_slot(&slot);
            }
        }
    }

    fn draw_header(&mut self, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            if self.audio_player_state.tracks.is_empty() {
                ui.label("No tracks playing");
                return;
            }

            let mut action = None;

            for track in &self.audio_player_state.tracks {
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
                    if let Some(act) = Self::draw_track_control(ui, &mut self.app_state, track) {
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

    fn draw_playback_controls(ui: &mut Ui, track: &TrackInfo) -> Option<TrackAction> {
        let mut action = None;

        let play_button = Button::new(if track.paused {
            ICON_PLAY_ARROW
        } else {
            ICON_PAUSE
        })
        .corner_radius(15.0);

        if ui.add_sized([30.0, 30.0], play_button).clicked() {
            action = Some(if track.paused {
                TrackAction::Resume(track.id)
            } else {
                TrackAction::Pause(track.id)
            });
        }

        let loop_button = Button::new(
            RichText::new(if track.looped {
                ICON_REPEAT_ONE
            } else {
                ICON_REPEAT
            })
            .size(18.0),
        )
        .frame(false);

        if ui.add_sized([15.0, 30.0], loop_button).clicked() {
            action = Some(TrackAction::ToggleLoop(track.id));
        }

        action
    }

    fn draw_position_control(
        ui: &mut Ui,
        ui_state: &mut pwsp::types::gui::TrackUiState,
        track: &TrackInfo,
        default_slider_width: f32,
    ) {
        let duration = track.duration.unwrap_or(1.0);
        let position_slider = Slider::new(&mut ui_state.position_slider_value, 0.0..=duration)
            .show_value(false)
            .step_by(0.01);

        let position_slider_width = ui.available_width()
            - (30.0 * 3.0)
            - default_slider_width
            - (ui.spacing().item_spacing.x * 6.0);

        ui.spacing_mut().slider_width = position_slider_width;
        if ui.add_sized([30.0, 30.0], position_slider).drag_stopped() {
            ui_state.position_dragged = true;
        }

        let time_label =
            Label::new(RichText::new(format_time_pair(track.position, duration)).monospace());
        ui.add_sized([30.0, 30.0], time_label);
    }

    fn draw_volume_control(
        ui: &mut Ui,
        ui_state: &mut pwsp::types::gui::TrackUiState,
        track: &TrackInfo,
        default_slider_width: f32,
    ) {
        let volume_icon = Self::get_volume_icon(track.volume);
        let volume_label = Label::new(RichText::new(volume_icon).size(18.0));
        ui.add_sized([30.0, 30.0], volume_label)
            .on_hover_text(format!("Volume: {:.0}%", track.volume * 100.0));

        let volume_slider = Slider::new(&mut ui_state.volume_slider_value, 0.0..=1.0)
            .show_value(false)
            .step_by(0.01);

        ui.spacing_mut().slider_width = default_slider_width - 30.0;
        ui.spacing_mut().item_spacing.x = 0.0;

        if ui.add_sized([30.0, 30.0], volume_slider).drag_stopped() {
            ui_state.volume_dragged = true;
        }
    }

    fn draw_stop_control(ui: &mut Ui, track: &TrackInfo) -> Option<TrackAction> {
        let stop_button = Button::new(ICON_CLOSE).frame(false);
        if ui.add_sized([30.0, 30.0], stop_button).clicked() {
            Some(TrackAction::Stop(track.id))
        } else {
            None
        }
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
            if let Some(act) = Self::draw_playback_controls(ui, track) {
                action = Some(act);
            }

            let default_slider_width = ui.spacing().slider_width;
            Self::draw_position_control(ui, ui_state, track, default_slider_width);
            Self::draw_volume_control(ui, ui_state, track, default_slider_width);

            if let Some(act) = Self::draw_stop_control(ui, track) {
                action = Some(act);
            }
        });

        action
    }

    fn draw_body(&mut self, ui: &mut Ui) {
        let left_panel_width = self
            .config
            .left_panel_width
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
                self.config.left_panel_width += vertical_separator_response.drag_delta().x;
                self.config.left_panel_width = self.config.left_panel_width.clamp(100.0, 500.0);
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

                let mut dirs = self.app_state.dirs.clone();

                dnd(ui, "dnd_directories").show_vec(&mut dirs, |ui, item, handle, _state| {
                    let path = item.clone();
                    ui.horizontal(|ui| {
                        handle.ui(ui, |ui| {
                            ui.label(ICON_DRAG_INDICATOR.codepoint);
                        });
                        let name = path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.to_string_lossy().to_string());

                        let mut dir_button_text = RichText::new(name.clone());
                        if let Some(current_dir) = &self.app_state.current_dir
                            && current_dir.eq(&path)
                        {
                            dir_button_text = dir_button_text.color(Color32::WHITE);
                        }

                        let dir_button =
                            Button::new(dir_button_text.atom_max_width(area_size.x)).frame(false);

                        let dir_button_response = ui.add(dir_button);
                        if dir_button_response.clicked() {
                            self.open_dir(&path);
                        }

                        let delete_dir_button = Button::new(ICON_DELETE).frame(false);
                        let delete_dir_button_response =
                            ui.add_sized([18.0, 18.0], delete_dir_button);
                        if delete_dir_button_response.clicked() {
                            self.app_state.dirs_to_remove.insert(path.clone());
                        }

                        // Context menu
                        dir_button_response.context_menu(|ui| {
                            if ui
                                .button(format!("{} {}", ICON_OPEN_IN_NEW.codepoint, "Show"))
                                .clicked()
                            {
                                self.open_dir(&path);
                            }

                            if ui
                                .button(format!(
                                    "{} {}",
                                    ICON_OPEN_IN_BROWSER.codepoint, "Open in File Manager"
                                ))
                                .clicked()
                                && let Err(e) = opener::open(&path)
                            {
                                eprintln!("Failed to open file manager: {}", e);
                            }

                            ui.separator();

                            if ui
                                .button(format!("{} {}", ICON_DELETE.codepoint, "Remove"))
                                .clicked()
                            {
                                self.app_state.dirs_to_remove.insert(path.clone());
                            }
                        });
                    });
                });
                self.app_state.dirs = dirs;

                ui.horizontal(|ui| {
                    let add_dirs_button = Button::new(ICON_ADD).frame(false);
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

    fn header_cell(&mut self, ui: &mut Ui, label: &str, col: SortColumn) {
        let glyph = if self.app_state.sort_column == col {
            match self.app_state.sort_dir {
                SortDir::Asc => ICON_ARROW_UPWARD.codepoint,
                SortDir::Desc => ICON_ARROW_DOWNWARD.codepoint,
            }
        } else {
            ""
        };
        let text = RichText::new(format!("{} {}", label, glyph)).strong();
        let resp = ui.add(Button::new(text).frame(false));
        if resp.clicked() {
            let (new_col, new_dir) =
                cycle_sort(self.app_state.sort_column, self.app_state.sort_dir, col);
            self.app_state.sort_column = new_col;
            self.app_state.sort_dir = new_dir;
        }
    }

    fn draw_files(&mut self, ui: &mut Ui, area_size: Vec2) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let search_field_response = ui.add_sized(
                    [ui.available_width(), 22.0],
                    TextEdit::singleline(&mut self.app_state.search_query).hint_text("Search..."),
                );

                if self.app_state.force_focus_search {
                    search_field_response.request_focus();
                    self.app_state.force_focus_search = false;
                }
                self.app_state.search_field_id = Some(search_field_response.id);
            });

            ui.separator();

            // Cache invalidation when category changes
            if self.app_state.mtime_cache_dir != self.app_state.current_dir {
                refresh_mtime_cache(&mut self.app_state.file_mtime_cache, &self.app_state.files);
                self.app_state.mtime_cache_dir = self.app_state.current_dir.clone();
            }

            // Build hotkey lookup once per frame
            let mut hotkeys: std::collections::HashMap<std::path::PathBuf, String> =
                std::collections::HashMap::new();
            for slot in &self.app_state.hotkey_config.slots {
                if slot.action.name == "play"
                    && let Some(p) = slot.action.args.get("file_path")
                {
                    let label = match &slot.key_chord {
                        Some(c) => c.clone(),
                        None => slot.slot.clone(),
                    };
                    hotkeys.insert(std::path::PathBuf::from(p), label);
                }
            }

            // Stable slot index over the unfiltered set
            let slot_idx = slot_index_map(&self.app_state.files);

            // Filtered + sorted view
            let filtered = self.get_filtered_files();

            // Fallback: if the active sort column was hidden via settings, revert to Index asc.
            let active_hidden = match self.app_state.sort_column {
                SortColumn::Index => !self.config.show_index_column,
                SortColumn::Hotkey => !self.config.show_hotkey_column,
                SortColumn::Modified => !self.config.show_modified_column,
                SortColumn::Name => false,
            };
            if active_hidden {
                self.app_state.sort_column = SortColumn::Index;
                self.app_state.sort_dir = SortDir::Asc;
            }

            let sorted = sort_files(
                &filtered,
                self.app_state.sort_column,
                self.app_state.sort_dir,
                &self.app_state.file_mtime_cache,
                &hotkeys,
            );

            // Table
            let mut table = TableBuilder::new(ui)
                .striped(false)
                .resizable(true)
                .cell_layout(Layout::left_to_right(Align::Center));
            if self.config.show_index_column {
                table = table.column(Column::initial(40.0).at_least(30.0));
            }
            if self.config.show_hotkey_column {
                table = table.column(Column::initial(70.0).at_least(40.0));
            }
            table = table.column(Column::remainder().clip(true));
            if self.config.show_modified_column {
                table = table.column(Column::initial(120.0).at_least(80.0));
            }
            let table = table.min_scrolled_height(area_size.y);

            table
                .header(20.0, |mut header| {
                    if self.config.show_index_column {
                        header.col(|ui| {
                            self.header_cell(ui, "#", SortColumn::Index);
                        });
                    }
                    if self.config.show_hotkey_column {
                        header.col(|ui| {
                            self.header_cell(ui, "Hotkey", SortColumn::Hotkey);
                        });
                    }
                    header.col(|ui| {
                        self.header_cell(ui, "Name", SortColumn::Name);
                    });
                    if self.config.show_modified_column {
                        header.col(|ui| {
                            self.header_cell(ui, "Last Modified", SortColumn::Modified);
                        });
                    }
                })
                .body(|mut body| {
                    for entry_path in sorted {
                        let file_name = entry_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let idx = slot_idx.get(&entry_path).copied().unwrap_or(0);
                        let hotkey_label =
                            hotkeys.get(&entry_path).cloned().unwrap_or_default();
                        let mtime = self.app_state.file_mtime_cache.get(&entry_path).copied();

                        body.row(20.0, |mut row| {
                            if self.config.show_index_column {
                                row.col(|ui| {
                                    ui.label(RichText::new(idx.to_string()).monospace());
                                });
                            }
                            if self.config.show_hotkey_column {
                                row.col(|ui| {
                                    if !hotkey_label.is_empty() {
                                        ui.label(
                                            RichText::new(&hotkey_label)
                                                .small()
                                                .monospace()
                                                .color(Color32::from_rgb(100, 200, 100)),
                                        );
                                    }
                                });
                            }
                            row.col(|ui| {
                                let mut file_button_text = RichText::new(&file_name);
                                if let Some(current_file) = &self.app_state.selected_file
                                    && current_file.eq(&entry_path)
                                {
                                    file_button_text = file_button_text.color(Color32::WHITE);
                                }
                                let file_button =
                                    Button::new(file_button_text).frame(false).truncate();
                                let file_button_response = ui.add(file_button);
                                if file_button_response.clicked() {
                                    ui.input(|i| {
                                        if i.modifiers.ctrl {
                                            self.play_file(&entry_path, true);
                                        } else if i.modifiers.shift
                                            && let Some(last_track) =
                                                self.audio_player_state.tracks.last()
                                        {
                                            self.stop(Some(last_track.id));
                                            self.play_file(&entry_path, true);
                                        } else {
                                            self.play_file(&entry_path, false);
                                        }
                                    });
                                    self.app_state.selected_file = Some(entry_path.clone());
                                }

                                file_button_response.context_menu(|ui| {
                                    if ui
                                        .button(format!(
                                            "{} {}",
                                            ICON_BOLT.codepoint, "Play Solo"
                                        ))
                                        .clicked()
                                    {
                                        self.play_file(&entry_path, false);
                                        self.app_state.selected_file =
                                            Some(entry_path.clone());
                                    }
                                    if ui
                                        .button(format!(
                                            "{} {}",
                                            ICON_ADD.codepoint, "Add New"
                                        ))
                                        .clicked()
                                    {
                                        self.play_file(&entry_path, true);
                                        self.app_state.selected_file =
                                            Some(entry_path.clone());
                                    }
                                    if ui
                                        .button(format!(
                                            "{} {}",
                                            ICON_SWAP_HORIZ.codepoint, "Replace Last"
                                        ))
                                        .clicked()
                                        && let Some(last_track) =
                                            self.audio_player_state.tracks.last()
                                    {
                                        self.stop(Some(last_track.id));
                                        self.play_file(&entry_path, true);
                                        self.app_state.selected_file =
                                            Some(entry_path.clone());
                                    }
                                    ui.separator();
                                    if ui
                                        .button(format!(
                                            "{} {}",
                                            ICON_OPEN_IN_BROWSER.codepoint,
                                            "Show in File Manager"
                                        ))
                                        .clicked()
                                        && let Err(e) = opener::reveal(&entry_path)
                                    {
                                        eprintln!("Failed to open file manager: {}", e);
                                    }
                                    ui.separator();
                                    if ui
                                        .button(format!(
                                            "{} {}",
                                            ICON_KEYBOARD.codepoint, "Assign Hotkey"
                                        ))
                                        .clicked()
                                    {
                                        self.app_state.assigning_hotkey_for_file =
                                            Some(entry_path.clone());
                                        self.app_state.hotkey_capture_active = true;
                                        ui.close();
                                    }
                                });
                            });
                            if self.config.show_modified_column {
                                row.col(|ui| {
                                    ui.label(
                                        RichText::new(format_mtime_opt(mtime)).monospace(),
                                    );
                                });
                            }
                        });
                    }
                });
        });
    }

    fn draw_footer(&mut self, ui: &mut Ui) {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            // ---------- Microphone selection ----------
            let mics = &self.audio_player_state.all_inputs_sorted;

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
                        ui.selectable_value(&mut selected_input, name.clone(), nick);
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

            ui.add_space(ui.available_width() - 18.0 * 2.0 - ui.spacing().item_spacing.x * 2.0);

            // ---------- Hotkeys button ----------
            let hotkeys_button =
                Button::new(ICON_KEYBOARD.atom_size(Vec2::new(18.0, 18.0))).frame(false);
            let hotkeys_button_response = ui.add_sized([18.0, 18.0], hotkeys_button);
            if hotkeys_button_response.clicked() {
                self.app_state.show_hotkeys = true;
            }
            hotkeys_button_response.on_hover_text("Hotkeys (H)");
            // --------------------------------

            // ---------- Settings button ----------
            let settings_button =
                Button::new(ICON_SETTINGS.atom_size(Vec2::new(18.0, 18.0))).frame(false);
            let settings_button_response = ui.add_sized([18.0, 18.0], settings_button);
            if settings_button_response.clicked() {
                self.app_state.show_settings = true;
            }
            // --------------------------------
        });
    }
}
