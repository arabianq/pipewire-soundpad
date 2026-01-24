use crate::{
    types::{
        audio_player::{PlayerState, TrackInfo},
        config::GuiConfig,
        gui::AudioPlayerState,
        socket::{Request, Response},
    },
    utils::daemon::{make_request, wait_for_daemon},
};
use std::{
    collections::HashMap,
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
            wait_for_daemon().await.ok();

            let state_req = Request::get_state();
            let tracks_req = Request::get_tracks();
            let current_input_req = Request::get_input();
            let all_inputs_req = Request::get_inputs();

            let (state_res, tracks_res, current_input_res, all_inputs_res) = tokio::join!(
                make_request(state_req),
                make_request(tracks_req),
                make_request(current_input_req),
                make_request(all_inputs_req),
            );

            let state_res = state_res.unwrap_or_default();
            let tracks_res = tracks_res.unwrap_or_default();
            let current_input_res = current_input_res.unwrap_or_default();
            let all_inputs_res = all_inputs_res.unwrap_or_default();

            let state = match state_res.status {
                true => serde_json::from_str::<PlayerState>(&state_res.message).unwrap(),
                false => PlayerState::default(),
            };

            let tracks = match tracks_res.status {
                true => {
                    serde_json::from_str::<Vec<TrackInfo>>(&tracks_res.message).unwrap_or_default()
                }
                false => vec![],
            };

            let current_input = match current_input_res.status {
                true => current_input_res
                    .message
                    .as_str()
                    .split(" - ")
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap()
                    .to_string(),
                false => String::new(),
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
                        entry
                            .split_once(" - ")
                            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
                    })
                    .collect::<HashMap<String, String>>(),
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
                guard.tracks = tracks.clone();

                guard.current_input = current_input;
                guard.all_inputs = all_inputs;
            }

            sleep(sleep_duration).await;
        }
    });
}
