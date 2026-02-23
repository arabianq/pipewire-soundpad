use pwsp::{
    types::socket::{Request, Response},
    utils::{
        commands::parse_command,
        daemon::{
            create_runtime_dir, get_audio_player, get_daemon_config, get_runtime_dir,
            is_daemon_running, link_player_to_virtual_mic,
        },
        pipewire::create_virtual_mic,
    },
};
use std::{error::Error, fs, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixListener,
    time::sleep,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    create_runtime_dir()?;

    if is_daemon_running()? {
        return Err("Another instance is already running.".into());
    }

    get_daemon_config(); // Initialize daemon config
    create_virtual_mic()?;
    get_audio_player().await; // Initialize audio player
    link_player_to_virtual_mic().await?;

    let runtime_dir = get_runtime_dir();

    let lock_file = fs::File::create(runtime_dir.join("daemon.lock"))?;
    lock_file.lock()?;

    let socket_path = runtime_dir.join("daemon.sock");
    if fs::metadata(&socket_path).is_ok() {
        fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    println!(
        "Daemon started. Listening on {}",
        socket_path.to_str().unwrap_or_default()
    );

    let commands_loop_handle = tokio::spawn(async {
        commands_loop(listener).await.ok();
    });

    let player_loop_handle = tokio::spawn(async {
        player_loop().await;
    });

    tokio::select! {
        _ = commands_loop_handle => {
            eprint!("Commands loop was finished, stopping program...");
        }
        _ = player_loop_handle => {
            eprint!("Audio Player loop was finished, stopping program...");
        }
    }

    Ok(())
}

async fn commands_loop(listener: UnixListener) -> Result<(), Box<dyn Error>> {
    loop {
        let (mut stream, _addr) = listener.accept().await?;

        tokio::spawn(async move {
            // ---------- Read request (start) ----------
            let mut len_bytes = [0u8; 4];
            if stream.read_exact(&mut len_bytes).await.is_err() {
                eprintln!("Failed to read message length from client!");
                return;
            }

            let request_len = u32::from_le_bytes(len_bytes) as usize;

            let mut buffer = vec![0u8; request_len];
            if stream.read_exact(&mut buffer).await.is_err() {
                eprintln!("Failed to read message from client!");
                return;
            }

            let request: Request = serde_json::from_slice(&buffer).unwrap();
            // ---------- Read request (end) ----------

            // ---------- Generate response (start) ----------
            let command = parse_command(&request);
            let response: Response;
            if let Some(command) = command {
                response = command.execute().await;
            } else {
                response = Response::new(false, "Unknown command");
            }
            // ---------- Generate response (end) ----------

            // ---------- Send response (start) ----------
            let response_data = serde_json::to_vec(&response).unwrap();
            let response_len = response_data.len() as u32;

            if stream.write_all(&response_len.to_le_bytes()).await.is_err() {
                eprintln!("Failed to write response length to client!");
                return;
            }
            if stream.write_all(&response_data).await.is_err() {
                eprintln!("Failed to write response to client!");
                return;
            }
            // ---------- Send response (end) ----------

            if response.status && response.message.eq("killed") {
                std::process::exit(0);
            }
        });
    }
}

async fn player_loop() {
    loop {
        let mut audio_player = get_audio_player().await.lock().await;

        audio_player.update().await;

        sleep(Duration::from_millis(100)).await;
    }
}
