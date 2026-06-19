pub mod github;

/// Auto-updater that polls GitHub Releases for new versions.
pub struct AutoUpdater {
    pub current_version: String,
}

impl AutoUpdater {
    pub fn new(current_version: &str) -> Self {
        Self {
            current_version: current_version.to_string(),
        }
    }
}
