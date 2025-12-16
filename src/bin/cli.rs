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
    /// Pause audio playback
    Pause,
    /// Resume audio playback
    Resume,
    /// Toggle pause
    TogglePause,
    /// Stop audio playback and clear the queue
    Stop,
    /// Play a file
    Play { file_path: PathBuf },
}

#[derive(Subcommand, Debug)]
enum GetCommands {
    /// Check if the player is paused
    IsPaused,
    /// Playback volume
    Volume,
    /// Playback position (in seconds)
    Position,
    /// Duration of the current file
    Duration,
    /// Player state (Playing, Paused or Stopped)
    State,
    /// Current playing file path
    CurrentFilePath,
    /// Current audio input
    Input,
    /// All audio inputs
    Inputs,
}

#[derive(Subcommand, Debug)]
enum SetCommands {
    /// Playback volume
    Volume { volume: f32 },
    /// Playback position (in seconds)
    Position { position: f32 },
    /// Audio input id (see pwsp-cli get inputs)
    Input { name: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    wait_for_daemon().await?;

    let request = match cli.command {
        Commands::Action { action } => match action {
            Actions::Ping => Request::ping(),
            Actions::Pause => Request::pause(),
            Actions::Resume => Request::resume(),
            Actions::TogglePause => Request::toggle_pause(),
            Actions::Stop => Request::stop(),
            Actions::Play { file_path } => Request::play(file_path.to_str().unwrap()),
        },
        Commands::Get { parameter } => match parameter {
            GetCommands::IsPaused => Request::get_is_paused(),
            GetCommands::Volume => Request::get_volume(),
            GetCommands::Position => Request::get_position(),
            GetCommands::Duration => Request::get_duration(),
            GetCommands::State => Request::get_state(),
            GetCommands::CurrentFilePath => Request::get_current_file_path(),
            GetCommands::Input => Request::get_input(),
            GetCommands::Inputs => Request::get_inputs(),
        },
        Commands::Set { parameter } => match parameter {
            SetCommands::Volume { volume } => Request::set_volume(volume),
            SetCommands::Position { position } => Request::seek(position),
            SetCommands::Input { name } => Request::set_input(&name),
        },
    };

    let response = make_request(request).await?;
    println!("{} : {}", response.status, response.message);

    Ok(())
}
