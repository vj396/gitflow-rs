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
/// * `Result<()>` - Returns an empty Ok result on success, or an error on failure.
pub fn merge_branch(repo: &Repository, from: &str, to: &str) -> Result<()> {
    info!("Merging {} into {}", from, to);

    // Check if there are uncommitted changes
    let status = get_repo_status(repo)?;
    if !status.is_empty() {
        return Err(GitFlowError::Aborted(
            "There are uncommitted changes. Please commit or stash them first.".to_string(),
        ));
    }

    // Get the current branch to restore later if needed
    let original_branch = get_current_branch(repo)?;

    // Checkout the target branch
    checkout_branch(repo, to)?;

    // Find the annotated commit to merge
    let reference = repo.find_reference(&format!("refs/heads/{}", from))?;
    let annotated_commit = repo.reference_to_annotated_commit(&reference)?;

    // Prepare merge options
    let mut merge_options = MergeOptions::new();
    merge_options.fail_on_conflict(false);

    // Do the merge analysis
    let analysis = repo.merge_analysis(&[&annotated_commit])?;

    // Handle different merge scenarios
    if analysis.0.is_up_to_date() {
        info!("Already up-to-date");
    } else if analysis.0.is_fast_forward() {
        // Fast-forward merge
        let commit = repo.find_annotated_commit(annotated_commit.id())?;

        info!("Fast-forward merge");

        // Do the fast-forward
        let mut target_ref = repo.find_reference(&format!("refs/heads/{}", to))?;
        target_ref.set_target(commit.id(), "Fast-forward")?;
        repo.checkout_tree(&repo.find_object(commit.id(), None)?, None)?;
        repo.set_head(&target_ref.name().unwrap())?;
    } else {
        // Normal merge
        let sig = repo.signature()?;

        // Perform the merge
        let result = repo.merge(&[&annotated_commit], Some(&mut merge_options), None);

        if let Err(e) = result {
            if e.code() == ErrorCode::Conflict {
                // Handle merge conflicts
                warn!("Merge conflicts detected");

                // Abort the merge
                repo.cleanup_state()?;

                // Restore the original branch
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

        // Get the index with conflicts (if any)
        let mut index = repo.index()?;

        if index.has_conflicts() {
            // Abort the merge if there are conflicts
            repo.cleanup_state()?;

            // Restore the original branch
            if original_branch != to {
                checkout_branch(repo, &original_branch)?;
            }

            return Err(GitFlowError::Aborted(format!(
                "Merge conflicts detected between {} and {}. Please resolve manually.",
                from, to
            )));
        }

        // No conflicts, create the commit
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        // Get parent commits
        let head_commit = repo.head()?.peel_to_commit()?;
        let merged_commit = repo.find_commit(annotated_commit.id())?;

        // Create the merge commit
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("Merge branch '{}' into '{}'", from, to),
            &tree,
            &[&head_commit, &merged_commit],
        )?;

        // Cleanup the merge state
        repo.cleanup_state()?;
    }

    // Return to the original branch if needed
    if original_branch != to {
        checkout_branch(repo, &original_branch)?;
    }

    info!("Successfully merged {} into {}", from, to);
    Ok(())
}
