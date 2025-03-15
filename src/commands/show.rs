//! Module for the 'show' command.
//!
//! This module handles displaying the Git branch hierarchy along with pull request (PR)
//! information and commit messages. It loads configuration, determines the branch detection
//! strategy, and prints the branch structure accordingly.
//!
//! # Details
//! Detailed documentation is provided for easier maintenance and clarity.

use crate::cli::BranchDetectionStrategy;
use crate::configuration::Config;
use crate::error::Result;
use crate::git;
use crate::utils::print_branch_hierarchy;
use git2::{BranchType, Repository};
use log::info;
use std::collections::HashMap;

/// Handle the 'show' command to display branch structure with PR information
///
/// # Arguments
/// * `repo` - A reference to the Git repository.
/// * `strategy_opt` - An optional branch detection strategy from the CLI.
///
/// # Returns
/// * `Result<()>` - Returns an empty Ok result on success or an error on failure.
///
/// # Examples
/// ```rust
/// // Example usage:
/// // let repo = Repository::open(".")?;
/// // handle_show(&repo, Some(BranchDetectionStrategy::Default))?;
/// ```
pub fn handle_show(repo: &Repository, strategy_opt: Option<BranchDetectionStrategy>) -> Result<()> {
    // Load configuration for branch detection strategy.
    let config = Config::load()?;

    // Determine which branch detection strategy to use.
    // Use the provided strategy if available; otherwise, fallback to the configuration setting.
    let strategy = match strategy_opt {
        Some(s) => s.into(),
        None => config.branch_detection_strategy,
    };

    // Log the selected branch detection strategy for debugging purposes.
    info!("Using branch detection strategy: {:?}", strategy);

    // Retrieve the branch hierarchy using the determined strategy.
    let branch_tree = git::get_branch_tree(repo, strategy, &config)?;

    // Retrieve the current branch to enable highlighting in the output.
    let current_branch = git::get_current_branch(repo)?;

    // If no branch hierarchy is detected, list all local branches.
    if branch_tree.is_empty() {
        info!("No branch hierarchy detected.");

        // Iterate over local branches to print their status.
        let branches = repo.branches(Some(BranchType::Local))?;
        for branch_result in branches {
            let (branch, _) = branch_result?;
            let name = branch.name()?.unwrap_or("").to_string();

            println!(
                "{} is {}",
                name,
                if name == current_branch {
                    "current"
                } else {
                    "not current"
                }
            );
        }
        return Ok(());
    }

    // Identify root branches (branches without parent branches).
    let root_branches = git::find_root_branches(&branch_tree);

    // Collect pull request (PR) information for each branch from the configuration.
    let mut pr_info = HashMap::new();
    for (branch, info) in &config.prs {
        pr_info.insert(branch.clone(), (info.number, info.url.clone()));
    }

    // Collect the first line of the commit message for each branch.
    let mut commit_messages = HashMap::new();
    for branch_name in branch_tree.keys() {
        if let Ok(commit) = git::get_branch_commit(repo, branch_name) {
            if let Some(message) = commit.message() {
                commit_messages.insert(
                    branch_name.clone(),
                    message.lines().next().unwrap_or("").to_string(),
                );
            }
        }
    }

    // Print the complete branch hierarchy along with PR and commit message details.
    print_branch_hierarchy(
        &branch_tree,
        &root_branches,
        &current_branch,
        &pr_info,
        &commit_messages,
    );

    Ok(())
}
