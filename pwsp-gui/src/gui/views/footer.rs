use crate::gui::SoundpadGui;
use egui::{AtomExt, Button, ComboBox, Label, RichText, Slider, Ui, Vec2};
use egui_material_icons::icons::*;
use pwsp_lib::types::gui::SliderLatch;
use rust_i18n::t;

/// Masters go past 100% on purpose: amplifying the microphone feed without deafening
/// yourself is the reason the two paths were split in the first place.
const MAX_MASTER_VOLUME: f32 = 2.0;
const VOLUME_SLIDER_WIDTH: f32 = 110.0;

impl SoundpadGui {
    pub fn draw_footer(&mut self, ui: &mut Ui) {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            self.draw_monitoring_volume(ui);
            self.draw_mic_volume(ui);

            ui.add_space(ui.available_width() - 18.0 * 2.0 - ui.spacing().item_spacing.x * 2.0);

            self.draw_hotkeys_button(ui);
            self.draw_settings_button(ui);
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            self.draw_mic_selection(ui);
            self.draw_output_selection(ui);
        });
    }

    fn draw_mic_selection(&mut self, ui: &mut Ui) {
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
    }

    fn draw_output_selection(&mut self, ui: &mut Ui) {
        let outputs = &self.audio_player_state.all_outputs_sorted;

        let mut selected_output = self.audio_player_state.current_output.to_owned();
        let prev_output = selected_output.to_owned();

        // An empty selection means no device is pinned and playback follows the system
        // default sink, which is also what a fresh install does.
        let selected_text = self
            .audio_player_state
            .all_outputs
            .get(&selected_output)
            .cloned()
            .unwrap_or_else(|| t!("gui.default_output").to_string());

        ComboBox::from_label(t!("gui.choose_output_select"))
            .height(30.0)
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for (name, nick) in outputs {
                    ui.selectable_value(&mut selected_output, name.clone(), nick);
                }
            });

        if selected_output != prev_output {
            self.set_output(selected_output);
        }
    }

    fn draw_monitoring_volume(&mut self, ui: &mut Ui) {
        let volume = self.audio_player_state.monitoring_volume;
        let icon = Self::get_volume_icon(volume);
        ui.add_sized([18.0, 18.0], Label::new(RichText::new(icon).size(18.0)))
            .on_hover_text(format!(
                "{}: {:.0}%",
                t!("gui.monitoring_volume"),
                volume * 100.0
            ));

        Self::draw_volume_slider(ui, &mut self.app_state.monitoring_volume, volume);
    }

    fn draw_mic_volume(&mut self, ui: &mut Ui) {
        let volume = self.audio_player_state.mic_volume;
        let icon = if volume <= 0.0 {
            ICON_MIC_OFF.codepoint
        } else {
            ICON_MIC.codepoint
        };
        ui.add_sized([18.0, 18.0], Label::new(RichText::new(icon).size(18.0)))
            .on_hover_text(format!("{}: {:.0}%", t!("gui.mic_volume"), volume * 100.0));

        Self::draw_volume_slider(ui, &mut self.app_state.mic_volume, volume);
    }

    fn draw_volume_slider(ui: &mut Ui, latch: &mut SliderLatch, daemon_value: f32) {
        latch.sync(daemon_value);

        let slider = Slider::new(&mut latch.value, 0.0..=MAX_MASTER_VOLUME)
            .show_value(false)
            .step_by(0.01);
        if ui
            .add_sized([VOLUME_SLIDER_WIDTH, 18.0], slider)
            .drag_stopped()
        {
            latch.dragged = true;
        }
    }

    fn draw_hotkeys_button(&mut self, ui: &mut Ui) {
        let hotkeys_button =
            Button::new(ICON_KEYBOARD.atom_size(Vec2::new(18.0, 18.0))).frame(false);
        let hotkeys_button_response = ui.add_sized([18.0, 18.0], hotkeys_button);
        if hotkeys_button_response.clicked() {
            self.app_state.show_hotkeys = true;
        }
        hotkeys_button_response.on_hover_text("Hotkeys (H)");
    }

    fn draw_settings_button(&mut self, ui: &mut Ui) {
        let settings_button =
            Button::new(ICON_SETTINGS.atom_size(Vec2::new(18.0, 18.0))).frame(false);
        let settings_button_response = ui.add_sized([18.0, 18.0], settings_button);
        if settings_button_response.clicked() {
            self.app_state.show_settings = true;
        }
    }
}
