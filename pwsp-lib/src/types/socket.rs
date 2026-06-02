use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const MAX_MESSAGE_SIZE: usize = 128 * 1024;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub name: String,
    pub args: HashMap<String, String>,
}

impl Request {
    pub fn new<T: AsRef<str>>(function_name: T, data: Vec<(T, T)>) -> Self {
        let hashmap_data: HashMap<String, String> = data
            .into_iter()
            .map(|(key, value)| (key.as_ref().to_string(), value.as_ref().to_string()))
            .collect();

        Request {
            name: function_name.as_ref().to_string(),
            args: hashmap_data,
        }
    }

    pub fn ping() -> Self {
        Request::new("ping", vec![])
    }

    pub fn kill() -> Self {
        Request::new("kill", vec![])
    }

    pub fn pause(id: Option<u32>) -> Self {
        let mut args = vec![];
        let id_str;
        if let Some(id) = id {
            id_str = id.to_string();
            args.push(("id", id_str.as_str()));
        }
        Request::new("pause", args)
    }

    pub fn resume(id: Option<u32>) -> Self {
        let mut args = vec![];
        let id_str;
        if let Some(id) = id {
            id_str = id.to_string();
            args.push(("id", id_str.as_str()));
        }
        Request::new("resume", args)
    }

    pub fn toggle_pause(id: Option<u32>) -> Self {
        let mut args = vec![];
        let id_str;
        if let Some(id) = id {
            id_str = id.to_string();
            args.push(("id", id_str.as_str()));
        }
        Request::new("toggle_pause", args)
    }

    pub fn stop(id: Option<u32>) -> Self {
        let mut args = vec![];
        let id_str;
        if let Some(id) = id {
            id_str = id.to_string();
            args.push(("id", id_str.as_str()));
        }
        Request::new("stop", args)
    }

    pub fn play(file_path: &str, concurrent: bool) -> Self {
        Request::new(
            "play",
            vec![
                ("file_path", file_path),
                ("concurrent", &concurrent.to_string()),
            ],
        )
    }

    pub fn get_is_paused() -> Self {
        Request::new("is_paused", vec![])
    }

    pub fn get_volume(id: Option<u32>) -> Self {
        let mut args = vec![];
        let id_str;
        if let Some(id) = id {
            id_str = id.to_string();
            args.push(("id", id_str.as_str()));
        }
        Request::new("get_volume", args)
    }

    pub fn get_position(id: Option<u32>) -> Self {
        let mut args = vec![];
        let id_str;
        if let Some(id) = id {
            id_str = id.to_string();
            args.push(("id", id_str.as_str()));
        }
        Request::new("get_position", args)
    }

    pub fn get_duration(id: Option<u32>) -> Self {
        let mut args = vec![];
        let id_str;
        if let Some(id) = id {
            id_str = id.to_string();
            args.push(("id", id_str.as_str()));
        }
        Request::new("get_duration", args)
    }

    pub fn get_state() -> Self {
        Request::new("get_state", vec![])
    }

    pub fn get_tracks() -> Self {
        Request::new("get_tracks", vec![])
    }

    pub fn get_input() -> Self {
        Request::new("get_input", vec![])
    }

    pub fn get_inputs() -> Self {
        Request::new("get_inputs", vec![])
    }

    pub fn set_volume(volume: f32, id: Option<u32>) -> Self {
        let mut args = vec![("volume".to_string(), volume.to_string())];
        if let Some(id) = id {
            args.push(("id".to_string(), id.to_string()));
        }
        Request::new("set_volume".to_string(), args)
    }

    pub fn seek(position: f32, id: Option<u32>) -> Self {
        let mut args = vec![("position".to_string(), position.to_string())];
        if let Some(id) = id {
            args.push(("id".to_string(), id.to_string()));
        }
        Request::new("seek".to_string(), args)
    }

    pub fn set_input(name: &str) -> Self {
        Request::new("set_input", vec![("input_name", name)])
    }

    pub fn set_loop(enabled: &str, id: Option<u32>) -> Self {
        let mut args = vec![("enabled".to_string(), enabled.to_string())];
        if let Some(id) = id {
            args.push(("id".to_string(), id.to_string()));
        }
        Request::new("set_loop".to_string(), args)
    }

    pub fn toggle_loop(id: Option<u32>) -> Self {
        let mut args = vec![];
        let id_str;
        if let Some(id) = id {
            id_str = id.to_string();
            args.push(("id", id_str.as_str()));
        }
        Request::new("toggle_loop", args)
    }

    pub fn get_daemon_version() -> Self {
        Request::new("get_daemon_version", vec![])
    }

    pub fn get_full_state() -> Self {
        Request::new("get_full_state", vec![])
    }

    pub fn get_hotkeys() -> Self {
        Request::new("get_hotkeys", vec![])
    }

    pub fn set_hotkey(slot: &str, file_path: &str) -> Self {
        Request::new("set_hotkey", vec![("slot", slot), ("file_path", file_path)])
    }

    pub fn set_hotkey_key(slot: &str, key_chord: &str) -> Self {
        Request::new(
            "set_hotkey_key",
            vec![("slot", slot), ("key_chord", key_chord)],
        )
    }

    pub fn clear_hotkey(slot: &str) -> Self {
        Request::new("clear_hotkey", vec![("slot", slot)])
    }

    pub fn play_hotkey(slot: &str) -> Self {
        Request::new("play_hotkey", vec![("slot", slot)])
    }

    pub fn set_hotkey_action(slot: &str, action: &Request) -> Self {
        let action_json = serde_json::to_string(action).unwrap_or_default();
        Request::new(
            "set_hotkey_action",
            vec![("slot", slot), ("action", &action_json)],
        )
    }

    pub fn clear_hotkey_key(slot: &str) -> Self {
        Request::new("clear_hotkey_key", vec![("slot", slot)])
    }

    pub fn set_hotkey_action_and_key(slot: &str, action: &Request, key_chord: &str) -> Self {
        let action_json = serde_json::to_string(action).unwrap_or_default();
        Request::new(
            "set_hotkey_action_and_key",
            vec![
                ("slot", slot),
                ("action", &action_json),
                ("key_chord", key_chord),
            ],
        )
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: bool,
    pub message: String,
}

impl Response {
    pub fn new<T: AsRef<str>>(status: bool, message: T) -> Self {
        Response {
            status,
            message: message.as_ref().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_new() {
        let res = Response::new(true, "success-msg");
        assert!(res.status);
        assert_eq!(res.message, "success-msg");
    }

    #[test]
    fn test_request_constructors() {
        // test ping
        let req_ping = Request::ping();
        assert_eq!(req_ping.name, "ping");
        assert!(req_ping.args.is_empty());

        // test kill
        let req_kill = Request::kill();
        assert_eq!(req_kill.name, "kill");

        // test pause (with and without id)
        let req_pause_no_id = Request::pause(None);
        assert_eq!(req_pause_no_id.name, "pause");
        assert!(req_pause_no_id.args.is_empty());

        let req_pause_with_id = Request::pause(Some(42));
        assert_eq!(req_pause_with_id.name, "pause");
        assert_eq!(
            req_pause_with_id.args.get("id").map(|s| s.as_str()),
            Some("42")
        );

        // test play
        let req_play = Request::play("/path/to/sound.mp3", true);
        assert_eq!(req_play.name, "play");
        assert_eq!(
            req_play.args.get("file_path").map(|s| s.as_str()),
            Some("/path/to/sound.mp3")
        );
        assert_eq!(
            req_play.args.get("concurrent").map(|s| s.as_str()),
            Some("true")
        );

        // test set_volume
        let req_volume = Request::set_volume(0.8, Some(10));
        assert_eq!(req_volume.name, "set_volume");
        assert_eq!(
            req_volume.args.get("volume").map(|s| s.as_str()),
            Some("0.8")
        );
        assert_eq!(req_volume.args.get("id").map(|s| s.as_str()), Some("10"));

        // test set_hotkey_action_and_key
        let action = Request::ping();
        let req_hotkey_action_and_key =
            Request::set_hotkey_action_and_key("slot1", &action, "Ctrl+P");
        assert_eq!(req_hotkey_action_and_key.name, "set_hotkey_action_and_key");
        assert_eq!(
            req_hotkey_action_and_key
                .args
                .get("slot")
                .map(|s| s.as_str()),
            Some("slot1")
        );
        assert_eq!(
            req_hotkey_action_and_key
                .args
                .get("key_chord")
                .map(|s| s.as_str()),
            Some("Ctrl+P")
        );
        let action_json = serde_json::to_string(&action).unwrap();
        assert_eq!(
            req_hotkey_action_and_key
                .args
                .get("action")
                .map(|s| s.as_str()),
            Some(action_json.as_str())
        );
    }
}
