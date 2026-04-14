use crate::{
    types::{
        audio_player::AudioPlayer,
        config::DaemonConfig,
        socket::{Request, Response},
    },
    utils::pipewire::get_device,
};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{error::Error, fs};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::{Mutex, OnceCell},
    time::{Duration, sleep},
};

static AUDIO_PLAYER: OnceCell<Mutex<AudioPlayer>> = OnceCell::const_new();

pub async fn get_audio_player() -> Result<&'static Mutex<AudioPlayer>, String> {
    AUDIO_PLAYER
        .get_or_try_init(|| async {
            println!("Initializing audio player");
            match AudioPlayer::new().await {
                Ok(player) => Ok(Mutex::new(player)),
                Err(err) => Err(err.to_string()),
            }
        })
        .await
}

pub fn get_daemon_config() -> DaemonConfig {
    DaemonConfig::load_from_file().unwrap_or_else(|_| {
        let config = DaemonConfig::default();
        config.save_to_file().ok();
        config
    })
}


pub fn get_runtime_dir() -> PathBuf {
    dirs::runtime_dir().unwrap_or(PathBuf::from("/run/pwsp"))
}

pub fn create_runtime_dir() -> Result<(), Box<dyn Error>> {
    let runtime_dir = get_runtime_dir();
    if !runtime_dir.exists() {
        fs::create_dir_all(&runtime_dir)?;
    }
    fs::set_permissions(&runtime_dir, fs::Permissions::from_mode(0o700))?;

    Ok(())
}

pub fn is_daemon_running() -> Result<bool, Box<dyn Error>> {
    let lock_file = fs::File::create(get_runtime_dir().join("daemon.lock"))?;
    match lock_file.try_lock() {
        Ok(_) => Ok(false),
        Err(_) => Ok(true),
    }
}

pub async fn wait_for_daemon() -> Result<(), Box<dyn Error>> {
    if is_daemon_running()? {
        return Ok(());
    }

    println!("Daemon not found, waiting for it...");
    while !is_daemon_running()? {
        sleep(Duration::from_millis(100)).await;
    }

    println!("Found running daemon");

    Ok(())
}

pub async fn make_request(request: Request) -> Result<Response, Box<dyn Error + Send + Sync>> {
    let socket_path = get_runtime_dir().join("daemon.sock");
    let mut stream = UnixStream::connect(socket_path).await?;

    // ---------- Send request (start) ----------
    let request_data = serde_json::to_vec(&request)?;
    let request_len = request_data.len() as u32;
    if stream.write_all(&request_len.to_le_bytes()).await.is_err() {
        return Err("Failed to send request length".into());
    };
    if stream.write_all(&request_data).await.is_err() {
        return Err("Failed to send request".into());
    }
    // ---------- Send request (end) ----------

    // ---------- Read response (start) ----------
    let mut len_bytes = [0u8; 4];
    if stream.read_exact(&mut len_bytes).await.is_err() {
        return Err("Failed to read response length".into());
    }
    let response_len = u32::from_le_bytes(len_bytes) as usize;

    let mut buffer = vec![0u8; response_len];
    if stream.read_exact(&mut buffer).await.is_err() {
        return Err("Failed to read response".into());
    };
    // ---------- Read response (end) ----------

    Ok(serde_json::from_slice(&buffer)?)
}
