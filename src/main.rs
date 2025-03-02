mod cli;
mod commands;
mod error;
mod git;

use cli::Cli;
use commands::create;
use error::Result;

use clap::Parser;
use git2::Repository;
use log::error;

/// The main entry point of the application
fn main() {
    // Parse command line arguments
    let cli = Cli::parse();

    // Run the application and handle any errors
    if let Err(e) = run(cli) {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Runs the application logic based on the parsed CLI arguments
///
/// # Arguments
///
/// * `cli` - A struct containing the parsed command line arguments
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - Returns an empty Ok result on success, or an error on failure
fn run(cli: cli::Cli) -> Result<()> {
    // Open the git repository located at the current directory
    let _repo = Repository::open(".")?;

    // Handle the appropriate command based on the parsed CLI arguments
    match cli.command {
        cli::Commands::Create { name, parent } => {
            create::handle_new_branch(&_repo, &name, parent.as_deref())?;
        }
    }
    Ok(())
}
