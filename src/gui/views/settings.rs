use crate::gui::SoundpadGui;
use egui::{Align, Button, Color32, ComboBox, Layout, RichText, Ui};
use egui_material_icons::icons::ICON_ARROW_BACK;
use pwsp::types::config::PreferredTheme;
use rust_i18n::t;

impl SoundpadGui {
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

                ui.label(
                    RichText::new(t!("gui.settings.header"))
                        .color(Color32::WHITE)
                        .monospace(),
                );
            });
            // --------------------------------

            ui.separator();
            ui.add_space(20.0);

            // --------- Checkboxes ----------
            let save_volume_response = ui.checkbox(
                &mut self.config.save_volume,
                t!("gui.settings.remember_volume"),
            );
            let save_input_response =
                ui.checkbox(&mut self.config.save_input, t!("gui.settings.remember_mic"));
            let save_scale_response = ui.checkbox(
                &mut self.config.save_scale_factor,
                t!("gui.settings.remember_ui_scale"),
            );
            let pause_on_exit_response = ui.checkbox(
                &mut self.config.pause_on_exit,
                t!("gui.settings.pause_on_window_close"),
            );

            if save_volume_response.changed()
                || save_input_response.changed()
                || save_scale_response.changed()
                || pause_on_exit_response.changed()
            {
                self.config.save_to_file().ok();
            }
            // --------------------------------

            ui.separator();

            // ---------- Selectors -----------
            let mut selected_theme = self.config.preferred_theme.clone();
            ComboBox::from_label(t!("gui.settings.theme.label"))
                .selected_text(match self.config.preferred_theme {
                    PreferredTheme::System => t!("gui.settings.theme.system"),
                    PreferredTheme::Light => t!("gui.settings.theme.light"),
                    PreferredTheme::Dark => t!("gui.settings.theme.dark"),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut selected_theme,
                        PreferredTheme::System,
                        t!("gui.settings.theme.system"),
                    );
                    ui.selectable_value(
                        &mut selected_theme,
                        PreferredTheme::Light,
                        t!("gui.settings.theme.light"),
                    );
                    ui.selectable_value(
                        &mut selected_theme,
                        PreferredTheme::Dark,
                        t!("gui.settings.theme.dark"),
                    );
                });

            if selected_theme != self.config.preferred_theme {
                self.config.preferred_theme = selected_theme;
                self.config.save_to_file().ok();
            }
            // --------------------------------

            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.label(t!(
                    "gui.settings.version",
                    version = env!("CARGO_PKG_VERSION")
                ));
            });
        });
    }
}
