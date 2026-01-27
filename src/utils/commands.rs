use crate::types::{commands::*, socket::Request};

use std::path::PathBuf;

pub fn parse_command(request: &Request) -> Option<Box<dyn Executable + Send>> {
    let id = request.args.get("id").and_then(|s| s.parse::<u32>().ok());

    match request.name.as_str() {
        "ping" => Some(Box::new(PingCommand {})),
        "pause" => Some(Box::new(PauseCommand { id })),
        "resume" => Some(Box::new(ResumeCommand { id })),
        "toggle_pause" => Some(Box::new(TogglePauseCommand { id })),
        "stop" => Some(Box::new(StopCommand { id })),
        "is_paused" => Some(Box::new(IsPausedCommand {})),
        "get_state" => Some(Box::new(GetStateCommand {})),
        "get_volume" => Some(Box::new(GetVolumeCommand {})),
        "set_volume" => {
            let volume = request
                .args
                .get("volume")
                .unwrap_or(&String::new())
                .parse::<f32>()
                .ok();
            Some(Box::new(SetVolumeCommand { volume, id }))
        }
        "get_position" => Some(Box::new(GetPositionCommand { id })),
        "seek" => {
            let position = request
                .args
                .get("position")
                .unwrap_or(&String::new())
                .parse::<f32>()
                .ok();
            Some(Box::new(SeekCommand { position, id }))
        }
        "get_duration" => Some(Box::new(GetDurationCommand { id })),
        "play" => {
            let file_path = request
                .args
                .get("file_path")
                .unwrap_or(&String::new())
                .parse::<PathBuf>()
                .ok();
            let concurrent = request
                .args
                .get("concurrent")
                .unwrap_or(&String::new())
                .parse::<bool>()
                .ok();
            Some(Box::new(PlayCommand {
                file_path,
                concurrent,
            }))
        }
        "get_tracks" => Some(Box::new(GetTracksCommand {})),
        "get_input" => Some(Box::new(GetCurrentInputCommand {})),
        "get_inputs" => Some(Box::new(GetAllInputsCommand {})),
        "set_input" => {
            let name = Some(request.args.get("input_name").unwrap_or(&String::new())).cloned();
            Some(Box::new(SetCurrentInputCommand { name }))
        }
        "set_loop" => {
            let enabled = request
                .args
                .get("enabled")
                .unwrap_or(&String::new())
                .parse::<bool>()
                .ok();
            Some(Box::new(SetLoopCommand { enabled, id }))
        }
        "toggle_loop" => Some(Box::new(ToggleLoopCommand { id })),
        "get_full_state" => Some(Box::new(GetFullStateCommand {})),
        _ => None,
    }
}
