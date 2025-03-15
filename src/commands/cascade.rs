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
/// * `repo` - A reference to the Git repository
/// * `yes` - A boolean flag to confirm all merges without prompting
///
/// # Returns
///
/// * `Result<()>` - Returns an empty Ok result on success, or an error on failure
pub fn handle_cascade(repo: &Repository, yes: bool) -> Result<()> {
    let branch_tree = git::get_branch_tree(repo)?;

    // Show the planned merges
    info!("Planning to perform the following merges:");
    for (parent, children) in &branch_tree {
        for child in children {
            info!("  {} -> {}", parent, child);
        }
    }

    // Ask for confirmation unless --yes flag is provided
    if !yes && !prompt_confirmation("Proceed with merges?")? {
        return Err(GitFlowError::Aborted(
            "Merge operation cancelled".to_string(),
        ));
    }

    let mut processed = HashMap::new();

    // Process each root branch
    let root_branches = git::find_root_branches(&branch_tree);
    for branch in root_branches {
        merge_recursive(repo, &branch, &branch_tree, &mut processed)?;
    }

    info!("Cascade merge completed successfully");
    Ok(())
}

/// Recursively merge branches based on the branch hierarchy
///
/// # Arguments
///
/// * `repo` - A reference to the Git repository
/// * `branch` - The current branch to process
/// * `branch_tree` - A hashmap representing the branch tree
/// * `processed` - A hashmap to keep track of processed branches
///
/// # Returns
///
/// * `Result<()>` - Returns an empty Ok result on success, or an error on failure
fn merge_recursive(
    repo: &Repository,
    branch: &str,
    branch_tree: &HashMap<String, Vec<String>>,
    processed: &mut HashMap<String, bool>,
) -> Result<()> {
    if processed.contains_key(branch) {
        return Ok(());
    }

    // Process this branch
    processed.insert(branch.to_string(), true);

    // Get children of this branch
    if let Some(children) = branch_tree.get(branch) {
        for child in children {
            // Merge this branch into child
            match git::merge_branch(repo, branch, child) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Failed to merge {} into {}: {}", branch, child, e);
                    // Continue with other branches even if one fails
                }
            }

            // Recursively process children
            merge_recursive(repo, child, branch_tree, processed)?;
        }
    }

    Ok(())
}
