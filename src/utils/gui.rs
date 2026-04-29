use crate::{
    types::{
        audio_player::FullState,
        config::{GuiConfig, HotkeyConfig},
        gui::AudioPlayerState,
        socket::{Request, Response},
    },
    utils::daemon::{is_daemon_running, make_request},
};
use std::{
    error::Error,
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

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::types::gui::{SortColumn, SortDir};

pub fn sort_files(
    files: &[PathBuf],
    column: SortColumn,
    dir: SortDir,
    mtimes: &HashMap<PathBuf, SystemTime>,
    hotkeys: &HashMap<PathBuf, String>,
) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = files.to_vec();

    match column {
        SortColumn::Index => {
            if dir == SortDir::Desc {
                out.reverse();
            }
        }
        SortColumn::Name => {
            out.sort_by(|a, b| compare_name(a, b));
            if dir == SortDir::Desc {
                out.reverse();
            }
        }
        SortColumn::Modified => {
            out.sort_by(|a, b| compare_optional(mtimes.get(a), mtimes.get(b), dir));
        }
        SortColumn::Hotkey => {
            out.sort_by(|a, b| compare_optional_str(hotkeys.get(a), hotkeys.get(b), dir));
        }
    }

    out
}

fn name_key(p: &Path) -> String {
    p.file_name().unwrap_or_default().to_string_lossy().to_lowercase()
}

fn compare_name(a: &Path, b: &Path) -> Ordering {
    name_key(a).cmp(&name_key(b))
}

/// Compare with "missing sorts last" semantics, regardless of asc/desc.
fn compare_optional<T: Ord>(a: Option<&T>, b: Option<&T>, dir: SortDir) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => {
            let ord = x.cmp(y);
            if dir == SortDir::Desc { ord.reverse() } else { ord }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_optional_str(a: Option<&String>, b: Option<&String>, dir: SortDir) -> Ordering {
    let a = a.filter(|s| !s.is_empty());
    let b = b.filter(|s| !s.is_empty());
    match (a, b) {
        (Some(x), Some(y)) => {
            let ord = x.to_lowercase().cmp(&y.to_lowercase());
            if dir == SortDir::Desc { ord.reverse() } else { ord }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::gui::{SortColumn, SortDir};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn pb(s: &str) -> PathBuf { PathBuf::from(s) }

    #[test]
    fn sort_index_keeps_input_order() {
        let files = vec![pb("/c/b.mp3"), pb("/c/a.mp3"), pb("/c/d.mp3")];
        let out = sort_files(&files, SortColumn::Index, SortDir::Asc, &HashMap::new(), &HashMap::new());
        assert_eq!(out, files);
    }

    #[test]
    fn sort_index_desc_reverses() {
        let files = vec![pb("/c/a.mp3"), pb("/c/b.mp3"), pb("/c/c.mp3")];
        let out = sort_files(&files, SortColumn::Index, SortDir::Desc, &HashMap::new(), &HashMap::new());
        assert_eq!(out, vec![pb("/c/c.mp3"), pb("/c/b.mp3"), pb("/c/a.mp3")]);
    }

    #[test]
    fn sort_name_case_insensitive_asc() {
        let files = vec![pb("/c/Banana.mp3"), pb("/c/apple.mp3"), pb("/c/cherry.mp3")];
        let out = sort_files(&files, SortColumn::Name, SortDir::Asc, &HashMap::new(), &HashMap::new());
        assert_eq!(out, vec![pb("/c/apple.mp3"), pb("/c/Banana.mp3"), pb("/c/cherry.mp3")]);
    }

    #[test]
    fn sort_modified_missing_goes_last_in_both_dirs() {
        let files = vec![pb("/c/a.mp3"), pb("/c/b.mp3"), pb("/c/c.mp3")];
        let mut mtimes = HashMap::new();
        mtimes.insert(pb("/c/a.mp3"), UNIX_EPOCH + Duration::from_secs(100));
        mtimes.insert(pb("/c/c.mp3"), UNIX_EPOCH + Duration::from_secs(200));
        // b is missing

        let asc = sort_files(&files, SortColumn::Modified, SortDir::Asc, &mtimes, &HashMap::new());
        assert_eq!(asc, vec![pb("/c/a.mp3"), pb("/c/c.mp3"), pb("/c/b.mp3")]);

        let desc = sort_files(&files, SortColumn::Modified, SortDir::Desc, &mtimes, &HashMap::new());
        assert_eq!(desc, vec![pb("/c/c.mp3"), pb("/c/a.mp3"), pb("/c/b.mp3")]);
    }

    #[test]
    fn sort_hotkey_empty_goes_last() {
        let files = vec![pb("/c/a.mp3"), pb("/c/b.mp3"), pb("/c/c.mp3")];
        let mut hk = HashMap::new();
        hk.insert(pb("/c/a.mp3"), "F2".to_string());
        hk.insert(pb("/c/c.mp3"), "F1".to_string());

        let asc = sort_files(&files, SortColumn::Hotkey, SortDir::Asc, &HashMap::new(), &hk);
        assert_eq!(asc, vec![pb("/c/c.mp3"), pb("/c/a.mp3"), pb("/c/b.mp3")]);

        let desc = sort_files(&files, SortColumn::Hotkey, SortDir::Desc, &HashMap::new(), &hk);
        assert_eq!(desc, vec![pb("/c/a.mp3"), pb("/c/c.mp3"), pb("/c/b.mp3")]);
    }
}
