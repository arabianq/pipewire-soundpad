use crate::{
    types::pipewire::{AudioDevice, DeviceType, Terminate},
    utils::{
        daemon::get_daemon_config,
        pipewire::{create_link, get_all_devices, get_device},
    },
};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use serde::{Deserialize, Serialize};
use std::{
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

pub struct AudioPlayer {
    _stream_handle: OutputStream,
    sink: Sink,

    input_link_sender: Option<pipewire::channel::Sender<Terminate>>,
    pub current_input_device: Option<AudioDevice>,

    pub volume: f32,
    pub duration: Option<f32>,

    pub current_file_path: Option<PathBuf>,
}

impl AudioPlayer {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let daemon_config = get_daemon_config();
        let default_volume = daemon_config.default_volume.unwrap_or(1.0);
        let mut default_input_device: Option<AudioDevice> = None;
        if let Some(name) = daemon_config.default_input_name
            && let Ok(device) = get_device(&name).await
            && device.device_type == DeviceType::Input
        {
            default_input_device = Some(device);
        }

        let stream_handle = OutputStreamBuilder::open_default_stream()?;
        let sink = Sink::connect_new(stream_handle.mixer());
        sink.set_volume(default_volume);

        let mut audio_player = AudioPlayer {
            _stream_handle: stream_handle,
            sink,

            input_link_sender: None,
            current_input_device: default_input_device.clone(),

            volume: default_volume,
            duration: None,

            current_file_path: None,
        };

        if default_input_device.is_some() {
            audio_player.link_devices().await?;
        }

        Ok(audio_player)
    }

    fn abort_link_thread(&mut self) {
        if let Some(sender) = &self.input_link_sender {
            match sender.send(Terminate {}) {
                Ok(_) => println!("Sent terminate signal to link thread"),
                Err(_) => println!("Failed to send terminate signal to link thread"),
            }
        }
    }

    async fn link_devices(&mut self) -> Result<(), Box<dyn Error>> {
        self.abort_link_thread();

        if self.current_input_device.is_none() {
            println!("No input device selected, skipping device linking");
            return Ok(());
        }

        let (input_devices, _) = get_all_devices().await?;

        let mut pwsp_daemon_input: Option<AudioDevice> = None;
        for input_device in input_devices {
            if input_device.name == "pwsp-virtual-mic" {
                pwsp_daemon_input = Some(input_device);
                break;
            }
        }

        if pwsp_daemon_input.is_none() {
            println!("Could not find pwsp-daemon input device, skipping device linking");
            return Ok(());
        }

        let pwsp_daemon_input = pwsp_daemon_input.unwrap();

        let current_input_device = self.current_input_device.clone().unwrap();
        let output_fl = current_input_device
            .clone()
            .output_fl
            .expect("Failed to get pwsp-daemon output_fl");
        let output_fr = current_input_device
            .clone()
            .output_fr
            .expect("Failed to get pwsp-daemon output_fl");
        let input_fl = pwsp_daemon_input
            .clone()
            .input_fl
            .expect("Failed to get pwsp-daemon input_fl");
        let input_fr = pwsp_daemon_input
            .clone()
            .input_fr
            .expect("Failed to get pwsp-daemon input_fr");
        self.input_link_sender = Some(create_link(output_fl, output_fr, input_fl, input_fr)?);

        Ok(())
    }

    pub fn pause(&mut self) {
        if self.get_state() == PlayerState::Playing {
            self.sink.pause();
        }
    }

    pub fn resume(&mut self) {
        if self.get_state() == PlayerState::Paused {
            self.sink.play();
        }
    }

    pub fn stop(&mut self) {
        self.sink.stop();
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn get_state(&mut self) -> PlayerState {
        if self.sink.len() == 0 {
            return PlayerState::Stopped;
        }

        if self.sink.is_paused() {
            return PlayerState::Paused;
        }

        PlayerState::Playing
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        self.sink.set_volume(volume);
    }

    pub fn get_position(&mut self) -> f32 {
        if self.get_state() == PlayerState::Stopped {
            return 0.0;
        }

        self.sink.get_pos().as_secs_f32()
    }

    pub fn seek(&mut self, mut position: f32) -> Result<(), Box<dyn Error>> {
        if position < 0.0 {
            position = 0.0;
        }

        match self.sink.try_seek(Duration::from_secs_f32(position)) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn get_duration(&mut self) -> Result<f32, Box<dyn Error>> {
        if self.get_state() == PlayerState::Stopped {
            Err("Nothing is playing right now".into())
        } else {
            match self.duration {
                Some(duration) => Ok(duration),
                None => Err("Couldn't determine duration for current file".into()),
            }
        }
    }

    pub async fn play(&mut self, file_path: &Path) -> Result<(), Box<dyn Error>> {
        if !file_path.exists() {
            return Err(format!("File does not exist: {}", file_path.display()).into());
        }

        let file = fs::File::open(file_path)?;
        match Decoder::try_from(file) {
            Ok(source) => {
                self.current_file_path = Some(file_path.to_path_buf());

                if let Some(duration) = source.total_duration() {
                    self.duration = Some(duration.as_secs_f32());
                } else {
                    self.duration = None;
                }

                self.sink.stop();
                self.sink.append(source);
                self.sink.play();
                self.link_devices().await?;

                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn get_current_file_path(&mut self) -> &Option<PathBuf> {
        if self.get_state() == PlayerState::Stopped {
            self.current_file_path = None;
        }
        &self.current_file_path
    }

    pub async fn set_current_input_device(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let input_device = get_device(name).await?;

        if input_device.device_type != DeviceType::Input {
            return Err("Selected device is not an input device".into());
        }

        self.current_input_device = Some(input_device);

        self.link_devices().await?;

        Ok(())
    }
}
