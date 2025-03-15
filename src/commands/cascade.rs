//! Module for the 'cascade' command.
//!
//! This module handles recursively merging branches according to the detected branch hierarchy.
//! It loads configuration, determines the branch detection strategy, plans the merges, and
//! prompts the user for confirmation before executing merge operations.
//!
//! # Details
//! Detailed documentation is provided for easier maintenance and clarity.

use crate::cli::BranchDetectionStrategy;
use crate::configuration::Config;
use crate::error::{GitFlowError, Result};
use crate::git;
use crate::utils::prompt_confirmation;
use git2::Repository;
use log::{info, warn};
use std::collections::HashMap;

/// Handle the 'cascade' command to merge branches recursively
///
/// # Arguments
///
/// * `repo`         - A reference to the Git repository.
/// * `yes`          - Flag to bypass confirmation prompts.
/// * `strategy_opt` - Optional branch detection strategy from the CLI.
///
/// # Returns
///
/// * `Result<()>`   - Ok on success, or an error if merging fails.
///
/// # Examples
///
/// ```rust
/// // Example usage:
/// // handle_cascade(&repo, false, Some(BranchDetectionStrategy::Default))?;
/// ```
pub fn handle_cascade(
    repo: &Repository,
    yes: bool,
    strategy_opt: Option<BranchDetectionStrategy>,
) -> Result<()> {
    // Load configuration for branch detection strategy.
    let config = Config::load()?;

    // Determine the branch detection strategy.
    let mut strategy = match strategy_opt {
        Some(s) => s.into(),
        None => config.branch_detection_strategy,
    };

    info!("Using branch detection strategy: {:?}", strategy);

    // Retrieve the branch tree using the selected strategy.
    let mut branch_tree = git::get_branch_tree(repo, strategy, &config)?;

    if branch_tree.is_empty() && strategy_opt.is_none() {
        info!("No branch hierarchy detected with current strategy.");

        // Attempt alternative strategies ordered by likelihood of success.
        let alternatives = [
            git::BranchRelationStrategy::CreationTime,
            git::BranchRelationStrategy::DefaultRoot,
            git::BranchRelationStrategy::Manual,
        ];

        for alt_strategy in alternatives.iter() {
            if *alt_strategy == strategy {
                continue; // Skip the current strategy.
            }

            if !prompt_confirmation(&format!("Try with {:?} strategy?", alt_strategy))? {
                continue; // User declined this alternative.
            }

            strategy = *alt_strategy;
            branch_tree = git::get_branch_tree(repo, strategy, &config)?;

            if !branch_tree.is_empty() {
                info!("Found branch hierarchy with {:?} strategy!", strategy);

                if prompt_confirmation("Set this as your default strategy?")? {
                    let mut config = Config::load()?;
                    config.set_branch_detection_strategy(strategy)?;
                    info!("Default strategy updated to {:?}", strategy);
                }
                break;
            }
        }
    }

    if branch_tree.is_empty() {
        info!("No branch hierarchy detected with any strategy. Try setting up manual relationships.");
        return Ok(());
    }

    // Display the planned merge operations.
    info!("Planning to perform the following merges:");
    for (parent, children) in &branch_tree {
        for child in children {
            info!("  {} -> {}", parent, child);
        }
    }

    // Confirm execution unless the '--yes' flag is provided.
    if !yes && !prompt_confirmation("Proceed with merges?")? {
        return Err(GitFlowError::Aborted("Merge operation cancelled".to_string()));
    }

    let mut processed = HashMap::new();

    // Recursively process each root branch.
    let root_branches = git::find_root_branches(&branch_tree);
    for branch in root_branches {
        merge_recursive(repo, &branch, &branch_tree, &mut processed)?;
    }

    info!("Cascade merge completed successfully");
    Ok(())
}

/// Recursively merge branches based on the branch hierarchy.
///
/// # Arguments
///
/// * `repo`         - The Git repository.
/// * `branch`       - The current branch to process.
/// * `branch_tree`  - Mapping from parent branches to child branches.
/// * `processed`    - A mutable map tracking processed branches to avoid duplication.
///
/// # Returns
///
/// * `Result<()>`   - Ok on success.
///
/// # Examples
///
/// ```rust
/// // Example usage:
/// // merge_recursive(&repo, "main", &branch_tree, &mut HashMap::new())?;
/// ```
fn merge_recursive(
    repo: &Repository,
    branch: &str,
    branch_tree: &HashMap<String, Vec<String>>,
    processed: &mut HashMap<String, bool>,
) -> Result<()> {
    if processed.contains_key(branch) {
        return Ok(());
    }

    // Mark this branch as processed.
    processed.insert(branch.to_string(), true);

    // For each child branch, merge the current branch and process recursively.
    if let Some(children) = branch_tree.get(branch) {
        for child in children {
            // Attempt merge of parent branch into child branch.
            match git::merge_branch(repo, branch, child) {
                Ok(_) => {},
                Err(e) => {
                    warn!("Failed to merge {} into {}: {}", branch, child, e);
                }
            }
            merge_recursive(repo, child, branch_tree, processed)?;
        }
    }

    Ok(())
}
