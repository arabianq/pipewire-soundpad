use crate::{
    types::{audio_player::PlayerState, socket::Response},
    utils::{daemon::get_audio_player, pipewire::get_all_devices},
};
use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait Executable {
    async fn execute(&self) -> Response;
}

pub struct PingCommand {}

pub struct PauseCommand {}

pub struct ResumeCommand {}

pub struct TogglePauseCommand {}

pub struct StopCommand {}

pub struct IsPausedCommand {}

pub struct GetStateCommand {}

pub struct GetVolumeCommand {}

pub struct SetVolumeCommand {
    pub volume: Option<f32>,
}

pub struct GetPositionCommand {}

pub struct SeekCommand {
    pub position: Option<f32>,
}

pub struct GetDurationCommand {}

pub struct PlayCommand {
    pub file_path: Option<PathBuf>,
}

pub struct GetCurrentFilePathCommand {}

pub struct GetCurrentInputCommand {}

pub struct GetAllInputsCommand {}

pub struct SetCurrentInputCommand {
    pub name: Option<String>,
}

pub struct GetLoopCommand {}

pub struct SetLoopCommand {
    pub enabled: Option<bool>,
}

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
        audio_player.pause();
        Response::new(true, "Audio was paused")
    }
}

#[async_trait]
impl Executable for ResumeCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;
        audio_player.resume();
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

        if audio_player.is_paused() {
            audio_player.resume();
            Response::new(true, "Audio was resumed")
        } else {
            audio_player.pause();
            Response::new(true, "Audio was paused")
        }
    }
}

#[async_trait]
impl Executable for StopCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;
        audio_player.stop();
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
            audio_player.set_volume(volume);
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
        let position = audio_player.get_position();
        Response::new(true, position.to_string())
    }
}

#[async_trait]
impl Executable for SeekCommand {
    async fn execute(&self) -> Response {
        if let Some(position) = self.position {
            let mut audio_player = get_audio_player().await.lock().await;
            match audio_player.seek(position) {
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
        match audio_player.get_duration() {
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
            match audio_player.play(file_path).await {
                Ok(_) => Response::new(true, format!("Now playing {}", file_path.display())),
                Err(err) => Response::new(false, err.to_string()),
            }
        } else {
            Response::new(false, "Invalid file path")
        }
    }
}

#[async_trait]
impl Executable for GetCurrentFilePathCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;
        let current_file_path = audio_player.get_current_file_path();
        if let Some(current_file_path) = current_file_path {
            Response::new(true, current_file_path.to_str().unwrap())
        } else {
            Response::new(false, "No file is playing")
        }
    }
}

#[async_trait]
impl Executable for GetCurrentInputCommand {
    async fn execute(&self) -> Response {
        let audio_player = get_audio_player().await.lock().await;
        if let Some(input_device) = &audio_player.current_input_device {
            Response::new(
                true,
                format!("{} - {}", input_device.name, input_device.nick),
            )
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
impl Executable for GetLoopCommand {
    async fn execute(&self) -> Response {
        let audio_player = get_audio_player().await.lock().await;
        Response::new(true, audio_player.looped.to_string())
    }
}

#[async_trait]
impl Executable for SetLoopCommand {
    async fn execute(&self) -> Response {
        let mut audio_player = get_audio_player().await.lock().await;

        match self.enabled {
            Some(enabled) => {
                audio_player.looped = enabled;
                Response::new(true, format!("Loop was set to {}", enabled))
            }
            None => Response::new(false, "Invalid enabled value"),
        }
    }
}
