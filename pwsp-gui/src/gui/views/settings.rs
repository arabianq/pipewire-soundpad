use crate::gui::SoundpadGui;
use egui::{Align, Button, Color32, ComboBox, Layout, RichText, Slider, Ui};
use egui_material_icons::icons::ICON_ARROW_BACK;
use pwsp_lib::types::config::PreferredTheme;
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
            let save_volume_multiplier_response = ui.checkbox(
                &mut self.config.save_volume_multiplier,
                t!("gui.settings.remember_volume_multiplier"),
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
                || save_volume_multiplier_response.changed()
                || save_input_response.changed()
                || save_scale_response.changed()
                || pause_on_exit_response.changed()
            {
                self.config.save_to_file().ok();
            }
            // --------------------------------

            ui.separator();

            // ---------- Selectors -----------
            self.draw_mic_selection(ui);
            self.draw_output_selection(ui);

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

            ui.separator();

            // ----------- Sliders ------------
            // Volume multiplier
            self.app_state
                .volume_multiplier
                .sync(self.audio_player_state.volume_multiplier);

            ui.horizontal(|ui| {
                let slider = Slider::new(&mut self.app_state.volume_multiplier.value, 0.01..=3.0);
                let response = ui.add(slider);
                ui.label(t!("gui.settings.volume_multiplier"));

                if response.changed() {
                    // This condition is required to avoid spamming requests while dragging, but to allow changing the value via TextEdit
                    if !response.dragged() || (response.dragged() && response.drag_stopped()) {
                        self.app_state.volume_multiplier.dragged = true;
                    }
                }
            });
            // --------------------------------

            ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                ui.label(t!(
                    "gui.settings.version",
                    version = env!("CARGO_PKG_VERSION")
                ));
            });
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

        // An empty selection means no device is pinned.
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
                // Listed first so pinning a device stays undoable.
                ui.selectable_value(
                    &mut selected_output,
                    String::new(),
                    t!("gui.default_output"),
                );
                for (name, nick) in outputs {
                    ui.selectable_value(&mut selected_output, name.clone(), nick);
                }
            });

        if selected_output != prev_output {
            self.set_output(selected_output);
        }
    }
}
