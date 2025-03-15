//! Module for the GitFlow CLI.
//!
//! This module uses Clap to parse command line arguments and defines the subcommands and
//! options available for managing the GitHub development workflow in GitFlow.
//!
//! # Details
//! Detailed documentation, including descriptions of subcommands and their options, is provided for clarity.

use crate::git::branch::BranchRelationStrategy;
use clap::{Parser, Subcommand, ValueEnum};

/// GitFlow CLI for managing GitHub development workflow
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

    #[clap(subcommand)]
    pub command: Commands,
}

/// GitFlow CLI subcommands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create a new branch based on the current branch or specified parent
    Create {
        /// Name of the new branch
        name: String,

        /// Parent branch to use (defaults to current branch)
        #[clap(long)]
        parent: Option<String>,
    },

    /// Merge parent branches into child branches recursively
    Cascade {
        /// Skip confirmation prompt
        #[clap(long)]
        yes: bool,

        /// Strategy for detecting branch relationships
        #[clap(long, value_enum)]
        strategy: Option<BranchDetectionStrategy>,
    },

    /// Sync the local branch with remote and create a pull request
    Sync {
        /// Title of the PR (if not provided, will use the last commit message)
        #[clap(long)]
        title: Option<String>,

        /// Skip confirmation prompt
        #[clap(long)]
        yes: bool,

        /// Base branch for PR (if not provided, will try to determine from branch structure)
        #[clap(long)]
        base: Option<String>,
    },

    /// Show the branch structure with PR information.
    Show {
        /// Strategy for detecting branch relationships
        #[clap(long, value_enum)]
        strategy: Option<BranchDetectionStrategy>,
    },

    /// Configure default settings
    Config {
        /// Set the default base branch
        #[clap(long)]
        default_base: Option<String>,

        /// Set the default branch detection strategy
        #[clap(long, value_enum)]
        detection_strategy: Option<BranchDetectionStrategy>,

        /// Add a manual branch relationship (format: parent:child)
        #[clap(long)]
        add_relationship: Option<String>,

        /// Remove a manual branch relationship (format: parent:child)
        #[clap(long)]
        remove_relationship: Option<String>,
    },
}

/// Command-line friendly enum for branch detection strategies
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum BranchDetectionStrategy {
    /// Use commit history to determine relationships
    History,
    /// Use branch creation timestamps
    Time,
    /// Consider default branch as parent of all others
    Default,
    /// Use explicit configuration
    Manual,
}

impl From<BranchDetectionStrategy> for BranchRelationStrategy {
    fn from(strategy: BranchDetectionStrategy) -> Self {
        match strategy {
            BranchDetectionStrategy::History => BranchRelationStrategy::CommitHistory,
            BranchDetectionStrategy::Time => BranchRelationStrategy::CreationTime,
            BranchDetectionStrategy::Default => BranchRelationStrategy::DefaultRoot,
            BranchDetectionStrategy::Manual => BranchRelationStrategy::Manual,
        }
    }
}
