mod pw;
pub mod settings;

pub use settings::Settings;

use eframe::{CreationContext, Frame, NativeOptions};
use egui::{
    Button, CentralPanel, ComboBox, Context, Label, ScrollArea, Separator, Slider, TextEdit, Ui,
    Vec2,
};
use egui_material_icons::icons;

use metadata::media_file::MediaFileMetadata;
use rfd::FileDialog;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::{fs, path::PathBuf};

#[derive(PartialEq, Default)]
enum PlayerState {
    Playing,
    #[default]
    Paused,
}

pub struct App {
    saved_settings: Settings,

    player_position: f32,
    prev_player_position: f32,
    max_player_position: f32,

    volume: f32,

    player_state: PlayerState,

    directories: Vec<PathBuf>,
    directory_to_delete: Option<usize>,
    current_directory: Option<usize>,

    selected_input_device: String,
    available_input_devices: Vec<pw::InputDevice>,

    current_file: PathBuf,

    search_query: String,

    _audio_stream_handle: OutputStream,
    audio_sink: Sink,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            // Render UI
            self.render_ui(ui);

            // Load all available input devices
            self.available_input_devices = pw::get_input_devices().unwrap_or_default();

            // Save new settings
            let current_settings = Settings {
                saved_dirs: self.directories.clone(),
                saved_mic: self.selected_input_device.clone(),
                saved_volume: self.volume,
            };
            if current_settings != self.saved_settings {
                current_settings.save_to_file(
                    &dirs::config_dir()
                        .unwrap_or_default()
                        .join("pwsp")
                        .join("pwsp.json"),
                );
                self.saved_settings = current_settings;
            }

            // Pause audio_sink on audio end
            if self.audio_sink.len() == 0 && !self.audio_sink.is_paused() {
                self.audio_sink.pause();
            }

            // Change player_state based on audio_sink state
            self.player_state = match self.audio_sink.is_paused() {
                true => PlayerState::Paused,
                false => PlayerState::Playing,
            };

            // Handle changing player position
            if self.player_position != self.prev_player_position {
                let mut target_pos = self.player_position;
                target_pos = target_pos.clamp(0.0, self.max_player_position - 0.1);

                let target_pos_dur = core::time::Duration::from_secs_f32(target_pos);
                self.audio_sink.try_seek(target_pos_dur).unwrap();
                self.prev_player_position = self.player_position;
            }

            // Update UI when playing
            if self.player_state == PlayerState::Playing {
                ctx.request_repaint();
                self.audio_sink.set_volume(self.volume);
                self.player_position = self.audio_sink.get_pos().as_secs_f32();
            }

            self.prev_player_position = self.player_position;

            if (self.max_player_position - self.player_position) <= 0.1 {
                self.audio_sink.pause();
            }
        });
    }
}

impl App {
    pub fn new(_cc: &CreationContext<'_>, settings: Settings) -> Self {
        let _audio_stream_handle = OutputStreamBuilder::open_default_stream().unwrap();
        let audio_sink = Sink::connect_new(_audio_stream_handle.mixer());
        audio_sink.pause();

        Self {
            saved_settings: settings.clone(),

            player_position: 0.0,
            prev_player_position: 0.0,
            max_player_position: 1.0,
            volume: settings.saved_volume,

            player_state: PlayerState::Paused,

            directories: settings.saved_dirs.clone(),
            directory_to_delete: None,
            current_directory: match settings.saved_dirs.len() {
                0 => None,
                _ => Some(0),
            },

            selected_input_device: settings.saved_mic,
            available_input_devices: Vec::new(),

            current_file: PathBuf::new(),

            search_query: String::new(),

            _audio_stream_handle,
            audio_sink,
        }
    }

    fn render_ui(&mut self, ui: &mut Ui) {
        self.render_player(ui);

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        self.render_content(ui);
    }

    fn render_player(&mut self, ui: &mut Ui) {
        let file_title_label = Label::new(
            self.current_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(""),
        );

        let play_button_icon = match self.player_state {
            PlayerState::Playing => icons::ICON_PAUSE,
            PlayerState::Paused => icons::ICON_PLAY_ARROW,
        };
        let play_button = Button::new(play_button_icon).corner_radius(15.0);

        let position_minutes = (self.player_position / 60.0) as i32;
        let position_seconds = (self.player_position - (position_minutes as f32) * 60.0) as i32;
        let position_minutes_str = match position_minutes.to_string().chars().count() {
            1 => "0".to_string() + &position_minutes.to_string(),
            2 => position_minutes.to_string(),
            _ => "00".to_string(),
        };
        let position_seconds_str = match position_seconds.to_string().chars().count() {
            1 => "0".to_string() + &position_seconds.to_string(),
            2 => position_seconds.to_string(),
            _ => "00".to_string(),
        };
        let player_position_label =
            Label::new(format!("{}:{}", position_minutes_str, position_seconds_str));

        let player_position_slider =
            Slider::new(&mut self.player_position, 0.0..=self.max_player_position)
                .show_value(false);

        let volume_slider = Slider::new(&mut self.volume, 0.0..=1.0).show_value(false);

        ui.add_space(10.0);

        ui.horizontal_top(|ui| {
            let play_button_response = ui.add_sized([30.0, 30.0], play_button);
            if !self.current_file.display().to_string().is_empty() && play_button_response.clicked()
            {
                match self.player_state {
                    PlayerState::Playing => {
                        self.audio_sink.pause();
                    }
                    PlayerState::Paused => {
                        self.audio_sink.play();
                    }
                }
            }

            ui.vertical(|ui| {
                ui.spacing_mut().slider_width = ui.available_width() - 150.0;
                ui.add_sized([ui.available_width() - 150.0, 15.0], file_title_label);
                ui.add_sized([ui.available_width() - 150.0, 15.0], player_position_slider);
            });

            ui.add_sized([30.0, 30.0], player_position_label);
            ui.add_sized([15.0, 30.0], volume_slider);
        });
    }

    fn render_content(&mut self, ui: &mut Ui) {
        let dirs_area_size = Vec2::new(120.0, ui.available_height());

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.render_directories_list(ui, dirs_area_size);
                self.render_mic_selection(ui);
            });
            ui.allocate_ui(dirs_area_size, |ui| {
                if !self.directories.is_empty() {
                    ui.add(Separator::default().vertical());
                }
            });

            let music_area_size = Vec2::new(ui.available_width(), ui.available_height());
            self.render_music_list(ui, music_area_size);
        });

        self.handle_directory_deletion();
    }

    fn render_directories_list(&mut self, ui: &mut Ui, scroll_area_size: Vec2) {
        let add_dir_button = Button::new(icons::ICON_ADD).frame(false);

        ui.vertical(|ui| {
            let add_dir_button_response = ui.add_sized([20.0, 20.0], add_dir_button);
            if add_dir_button_response.clicked() {
                self.handle_directory_adding();
            }

            ui.add_space(10.0);

            ui.allocate_ui(scroll_area_size, |ui| {
                ScrollArea::vertical().id_salt(0).show(ui, |ui| {
                    for (index, dir) in self.directories.iter().enumerate() {
                        let dir_name = dir
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Invalid Directory");
                        let dir_button = Button::new(dir_name).frame(false);

                        let dir_delete_button = Button::new(icons::ICON_DELETE).frame(false);

                        ui.horizontal(|ui| {
                            let dir_button_response = ui.add(dir_button);
                            if dir_button_response.clicked() {
                                self.current_directory = Some(index);
                            }

                            let directory_delete_button_response = ui.add(dir_delete_button);
                            if directory_delete_button_response.clicked() {
                                self.directory_to_delete = Some(index);
                            }
                        });
                        ui.separator();
                    }
                });
            });
        });
    }

    fn handle_directory_adding(&mut self) {
        if let Some(dirs) = FileDialog::pick_folders(Default::default()) {
            for path in dirs {
                if !self.directories.contains(&path) {
                    self.directories.push(path.clone());
                }
            }
        }
    }

    fn handle_directory_deletion(&mut self) {
        if let Some(index) = self.directory_to_delete {
            if let Some(current_index) = self.current_directory {
                if current_index > index {
                    self.current_directory = Some(current_index - 1);
                } else if current_index == index {
                    self.current_directory = None;
                }
            }

            self.directories.remove(index);
            self.directory_to_delete = None;
        }
    }

    fn render_music_list(&mut self, ui: &mut Ui, scroll_area_size: Vec2) {
        if self.current_directory.is_none() {
            return;
        }

        let current_path = self
            .directories
            .get(self.current_directory.unwrap())
            .unwrap();
        if !fs::exists(current_path).ok().unwrap_or(false) {
            self.directory_to_delete = self.current_directory;
            return;
        }

        let files = fs::read_dir(current_path).ok().unwrap();
        let mut music_files = Vec::new();
        let music_extensions = ["mp3", "wav", "ogg", "flac", "mp4", "aac"].to_vec();
        for file in files {
            let path = file.unwrap().path();

            if !path.is_file() {
                continue;
            }

            if let Some(extension) = path.extension().and_then(|n| n.to_str())
                && music_extensions.contains(&extension.to_lowercase().as_str())
                && path
                    .to_str()
                    .unwrap()
                    .to_string()
                    .to_lowercase()
                    .contains(self.search_query.as_str())
            {
                music_files.push(path);
            }
        }
        ui.vertical(|ui| {
            let search_entry = TextEdit::singleline(&mut self.search_query);
            ui.add_sized([ui.available_width(), 20.0], search_entry);

            ui.separator();

            ui.allocate_ui(scroll_area_size, |ui| {
                ScrollArea::vertical().id_salt(1).show(ui, |ui| {
                    for file in music_files.iter() {
                        let file_name = file.file_name().and_then(|n| n.to_str()).unwrap();
                        let file_button = Button::new(file_name).frame(false);
                        let file_button_response = ui.add(file_button);
                        if file_button_response.clicked() {
                            self.current_file = file.to_path_buf();
                            self.play_current_file();
                        }
                        ui.separator();
                    }
                });
            });
        });
    }

    fn render_mic_selection(&mut self, ui: &mut Ui) {
        ComboBox::from_label("Choose MIC")
            .selected_text(format!("{:?}", self.selected_input_device))
            .show_ui(ui, |ui| {
                for device in self.available_input_devices.iter() {
                    ui.selectable_value(
                        &mut self.selected_input_device,
                        device.audio_device.name.clone(),
                        device.audio_device.nick.clone(),
                    );
                }
            });
    }

    fn play_current_file(&mut self) {
        if self.current_file.to_str().unwrap().is_empty() {
            return;
        }

        if !self.current_file.exists() {
            return;
        }

        if !self.selected_input_device.is_empty() {
            self.link_devices();
        }

        let file = fs::File::open(self.current_file.display().to_string()).unwrap();
        let source = Decoder::try_from(file).unwrap();

        self.audio_sink.stop();
        self.audio_sink.play();
        self.audio_sink.append(source);

        let metadata = MediaFileMetadata::new(&self.current_file.as_path()).unwrap();
        self.max_player_position = metadata._duration.unwrap() as f32;
        self.player_position = 0.0;
        self.prev_player_position = 0.0;
    }

    fn link_devices(&self) {
        let output_devices = pw::get_output_devices().unwrap();
        let mut pwsp_output: Option<&pw::OutputDevice> = None;

        for device in output_devices.iter() {
            if device.audio_device.name == "alsa_playback.pwsp" {
                pwsp_output = Some(device);
                break;
            }
        }

        if pwsp_output.is_none() {
            return;
        }

        let input_devices = pw::get_input_devices().unwrap();
        let mut mic: Option<&pw::InputDevice> = None;

        for device in input_devices.iter() {
            if device.audio_device.name == self.selected_input_device {
                mic = Some(device);
                break;
            }
        }

        if mic.is_none() {
            return;
        }

        pwsp_output.unwrap().unlink(mic.unwrap());
        pwsp_output.unwrap().link(mic.unwrap());
    }
}

pub fn run(settings: Settings) -> Result<(), eframe::Error> {
    let options = NativeOptions {
        vsync: true,
        centered: true,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,

        viewport: egui::ViewportBuilder::default()
            .with_app_id("ru.arabianq.pwsp")
            .with_inner_size(Vec2::new(800.0, 600.0))
            .with_min_inner_size(Vec2::new(400.0, 400.0)),

        ..Default::default()
    };

    eframe::run_native(
        "PipeWire SoundPad",
        options,
        Box::new(|cc| {
            egui_material_icons::initialize(&cc.egui_ctx);
            Ok(Box::new(App::new(cc, settings)))
        }),
    )
}
