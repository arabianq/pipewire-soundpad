use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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

    pub fn get_volume() -> Self {
        Request::new("get_volume", vec![])
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
