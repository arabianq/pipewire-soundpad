use crate::gui::SoundpadGui;
use egui::Ui;
use egui_material_icons::icons::*;

mod body;
mod footer;
mod header;
mod hotkey_capture;
mod hotkeys;
mod settings;
mod waiting_for_daemon;

impl SoundpadGui {
    pub(crate) fn get_volume_icon(volume: f32) -> &'static str {
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
}
