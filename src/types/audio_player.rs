use crate::{
    types::pipewire::{DeviceType, Terminate},
    utils::{
        daemon::get_daemon_config,
        pipewire::{create_link, get_device},
    },
};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug, Eq, PartialEq, Default, Clone, Serialize, Deserialize)]
pub enum PlayerState {
    #[default]
    Stopped,
    Paused,
    Playing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    pub id: u32,
    pub path: PathBuf,
    pub duration: Option<f32>,
    pub position: f32,
    pub volume: f32,
    pub looped: bool,
    pub paused: bool,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FullState {
    pub state: PlayerState,
    pub tracks: Vec<TrackInfo>,
    pub volume: f32,
    pub current_input: String,
    pub all_inputs: HashMap<String, String>,
}

pub struct PlayingSound {
    pub id: u32,
    pub sink: Player,
    pub path: PathBuf,
    pub duration: Option<f32>,
    pub looped: bool,
    pub volume: f32,
}

pub struct AudioPlayer {
    pub stream_handle: MixerDeviceSink,
    pub tracks: HashMap<u32, PlayingSound>,
    pub next_id: u32,

    input_link_sender: Option<pipewire::channel::Sender<Terminate>>,
    pub input_device_name: Option<String>,

    pub volume: f32, // Master volume
}

impl AudioPlayer {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let daemon_config = get_daemon_config();
        let default_volume = daemon_config.default_volume.unwrap_or(1.0);

        let stream_handle = DeviceSinkBuilder::open_default_sink()?;

        let mut audio_player = AudioPlayer {
            stream_handle,
            tracks: HashMap::new(),
            next_id: 1,

            input_link_sender: None,
            input_device_name: daemon_config.default_input_name.clone(),

            volume: default_volume,
        };

        if audio_player.input_device_name.is_some() {
            audio_player.link_devices().await?;
        }

        Ok(audio_player)
    }

    fn abort_link_thread(&mut self) {
        if let Some(sender) = &self.input_link_sender {
            match sender.send(Terminate {}) {
                Ok(_) => {
                    println!("Sent terminate signal to link thread");
                    self.input_link_sender = None;
                }
                Err(_) => eprintln!("Failed to send terminate signal to link thread"),
            }
        }
    }

    async fn link_devices(&mut self) -> Result<(), Box<dyn Error>> {
        self.abort_link_thread();

        let input_device;
        if let Some(input_device_name) = &self.input_device_name {
            if let Ok(device) = get_device(input_device_name).await {
                input_device = device;
            } else {
                eprintln!(
                    "Could not find selected input device {}, skipping device linking",
                    input_device_name
                );
                return Ok(());
            }
        } else {
            eprintln!("No input device selected, skipping device linking");
            return Ok(());
        }

        let daemon_input;
        if let Ok(device) = get_device("pwsp-virtual-mic").await {
            daemon_input = device;
        } else {
            eprintln!("Could not find pwsp-virtual-mic device, skipping device linking");
            return Ok(());
        }

        let Some(output_fl) = input_device.output_fl.clone() else {
            eprintln!("Failed to get pwsp-daemon output_fl");
            return Ok(());
        };
        let Some(output_fr) = input_device.output_fr.clone() else {
            eprintln!("Failed to get pwsp-daemon output_fr");
            return Ok(());
        };
        let Some(input_fl) = daemon_input.input_fl.clone() else {
            eprintln!("Failed to get pwsp-daemon input_fl");
            return Ok(());
        };
        let Some(input_fr) = daemon_input.input_fr.clone() else {
            eprintln!("Failed to get pwsp-daemon input_fr");
            return Ok(());
        };

        self.input_link_sender = Some(create_link(output_fl, output_fr, input_fl, input_fr)?);

        Ok(())
    }

    pub fn pause(&mut self, id: Option<u32>) {
        if let Some(id) = id {
            if let Some(sound) = self.tracks.get_mut(&id) {
                sound.sink.pause();
            }
        } else {
            for sound in self.tracks.values_mut() {
                sound.sink.pause();
            }
        }
    }

    pub fn resume(&mut self, id: Option<u32>) {
        if let Some(id) = id {
            if let Some(sound) = self.tracks.get_mut(&id) {
                sound.sink.play();
            }
        } else {
            for sound in self.tracks.values_mut() {
                sound.sink.play();
            }
        }
    }

    pub fn stop(&mut self, id: Option<u32>) {
        if let Some(id) = id {
            self.tracks.remove(&id);
        } else {
            self.tracks.clear();
        }
    }

    pub fn is_paused(&self) -> bool {
        if self.tracks.is_empty() {
            return false;
        }
        self.tracks.values().all(|s| s.sink.is_paused())
    }

    pub fn get_state(&self) -> PlayerState {
        if self.tracks.is_empty() {
            return PlayerState::Stopped;
        }

        if self
            .tracks
            .values()
            .any(|s| !s.sink.is_paused() && !s.sink.empty())
        {
            return PlayerState::Playing;
        }

        if self.is_paused() {
            return PlayerState::Paused;
        }

        PlayerState::Stopped
    }

    pub fn set_volume(&mut self, volume: f32, id: Option<u32>) {
        if let Some(id) = id {
            if let Some(sound) = self.tracks.get_mut(&id) {
                sound.volume = volume;
                sound.sink.set_volume(self.volume * volume);
            }
        } else {
            self.volume = volume;
            for sound in self.tracks.values_mut() {
                sound.sink.set_volume(self.volume * sound.volume);
            }
        }
    }

    pub fn get_position(&self, id: Option<u32>) -> f32 {
        if let Some(id) = id {
            if let Some(sound) = self.tracks.get(&id) {
                return sound.sink.get_pos().as_secs_f32();
            }
        } else if let Some(sound) = self.tracks.values().last() {
            // Fallback to last added track if no ID
            return sound.sink.get_pos().as_secs_f32();
        }
        0.0
    }

    pub fn seek(&mut self, position: f32, id: Option<u32>) -> Result<(), Box<dyn Error>> {
        let position = if position < 0.0 { 0.0 } else { position };

        if let Some(id) = id {
            if let Some(sound) = self.tracks.get_mut(&id) {
                sound.sink.try_seek(Duration::from_secs_f32(position))?;
            }
        } else {
            // Seek all? Or last? Let's seek all for now if no ID provided
            for sound in self.tracks.values_mut() {
                sound.sink.try_seek(Duration::from_secs_f32(position)).ok();
            }
        }
        Ok(())
    }

    pub fn get_duration(&mut self, id: Option<u32>) -> Result<f32, Box<dyn Error>> {
        if let Some(id) = id {
            if let Some(sound) = self.tracks.get(&id) {
                return sound.duration.ok_or("Unknown duration".into());
            }
        } else if let Some(sound) = self.tracks.values().last() {
            return sound.duration.ok_or("Unknown duration".into());
        }
        Err("No track playing".into())
    }

    pub async fn play(
        &mut self,
        file_path: &Path,
        concurrent: bool,
    ) -> Result<u32, Box<dyn Error>> {
        if !file_path.exists() {
            return Err(format!("File does not exist: {}", file_path.display()).into());
        }

        let file = fs::File::open(file_path)?;
        match Decoder::try_from(file) {
            Ok(source) => {
                if !concurrent {
                    self.tracks.clear();
                }

                let id = self.next_id;
                self.next_id += 1;

                let duration = source.total_duration().map(|d| d.as_secs_f32());

                let sink = Player::connect_new(self.stream_handle.mixer());
                sink.set_volume(self.volume); // Default volume is 1.0 * master
                sink.append(source);
                sink.play();

                let sound = PlayingSound {
                    id,
                    sink,
                    path: file_path.to_path_buf(),
                    duration,
                    looped: false,
                    volume: 1.0,
                };

                self.tracks.insert(id, sound);

                Ok(id)
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn set_loop(&mut self, enabled: bool, id: Option<u32>) {
        if let Some(id) = id {
            if let Some(sound) = self.tracks.get_mut(&id) {
                sound.looped = enabled;
            }
        } else {
            // Set loop for all? Or just last?
            // Let's set for all.
            for sound in self.tracks.values_mut() {
                sound.looped = enabled;
            }
        }
    }

    pub fn get_tracks(&self) -> Vec<TrackInfo> {
        let mut tracks: Vec<_> = self
            .tracks
            .values()
            .map(|sound| TrackInfo {
                id: sound.id,
                path: sound.path.clone(),
                duration: sound.duration,
                position: sound.sink.get_pos().as_secs_f32(),
                volume: sound.volume,
                looped: sound.looped,
                paused: sound.sink.is_paused(),
            })
            .collect();
        tracks.sort_by_key(|t| t.id);
        tracks
    }

    pub async fn update(&mut self) {
        if let Some(input_device_name) = &self.input_device_name {
            // Unlink devices if selected input device was removed
            if self.input_link_sender.is_some() && get_device(input_device_name).await.is_err() {
                // Selected input device was removed
                eprintln!(
                    "Selected input device {} was removed, unlinking devices",
                    input_device_name
                );
                self.abort_link_thread();
            }
            // Link devices if not linked
            else if self.input_link_sender.is_none() {
                self.link_devices().await.ok();
            }
        }

        // Handle looped sounds
        let mut restarts = vec![];

        for (id, sound) in &self.tracks {
            if sound.sink.empty() && sound.looped {
                restarts.push(*id);
            }
        }

        for id in restarts {
            if let Some(sound) = self.tracks.get_mut(&id) {
                if let Ok(file) = fs::File::open(&sound.path) {
                    if let Ok(source) = Decoder::try_from(file) {
                        sound.sink.append(source);
                        sound.sink.play();
                    }
                }
            }
        }

        self.tracks
            .retain(|_, sound| !sound.sink.empty() || sound.looped);
    }

    pub async fn set_current_input_device(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let input_device = get_device(name).await?;

        if input_device.device_type != DeviceType::Input {
            return Err("Selected device is not an input device".into());
        }

        self.input_device_name = Some(name.to_string());

        self.link_devices().await?;

        Ok(())
    }
}
