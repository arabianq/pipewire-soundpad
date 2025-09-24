use crate::types::audio_player::PlayerState;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Default, Debug)]
pub struct AppState {
    pub search_query: String,

    pub position_slider_value: f32,
    pub volume_slider_value: f32,

    pub position_dragged: bool,
    pub volume_dragged: bool,

    pub show_settings: bool,

    pub current_dir: Option<PathBuf>,
    pub dirs: HashSet<PathBuf>,
}

#[derive(Default, Debug, Clone)]
pub struct AudioPlayerState {
    pub state: PlayerState,
    pub new_state: Option<PlayerState>,
    pub current_file_path: PathBuf,

    pub is_paused: bool,

    pub volume: f32,
    pub new_volume: Option<f32>,
    pub position: f32,
    pub new_position: Option<f32>,
    pub duration: f32,

    pub current_input: u32,
    pub all_inputs: HashMap<u32, String>,
}
