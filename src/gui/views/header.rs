use crate::gui::SoundpadGui;
use egui::{Button, CollapsingHeader, FontFamily, Label, RichText, Slider, Ui};
use egui_material_icons::icons::*;
use pwsp::types::{audio_player::TrackInfo, gui::AppState};
use pwsp::utils::gui::format_time_pair;
use std::time::Instant;

pub(crate) enum TrackAction {
    Pause(u32),
    Resume(u32),
    ToggleLoop(u32),
    Stop(u32),
}

impl SoundpadGui {
    pub fn draw_header(&mut self, ui: &mut Ui) {
        ui.vertical_centered_justified(|ui| {
            if self.audio_player_state.tracks.is_empty() {
                ui.label("No tracks playing");
                return;
            }

            let mut action = None;

            for track in &self.audio_player_state.tracks {
                CollapsingHeader::new(
                    RichText::new(
                        track
                            .path
                            .file_stem()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    )
                    .family(FontFamily::Monospace),
                )
                .default_open(true)
                .show(ui, |ui| {
                    if let Some(act) = Self::draw_track_control(ui, &mut self.app_state, track) {
                        action = Some(act);
                    }
                });
                ui.separator();
            }

            if let Some(action) = action {
                match action {
                    TrackAction::Pause(id) => self.pause(Some(id)),
                    TrackAction::Resume(id) => self.resume(Some(id)),
                    TrackAction::ToggleLoop(id) => self.toggle_loop(Some(id)),
                    TrackAction::Stop(id) => self.stop(Some(id)),
                }
            }
        });
    }

    fn draw_playback_controls(ui: &mut Ui, track: &TrackInfo) -> Option<TrackAction> {
        let mut action = None;

        let play_button = Button::new(if track.paused {
            ICON_PLAY_ARROW
        } else {
            ICON_PAUSE
        })
        .corner_radius(15.0);

        if ui.add_sized([30.0, 30.0], play_button).clicked() {
            action = Some(if track.paused {
                TrackAction::Resume(track.id)
            } else {
                TrackAction::Pause(track.id)
            });
        }

        let loop_button = Button::new(
            RichText::new(if track.looped {
                ICON_REPEAT_ONE
            } else {
                ICON_REPEAT
            })
            .size(18.0),
        )
        .frame(false);

        if ui.add_sized([15.0, 30.0], loop_button).clicked() {
            action = Some(TrackAction::ToggleLoop(track.id));
        }

        action
    }

    fn draw_position_control(
        ui: &mut Ui,
        ui_state: &mut pwsp::types::gui::TrackUiState,
        track: &TrackInfo,
        default_slider_width: f32,
    ) {
        let duration = track.duration.unwrap_or(1.0);
        let position_slider = Slider::new(&mut ui_state.position_slider_value, 0.0..=duration)
            .show_value(false)
            .step_by(0.01);

        let position_slider_width = ui.available_width()
            - (30.0 * 3.0)
            - default_slider_width
            - (ui.spacing().item_spacing.x * 6.0);

        ui.spacing_mut().slider_width = position_slider_width;
        if ui.add_sized([30.0, 30.0], position_slider).drag_stopped() {
            ui_state.position_dragged = true;
        }

        let time_label =
            Label::new(RichText::new(format_time_pair(track.position, duration)).monospace());
        ui.add_sized([30.0, 30.0], time_label);
    }

    fn draw_volume_control(
        ui: &mut Ui,
        ui_state: &mut pwsp::types::gui::TrackUiState,
        track: &TrackInfo,
        default_slider_width: f32,
    ) {
        let volume_icon = Self::get_volume_icon(track.volume);
        let volume_label = Label::new(RichText::new(volume_icon).size(18.0));
        ui.add_sized([30.0, 30.0], volume_label)
            .on_hover_text(format!("Volume: {:.0}%", track.volume * 100.0));

        let volume_slider = Slider::new(&mut ui_state.volume_slider_value, 0.0..=1.0)
            .show_value(false)
            .step_by(0.01);

        ui.spacing_mut().slider_width = default_slider_width - 30.0;
        ui.spacing_mut().item_spacing.x = 0.0;

        if ui.add_sized([30.0, 30.0], volume_slider).drag_stopped() {
            ui_state.volume_dragged = true;
        }
    }

    fn draw_stop_control(ui: &mut Ui, track: &TrackInfo) -> Option<TrackAction> {
        let stop_button = Button::new(ICON_CLOSE).frame(false);
        if ui.add_sized([30.0, 30.0], stop_button).clicked() {
            Some(TrackAction::Stop(track.id))
        } else {
            None
        }
    }

    fn draw_track_control(
        ui: &mut Ui,
        app_state: &mut AppState,
        track: &TrackInfo,
    ) -> Option<TrackAction> {
        let ui_state = app_state.track_ui_states.entry(track.id).or_default();

        let should_update_position = !ui_state.position_dragged
            && ui_state
                .ignore_position_update_until
                .map(|t| Instant::now() > t)
                .unwrap_or(true);

        if should_update_position {
            ui_state.position_slider_value = track.position;
        }

        let should_update_volume = !ui_state.volume_dragged
            && ui_state
                .ignore_volume_update_until
                .map(|t| Instant::now() > t)
                .unwrap_or(true);

        if should_update_volume {
            ui_state.volume_slider_value = track.volume;
        }

        let mut action = None;

        ui.horizontal_top(|ui| {
            if let Some(act) = Self::draw_playback_controls(ui, track) {
                action = Some(act);
            }

            let default_slider_width = ui.spacing().slider_width;
            Self::draw_position_control(ui, ui_state, track, default_slider_width);
            Self::draw_volume_control(ui, ui_state, track, default_slider_width);

            if let Some(act) = Self::draw_stop_control(ui, track) {
                action = Some(act);
            }
        });

        action
    }
}
