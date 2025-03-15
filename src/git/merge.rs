//! Module for Git merge operations.
//!
//! This module provides functionality to merge one branch into another with proper conflict handling.
//! It supports fast-forward merges as well as normal merges, and cleans up merge state on conflict.
//!
//! # Details
//! Detailed documentation and example usage are provided to simplify future maintenance.

use crate::error::{GitFlowError, Result};
use crate::git::branch::{checkout_branch, get_current_branch};
use crate::git::status::get_repo_status;
use git2::{ErrorCode, MergeOptions, Repository};
use log::{info, warn};

/// Merge one branch into another with proper conflict handling.
///
/// # Arguments
///
/// * `repo` - A reference to the Git repository.
/// * `from` - The source branch name.
/// * `to` - The target branch name.
///
/// # Returns
///
/// * `Result<()>` - Ok on success, or an error if the merge fails.
///
/// # Examples
/// ```rust
/// // Example: Merge branch "feature" into "main"
/// // let repo = Repository::open(".")?;
/// // merge_branch(&repo, "feature", "main")?;
/// ```
pub fn merge_branch(repo: &Repository, from: &str, to: &str) -> Result<()> {
    info!("Merging {} into {}", from, to);

    // Ensure there are no uncommitted changes in the repository.
    let status = get_repo_status(repo, false)?;
    if !status.is_empty() {
        return Err(GitFlowError::Aborted(
            "There are uncommitted changes. Please commit or stash them first.".to_string(),
        ));
    }

    // Save the current branch to restore later.
    let original_branch = get_current_branch(repo)?;

    // Checkout the target branch.
    checkout_branch(repo, to)?;

    // Find the annotated commit of the source branch.
    let reference = repo.find_reference(&format!("refs/heads/{}", from))?;
    let annotated_commit = repo.reference_to_annotated_commit(&reference)?;

    // Prepare merge options.
    let mut merge_options = MergeOptions::new();
    merge_options.fail_on_conflict(false);

    // Perform merge analysis.
    let analysis = repo.merge_analysis(&[&annotated_commit])?;

    if analysis.0.is_up_to_date() {
        info!("Already up-to-date");
    } else if analysis.0.is_fast_forward() {
        // Fast-forward merge.
        let commit = repo.find_annotated_commit(annotated_commit.id())?;
        info!("Performing fast-forward merge");

        let mut target_ref = repo.find_reference(&format!("refs/heads/{}", to))?;
        target_ref.set_target(commit.id(), "Fast-forward")?;
        repo.checkout_tree(&repo.find_object(commit.id(), None)?, None)?;
        repo.set_head(target_ref.name().unwrap())?;
    } else {
        // Normal merge process.
        let sig = repo.signature()?;
        let result = repo.merge(&[&annotated_commit], Some(&mut merge_options), None);

        if let Err(e) = result {
            if e.code() == ErrorCode::Conflict {
                warn!("Merge conflicts detected");
                repo.cleanup_state()?;
                if original_branch != to {
                    checkout_branch(repo, &original_branch)?;
                }
                return Err(GitFlowError::Aborted(format!(
                    "Merge conflicts detected between {} and {}. Please resolve manually.",
                    from, to
                )));
            } else {
                return Err(GitFlowError::Git(e));
            }
        }

        // Verify if conflicts exist in the merge index.
        let mut index = repo.index()?;
        if index.has_conflicts() {
            repo.cleanup_state()?;
            if original_branch != to {
                checkout_branch(repo, &original_branch)?;
            }
            return Err(GitFlowError::Aborted(format!(
                "Merge conflicts detected between {} and {}. Please resolve manually.",
                from, to
            )));
        }

        // Create the merge commit.
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let head_commit = repo.head()?.peel_to_commit()?;
        let merged_commit = repo.find_commit(annotated_commit.id())?;
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("Merge branch '{}' into '{}'", from, to),
            &tree,
            &[&head_commit, &merged_commit],
        )?;
        repo.cleanup_state()?;
    }

    // Return to the original branch if necessary.
    if original_branch != to {
        checkout_branch(repo, &original_branch)?;
    }

    info!("Successfully merged {} into {}", from, to);
    Ok(())
}
