use crate::gui::SoundpadGui;
use egui::{AtomExt, Button, Label, RichText, Slider, Ui, Vec2};
use egui_material_icons::icons::*;
use pwsp_lib::types::gui::SliderLatch;
use rust_i18n::t;

/// Masters go past 100% on purpose: amplifying the microphone feed without deafening
/// yourself is the reason the two paths were split in the first place.
const MAX_MASTER_VOLUME: f32 = 2.0;
const VOLUME_SLIDER_WIDTH: f32 = 90.0;
const ICON_SIZE: f32 = 18.0;

impl SoundpadGui {
    pub fn draw_footer(&mut self, ui: &mut Ui) {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            self.draw_monitoring_volume(ui);
            self.draw_mic_volume(ui);

            // Pushes the two icon buttons to the right edge. Clamped because a window
            // narrow enough to make this negative would otherwise shove them out of
            // view instead of just crowding them.
            let spacer = ui.available_width() - ICON_SIZE * 2.0 - ui.spacing().item_spacing.x * 2.0;
            ui.add_space(spacer.max(0.0));

            self.draw_hotkeys_button(ui);
            self.draw_settings_button(ui);
        });
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

        // A Slider draws its rail at spacing().slider_width no matter what add_sized
        // allocates, so both have to be told the same number or the widget overruns the
        // space reserved for it.
        ui.spacing_mut().slider_width = VOLUME_SLIDER_WIDTH;

        let slider = Slider::new(&mut latch.value, 0.0..=MAX_MASTER_VOLUME)
            .show_value(false)
            .step_by(0.01);
        if ui
            .add_sized([VOLUME_SLIDER_WIDTH, ICON_SIZE], slider)
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
