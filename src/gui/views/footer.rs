use crate::gui::SoundpadGui;
use egui::{AtomExt, Button, ComboBox, Label, RichText, Slider, Ui, Vec2};
use egui_material_icons::icons::*;
use rust_i18n::t;
use std::time::Instant;

impl SoundpadGui {
    pub fn draw_footer(&mut self, ui: &mut Ui) {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            // ---------- Microphone selection ----------
            let mics = &self.audio_player_state.all_inputs_sorted;

            let mut selected_input = self.audio_player_state.current_input.to_owned();
            let prev_input = selected_input.to_owned();
            ComboBox::from_label(t!("gui.choose_mic_select"))
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
