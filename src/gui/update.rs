use crate::gui::SoundpadGui;
use eframe::{App, Frame as EFrame};
use egui::{CentralPanel, Context};
use pwsp::{
    types::socket::Request,
    utils::{daemon::get_daemon_config, gui::make_request_async},
};
use std::time::{Duration, Instant};

impl App for SoundpadGui {
    fn update(&mut self, ctx: &Context, _frame: &mut EFrame) {
        let mut seek_requests = vec![];
        let mut volume_requests = vec![];

        for (id, ui_state) in &mut self.app_state.track_ui_states {
            if ui_state.position_dragged {
                seek_requests.push((*id, ui_state.position_slider_value));
            }
            if ui_state.volume_dragged {
                volume_requests.push((*id, ui_state.volume_slider_value));
                ui_state.volume_dragged = false;
            }
        }

        for (id, pos) in seek_requests {
            make_request_async(Request::seek(pos, Some(id)));
            if let Some(ui_state) = self.app_state.track_ui_states.get_mut(&id) {
                ui_state.position_dragged = false;
                ui_state.ignore_position_update_until =
                    Some(Instant::now() + Duration::from_millis(300));
            }
        }

        for (id, vol) in volume_requests {
            make_request_async(Request::set_volume(vol, Some(id)));
            if let Some(ui_state) = self.app_state.track_ui_states.get_mut(&id) {
                ui_state.volume_dragged = false;
                ui_state.ignore_volume_update_until =
                    Some(Instant::now() + Duration::from_millis(300));
            }
        }

        if self.app_state.volume_dragged {
            make_request_async(Request::set_volume(
                self.app_state.volume_slider_value,
                None,
            ));

            self.app_state.volume_dragged = false;
            self.app_state.ignore_volume_update_until =
                Some(Instant::now() + Duration::from_millis(300));

            if self.config.save_volume {
                let mut daemon_config = get_daemon_config();
                daemon_config.default_volume = Some(self.app_state.volume_slider_value);
                daemon_config.save_to_file().ok();
            }
        }

        {
            let guard = self.audio_player_state_shared.lock().unwrap();
            self.audio_player_state = guard.clone();
        }

        let old_scale_factor = self.config.scale_factor;
        let new_scale_factor = ctx.zoom_factor().clamp(0.5, 2.0);

        ctx.set_zoom_factor(new_scale_factor);
        self.config.scale_factor = new_scale_factor;

        if new_scale_factor != old_scale_factor && self.config.save_scale_factor {
            self.config.save_to_file().ok();
        }

        self.handle_input(ctx);

        CentralPanel::default().show(ctx, |ui| {
            if !self.audio_player_state.is_daemon_running {
                self.draw_waiting_for_daemon(ui);
                return;
            }

            if self.app_state.show_settings {
                self.draw_settings(ui);
                return;
            }

            self.draw(ui).ok();
        });

        ctx.request_repaint_after_secs(1.0 / 60.0);
    }
}
