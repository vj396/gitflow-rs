//! Module for GitFlow configuration settings.
//!
//! This module defines the configuration structures for GitFlow including pull request information,
//! default base branch settings, branch relationships, and the branch detection strategy.
//! It also provides functions to load, save, and update the configuration persisted on disk.
//!
//! # Details
//! Detailed documentation is provided for clear maintenance and future updates.

use crate::error::{GitFlowError, Result};
use crate::git::branch::BranchRelationStrategy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// PR information stored in configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrInfo {
    pub url: String,
    pub number: u64,
    pub title: String,
    pub created_at: String,
}

/// GitFlow configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Map of branch names to PR information.
    pub prs: HashMap<String, PrInfo>,

    /// Default base branch (usually main or master).
    pub default_base_branch: String,

    /// Manual branch relationships for explicit configuration.
    #[serde(default)]
    pub branch_relationships: HashMap<String, Vec<String>>,

    /// Strategy to use for detecting branch relationships.
    #[serde(default)]
    pub branch_detection_strategy: BranchRelationStrategy,
}

impl Config {
    /// Load configuration from disk.
    ///
    /// # Returns
    ///
    /// * `Result<Config>` - The loaded configuration on success.
    ///
    /// # Examples
    /// ```rust
    /// // let config = Config::load()?;
    /// ```
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            // Create default configuration if none exists.
            let config = Config {
                prs: HashMap::new(),
                default_base_branch: "main".to_string(),
                branch_relationships: HashMap::new(),
                branch_detection_strategy: BranchRelationStrategy::default(),
            };
            config.save()?;
            return Ok(config);
        }

        let json = fs::read_to_string(&config_path)
            .map_err(|e| GitFlowError::Config(format!("Could not read config file: {}", e)))?;
        serde_json::from_str(&json)
            .map_err(|e| GitFlowError::Config(format!("Invalid config file format: {}", e)))
    }

    /// Save configuration to disk.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success or an error if saving fails.
    ///
    /// # Examples
    /// ```rust
    /// // config.save()?;
    /// ```
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        // Ensure the configuration directory exists.
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                GitFlowError::Config(format!("Could not create config directory: {}", e))
            })?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, json)
            .map_err(|e| GitFlowError::Config(format!("Could not write config file: {}", e)))?;
        Ok(())
    }

    /// Add a PR to the configuration.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name.
    /// * `pr_info` - The pull request information to add.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success.
    ///
    /// # Examples
    /// ```rust
    /// // config.add_pr("feature".to_string(), pr_info)?;
    /// ```
    pub fn add_pr(&mut self, branch: String, pr_info: PrInfo) -> Result<()> {
        self.prs.insert(branch, pr_info);
        self.save()?;
        Ok(())
    }

    /// Get PR information for a specific branch.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name.
    ///
    /// # Returns
    ///
    /// * `Option<&PrInfo>` - The PR information if available.
    ///
    /// # Examples
    /// ```rust
    /// // let info = config.get_pr("feature");
    /// ```
    pub fn get_pr(&self, branch: &str) -> Option<&PrInfo> {
        self.prs.get(branch)
    }

    /// Set the default base branch.
    ///
    /// # Arguments
    ///
    /// * `branch` - The branch name to set as the default base.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success.
    ///
    /// # Examples
    /// ```rust
    /// // config.set_default_base_branch("main".to_string())?;
    /// ```
    pub fn set_default_base_branch(&mut self, branch: String) -> Result<()> {
        self.default_base_branch = branch;
        self.save()?;
        Ok(())
    }

    /// Set the branch detection strategy.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The branch relation strategy to use.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success.
    ///
    /// # Examples
    /// ```rust
    /// // config.set_branch_detection_strategy(BranchRelationStrategy::Manual)?;
    /// ```
    pub fn set_branch_detection_strategy(
        &mut self,
        strategy: BranchRelationStrategy,
    ) -> Result<()> {
        self.branch_detection_strategy = strategy;
        self.save()?;
        Ok(())
    }

    /// Add a manual branch relationship.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent branch.
    /// * `child` - The child branch.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success.
    ///
    /// # Examples
    /// ```rust
    /// // config.add_branch_relationship("main".to_string(), "feature".to_string())?;
    /// ```
    pub fn add_branch_relationship(&mut self, parent: String, child: String) -> Result<()> {
        self.branch_relationships
            .entry(parent)
            .or_insert_with(Vec::new)
            .push(child);
        self.save()?;
        Ok(())
    }

    /// Remove a manual branch relationship.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent branch.
    /// * `child` - The child branch to remove.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success.
    ///
    /// # Examples
    /// ```rust
    /// // config.remove_branch_relationship("main", "feature")?;
    /// ```
    pub fn remove_branch_relationship(&mut self, parent: &str, child: &str) -> Result<()> {
        if let Some(children) = self.branch_relationships.get_mut(parent) {
            children.retain(|c| c != child);
            if children.is_empty() {
                self.branch_relationships.remove(parent);
            }
        }
        self.save()?;
        Ok(())
    }
}

/// Get the path to the configuration file.
///
/// # Returns
///
/// * `Result<PathBuf>` - The configuration file path on success.
///
/// # Examples
/// ```rust
/// // let path = get_config_path()?;
/// ```
pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| GitFlowError::Config("Could not determine config directory".to_string()))?
        .join("gitflow");
    Ok(config_dir.join("config.json"))
}
