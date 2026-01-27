use crate::{
    types::{
        audio_player::FullState,
        config::GuiConfig,
        gui::AudioPlayerState,
        socket::{Request, Response},
    },
    utils::daemon::{is_daemon_running, make_request},
};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use tokio::time::{Duration, sleep};

pub fn get_gui_config() -> GuiConfig {
    GuiConfig::load_from_file().unwrap_or_else(|_| {
        let mut config = GuiConfig::default();
        config.save_to_file().ok();
        config
    })
}

pub fn make_request_sync(request: Request) -> Result<Response, Box<dyn Error>> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(make_request(request))
            .map_err(|e| e as Box<dyn Error>)
    })
}

pub fn make_request_async(request: Request) {
    tokio::spawn(async move {
        make_request(request).await.ok();
    });
}

pub fn format_time_pair(position: f32, duration: f32) -> String {
    fn format_time(seconds: f32) -> String {
        let total_seconds = seconds.round() as u32;
        let minutes = total_seconds / 60;
        let secs = total_seconds % 60;
        format!("{:02}:{:02}", minutes, secs)
    }

    format!("{}/{}", format_time(position), format_time(duration))
}

pub fn start_app_state_thread(audio_player_state_shared: Arc<Mutex<AudioPlayerState>>) {
    tokio::spawn(async move {
        let sleep_duration = Duration::from_secs_f32(1.0 / 60.0);

        loop {
            let is_running = is_daemon_running().unwrap_or(false);

            if !is_running {
                {
                    let mut guard = audio_player_state_shared.lock().unwrap();
                    guard.is_daemon_running = false;
                }
                sleep(Duration::from_millis(500)).await;
                continue;
            }

            let full_state_req = Request::get_full_state();
            let full_state_res = make_request(full_state_req).await.unwrap_or_default();

            if full_state_res.status {
                let full_state: FullState =
                    serde_json::from_str(&full_state_res.message).unwrap_or_default();

                let mut guard = audio_player_state_shared.lock().unwrap();

                guard.state = match guard.new_state.clone() {
                    Some(new_state) => {
                        guard.new_state = None;
                        new_state
                    }
                    None => full_state.state,
                };
                guard.tracks = full_state.tracks;
                guard.volume = full_state.volume;
                guard.current_input = full_state
                    .current_input
                    .split(" - ")
                    .next()
                    .unwrap_or_default()
                    .to_string();
                guard.all_inputs = full_state.all_inputs;
                guard.is_daemon_running = true;
            }

            sleep(sleep_duration).await;
        }
    });
}
