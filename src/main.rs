mod gui;

use std::error::Error;
use std::process;

use clap::Parser;
use pwsp::types::gui::{SortColumn, SortDir};
use pwsp::utils::gui::parse_sort_flag;

#[derive(Parser, Debug)]
#[command(name = "pwsp-gui", version)]
struct Args {
    /// Initial sort: <column>[:<dir>] where column is index|hotkey|name|modified
    /// and dir is asc|desc (default asc).
    #[arg(long)]
    sort: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let initial_sort: Option<(SortColumn, SortDir)> = match args.sort {
        Some(s) => match parse_sort_flag(&s) {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("error: {}", e);
                process::exit(2);
            }
        },
        None => None,
    };

    gui::run(initial_sort).await
}
