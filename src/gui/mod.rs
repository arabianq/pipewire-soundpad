mod draw;
mod input;
mod update;

use eframe::{HardwareAcceleration, NativeOptions, icon_data::from_png_bytes, run_native};
use egui::{Context, Vec2, ViewportBuilder};
use pwsp::{
    types::{
        audio_player::PlayerState,
        config::GuiConfig,
        gui::{AppState, AudioPlayerState},
        socket::Request,
    },
    utils::{
        daemon::get_daemon_config,
        gui::{get_gui_config, make_request_sync, start_app_state_thread},
    },
};
use rfd::FileDialog;
use std::path::PathBuf;
use std::{
    error::Error,
    sync::{Arc, Mutex},
};

struct SoundpadGui {
    pub app_state: AppState,
    pub config: GuiConfig,
    pub audio_player_state: AudioPlayerState,
    pub audio_player_state_shared: Arc<Mutex<AudioPlayerState>>,
}

impl SoundpadGui {
    fn new(ctx: &Context) -> Self {
        let audio_player_state = Arc::new(Mutex::new(AudioPlayerState::default()));
        start_app_state_thread(audio_player_state.clone());

        let config = get_gui_config();

        ctx.set_zoom_factor(config.scale_factor);

        let mut soundpad_gui = SoundpadGui {
            app_state: AppState::default(),
            config: config.clone(),
            audio_player_state: AudioPlayerState::default(),
            audio_player_state_shared: audio_player_state.clone(),
        };

        soundpad_gui.app_state.dirs = config.dirs;

        soundpad_gui
    }

    pub fn play_toggle(&mut self) {
        let mut guard = self.audio_player_state_shared.lock().unwrap();
        guard.state = match guard.state {
            PlayerState::Playing => {
                make_request_sync(Request::pause()).ok();
                guard.new_state = Some(PlayerState::Paused);
                PlayerState::Paused
            }
            PlayerState::Paused => {
                make_request_sync(Request::resume()).ok();
                guard.new_state = Some(PlayerState::Playing);
                PlayerState::Playing
            }
            PlayerState::Stopped => PlayerState::Stopped,
        };
    }

    pub fn add_dir(&mut self) {
        let file_dialog = FileDialog::new();
        if let Some(path) = file_dialog.pick_folder() {
            self.app_state.dirs.insert(path);
            self.config.dirs = self.app_state.dirs.clone();
            self.config.save_to_file().ok();
        }
    }

    pub fn remove_dir(&mut self, path: &PathBuf) {
        self.app_state.dirs.remove(path);
        if let Some(current_dir) = &self.app_state.current_dir
            && current_dir == path
        {
            self.app_state.current_dir = None;
        }
        self.config.dirs = self.app_state.dirs.clone();
        self.config.save_to_file().ok();
    }

    pub fn open_dir(&mut self, path: &PathBuf) {
        self.app_state.current_dir = Some(path.clone());
        self.app_state.files = path
            .read_dir()
            .unwrap()
            .filter_map(|res| res.ok())
            .map(|entry| entry.path())
            .collect();
    }

    pub fn play_file(&mut self, path: &PathBuf) {
        make_request_sync(Request::play(path.to_str().unwrap())).ok();
    }

    pub fn set_input(&mut self, name: String) {
        make_request_sync(Request::set_input(&name)).ok();

        if self.config.save_input {
            let mut daemon_config = get_daemon_config();
            daemon_config.default_input_name = Some(name);
            daemon_config.save_to_file().ok();
        }
    }
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    const ICON: &[u8] = include_bytes!("../../assets/icon.png");

    let options = NativeOptions {
        vsync: true,
        centered: true,
        hardware_acceleration: HardwareAcceleration::Preferred,

        viewport: ViewportBuilder::default()
            .with_app_id("ru.arabianq.pwsp")
            .with_inner_size(Vec2::new(1200.0, 800.0))
            .with_min_inner_size(Vec2::new(800.0, 600.0))
            .with_icon(from_png_bytes(ICON)?),

        ..Default::default()
    };

    match run_native(
        "Pipewire Soundpad",
        options,
        Box::new(|cc| {
            egui_material_icons::initialize(&cc.egui_ctx);
            Ok(Box::new(SoundpadGui::new(&cc.egui_ctx)))
        }),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
