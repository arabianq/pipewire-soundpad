use crate::types::{commands::*, socket::Request};

use std::path::PathBuf;

pub fn parse_command(request: &Request) -> Option<Box<dyn Executable + Send>> {
    match request.name.as_str() {
        "ping" => Some(Box::new(PingCommand {})),
        "pause" => Some(Box::new(PauseCommand {})),
        "resume" => Some(Box::new(ResumeCommand {})),
        "stop" => Some(Box::new(StopCommand {})),
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
            Some(Box::new(SetVolumeCommand { volume }))
        }
        "get_position" => Some(Box::new(GetPositionCommand {})),
        "seek" => {
            let position = request
                .args
                .get("position")
                .unwrap_or(&String::new())
                .parse::<f32>()
                .ok();
            Some(Box::new(SeekCommand { position }))
        }
        "get_duration" => Some(Box::new(GetDurationCommand {})),
        "play" => {
            let file_path = request
                .args
                .get("file_path")
                .unwrap_or(&String::new())
                .parse::<PathBuf>()
                .ok();
            Some(Box::new(PlayCommand { file_path }))
        }
        "get_current_file_path" => Some(Box::new(GetCurrentFilePathCommand {})),
        "get_input" => Some(Box::new(GetCurrentInputCommand {})),
        "get_inputs" => Some(Box::new(GetAllInputsCommand {})),
        "set_input" => {
            let name = Some(request.args.get("input_name").unwrap_or(&String::new())).cloned();
            Some(Box::new(SetCurrentInputCommand { name }))
        }
        _ => None,
    }
}
