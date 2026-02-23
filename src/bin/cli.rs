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
}

#[derive(Subcommand, Debug)]
enum GetCommands {
    /// Check if the player is paused
    IsPaused,
    /// Playback volume
    Volume,
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
            } => Request::play(file_path.to_str().unwrap(), concurrent),
            Actions::ToggleLoop { id } => Request::toggle_loop(id),
        },
        Commands::Get { parameter } => match parameter {
            GetCommands::IsPaused => Request::get_is_paused(),
            GetCommands::Volume => Request::get_volume(),
            GetCommands::Position { id } => Request::get_position(id),
            GetCommands::Duration { id } => Request::get_duration(id),
            GetCommands::State => Request::get_state(),
            GetCommands::Tracks => Request::get_tracks(),
            GetCommands::Input => Request::get_input(),
            GetCommands::Inputs => Request::get_inputs(),
            GetCommands::DaemonVersion => Request::get_daemon_version(),
            GetCommands::FullState => Request::get_full_state(),
        },
        Commands::Set { parameter } => match parameter {
            SetCommands::Volume { volume, id } => Request::set_volume(volume, id),
            SetCommands::Position { position, id } => Request::seek(position, id),
            SetCommands::Input { name } => Request::set_input(&name),
            SetCommands::Loop { enabled, id } => Request::set_loop(&enabled, id),
        },
    };

    let response = make_request(request)
        .await
        .map_err(|e| e as Box<dyn Error>)?;
    println!("{} : {}", response.status, response.message);

    Ok(())
}
