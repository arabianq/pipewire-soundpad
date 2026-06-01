use crate::types::{commands::*, socket::Request};

use std::path::PathBuf;

pub fn parse_command(request: &Request) -> Option<Box<dyn Executable + Send>> {
    let id = request.args.get("id").and_then(|s| s.parse::<u32>().ok());

    match request.name.as_str() {
        "ping" => Some(Box::new(PingCommand {})),
        "kill" => Some(Box::new(KillCommand {})),
        "pause" => Some(Box::new(PauseCommand { id })),
        "resume" => Some(Box::new(ResumeCommand { id })),
        "toggle_pause" => Some(Box::new(TogglePauseCommand { id })),
        "stop" => Some(Box::new(StopCommand { id })),
        "is_paused" => Some(Box::new(IsPausedCommand {})),
        "get_state" => Some(Box::new(GetStateCommand {})),
        "get_volume" => Some(Box::new(GetVolumeCommand { id })),
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
        "get_daemon_version" => Some(Box::new(GetDaemonVersionCommand {})),
        "get_full_state" => Some(Box::new(GetFullStateCommand {})),
        "get_hotkeys" => Some(Box::new(GetHotkeysCommand {})),
        "set_hotkey" => {
            let slot = request.args.get("slot").cloned();
            let file_path = request
                .args
                .get("file_path")
                .and_then(|s| s.parse::<PathBuf>().ok());
            Some(Box::new(SetHotkeyCommand { slot, file_path }))
        }
        "set_hotkey_key" => {
            let slot = request.args.get("slot").cloned();
            let key_chord = request.args.get("key_chord").cloned();
            Some(Box::new(SetHotkeyKeyCommand { slot, key_chord }))
        }
        "clear_hotkey" => {
            let slot = request.args.get("slot").cloned();
            Some(Box::new(ClearHotkeyCommand { slot }))
        }
        "play_hotkey" => {
            let slot = request.args.get("slot").cloned();
            Some(Box::new(PlayHotkeyCommand { slot }))
        }
        "set_hotkey_action" => {
            let slot = request.args.get("slot").cloned();
            let action = request
                .args
                .get("action")
                .and_then(|s| serde_json::from_str::<Request>(s).ok());
            Some(Box::new(SetHotkeyActionCommand { slot, action }))
        }
        "clear_hotkey_key" => {
            let slot = request.args.get("slot").cloned();
            Some(Box::new(ClearHotkeyKeyCommand { slot }))
        }
        "set_hotkey_action_and_key" => {
            let slot = request.args.get("slot").cloned();
            let action = request
                .args
                .get("action")
                .and_then(|s| serde_json::from_str::<Request>(s).ok());
            let key_chord = request.args.get("key_chord").cloned();
            Some(Box::new(SetHotkeyActionAndKeyCommand {
                slot,
                action,
                key_chord,
            }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::socket::Request;
    use std::collections::HashMap;

    #[test]
    fn test_parse_set_volume_valid() {
        let mut args = HashMap::new();
        args.insert("volume".to_string(), "0.5".to_string());
        args.insert("id".to_string(), "1".to_string());
        let request = Request {
            name: "set_volume".to_string(),
            args,
        };

        let cmd = parse_command(&request);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_parse_set_volume_missing_volume() {
        let mut args = HashMap::new();
        args.insert("id".to_string(), "1".to_string());
        let request = Request {
            name: "set_volume".to_string(),
            args,
        };

        let cmd = parse_command(&request);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_parse_set_volume_invalid_volume() {
        let mut args = HashMap::new();
        args.insert("volume".to_string(), "not-a-float".to_string());
        let request = Request {
            name: "set_volume".to_string(),
            args,
        };

        let cmd = parse_command(&request);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_parse_set_volume_missing_id() {
        let mut args = HashMap::new();
        args.insert("volume".to_string(), "0.5".to_string());
        let request = Request {
            name: "set_volume".to_string(),
            args,
        };

        let cmd = parse_command(&request);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_parse_set_volume_invalid_id() {
        let mut args = HashMap::new();
        args.insert("id".to_string(), "not-an-int".to_string());
        args.insert("volume".to_string(), "0.5".to_string());
        let request = Request {
            name: "set_volume".to_string(),
            args,
        };

        let cmd = parse_command(&request);
        assert!(cmd.is_some());
    }

    #[test]
    fn test_parse_set_volume_empty_args() {
        let request = Request {
            name: "set_volume".to_string(),
            args: HashMap::new(),
        };

        let cmd = parse_command(&request);
        assert!(cmd.is_some());
    }
}
