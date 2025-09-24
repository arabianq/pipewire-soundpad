use crate::{
    types::{
        audio_player::PlayerState,
        config::GuiConfig,
        gui::AudioPlayerState,
        socket::{Request, Response},
    },
    utils::daemon::{make_request, wait_for_daemon},
};
use std::{
    collections::HashMap,
    error::Error,
    path::PathBuf,
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
    futures::executor::block_on(make_request(request))
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
        let sleep_duration = Duration::from_millis(100);

        loop {
            wait_for_daemon().await.ok();

            let state_req = Request::get_state();
            let file_path_req = Request::get_current_file_path();
            let is_paused_req = Request::get_is_paused();
            let volume_req = Request::get_volume();
            let position_req = Request::get_position();
            let duration_req = Request::get_duration();
            let current_input_req = Request::get_input();
            let all_inputs_req = Request::get_inputs();

            let state_res = make_request(state_req).await.unwrap_or_default();
            let file_path_res = make_request(file_path_req).await.unwrap_or_default();
            let is_paused_res = make_request(is_paused_req).await.unwrap_or_default();
            let volume_res = make_request(volume_req).await.unwrap_or_default();
            let position_res = make_request(position_req).await.unwrap_or_default();
            let duration_res = make_request(duration_req).await.unwrap_or_default();
            let current_input_res = make_request(current_input_req).await.unwrap_or_default();
            let all_inputs_res = make_request(all_inputs_req).await.unwrap_or_default();

            let state = match state_res.status {
                true => serde_json::from_str::<PlayerState>(&state_res.message).unwrap(),
                false => PlayerState::default(),
            };

            let file_path = match file_path_res.status {
                true => PathBuf::from(file_path_res.message),
                false => PathBuf::new(),
            };
            let is_paused = match is_paused_res.status {
                true => is_paused_res.message == "true",
                false => false,
            };
            let volume = match volume_res.status {
                true => volume_res.message.parse::<f32>().unwrap(),
                false => 0.0,
            };
            let position = match position_res.status {
                true => position_res.message.parse::<f32>().unwrap(),
                false => 0.0,
            };
            let duration = match duration_res.status {
                true => duration_res.message.parse::<f32>().unwrap(),
                false => 0.0,
            };
            let current_input = match current_input_res.status {
                true => current_input_res
                    .message
                    .as_str()
                    .split(" - ")
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap()
                    .to_string()
                    .parse::<u32>()
                    .unwrap_or_default(),
                false => 0,
            };
            let all_inputs = match all_inputs_res.status {
                true => all_inputs_res
                    .message
                    .as_str()
                    .split(';')
                    .filter_map(|entry| {
                        let entry = entry.trim();
                        if entry.is_empty() {
                            return None;
                        }
                        entry.split_once(" - ").and_then(|(k, v)| {
                            k.trim()
                                .parse::<u32>()
                                .ok()
                                .map(|key| (key, v.trim().to_string()))
                        })
                    })
                    .collect(),
                false => HashMap::new(),
            };

            {
                let mut guard = audio_player_state_shared.lock().unwrap();

                guard.state = match guard.new_state.clone() {
                    Some(new_state) => {
                        guard.new_state = None;
                        new_state
                    }
                    None => state,
                };
                guard.current_file_path = file_path;
                guard.is_paused = is_paused;
                guard.volume = match guard.new_volume {
                    Some(new_volume) => {
                        guard.new_volume = None;
                        new_volume
                    }
                    None => volume,
                };
                guard.position = match guard.new_position {
                    Some(new_position) => {
                        guard.new_position = None;
                        new_position
                    }
                    None => position,
                };
                guard.duration = if duration > 0.0 { duration } else { 1.0 };
                guard.current_input = current_input;
                guard.all_inputs = all_inputs;
            }

            sleep(sleep_duration).await;
        }
    });
}
