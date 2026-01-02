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

    pub fn pause() -> Self {
        Request::new("pause", vec![])
    }

    pub fn resume() -> Self {
        Request::new("resume", vec![])
    }

    pub fn toggle_pause() -> Self {
        Request::new("toggle_pause", vec![])
    }

    pub fn stop() -> Self {
        Request::new("stop", vec![])
    }

    pub fn play(file_path: &str) -> Self {
        Request::new("play", vec![("file_path", file_path)])
    }

    pub fn get_is_paused() -> Self {
        Request::new("is_paused", vec![])
    }

    pub fn get_volume() -> Self {
        Request::new("get_volume", vec![])
    }

    pub fn get_position() -> Self {
        Request::new("get_position", vec![])
    }

    pub fn get_duration() -> Self {
        Request::new("get_duration", vec![])
    }

    pub fn get_state() -> Self {
        Request::new("get_state", vec![])
    }

    pub fn get_current_file_path() -> Self {
        Request::new("get_current_file_path", vec![])
    }

    pub fn get_input() -> Self {
        Request::new("get_input", vec![])
    }

    pub fn get_inputs() -> Self {
        Request::new("get_inputs", vec![])
    }

    pub fn set_volume(volume: f32) -> Self {
        Request::new("set_volume", vec![("volume", &volume.to_string())])
    }

    pub fn seek(position: f32) -> Self {
        Request::new("seek", vec![("position", &position.to_string())])
    }

    pub fn set_input(name: &str) -> Self {
        Request::new("set_input", vec![("input_name", name)])
    }

    pub fn get_loop() -> Self {
        Request::new("get_loop", vec![])
    }

    pub fn set_loop(enabled: &str) -> Self {
        Request::new("set_loop", vec![("enabled", enabled)])
    }

    pub fn toggle_loop() -> Self {
        Request::new("toggle_loop", vec![])
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
