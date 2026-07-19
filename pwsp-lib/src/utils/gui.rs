use crate::{
    types::{
        audio_player::FullState,
        config::{GuiConfig, HotkeyConfig},
        gui::AudioPlayerState,
        socket::{Request, Response},
    },
    utils::daemon::{is_daemon_running, make_request},
};
use anyhow::{Result, anyhow};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::time::{Duration, sleep};

pub fn get_gui_config() -> GuiConfig {
    GuiConfig::load_from_file().unwrap_or_else(|_| {
        let mut config = GuiConfig::default();
        config.save_to_file().ok();
        config
    })
}

pub fn make_request_sync(request: Request) -> Result<Response> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(make_request(request))
            .map_err(|e| anyhow!(e))
    })
}

pub fn make_request_async(request: Request) {
    tokio::spawn(async move {
        make_request(request).await.ok();
    });
}

pub fn ensure_pwsp_audio_dir() -> PathBuf {
    let audio_dir = dirs::audio_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .map(|p| p.join("Music"))
            .unwrap_or_else(|| "Music".into()) // already relative to $HOME afaik
    });
    let pwsp_audio_dir = audio_dir.join("PWSP");

    if !pwsp_audio_dir.exists() {
        std::fs::create_dir_all(&pwsp_audio_dir).ok();
    }

    pwsp_audio_dir
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
        let mut last_hotkey_poll = Instant::now();

        loop {
            let is_running = is_daemon_running().unwrap_or(false);

            if !is_running {
                {
                    let mut guard = audio_player_state_shared
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
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

                let mut guard = audio_player_state_shared
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());

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

                if guard.all_inputs != full_state.all_inputs {
                    guard.all_inputs = full_state.all_inputs;
                    let mut sorted: Vec<(String, String)> = guard
                        .all_inputs
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    sorted.sort_by(|a, b| a.0.cmp(&b.0));
                    guard.all_inputs_sorted = sorted;
                }

                guard.is_daemon_running = true;
            }

            // Poll hotkey config at a lower frequency (~every 2 seconds)
            if last_hotkey_poll.elapsed() >= Duration::from_secs(2) {
                let hotkey_res = make_request(Request::get_hotkeys())
                    .await
                    .unwrap_or_default();
                if hotkey_res.status
                    && let Ok(config) = serde_json::from_str::<HotkeyConfig>(&hotkey_res.message)
                {
                    let mut guard = audio_player_state_shared
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    guard.hotkey_config = Some(config);
                }
                last_hotkey_poll = Instant::now();
            }

            sleep(sleep_duration).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time_pair() {
        assert_eq!(format_time_pair(0.0, 0.0), "00:00/00:00");
        assert_eq!(format_time_pair(5.4, 10.0), "00:05/00:10");
        assert_eq!(format_time_pair(59.9, 125.1), "01:00/02:05");
        assert_eq!(format_time_pair(3600.0, 7205.0), "60:00/120:05");
    }
}
