//! Main entry point for the GitFlow application.
//!
//! This module parses command line arguments, initializes logging, opens the Git repository,
//! and dispatches commands to the corresponding handlers.
//!
//! # Details
//! The main function coordinates application startup and error handling. The run() function
//! encapsulates the application logic with clear documentation on arguments and behavior.

mod cli;
mod commands;
mod configuration;
mod error;
mod git;
mod github;
mod utils;

use cli::Cli;
use commands::{cascade, config, create, show, sync};
use error::Result;

use clap::Parser;
use git2::Repository;
use log::error;

/// Entry point of the application.
fn main() {
    // Parse command line arguments.
    let cli = Cli::parse();
    utils::init_logger(cli.verbose);

    // Run the application logic and handle any errors.
    if let Err(e) = run(cli) {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Runs the application logic based on the parsed CLI arguments.
///
/// # Arguments
///
/// * `cli` - A struct containing the parsed command line arguments.
///
/// # Returns
///
/// * `Result<()>` - Returns Ok on success, or an error on failure.
fn run(cli: cli::Cli) -> Result<()> {
    match &cli.command {
        cli::Commands::Config {
            default_base,
            detection_strategy,
            add_relationship,
            remove_relationship,
        } => {
            return config::handle_config(
                default_base.as_deref(),
                *detection_strategy,
                add_relationship.as_deref(),
                remove_relationship.as_deref(),
            );
        }
        _ => {}
    }

    // Open the Git repository located in the current directory.
    let repo = Repository::open(".")?;

    // Dispatch based on the user's command.
    match cli.command {
        cli::Commands::Create { name, parent } => {
            create::handle_new_branch(&repo, &name, parent.as_deref()).map_err(|e| {
                println!("Error: {}", e);
                e
            })?;
        }
        cli::Commands::Cascade { yes, strategy } => {
            cascade::handle_cascade(&repo, yes, strategy).map_err(|e| {
                println!("Error: {}", e);
                e
            })?;
        }
        cli::Commands::Show { strategy } => {
            show::handle_show(&repo, strategy).map_err(|e| {
                println!("Error: {}", e);
                e
            })?;
        }
        cli::Commands::Sync { title, yes, base } => {
            sync::handle_sync(&repo, title.as_deref(), yes, base.as_deref()).map_err(|e| {
                println!("Error: {}", e);
                e
            })?;
        }

        cli::Commands::Config { .. } => {
            // Already handled above.
        }
    }
    Ok(())
}
