use crate::{
    types::{
        audio_player::{FullState, PlayerState},
        socket::Response,
    },
    utils::{
        daemon::get_audio_player,
        pipewire::{get_all_devices, get_device},
    },
};
use async_trait::async_trait;
use std::{collections::HashMap, path::PathBuf};

#[async_trait]
pub trait Executable {
    async fn execute(&self) -> Response;
}

pub struct PingCommand {}

pub struct PauseCommand {
    pub id: Option<u32>,
}

pub struct ResumeCommand {
    pub id: Option<u32>,
}

pub struct TogglePauseCommand {
    pub id: Option<u32>,
}

pub struct StopCommand {
    pub id: Option<u32>,
}

pub struct IsPausedCommand {}

pub struct GetStateCommand {}

pub struct GetVolumeCommand {}

pub struct SetVolumeCommand {
    pub volume: Option<f32>,
    pub id: Option<u32>,
}

pub struct GetPositionCommand {
    pub id: Option<u32>,
}

pub struct SeekCommand {
    pub position: Option<f32>,
    pub id: Option<u32>,
}

pub struct GetDurationCommand {
    pub id: Option<u32>,
}

pub struct PlayCommand {
    pub file_path: Option<PathBuf>,
    pub concurrent: Option<bool>,
}

pub struct GetTracksCommand {}

pub struct GetCurrentInputCommand {}

pub struct GetAllInputsCommand {}

pub struct SetCurrentInputCommand {
    pub name: Option<String>,
}

pub struct SetLoopCommand {
    pub enabled: Option<bool>,
    pub id: Option<u32>,
}

pub struct ToggleLoopCommand {
    pub id: Option<u32>,
}

pub struct GetFullStateCommand {}

#[async_trait]
impl Executable for PingCommand {
    async fn execute(&self) -> Response {
        Response::new(true, "pong")
    }
}

#[async_trait]
impl Executable for PauseCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;
        audio_player.pause(self.id);
        Response::new(true, "Audio was paused")
    }
}

#[async_trait]
impl Executable for ResumeCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;
        audio_player.resume(self.id);
        Response::new(true, "Audio was resumed")
    }
}

#[async_trait]
impl Executable for TogglePauseCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;

        if audio_player.get_state() == PlayerState::Stopped {
            return Response::new(false, "Audio is not playing");
        }

        // This logic is a bit tricky with multiple tracks.
        // If ID is provided, toggle that track.
        // If not, toggle global pause state?
        // For now, let's just use pause/resume based on global state if no ID.

        if let Some(id) = self.id {
            if let Some(track) = audio_player.tracks.get(&id) {
                if track.sink.is_paused() {
                    audio_player.resume(Some(id));
                    Response::new(true, "Audio was resumed")
                } else {
                    audio_player.pause(Some(id));
                    Response::new(true, "Audio was paused")
                }
            } else {
                Response::new(false, "Track not found")
            }
        } else {
            if audio_player.is_paused() {
                audio_player.resume(None);
                Response::new(true, "Audio was resumed")
            } else {
                audio_player.pause(None);
                Response::new(true, "Audio was paused")
            }
        }
    }
}

#[async_trait]
impl Executable for StopCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;
        audio_player.stop(self.id);
        Response::new(true, "Audio was stopped")
    }
}

#[async_trait]
impl Executable for IsPausedCommand {
    async fn execute(&self) -> Response {
        let audio_player = get_audio_player().await.lock().await;
        let is_paused = audio_player.is_paused().to_string();
        Response::new(true, is_paused)
    }
}

#[async_trait]
impl Executable for GetStateCommand {
    async fn execute(&self) -> Response {
        let audio_player = get_audio_player().await.lock().await;
        let state = audio_player.get_state();
        Response::new(true, serde_json::to_string(&state).unwrap())
    }
}

#[async_trait]
impl Executable for GetVolumeCommand {
    async fn execute(&self) -> Response {
        let audio_player = get_audio_player().await.lock().await;
        let volume = audio_player.volume;
        Response::new(true, volume.to_string())
    }
}

#[async_trait]
impl Executable for SetVolumeCommand {
    async fn execute(&self) -> Response {
        if let Some(volume) = self.volume {
            let mut audio_player = get_audio_player().await.lock().await;
            audio_player.set_volume(volume, self.id);
            Response::new(true, format!("Audio volume was set to {}", volume))
        } else {
            Response::new(false, "Invalid volume value")
        }
    }
}

#[async_trait]
impl Executable for GetPositionCommand {
    async fn execute(&self) -> Response {
        let audio_player = get_audio_player().await.lock().await;
        let position = audio_player.get_position(self.id);
        Response::new(true, position.to_string())
    }
}

#[async_trait]
impl Executable for SeekCommand {
    async fn execute(&self) -> Response {
        if let Some(position) = self.position {
            let mut audio_player = get_audio_player().await.lock().await;
            match audio_player.seek(position, self.id) {
                Ok(_) => Response::new(true, format!("Audio position was set to {}", position)),
                Err(err) => Response::new(false, err.to_string()),
            }
        } else {
            Response::new(false, "Invalid position value")
        }
    }
}

#[async_trait]
impl Executable for GetDurationCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;
        match audio_player.get_duration(self.id) {
            Ok(duration) => Response::new(true, duration.to_string()),
            Err(err) => Response::new(false, err.to_string()),
        }
    }
}

#[async_trait]
impl Executable for PlayCommand {
    async fn execute(&self) -> Response {
        if let Some(file_path) = &self.file_path {
            let mut audio_player = get_audio_player().await.lock().await;
            match audio_player
                .play(file_path, self.concurrent.unwrap_or(false))
                .await
            {
                Ok(id) => Response::new(true, id.to_string()),
                Err(err) => Response::new(false, err.to_string()),
            }
        } else {
            Response::new(false, "Invalid file path")
        }
    }
}

#[async_trait]
impl Executable for GetTracksCommand {
    async fn execute(&self) -> Response {
        let audio_player = get_audio_player().await.lock().await;
        let tracks = audio_player.get_tracks();
        Response::new(true, serde_json::to_string(&tracks).unwrap())
    }
}

#[async_trait]
impl Executable for GetCurrentInputCommand {
    async fn execute(&self) -> Response {
        let audio_player = get_audio_player().await.lock().await;
        if let Some(input_device_name) = &audio_player.input_device_name {
            if let Ok(input_device) = get_device(input_device_name).await {
                Response::new(
                    true,
                    format!("{} - {}", input_device.name, input_device.nick),
                )
            } else {
                Response::new(false, "Failed to get current input device")
            }
        } else {
            Response::new(false, "No input device selected")
        }
    }
}

#[async_trait]
impl Executable for GetAllInputsCommand {
    async fn execute(&self) -> Response {
        let (input_devices, _output_devices) = get_all_devices().await.unwrap();
        let mut input_devices_strings = vec![];
        for device in input_devices {
            if device.name == "pwsp-virtual-mic" {
                continue;
            }

            let string = format!("{} - {}", device.name, device.nick);
            input_devices_strings.push(string);
        }
        let response_message = input_devices_strings.join("; ");

        Response::new(true, response_message)
    }
}

#[async_trait]
impl Executable for SetCurrentInputCommand {
    async fn execute(&self) -> Response {
        if let Some(name) = &self.name {
            let mut audio_player = get_audio_player().await.lock().await;
            match audio_player.set_current_input_device(name).await {
                Ok(_) => Response::new(true, "Input device was set"),
                Err(err) => Response::new(false, err.to_string()),
            }
        } else {
            Response::new(false, "Invalid index value")
        }
    }
}

#[async_trait]
impl Executable for SetLoopCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;

        match self.enabled {
            Some(enabled) => {
                audio_player.set_loop(enabled, self.id);
                Response::new(true, format!("Loop was set to {}", enabled))
            }
            None => Response::new(false, "Invalid enabled value"),
        }
    }
}

#[async_trait]
impl Executable for ToggleLoopCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;
        if let Some(id) = self.id {
            if let Some(track) = audio_player.tracks.get_mut(&id) {
                track.looped = !track.looped;
                Response::new(true, format!("Loop was set to {}", track.looped))
            } else {
                Response::new(false, "Track not found")
            }
        } else {
            // Toggle all?
            for track in audio_player.tracks.values_mut() {
                track.looped = !track.looped;
            }
            Response::new(true, "Loop toggled for all tracks")
        }
    }
}

#[async_trait]
impl Executable for GetFullStateCommand {
    async fn execute(&self) -> Response {
        let (input_devices, _output_devices) = get_all_devices().await.unwrap();
        let mut all_inputs = HashMap::new();
        let mut current_input_nick = String::new();

        let audio_player = get_audio_player().await.lock().await;
        for device in input_devices {
            if device.name == "pwsp-virtual-mic" {
                continue;
            }

            if let Some(current_input_name) = &audio_player.input_device_name {
                if device.name == *current_input_name {
                    current_input_nick = format!("{} - {}", device.name, device.nick);
                }
            }

            all_inputs.insert(device.name, device.nick);
        }

        let full_state = FullState {
            state: audio_player.get_state(),
            tracks: audio_player.get_tracks(),
            volume: audio_player.volume,
            current_input: current_input_nick,
            all_inputs,
        };

        Response::new(true, serde_json::to_string(&full_state).unwrap())
    }
}
