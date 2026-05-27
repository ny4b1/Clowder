use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub downloads: DownloadSettings,
    pub playback: PlaybackSettings,
    pub appearance: AppearanceSettings,
    pub vpn: VpnSettings,
}

impl Settings {
    pub fn normalize(&mut self) {
        self.downloads.normalize();
        self.playback.normalize();
        self.appearance.normalize();
        self.vpn.normalize();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VpnSettings {
    pub enabled: bool,
}

impl VpnSettings {
    pub fn normalize(&mut self) {

    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DownloadSettings {
    pub directory: Option<String>,
    pub filename_template: String,
}

impl Default for DownloadSettings {
    fn default() -> Self {
        Self {
            directory: None,
            filename_template: DEFAULT_FILENAME_TEMPLATE.to_string(),
        }
    }
}

impl DownloadSettings {
    pub fn normalize(&mut self) {
        if let Some(dir) = &self.directory
            && dir.trim().is_empty()
        {
            self.directory = None;
        }
        if self.filename_template.trim().is_empty() {
            self.filename_template = DEFAULT_FILENAME_TEMPLATE.to_string();
        }
    }
}

pub const DEFAULT_FILENAME_TEMPLATE: &str = "{artist}_{id}.{ext}";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaybackSettings {
    pub autoplay: bool,
    pub remember_volume: bool,
    pub video_chunk_mb: u32,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            autoplay: true,
            remember_volume: true,
            video_chunk_mb: 2,
        }
    }
}

impl PlaybackSettings {
    pub fn video_chunk_bytes(&self) -> u64 {
        let mb = self.video_chunk_mb.clamp(1, 64);
        u64::from(mb) * 1024 * 1024
    }

    pub fn normalize(&mut self) {
        self.video_chunk_mb = self.video_chunk_mb.clamp(1, 64);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionPreference {
    #[default]
    System,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceSettings {
    pub theme: Theme,
    pub motion: MotionPreference,
    pub grid_min_tile_px: u32,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            motion: MotionPreference::System,
            grid_min_tile_px: 176,
        }
    }
}

impl AppearanceSettings {
    pub fn normalize(&mut self) {
        self.grid_min_tile_px = self.grid_min_tile_px.clamp(120, 320);
    }
}

pub fn load(app: &tauri::AppHandle) -> Settings {
    let path = match settings_path(app) {
        Ok(p) => p,
        Err(err) => {
            tracing::warn!(error = %format!("{err:#}"), "could not resolve settings path");
            return Settings::default();
        }
    };
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Settings::default(),
        Err(err) => {
            tracing::warn!(error = %err, "could not read settings file");
            return Settings::default();
        }
    };
    match serde_json::from_str::<Settings>(&contents) {
        Ok(mut s) => {
            s.normalize();
            s
        }
        Err(err) => {
            tracing::warn!(error = %err, "could not parse settings file; using defaults");
            Settings::default()
        }
    }
}

pub fn save(app: &tauri::AppHandle, settings: &Settings) -> Result<()> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("create settings directory")?;
    }
    let contents = serde_json::to_string_pretty(settings).context("encode settings")?;
    fs::write(&path, contents).context("write settings file")?;
    Ok(())
}

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .context("resolve app config dir")?;
    Ok(dir.join("settings.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_serializes() {
        let s = Settings {
            downloads: DownloadSettings {
                directory: Some("/tmp/clowder".to_string()),
                filename_template: "{id}.{ext}".to_string(),
            },
            playback: PlaybackSettings {
                autoplay: false,
                remember_volume: false,
                video_chunk_mb: 8,
            },
            appearance: AppearanceSettings {
                theme: Theme::Light,
                motion: MotionPreference::Never,
                grid_min_tile_px: 220,
            },
            vpn: VpnSettings { enabled: true },
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn settings_uses_defaults_on_missing_fields() {
        let s: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(s.downloads, DownloadSettings::default());
        assert_eq!(s.playback, PlaybackSettings::default());
        assert_eq!(s.appearance, AppearanceSettings::default());
        assert_eq!(s.vpn, VpnSettings::default());
    }

    #[test]
    fn vpn_defaults_to_disabled() {
        let v = VpnSettings::default();
        assert!(!v.enabled);
    }

    #[test]
    fn settings_ignores_unknown_legacy_fields() {
        let json = r#"{
            "connection_mode": "wg",
            "doh_provider": "cloudflare",
            "fail_closed_ech": true,
            "playback": {"video_chunk_mb": 4}
        }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.playback.video_chunk_mb, 4);
        assert_eq!(s.downloads, DownloadSettings::default());
    }

    #[test]
    fn download_settings_defaults_match_template() {
        let d = DownloadSettings::default();
        assert!(d.directory.is_none());
        assert_eq!(d.filename_template, "{artist}_{id}.{ext}");
    }

    #[test]
    fn playback_chunk_clamps_to_safe_range() {
        let p = PlaybackSettings {
            autoplay: true,
            remember_volume: true,
            video_chunk_mb: 0,
        };
        assert_eq!(p.video_chunk_bytes(), 1024 * 1024);

        let p = PlaybackSettings {
            autoplay: true,
            remember_volume: true,
            video_chunk_mb: 200,
        };
        assert_eq!(p.video_chunk_bytes(), 64 * 1024 * 1024);

        let p = PlaybackSettings {
            autoplay: true,
            remember_volume: true,
            video_chunk_mb: 4,
        };
        assert_eq!(p.video_chunk_bytes(), 4 * 1024 * 1024);
    }

    #[test]
    fn appearance_defaults_to_system_theme() {
        let a = AppearanceSettings::default();
        assert_eq!(a.theme, Theme::System);
        assert_eq!(a.motion, MotionPreference::System);
        assert_eq!(a.grid_min_tile_px, 176);
    }

    #[test]
    fn normalize_clamps_out_of_range_values() {
        let mut s = Settings {
            downloads: DownloadSettings {
                directory: Some("   ".to_string()),
                filename_template: "   ".to_string(),
            },
            playback: PlaybackSettings {
                autoplay: true,
                remember_volume: true,
                video_chunk_mb: 9999,
            },
            appearance: AppearanceSettings {
                theme: Theme::Dark,
                motion: MotionPreference::Always,
                grid_min_tile_px: 9999,
            },
            vpn: VpnSettings::default(),
        };
        s.normalize();
        assert_eq!(s.downloads.directory, None);
        assert_eq!(s.downloads.filename_template, DEFAULT_FILENAME_TEMPLATE);
        assert_eq!(s.playback.video_chunk_mb, 64);
        assert_eq!(s.appearance.grid_min_tile_px, 320);
    }

    #[test]
    fn normalize_floors_below_minimum() {
        let mut s = Settings::default();
        s.playback.video_chunk_mb = 0;
        s.appearance.grid_min_tile_px = 1;
        s.normalize();
        assert_eq!(s.playback.video_chunk_mb, 1);
        assert_eq!(s.appearance.grid_min_tile_px, 120);
    }
}
