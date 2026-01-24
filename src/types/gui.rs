use crate::types::audio_player::{PlayerState, TrackInfo};

use egui::Id;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::Instant,
};

#[derive(Default, Debug)]
pub struct TrackUiState {
    pub position_slider_value: f32,
    pub volume_slider_value: f32,
    pub position_dragged: bool,
    pub volume_dragged: bool,
    pub ignore_position_update_until: Option<Instant>,
    pub ignore_volume_update_until: Option<Instant>,
}

#[derive(Default, Debug)]
pub struct AppState {
    pub search_query: String,

    pub track_ui_states: HashMap<u32, TrackUiState>,

    pub show_settings: bool,

    pub current_dir: Option<PathBuf>,
    pub dirs: HashSet<PathBuf>,

    pub selected_file: Option<PathBuf>,
    pub files: HashSet<PathBuf>,

    pub search_field_id: Option<Id>,
    pub force_focus_id: Option<Id>,
}

#[derive(Default, Debug, Clone)]
pub struct AudioPlayerState {
    pub state: PlayerState,
    pub new_state: Option<PlayerState>,

    pub tracks: Vec<TrackInfo>,

    pub current_file_path: PathBuf,

    pub is_paused: bool,
    pub looped: bool,

    pub volume: f32,
    pub new_volume: Option<f32>,
    pub position: f32,
    pub new_position: Option<f32>,
    pub duration: f32,

    pub current_input: String,
    pub all_inputs: HashMap<String, String>,
}
