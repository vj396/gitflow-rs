use clap::{Parser, Subcommand};

/// GitFlow CLI for managing GitHub development workflow
///
/// # Attributes
///
/// * `verbose` - Verbose output (use multiple for increased verbosity)
/// * `command` - The subcommand to execute
#[derive(Debug, Parser)]
#[clap(
    name = "gitflow",
    about = "A CLI to manage GitHub development workflow",
    version,
    author
)]
pub struct Cli {
    /// Verbose output (use multiple for increased verbosity)
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// The subcommand to execute
    #[clap(subcommand)]
    pub command: Commands,
}

/// GitFlow CLI subcommands
///
/// # Variants
///
/// * `Create` - Create a new branch based on the current branch or specified parent
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create a new branch based on the current branch or specified parent
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the new branch
    /// * `parent` - Parent branch to use (defaults to current branch)
    Create {
        /// Name of the new branch
        name: String,

        /// Parent branch to use (defaults to current branch)
        #[clap(long)]
        parent: Option<String>,
    },
}
