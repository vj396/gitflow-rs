mod cli;

use clap::Parser;
use git2::Repository;
use std::result::Result;
use log::error;

/// The main entry point of the application
fn main() {
    // Parse command line arguments
    let cli = cli::Cli::parse();
    
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
fn run(cli: cli::Cli) -> Result<(), Box<dyn std::error::Error>> {    
    // Open the git repository located at the current directory
    let _repo = Repository::open(".")?;
    
    // Handle the appropriate command based on the parsed CLI arguments
    match cli.command {
        cli::Commands::New { name: _, parent: _} => {
            // TODO: Implement the logic for the 'New' command
        }
    }
    
    Ok(())
}
