use anyhow::{Context, Result, anyhow};
use keyring_core::Error as KeyringError;

use crate::e621::Credentials;
use crate::site::Site;

const SERVICE: &str = "com.nyabi.clowder";
const LEGACY_ACCOUNT: &str = "default";

fn entry(site: Site) -> Result<keyring_core::Entry> {
    crate::keychain::entry(SERVICE, site.keychain_account())
}

pub async fn load(site: Site) -> Result<Option<Credentials>> {
    tokio::task::spawn_blocking(move || load_blocking(site))
        .await
        .context("join credentials load task")?
}

fn load_blocking(site: Site) -> Result<Option<Credentials>> {
    let entry = entry(site)?;
    match entry.get_password() {
        Ok(payload) => {
            let creds: Credentials =
                serde_json::from_str(&payload).context("decode stored credentials")?;
            Ok(Some(creds))
        }
        Err(KeyringError::NoEntry) if site == Site::E621 => migrate_legacy_blocking(),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(err) => Err(anyhow!("read keychain entry: {err}")),
    }
}

fn migrate_legacy_blocking() -> Result<Option<Credentials>> {
    let legacy = crate::keychain::entry(SERVICE, LEGACY_ACCOUNT)?;
    match legacy.get_password() {
        Ok(payload) => {
            let creds: Credentials =
                serde_json::from_str(&payload).context("decode legacy credentials")?;
            save_blocking(Site::E621, &creds)?;
            let _ = legacy.delete_credential();
            Ok(Some(creds))
        }
        Err(KeyringError::NoEntry) => Ok(None),
        Err(err) => Err(anyhow!("read legacy keychain entry: {err}")),
    }
}

pub async fn save(site: Site, creds: &Credentials) -> Result<()> {
    let creds = creds.clone();
    tokio::task::spawn_blocking(move || save_blocking(site, &creds))
        .await
        .context("join credentials save task")?
}

fn save_blocking(site: Site, creds: &Credentials) -> Result<()> {
    let payload = serde_json::to_string(creds).context("encode credentials")?;
    entry(site)?
        .set_password(&payload)
        .map_err(|err| anyhow!("write keychain entry: {err}"))
}

pub async fn clear(site: Site) -> Result<()> {
    tokio::task::spawn_blocking(move || clear_blocking(site))
        .await
        .context("join credentials clear task")?
}

fn clear_blocking(site: Site) -> Result<()> {
    let entry = entry(site)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(KeyringError::NoEntry) => Ok(()),
        Err(err) => Err(anyhow!("delete keychain entry: {err}")),
    }
}
