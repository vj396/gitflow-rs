//! Module for the 'config' command.
//!
//! This module handles configuration of global GitFlow settings including the default base branch,
//! branch detection strategy, and manual branch relationships.
//!
//! # Details
//! Enhanced documentation is provided for clearer maintenance and easier future updates.

use crate::cli::BranchDetectionStrategy;
use crate::configuration::Config;
use crate::error::{GitFlowError, Result};
use log::info;

/// Handle the 'config' command to configure global settings
///
/// # Arguments
///
/// * `default_base`         - Optional default base branch name.
/// * `detection_strategy`   - Optional detection strategy for branch detection.
/// * `add_relationship`     - Optional string in "parent:child" format to add a branch relationship.
/// * `remove_relationship`  - Optional string in "parent:child" format to remove a branch relationship.
///
/// # Returns
///
/// * `Result<()>` - Ok on success; otherwise returns a GitFlowError wrapped in Err.
///
/// # Examples
///
/// ```rust
/// // Example usage:
/// // handle_config(Some("main"), Some(BranchDetectionStrategy::Default), Some("main:feature"), None)?;
/// ```
pub fn handle_config(
    default_base: Option<&str>,
    detection_strategy: Option<BranchDetectionStrategy>,
    add_relationship: Option<&str>,
    remove_relationship: Option<&str>,
) -> Result<()> {
    let mut config = Config::load()?;

    // Update configuration based on provided options
    if let Some(base) = default_base {
        config.set_default_base_branch(base.to_string())?;
        info!("Default base branch set to: {}", base);
    }

    if let Some(strategy) = detection_strategy {
        config.set_branch_detection_strategy(strategy.into())?;
        info!("Default branch detection strategy set to: {:?}", strategy);
    }

    if let Some(relation) = add_relationship {
        // Parse parent:child format
        let parts: Vec<&str> = relation.split(':').collect();
        if parts.len() != 2 {
            return Err(GitFlowError::Config(
                "Relationship must be in format 'parent:child'".to_string(),
            ));
        }

        let parent = parts[0].trim();
        let child = parts[1].trim();

        if parent.is_empty() || child.is_empty() {
            return Err(GitFlowError::Config(
                "Parent and child branch names cannot be empty".to_string(),
            ));
        }

        config.add_branch_relationship(parent.to_string(), child.to_string())?;
        info!(
            "Added branch relationship: {} is parent of {}",
            parent, child
        );
    }

    if let Some(relation) = remove_relationship {
        // Parse parent:child format
        let parts: Vec<&str> = relation.split(':').collect();
        if parts.len() != 2 {
            return Err(GitFlowError::Config(
                "Relationship must be in format 'parent:child'".to_string(),
            ));
        }

        let parent = parts[0].trim();
        let child = parts[1].trim();

        config.remove_branch_relationship(parent, child)?;
        info!(
            "Removed branch relationship: {} is parent of {}",
            parent, child
        );
    }

    // If no options were provided, show current configuration
    if default_base.is_none()
        && detection_strategy.is_none()
        && add_relationship.is_none()
        && remove_relationship.is_none()
    {
        info!("Current configuration:");
        info!("Default base branch: {}", config.default_base_branch);
        info!(
            "Branch detection strategy: {:?}",
            config.branch_detection_strategy
        );
        info!("Manual branch relationships:");

        if config.branch_relationships.is_empty() {
            info!("  None defined");
        } else {
            for (parent, children) in &config.branch_relationships {
                for child in children {
                    info!("  {} -> {}", parent, child);
                }
            }
        }

        info!("Tracked PRs: {}", config.prs.len());
    }

    Ok(())
}
