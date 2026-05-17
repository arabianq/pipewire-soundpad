use crate::gui::SoundpadGui;
use egui::{Color32, RichText, Ui};
use rust_i18n::t;

impl SoundpadGui {
    pub fn draw_hotkey_capture(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.label(
                RichText::new(t!("gui.hotkeys.capture.header"))
                    .size(18.0)
                    .color(Color32::YELLOW)
                    .monospace(),
            );
            ui.add_space(10.0);
            let target = if let Some(slot) = &self.app_state.assigning_hotkey_slot {
                format!("{} '{}'", t!("gui.hotkeys.capture.for"), slot)
            } else if let Some(path) = &self.app_state.assigning_hotkey_for_file {
                format!(
                    "{} '{}'",
                    t!("gui.hotkeys.capture.for"),
                    path.file_name().unwrap_or_default().to_string_lossy()
                )
            } else {
                String::new()
            };
            ui.label(RichText::new(target).size(16.0));
            ui.add_space(10.0);
            ui.label(t!("gui.hotkeys.capture.cancel"));
        });
    }
}
