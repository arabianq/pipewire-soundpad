use crate::gui::SoundpadGui;
use egui::{Context, Key};

impl SoundpadGui {
    pub fn handle_input(&mut self, ctx: &Context) {
        ctx.input(|i| {
            if i.key_pressed(Key::Escape) {
                std::process::exit(0);
            }

            if !self.app_state.show_settings && i.key_pressed(Key::Space) {
                self.play_toggle();
            }

            if i.key_pressed(Key::Slash) {
                self.app_state.show_settings = !self.app_state.show_settings;
            }

            if self.app_state.show_settings && i.key_pressed(Key::Backspace) {
                self.app_state.show_settings = false;
            }
        });
    }
}
