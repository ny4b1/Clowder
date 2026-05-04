use std::fs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tauri::Manager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DohProvider {
    #[default]
    Cloudflare,
    Google,
    Quad9,
    AdGuard,
}

impl DohProvider {
    pub fn url(&self) -> &'static str {
        match self {
            Self::Cloudflare => "https://cloudflare-dns.com/dns-query",
            Self::Google => "https://dns.google/dns-query",
            Self::Quad9 => "https://dns.quad9.net/dns-query",
            Self::AdGuard => "https://dns.adguard.com/dns-query",
        }
    }

    pub fn host(&self) -> &'static str {
        match self {
            Self::Cloudflare => "cloudflare-dns.com",
            Self::Google => "dns.google",
            Self::Quad9 => "dns.quad9.net",
            Self::AdGuard => "dns.adguard.com",
        }
    }

    pub fn bootstrap_addrs(&self) -> Vec<SocketAddr> {
        match self {
            Self::Cloudflare => vec![
                v4(1, 1, 1, 1),
                v4(1, 0, 0, 1),
                v6(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111),
                v6(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1001),
            ],
            Self::Google => vec![
                v4(8, 8, 8, 8),
                v4(8, 8, 4, 4),
                v6(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888),
                v6(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8844),
            ],
            Self::Quad9 => vec![
                v4(9, 9, 9, 9),
                v4(149, 112, 112, 112),
                v6(0x2620, 0x00fe, 0, 0, 0, 0, 0, 0x00fe),
                v6(0x2620, 0x00fe, 0, 0, 0, 0, 0, 0x0009),
            ],
            Self::AdGuard => vec![
                v4(94, 140, 14, 14),
                v4(94, 140, 15, 15),
                v6(0x2a10, 0x50c0, 0, 0, 0, 0, 0x0ad1, 0x00ff),
                v6(0x2a10, 0x50c0, 0, 0, 0, 0, 0x0ad2, 0x00ff),
            ],
        }
    }
}

const fn v4(a: u8, b: u8, c: u8, d: u8) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(a, b, c, d)), 443)
}

#[allow(clippy::too_many_arguments)]
const fn v6(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V6(Ipv6Addr::new(a, b, c, d, e, f, g, h)), 443)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub doh_provider: DohProvider,
    pub fail_closed_ech: bool,
    pub downloads: DownloadSettings,
    pub playback: PlaybackSettings,
    pub appearance: AppearanceSettings,
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
        Ok(s) => s,
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
            doh_provider: DohProvider::Quad9,
            fail_closed_ech: true,
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
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn settings_uses_defaults_on_missing_fields() {
        let s: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(s.doh_provider, DohProvider::Cloudflare);
        assert!(!s.fail_closed_ech);
        assert_eq!(s.downloads, DownloadSettings::default());
        assert_eq!(s.playback, PlaybackSettings::default());
        assert_eq!(s.appearance, AppearanceSettings::default());
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
    fn settings_uses_defaults_on_partial_fields() {
        let s: Settings = serde_json::from_str(r#"{"fail_closed_ech": true}"#).unwrap();
        assert_eq!(s.doh_provider, DohProvider::Cloudflare);
        assert!(s.fail_closed_ech);
    }

    #[test]
    fn doh_providers_have_https_urls_and_bootstrap_addrs() {
        for provider in [
            DohProvider::Cloudflare,
            DohProvider::Google,
            DohProvider::Quad9,
            DohProvider::AdGuard,
        ] {
            assert!(provider.url().starts_with("https://"));
            assert!(provider.host().contains('.'));
            let addrs = provider.bootstrap_addrs();
            assert!(!addrs.is_empty());
            assert!(addrs.iter().all(|a| a.port() == 443));
        }
    }

    #[test]
    fn doh_provider_serializes_as_snake_case() {
        let json = serde_json::to_string(&DohProvider::AdGuard).unwrap();
        assert_eq!(json, "\"ad_guard\"");
        let back: DohProvider = serde_json::from_str(&json).unwrap();
        assert_eq!(back, DohProvider::AdGuard);
    }
}
