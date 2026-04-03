use clap::{Parser, Subcommand};
use pwsp::{
    types::socket::Request,
    utils::daemon::{make_request, wait_for_daemon},
};
use std::{error::Error, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Perform an action (ping, pause, resume, toggle-pause, stop, play)
    Action {
        #[clap(subcommand)]
        action: Actions,
    },
    /// Get information from the player (is paused, volume, position, duration, state, current-file-path, input, inputs)
    Get {
        #[clap(subcommand)]
        parameter: GetCommands,
    },
    /// Set information in the player (volume, position, input)
    Set {
        #[clap(subcommand)]
        parameter: SetCommands,
    },
}

#[derive(Subcommand, Debug)]
enum Actions {
    /// Ping the daemon
    Ping,
    /// Kill the daemon
    Kill,
    /// Pause audio playback
    Pause {
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Resume audio playback
    Resume {
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Toggle pause
    TogglePause {
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Stop audio playback and clear the queue
    Stop {
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Play a file
    Play {
        file_path: PathBuf,
        #[clap(short, long)]
        concurrent: bool,
    },
    /// Toggle loop
    ToggleLoop {
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Play a sound by hotkey slot name
    PlayHotkey { slot: String },
}

#[derive(Subcommand, Debug)]
enum GetCommands {
    /// Check if the player is paused
    IsPaused,
    /// Playback volume
    Volume {
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Playback position (in seconds)
    Position {
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Duration of the current file
    Duration {
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Player state (Playing, Paused or Stopped)
    State,
    /// Get all playing tracks
    Tracks,
    /// Current audio input
    Input,
    /// All audio inputs
    Inputs,
    /// Version of the daemon
    DaemonVersion,
    /// Full player state
    FullState,
    /// All hotkey slots
    Hotkeys,
}

#[derive(Subcommand, Debug)]
enum SetCommands {
    /// Playback volume
    Volume {
        volume: f32,
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Playback position (in seconds)
    Position {
        position: f32,
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Audio input id (see pwsp-cli get inputs)
    Input { name: String },
    /// Enable or disable loop (true or false)
    Loop {
        enabled: String,
        #[clap(short, long)]
        id: Option<u32>,
    },
    /// Assign a sound file to a hotkey slot
    Hotkey { slot: String, file_path: PathBuf },
    /// Set the key chord for a hotkey slot (e.g. "Ctrl+Alt+1")
    HotkeyKey { slot: String, key_chord: String },
    /// Remove a hotkey slot
    ClearHotkey { slot: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    wait_for_daemon().await?;

    let request = match cli.command {
        Commands::Action { action } => match action {
            Actions::Ping => Request::ping(),
            Actions::Kill => Request::kill(),
            Actions::Pause { id } => Request::pause(id),
            Actions::Resume { id } => Request::resume(id),
            Actions::TogglePause { id } => Request::toggle_pause(id),
            Actions::Stop { id } => Request::stop(id),
            Actions::Play {
                file_path,
                concurrent,
            } => Request::play(&file_path.to_string_lossy(), concurrent),
            Actions::ToggleLoop { id } => Request::toggle_loop(id),
            Actions::PlayHotkey { slot } => Request::play_hotkey(&slot),
        },
        Commands::Get { parameter } => match parameter {
            GetCommands::IsPaused => Request::get_is_paused(),
            GetCommands::Volume { id } => Request::get_volume(id),
            GetCommands::Position { id } => Request::get_position(id),
            GetCommands::Duration { id } => Request::get_duration(id),
            GetCommands::State => Request::get_state(),
            GetCommands::Tracks => Request::get_tracks(),
            GetCommands::Input => Request::get_input(),
            GetCommands::Inputs => Request::get_inputs(),
            GetCommands::DaemonVersion => Request::get_daemon_version(),
            GetCommands::FullState => Request::get_full_state(),
            GetCommands::Hotkeys => Request::get_hotkeys(),
        },
        Commands::Set { parameter } => match parameter {
            SetCommands::Volume { volume, id } => Request::set_volume(volume, id),
            SetCommands::Position { position, id } => Request::seek(position, id),
            SetCommands::Input { name } => Request::set_input(&name),
            SetCommands::Loop { enabled, id } => Request::set_loop(&enabled, id),
            SetCommands::Hotkey { slot, file_path } => {
                Request::set_hotkey(&slot, &file_path.to_string_lossy())
            }
            SetCommands::HotkeyKey { slot, key_chord } => {
                Request::set_hotkey_key(&slot, &key_chord)
            }
            SetCommands::ClearHotkey { slot } => Request::clear_hotkey(&slot),
        },
    };

    let response = make_request(request)
        .await
        .map_err(|e| e as Box<dyn Error>)?;
    println!("{} : {}", response.status, response.message);

    Ok(())
}
