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

pub fn load() -> Result<Option<WgConfig>> {
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

pub fn save(cfg: &WgConfig) -> Result<()> {
    let payload = serde_json::to_string(cfg).context("encode VPN config")?;
    entry()?
        .set_password(&payload)
        .map_err(|err| anyhow!("write VPN keychain entry: {err}"))
}

pub fn clear() -> Result<()> {
    let entry = entry()?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(KeyringError::NoEntry) => Ok(()),
        Err(err) => Err(anyhow!("delete VPN keychain entry: {err}")),
    }
}

pub fn load_mullvad() -> Result<Option<MullvadProfile>> {
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

pub fn save_mullvad(profile: &MullvadProfile) -> Result<()> {
    let payload = serde_json::to_string(profile).context("encode Mullvad profile")?;
    mullvad_entry()?
        .set_password(&payload)
        .map_err(|err| anyhow!("write Mullvad keychain entry: {err}"))
}

pub fn clear_mullvad() -> Result<()> {
    let entry = mullvad_entry()?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(KeyringError::NoEntry) => Ok(()),
        Err(err) => Err(anyhow!("delete Mullvad keychain entry: {err}")),
    }
}
