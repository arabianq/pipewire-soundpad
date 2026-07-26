use crate::types::{
    audio_player::{PlayerState, TrackInfo},
    config::HotkeyConfig,
};

use egui::Id;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub type ScanResult = (PathBuf, Vec<PathBuf>, HashMap<PathBuf, Vec<PathBuf>>);

/// How long the daemon's value is ignored after a slider commits, so the knob does not
/// snap back before the daemon has caught up.
const SETTLE_TIME: Duration = Duration::from_millis(300);

/// A slider whose local value wins over the daemon's while the user is interacting.
///
/// The daemon is polled at 60 Hz, so without this the value would fight the user mid-drag
/// and jump back right after release.
#[derive(Default, Debug)]
pub struct SliderLatch {
    pub value: f32,
    pub dragged: bool,
    pub ignore_update_until: Option<Instant>,
}

impl SliderLatch {
    /// Whether the daemon's value may overwrite the local one this frame.
    pub fn should_sync(&self) -> bool {
        !self.dragged
            && self
                .ignore_update_until
                .is_none_or(|until| Instant::now() > until)
    }

    /// Adopts the daemon's value unless the user is currently driving the slider.
    pub fn sync(&mut self, value: f32) {
        if self.should_sync() {
            self.value = value;
        }
    }

    /// Called once the pending change has been sent to the daemon.
    pub fn commit(&mut self) {
        self.dragged = false;
        self.ignore_update_until = Some(Instant::now() + SETTLE_TIME);
    }

    /// Returns the value to send, if the user finished a change that has not been sent yet.
    pub fn take_pending(&mut self) -> Option<f32> {
        if self.dragged {
            let value = self.value;
            self.commit();
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Default, Debug)]
pub struct TrackUiState {
    pub position: SliderLatch,
    pub volume: SliderLatch,
}

#[derive(Default, Debug)]
pub struct AppState {
    pub search_query: String,

    pub track_ui_states: HashMap<u32, TrackUiState>,

    pub show_settings: bool,
    pub force_focus_search: bool,

    pub monitoring_volume: SliderLatch,
    pub mic_volume: SliderLatch,
    pub volume_multiplier: SliderLatch,

    pub search_field_id: Option<Id>,

    pub current_dir: Option<PathBuf>,
    pub dirs: Vec<PathBuf>,
    pub dirs_to_remove: HashSet<PathBuf>,

    pub listed_files: HashSet<PathBuf>,
    pub listed_dirs: HashSet<PathBuf>,
    pub dir_cache: HashMap<PathBuf, Vec<PathBuf>>,

    pub scanning_dirs: Arc<Mutex<HashSet<PathBuf>>>,
    pub scanned_this_session: HashSet<PathBuf>,
    pub finished_scans: Arc<Mutex<Vec<ScanResult>>>,
    pub recursive_files_cache: HashMap<PathBuf, Vec<PathBuf>>,

    pub show_hotkeys: bool,
    pub hotkey_capture_active: bool,

    pub hotkey_config: HotkeyConfig,
    pub hotkey_search_query: String,

    pub assigning_hotkey_slot: Option<String>,
    pub assigning_hotkey_for_file: Option<PathBuf>,
}

#[derive(Default, Debug, Clone)]
pub struct AudioPlayerState {
    pub state: PlayerState,
    pub new_state: Option<PlayerState>,

    pub tracks: Vec<TrackInfo>,

    /// What the user hears locally.
    pub monitoring_volume: f32,
    /// What is sent to the virtual microphone.
    pub mic_volume: f32,
    pub volume_multiplier: f32,

    pub current_input: String,
    pub all_inputs: HashMap<String, String>,
    pub all_inputs_sorted: Vec<(String, String)>,

    /// Empty means "follow the system default sink".
    pub current_output: String,
    pub all_outputs: HashMap<String, String>,
    pub all_outputs_sorted: Vec<(String, String)>,

    pub is_daemon_running: bool,

    pub hotkey_config: Option<HotkeyConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_latch_syncs_when_idle() {
        let mut latch = SliderLatch::default();

        latch.sync(0.7);
        assert_eq!(latch.value, 0.7);

        // While the user drags, the daemon must not move the knob out from under them.
        latch.dragged = true;
        latch.sync(0.2);
        assert_eq!(latch.value, 0.7);
    }

    #[test]
    fn test_slider_latch_take_pending() {
        let mut latch = SliderLatch::default();

        // Nothing to send until the user actually changes something.
        assert_eq!(latch.take_pending(), None);

        latch.value = 1.5;
        latch.dragged = true;
        assert_eq!(latch.take_pending(), Some(1.5));

        // The change is sent exactly once.
        assert_eq!(latch.take_pending(), None);

        // And the daemon is ignored for a moment, so the slider does not snap back to a
        // stale value that was already in flight when we sent ours.
        assert!(!latch.should_sync());
        latch.sync(0.1);
        assert_eq!(latch.value, 1.5);
    }
}
