use crate::gui::SoundpadGui;
use egui::{RichText, Ui};

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
}
