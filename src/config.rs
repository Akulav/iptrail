//! User settings and on-disk locations.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const MIN_INTERVAL_SECS: u64 = 5;
pub const MAX_INTERVAL_SECS: u64 = 3600;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Seconds between public IP checks.
    pub interval_secs: u64,
    /// Start monitoring as soon as the app launches.
    pub auto_start: bool,
    /// Show a desktop notification when the IP changes.
    pub notifications: bool,
    /// Look up location / network / proxy info for each new IP.
    pub ip_intel: bool,
    /// Check each new IP against public Tor and abuse blocklists.
    pub threat_feeds: bool,
    /// Hide to the system tray when minimized (Windows).
    pub minimize_to_tray: bool,
    /// Hide to the system tray instead of quitting when the window is closed (Windows).
    pub close_to_tray: bool,
    /// Custom location of the human-readable change log. `None` uses the data directory.
    pub log_path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            interval_secs: 15,
            auto_start: true,
            notifications: true,
            ip_intel: true,
            threat_feeds: true,
            minimize_to_tray: true,
            close_to_tray: false,
            log_path: None,
        }
    }
}

impl Settings {
    pub fn load(paths: &Paths) -> Self {
        std::fs::read_to_string(&paths.settings)
            .ok()
            .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
            .map(Settings::sanitized)
            .unwrap_or_default()
    }

    pub fn save(&self, paths: &Paths) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        crate::history::write_private(&paths.settings, json.as_bytes())
    }

    pub fn sanitized(mut self) -> Self {
        self.interval_secs = self.interval_secs.clamp(MIN_INTERVAL_SECS, MAX_INTERVAL_SECS);
        if self.log_path.as_ref().is_some_and(|p| p.as_os_str().is_empty()) {
            self.log_path = None;
        }
        self
    }

    /// The change log. A relative custom path is taken relative to the data folder, not to
    /// whatever folder the app happened to be started from.
    pub fn log_file(&self, paths: &Paths) -> PathBuf {
        match &self.log_path {
            Some(p) if p.is_relative() => paths.data_dir().join(p),
            Some(p) => p.clone(),
            None => paths.default_log.clone(),
        }
    }
}

/// Every file IPTrail reads or writes.
#[derive(Debug, Clone)]
pub struct Paths {
    pub settings: PathBuf,
    pub history: PathBuf,
    pub default_log: PathBuf,
    pub cache_dir: PathBuf,
}

impl Paths {
    /// Everything lives in the folder IPTrail is installed in (next to the program when not
    /// installed); never in the OS's config / data / cache folders. See `crate::install`.
    pub fn discover() -> Self {
        Self::in_dir(&crate::install::data_root())
    }

    /// Every file inside `base` (also used by `--demo`).
    pub fn in_dir(base: &Path) -> Self {
        Self {
            settings: base.join("settings.json"),
            history: base.join("history.jsonl"),
            default_log: base.join("ip-changes.log"),
            cache_dir: base.join("cache"),
        }
    }

    pub fn data_dir(&self) -> &Path {
        self.history.parent().unwrap_or(Path::new("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_is_clamped() {
        let s = Settings { interval_secs: 1, ..Default::default() }.sanitized();
        assert_eq!(s.interval_secs, MIN_INTERVAL_SECS);
        let s = Settings { interval_secs: 99_999, ..Default::default() }.sanitized();
        assert_eq!(s.interval_secs, MAX_INTERVAL_SECS);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let s: Settings = serde_json::from_str(r#"{ "interval_secs": 42 }"#).unwrap();
        assert_eq!(s.interval_secs, 42);
        assert!(s.notifications);
    }

    #[test]
    fn relative_log_path_is_inside_the_data_folder() {
        let base = std::env::temp_dir().join("iptrail-paths");
        let paths = Paths::in_dir(&base);
        let s = Settings { log_path: Some("logs/ip.log".into()), ..Default::default() };
        assert_eq!(s.log_file(&paths), base.join("logs/ip.log"));
        assert_eq!(Settings::default().log_file(&paths), base.join("ip-changes.log"));
    }
}
