use crate::gui::SoundpadGui;
use egui::{Button, Color32, Label, RichText, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};
use egui_material_icons::icons::*;
use pwsp_lib::types::socket::Request;
use pwsp_lib::utils::gui::make_request_async;
use rust_i18n::t;
use std::path::Path;

pub(crate) enum HotkeyAction {
    Remove(String),
    Capture(String),
    ClearChord(String),
    Play(String),
}

impl SoundpadGui {
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
                ui.label(
                    RichText::new(t!("gui.hotkeys.header"))
                        .color(Color32::WHITE)
                        .monospace(),
                );
            });
        });
    }

    fn draw_hotkeys_search(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.menu_button(
                format!(
                    "{} {}",
                    ICON_ADD.codepoint,
                    t!("gui.hotkeys.add_command_select")
                ),
                |ui| {
                    let mut selected_cmd = None;
                    if ui.button(t!("gui.hotkeys.toggle_pause_command")).clicked() {
                        selected_cmd = Some(("cmd_toggle_pause", Request::toggle_pause(None)));
                    }
                    if ui.button(t!("gui.hotkeys.stop_playback_command")).clicked() {
                        selected_cmd = Some(("cmd_stop", Request::stop(None)));
                    }
                    if ui
                        .button(t!("gui.hotkeys.pause_playback_command"))
                        .clicked()
                    {
                        selected_cmd = Some(("cmd_pause", Request::pause(None)));
                    }
                    if ui
                        .button(t!("gui.hotkeys.resume_playback_command"))
                        .clicked()
                    {
                        selected_cmd = Some(("cmd_resume", Request::resume(None)));
                    }
                    if ui.button(t!("gui.hotkeys.toggle_loop_command")).clicked() {
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
                },
            );

            ui.add_space(10.0);

            ui.add(
                TextEdit::singleline(&mut self.app_state.hotkey_search_query)
                    .hint_text(t!("gui.hotkeys.search_placeholder"))
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
                        RichText::new(t!("gui.hotkeys.column_slot"))
                            .strong()
                            .monospace(),
                    );
                });
                header.col(|ui| {
                    ui.label(
                        RichText::new(t!("gui.hotkeys.column_sound"))
                            .strong()
                            .monospace(),
                    );
                });
                header.col(|ui| {
                    ui.label(
                        RichText::new(t!("gui.hotkeys.column_key_chord"))
                            .strong()
                            .monospace(),
                    );
                });
                header.col(|ui| {
                    ui.label(
                        RichText::new(t!("gui.hotkeys.column_actions"))
                            .strong()
                            .monospace(),
                    );
                });
            })
            .body(|mut body| {
                if slots.is_empty() {
                    body.row(30.0, |mut row| {
                        row.col(|_| {});
                        row.col(|ui| {
                            ui.label(RichText::new(t!("gui.hotkeys.no_hotkeys_configured")));
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
                                    Label::new(RichText::new(&slot.slot).monospace()).truncate(),
                                );
                            });
                        });

                        // Column 2: Sound / Action name
                        row.col(|ui| {
                            let action_name = match slot.action.name.as_str() {
                                "play" => {
                                    if let Some(file_path_str) = slot.action.args.get("file_path") {
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
                            ui.add(Label::new(RichText::new(action_name).monospace()).truncate());
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
}
