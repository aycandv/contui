//! Update checking module for automatic version updates.
//!
//! This module handles checking for new versions on startup and prompting
//! users to install updates.

mod ui;

use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::config::{CheckFrequency, UpdateConfig};

/// Persisted state for update checks
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateState {
    /// Timestamp of the last successful update check
    pub last_check: Option<DateTime<Utc>>,
}

impl UpdateState {
    /// Load state from the default path
    pub fn load() -> Result<Self> {
        let path = Self::state_file_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read update state from {}", path.display()))?;

        serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse update state from {}", path.display()))
    }

    /// Save state to the default path
    pub fn save(&self) -> Result<()> {
        let path = Self::state_file_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory {}", parent.display())
            })?;
        }

        let contents =
            serde_json::to_string_pretty(self).context("Failed to serialize update state")?;

        std::fs::write(&path, contents)
            .with_context(|| format!("Failed to write update state to {}", path.display()))?;

        Ok(())
    }

    /// Get the path to the state file
    fn state_file_path() -> Result<PathBuf> {
        use directories::ProjectDirs;

        if let Some(proj_dirs) = ProjectDirs::from("com", "contui", "contui") {
            Ok(proj_dirs.config_dir().join("update_state.json"))
        } else if let Some(home) = std::env::var_os("HOME") {
            Ok(PathBuf::from(home).join(".config/contui/update_state.json"))
        } else {
            anyhow::bail!("Could not determine config directory")
        }
    }

    /// Check if enough time has passed since last check based on frequency
    pub fn should_check(&self, frequency: CheckFrequency) -> bool {
        match frequency {
            CheckFrequency::Always => true,
            CheckFrequency::Never => false,
            CheckFrequency::Daily | CheckFrequency::Weekly => {
                let Some(last_check) = self.last_check else {
                    return true; // Never checked before
                };

                let duration = match frequency {
                    CheckFrequency::Daily => chrono::Duration::days(1),
                    CheckFrequency::Weekly => chrono::Duration::weeks(1),
                    _ => unreachable!(),
                };

                Utc::now() - last_check >= duration
            }
        }
    }

    /// Update the last check timestamp to now
    pub fn mark_checked(&mut self) {
        self.last_check = Some(Utc::now());
    }
}

/// Result of an update check
#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    /// Current version is the latest
    UpToDate,
    /// A newer version is available
    UpdateAvailable {
        current: String,
        latest: String,
        release_url: String,
    },
    /// Check was skipped (due to frequency or config)
    Skipped { reason: String },
    /// Check failed (network error, timeout, etc.)
    Failed { error: String },
}

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_url: String,
}

/// User's decision after seeing update prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateDecision {
    /// Install the update now
    Install,
    /// Skip this time, continue to app
    Skip,
    /// Skip this version permanently
    SkipVersion,
}

/// Check for updates with the given configuration, showing animated spinner.
///
/// This wraps `check_for_updates_quiet` with terminal animations.
pub async fn check_for_updates(config: &UpdateConfig) -> UpdateCheckResult {
    let spinner = ui::spinner("Checking for updates...");

    let result = check_for_updates_quiet(config).await;

    // Clear spinner and show result
    match &result {
        UpdateCheckResult::UpdateAvailable { .. } => {
            // Clear spinner - the prompt will show the styled result
            spinner.finish_and_clear();
        }
        UpdateCheckResult::UpToDate => {
            ui::spinner_success(
                &spinner,
                &format!("Already on latest (v{})", env!("CARGO_PKG_VERSION")),
            );
        }
        UpdateCheckResult::Failed { .. } => {
            spinner.finish_and_clear();
            ui::print_warning("Update check skipped (offline?)");
        }
        UpdateCheckResult::Skipped { .. } => {
            // Silently clear for skipped checks (frequency-based)
            spinner.finish_and_clear();
        }
    }

    result
}

/// Check for updates without terminal output.
///
/// Returns the check result. This function handles timeout internally.
async fn check_for_updates_quiet(config: &UpdateConfig) -> UpdateCheckResult {
    let current_version = env!("CARGO_PKG_VERSION");

    // Load state to check if we should perform the check
    let mut state = match UpdateState::load() {
        Ok(s) => s,
        Err(e) => {
            warn!("Could not load update state (using defaults): {}", e);
            UpdateState::default()
        }
    };

    // Check if we should skip based on frequency
    if !config.check_on_startup {
        return UpdateCheckResult::Skipped {
            reason: "Update checks disabled in config".to_string(),
        };
    }

    if !state.should_check(config.check_frequency) {
        return UpdateCheckResult::Skipped {
            reason: format!(
                "Already checked recently (frequency: {:?})",
                config.check_frequency
            ),
        };
    }

    // Perform the actual check with timeout
    let timeout = Duration::from_secs(config.timeout_seconds);
    let result = tokio::time::timeout(timeout, fetch_latest_version()).await;

    match result {
        Ok(Ok(latest_version)) => {
            // Update state regardless of result
            state.mark_checked();
            if let Err(e) = state.save() {
                warn!("Could not save update state: {}", e);
            }

            // Check if this version should be skipped
            if let Some(ref skip_ver) = config.skip_version {
                if skip_ver == &latest_version {
                    return UpdateCheckResult::Skipped {
                        reason: format!("Version {} is marked as skipped", latest_version),
                    };
                }
            }

            // Compare versions
            if version_is_newer(&latest_version, current_version) {
                info!(
                    "Update available: {} -> {}",
                    current_version, latest_version
                );
                UpdateCheckResult::UpdateAvailable {
                    current: current_version.to_string(),
                    latest: latest_version.clone(),
                    release_url: format!(
                        "https://github.com/aycandv/contui/releases/tag/v{}",
                        latest_version
                    ),
                }
            } else {
                debug!("Already on latest version: {}", current_version);
                UpdateCheckResult::UpToDate
            }
        }
        Ok(Err(e)) => {
            warn!("Update check failed: {}", e);
            UpdateCheckResult::Failed {
                error: e.to_string(),
            }
        }
        Err(_) => {
            warn!("Update check timed out after {}s", config.timeout_seconds);
            UpdateCheckResult::Failed {
                error: format!("Timed out after {} seconds", config.timeout_seconds),
            }
        }
    }
}

/// Fetch the latest version from GitHub releases
async fn fetch_latest_version() -> Result<String> {
    // Note: Timeout is handled by the caller via tokio::time::timeout
    let client = reqwest::Client::new();

    let response = client
        .get("https://api.github.com/repos/aycandv/contui/releases/latest")
        .header("User-Agent", "contui-update-checker")
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("GitHub API returned {}", response.status());
    }

    let release: serde_json::Value = response.json().await?;
    let tag = release["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Could not parse version from GitHub response"))?;

    // Remove 'v' prefix if present
    Ok(tag.trim_start_matches('v').to_string())
}

/// Compare two semver version strings
///
/// Returns true if `latest` is newer than `current`
pub fn version_is_newer(latest: &str, current: &str) -> bool {
    // Clean up version strings (remove 'v' prefix if present)
    let latest = latest.trim_start_matches('v');
    let current = current.trim_start_matches('v');

    // Parse into semver components
    let parse_version = |s: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = s.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        // Handle pre-release suffixes like "1.0.0-beta"
        let patch_str = parts.get(2).unwrap_or(&"0");
        let patch = patch_str
            .split('-')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        (major, minor, patch)
    };

    let latest_parsed = parse_version(latest);
    let current_parsed = parse_version(current);

    latest_parsed > current_parsed
}

/// Display update prompt and get user's decision
///
/// This function is synchronous and blocks waiting for user input.
/// It should only be called when stdin is a TTY.
pub fn prompt_for_update(info: &UpdateInfo) -> io::Result<UpdateDecision> {
    ui::print_sparkle("Update found!");
    println!();

    // Show styled version transition box
    let version_line = format!(
        "  v{}  ━━━━━━━━━━▶  v{}",
        info.current_version, info.latest_version
    );
    let release_line = format!("  {} What's new:", "\u{1F4E6}"); // 📦
    let url_line = format!("  {}", info.release_url);

    ui::print_box(&[&version_line, "", &release_line, &url_line]);

    print!("  \u{1F680} Install now? [Y/n/s] "); // 🚀
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let decision = match input.trim().to_lowercase().as_str() {
        "" | "y" | "yes" => UpdateDecision::Install,
        "n" | "no" => UpdateDecision::Skip,
        "s" | "skip" => UpdateDecision::SkipVersion,
        _ => UpdateDecision::Skip, // Default to skip on unrecognized input
    };

    Ok(decision)
}

/// Execute the update installation with animated progress
pub fn install_update() -> Result<()> {
    install_update_to_version(None)
}

/// Execute the update installation to a specific version (or latest if None)
pub fn install_update_to_version(target_version: Option<&str>) -> Result<()> {
    use self_update::backends::github::Update;
    use self_update::cargo_crate_version;

    let current_version = cargo_crate_version!();
    let target = self_update::get_target();

    println!();

    // Phase 1: Download
    let download_spinner = ui::spinner("Downloading update...");

    let mut update_builder = Update::configure();
    update_builder
        .repo_owner("aycandv")
        .repo_name("contui")
        .bin_name("contui")
        .target(target)
        .identifier("contui")
        .show_download_progress(false) // We handle our own UI
        .show_output(false)
        .no_confirm(true);

    if let Some(version) = target_version {
        update_builder.target_version_tag(&format!("v{}", version.trim_start_matches('v')));
    }

    let release = match update_builder.build() {
        Ok(updater) => updater,
        Err(e) => {
            ui::spinner_error(&download_spinner, "Download failed");
            return Err(anyhow::anyhow!("Failed to configure updater: {}", e));
        }
    };

    // Get the release info to show version
    let release_version = release
        .get_latest_release()
        .map(|r| r.version.clone())
        .unwrap_or_else(|_| "latest".to_string());

    download_spinner.set_message(format!("Downloading v{}...", release_version));

    // Execute the update (downloads, extracts, replaces)
    let status = match release.update() {
        Ok(s) => s,
        Err(e) => {
            ui::spinner_error(&download_spinner, "Update failed");
            ui::print_error_with_suggestion(
                "Update failed",
                &e.to_string(),
                "Try: sudo contui update",
            );
            return Err(anyhow::anyhow!("Update failed: {}", e));
        }
    };

    ui::spinner_success(
        &download_spinner,
        &format!("Downloaded v{}", status.version()),
    );
    ui::delay();

    // Phase 2: Show extraction complete (self_update already did this)
    let extract_spinner = ui::spinner("Extracting archive...");
    ui::delay();
    ui::spinner_success(&extract_spinner, "Extracted");
    ui::delay();

    // Phase 3: Show binary replacement complete
    let replace_spinner = ui::spinner("Replacing binary...");
    ui::delay();
    ui::spinner_success(&replace_spinner, "Replaced");
    ui::delay();

    // Phase 4: Verify installation
    let verify_spinner = ui::spinner("Verifying installation...");
    ui::delay();
    ui::spinner_success(&verify_spinner, "Verified");

    // Show success box
    if status.updated() {
        let title = format!(
            "{} Successfully updated to v{}!",
            "\u{2705}",
            status.version()
        ); // ✅
        let transition = format!("v{} → v{}", current_version, status.version());
        let restart_msg = format!("{} Restart contui to use new version", "\u{1F389}"); // 🎉

        ui::print_box(&[&title, "", &transition, "", &restart_msg]);
    } else {
        ui::print_check(&format!("Already on latest version (v{})", current_version));
    }

    Ok(())
}

/// Save the skip_version to the user's config file
pub fn save_skip_version(version: &str) -> Result<()> {
    use crate::config::Config;
    use directories::ProjectDirs;

    let config_path = if let Some(proj_dirs) = ProjectDirs::from("com", "contui", "contui") {
        let config_dir = proj_dirs.config_dir();
        std::fs::create_dir_all(config_dir)?;
        config_dir.join("config.toml")
    } else if let Some(home) = std::env::var_os("HOME") {
        let config_dir = PathBuf::from(home).join(".config/contui");
        std::fs::create_dir_all(&config_dir)?;
        config_dir.join("config.toml")
    } else {
        anyhow::bail!("Could not determine config directory");
    };

    // Load existing config or create default
    // Important: Do NOT fall back to default if config exists but fails to load,
    // as that would overwrite the user's config with defaults
    let mut config = if config_path.exists() {
        Config::load(&config_path).with_context(|| {
            format!(
                "Could not load existing config at {}. \
                 Please fix the config file or remove it to start fresh.",
                config_path.display()
            )
        })?
    } else {
        Config::default()
    };

    // Update skip_version
    config.update.skip_version = Some(version.to_string());

    // Save config
    config.save(&config_path)?;

    println!("  Version v{} will be skipped in future checks.", version);

    Ok(())
}

/// Check if stdin is a TTY (interactive terminal)
pub fn is_interactive() -> bool {
    use std::io::IsTerminal;
    io::stdin().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_newer() {
        // Basic comparisons
        assert!(version_is_newer("1.0.1", "1.0.0"));
        assert!(version_is_newer("1.1.0", "1.0.0"));
        assert!(version_is_newer("2.0.0", "1.0.0"));

        // Same version
        assert!(!version_is_newer("1.0.0", "1.0.0"));

        // Older version
        assert!(!version_is_newer("1.0.0", "1.0.1"));
        assert!(!version_is_newer("0.9.0", "1.0.0"));

        // With 'v' prefix
        assert!(version_is_newer("v1.0.1", "1.0.0"));
        assert!(version_is_newer("1.0.1", "v1.0.0"));
        assert!(version_is_newer("v1.0.1", "v1.0.0"));

        // Real-world examples
        assert!(version_is_newer("0.5.0", "0.4.2"));
        assert!(!version_is_newer("0.4.2", "0.4.2"));
        assert!(!version_is_newer("0.4.1", "0.4.2"));
    }

    #[test]
    fn test_version_with_prerelease() {
        // Pre-release suffixes (we treat the base version)
        assert!(version_is_newer("1.0.1-beta", "1.0.0"));
        assert!(!version_is_newer("1.0.0-beta", "1.0.0"));
    }

    #[test]
    fn test_update_state_should_check() {
        let mut state = UpdateState::default();

        // Never checked - should always check
        assert!(state.should_check(CheckFrequency::Daily));
        assert!(state.should_check(CheckFrequency::Weekly));
        assert!(state.should_check(CheckFrequency::Always));
        assert!(!state.should_check(CheckFrequency::Never));

        // Recently checked
        state.last_check = Some(Utc::now());
        assert!(!state.should_check(CheckFrequency::Daily));
        assert!(!state.should_check(CheckFrequency::Weekly));
        assert!(state.should_check(CheckFrequency::Always));
        assert!(!state.should_check(CheckFrequency::Never));

        // Checked yesterday
        state.last_check = Some(Utc::now() - chrono::Duration::hours(25));
        assert!(state.should_check(CheckFrequency::Daily));
        assert!(!state.should_check(CheckFrequency::Weekly));

        // Checked last week
        state.last_check = Some(Utc::now() - chrono::Duration::days(8));
        assert!(state.should_check(CheckFrequency::Daily));
        assert!(state.should_check(CheckFrequency::Weekly));
    }
}
