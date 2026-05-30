use anyhow::{Context, Result, anyhow};
use keyring_core::Error as KeyringError;

use super::config::WgConfig;
use super::mullvad::MullvadProfile;

const SERVICE: &str = "com.nyabi.clowder";
const ACCOUNT: &str = "vpn";
const MULLVAD_ACCOUNT: &str = "vpn-mullvad";

fn entry() -> Result<keyring_core::Entry> {
    crate::keychain::entry(SERVICE, ACCOUNT).context("open VPN keychain entry")
}

fn mullvad_entry() -> Result<keyring_core::Entry> {
    crate::keychain::entry(SERVICE, MULLVAD_ACCOUNT).context("open Mullvad keychain entry")
}

pub async fn load() -> Result<Option<WgConfig>> {
    tokio::task::spawn_blocking(load_blocking)
        .await
        .context("join VPN config load task")?
}

fn load_blocking() -> Result<Option<WgConfig>> {
    let entry = entry()?;
    match entry.get_password() {
        Ok(payload) => {
            let cfg: WgConfig =
                serde_json::from_str(&payload).context("decode stored VPN config")?;
            Ok(Some(cfg))
        }
        Err(KeyringError::NoEntry) => Ok(None),
        Err(err) => Err(anyhow!("read VPN keychain entry: {err}")),
    }
}

pub async fn save(cfg: &WgConfig) -> Result<()> {
    let cfg = cfg.clone();
    tokio::task::spawn_blocking(move || save_blocking(&cfg))
        .await
        .context("join VPN config save task")?
}

fn save_blocking(cfg: &WgConfig) -> Result<()> {
    let payload = serde_json::to_string(cfg).context("encode VPN config")?;
    entry()?
        .set_password(&payload)
        .map_err(|err| anyhow!("write VPN keychain entry: {err}"))
}

pub async fn clear() -> Result<()> {
    tokio::task::spawn_blocking(clear_blocking)
        .await
        .context("join VPN config clear task")?
}

fn clear_blocking() -> Result<()> {
    let entry = entry()?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(KeyringError::NoEntry) => Ok(()),
        Err(err) => Err(anyhow!("delete VPN keychain entry: {err}")),
    }
}

pub async fn load_mullvad() -> Result<Option<MullvadProfile>> {
    tokio::task::spawn_blocking(load_mullvad_blocking)
        .await
        .context("join Mullvad profile load task")?
}

fn load_mullvad_blocking() -> Result<Option<MullvadProfile>> {
    let entry = mullvad_entry()?;
    match entry.get_password() {
        Ok(payload) => {
            let profile: MullvadProfile =
                serde_json::from_str(&payload).context("decode stored Mullvad profile")?;
            Ok(Some(profile))
        }
        Err(KeyringError::NoEntry) => Ok(None),
        Err(err) => Err(anyhow!("read Mullvad keychain entry: {err}")),
    }
}

pub async fn save_mullvad(profile: &MullvadProfile) -> Result<()> {
    let profile = profile.clone();
    tokio::task::spawn_blocking(move || save_mullvad_blocking(&profile))
        .await
        .context("join Mullvad profile save task")?
}

fn save_mullvad_blocking(profile: &MullvadProfile) -> Result<()> {
    let payload = serde_json::to_string(profile).context("encode Mullvad profile")?;
    mullvad_entry()?
        .set_password(&payload)
        .map_err(|err| anyhow!("write Mullvad keychain entry: {err}"))
}

pub async fn clear_mullvad() -> Result<()> {
    tokio::task::spawn_blocking(clear_mullvad_blocking)
        .await
        .context("join Mullvad profile clear task")?
}

fn clear_mullvad_blocking() -> Result<()> {
    let entry = mullvad_entry()?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(KeyringError::NoEntry) => Ok(()),
        Err(err) => Err(anyhow!("delete Mullvad keychain entry: {err}")),
    }
}
