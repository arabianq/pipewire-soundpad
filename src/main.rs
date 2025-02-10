use eframe::{CreationContext, Frame, NativeOptions};
use egui::{
    Button, CentralPanel, ComboBox, Context, Label, ScrollArea, Separator, Slider, TextEdit, Ui,
    Vec2,
};
use egui_material_icons::icons;
use metadata::media_file::MediaFileMetadata;
use rfd::FileDialog;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::Write;
use std::{fs, path::PathBuf};

mod pw;

enum PlayerState {
    PLAYING,
    PAUSED,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::PAUSED
    }
}

#[derive(Default)]
struct App {
    player_position: f32,
    prev_player_position: f32,
    max_player_position: f32,

    volume: f32,

    player_state: PlayerState,

    directories: Vec<PathBuf>,
    deleted_directory: Option<usize>,
    current_directory: Option<usize>,

    selected_input_device: String,
    available_input_devices: Vec<pw::InputDevice>,

    current_file: PathBuf,

    search_query: String,

    _audio_stream: Option<OutputStream>,
    _audio_stream_handle: Option<OutputStreamHandle>,
    audio_sink: Option<Sink>,
}

impl App {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let saved_dirs_path = dirs::config_dir().unwrap().join("pwsp").join("saved_dirs");
        let saved_dirs_content = fs::read_to_string(&saved_dirs_path).unwrap_or_default();
        let saved_dirs: Vec<_> = saved_dirs_content.lines().map(PathBuf::from).collect();
        let current_directory = match saved_dirs.is_empty() {
            false => Some(0),
            true => None,
        };

        let saved_mic_path = dirs::config_dir().unwrap().join("pwsp").join("saved_mic");
        let saved_mic_content = fs::read_to_string(&saved_mic_path).unwrap_or_default();

        let (_audio_stream, audio_stream_handle) = OutputStream::try_default().unwrap();
        let audio_sink = Sink::try_new(&audio_stream_handle).unwrap();
        audio_sink.pause();

        Self {
            max_player_position: 1.0,
            directories: saved_dirs,
            selected_input_device: saved_mic_content,
            _audio_stream: Some(_audio_stream),
            _audio_stream_handle: Some(audio_stream_handle),
            audio_sink: Some(audio_sink),
            volume: 1.0,
            current_directory,

            ..Default::default()
        }
    }

    fn upd(&mut self, ui: &mut Ui, ctx: &Context, _frame: &mut Frame) {
        self.available_input_devices = pw::get_input_devices().unwrap();

        let saved_mic_path = dirs::config_dir().unwrap().join("pwsp").join("saved_mic");
        let saved_mic_content = fs::read_to_string(&saved_mic_path).unwrap_or_default();
        if self.selected_input_device != saved_mic_content {
            fs::write(saved_mic_path, self.selected_input_device.clone()).ok();
        }

        if let PlayerState::PLAYING = self.player_state {
            ctx.request_repaint();
            self.audio_sink.as_ref().unwrap().set_volume(self.volume);
        }

        self.player_state = match self.audio_sink.as_ref().unwrap().is_paused() {
            true => PlayerState::PAUSED,
            false => PlayerState::PLAYING,
        };

        if self.player_position != self.prev_player_position {
            let target_pos = core::time::Duration::from_secs_f32(self.player_position);
            self.audio_sink
                .as_ref()
                .unwrap()
                .try_seek(target_pos)
                .unwrap();
            self.prev_player_position = self.player_position;
        } else {
            self.player_position = self.audio_sink.as_ref().unwrap().get_pos().as_secs_f32();
        }

        self.prev_player_position = self.player_position;

        self.render_ui(ui);
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
            PlayerState::PLAYING => icons::ICON_PAUSE,
            PlayerState::PAUSED => icons::ICON_PLAY_ARROW,
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
                    PlayerState::PLAYING => {
                        self.audio_sink.as_ref().unwrap().pause();
                    }
                    PlayerState::PAUSED => {
                        self.audio_sink.as_ref().unwrap().play();
                    }
                }
            }

            ui.vertical(|ui| {
                ui.spacing_mut().slider_width = ui.available_width() - 150.0;
                ui.add_sized([ui.available_width() - 150.0, 15.0], file_title_label);
                ui.add_sized([ui.available_width() - 150.0, 15.0], player_position_slider);
            });

            ui.add_sized([30.0, 30.0], player_position_label);

            let volume_slider_response = ui.add_sized([15.0, 30.0], volume_slider);
            if volume_slider_response.changed() {
                // println!("{}", self.volume.);
            }
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
                                self.deleted_directory = Some(index);
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
            let saved_dirs_path = dirs::config_dir().unwrap().join("pwsp").join("saved_dirs");
            if let Ok(mut file) = fs::OpenOptions::new().append(true).open(saved_dirs_path) {
                for path in dirs {
                    if !self.directories.contains(&path) {
                        self.directories.push(path.clone());
                        writeln!(file, "{}", path.display()).ok();
                    }
                }
            }
        }
    }

    fn handle_directory_deletion(&mut self) {
        if let Some(index) = self.deleted_directory {
            if let Some(current_index) = self.current_directory {
                if current_index > index {
                    self.current_directory = Some(current_index - 1);
                } else if current_index == index {
                    self.current_directory = None;
                }
            }

            self.directories.remove(index);
            self.deleted_directory = None;

            let saved_dirs_path = dirs::config_dir().unwrap().join("pwsp").join("saved_dirs");
            let content = self
                .directories
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n");

            fs::write(saved_dirs_path, content).ok();
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
            self.deleted_directory = self.current_directory;
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

            if let Some(extension) = path.extension().and_then(|n| n.to_str()) {
                if music_extensions.contains(&extension.to_lowercase().as_str()) {
                    if path
                        .to_str()
                        .unwrap()
                        .to_string()
                        .to_lowercase()
                        .contains(self.search_query.as_str())
                    {
                        music_files.push(path);
                    }
                }
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
        let source = Decoder::new(file).unwrap();

        let metadata = MediaFileMetadata::new(&self.current_file.as_path()).unwrap();
        self.max_player_position = metadata._duration.unwrap() as f32;

        self.audio_sink.as_ref().unwrap().stop();
        self.audio_sink.as_ref().unwrap().play();
        self.audio_sink.as_ref().unwrap().append(source);
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

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| self.upd(ui, ctx, frame));
    }
}

fn main() -> Result<(), eframe::Error> {
    let config_dir_path = dirs::config_dir().unwrap().join("pwsp");
    fs::create_dir_all(&config_dir_path).ok();

    if !fs::exists(config_dir_path.join("saved_dirs"))
        .ok()
        .unwrap_or(false)
    {
        fs::File::create(config_dir_path.join("saved_dirs")).ok();
    }
    if !fs::exists(config_dir_path.join("saved_mic"))
        .ok()
        .unwrap_or(false)
    {
        fs::File::create(config_dir_path.join("saved_mic")).ok();
    }

    let mut options = NativeOptions {
        ..Default::default()
    };
    options.viewport.min_inner_size = Some(Vec2::new(400.0, 400.0));
    options.vsync = true;
    options.hardware_acceleration = eframe::HardwareAcceleration::Preferred;

    eframe::run_native(
        "PipeWire SoundPad",
        options,
        Box::new(|cc| {
            egui_material_icons::initialize(&cc.egui_ctx);
            Ok(Box::new(App::new(cc)))
        }),
    )
}
