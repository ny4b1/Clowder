use anyhow::{Context, Result, anyhow};
use keyring_core::Error as KeyringError;

use crate::e621::Credentials;

const SERVICE: &str = "com.nyabi.clowder";
const ACCOUNT: &str = "default";

fn entry() -> Result<keyring_core::Entry> {
    crate::keychain::entry(SERVICE, ACCOUNT)
}

pub async fn load() -> Result<Option<Credentials>> {
    tokio::task::spawn_blocking(load_blocking)
        .await
        .context("join credentials load task")?
}

fn load_blocking() -> Result<Option<Credentials>> {
    let entry = entry()?;
    match entry.get_password() {
        Ok(payload) => {
            let creds: Credentials =
                serde_json::from_str(&payload).context("decode stored credentials")?;
            Ok(Some(creds))
        }
        Err(KeyringError::NoEntry) => Ok(None),
        Err(err) => Err(anyhow!("read keychain entry: {err}")),
    }
}

pub async fn save(creds: &Credentials) -> Result<()> {
    let creds = creds.clone();
    tokio::task::spawn_blocking(move || save_blocking(&creds))
        .await
        .context("join credentials save task")?
}

fn save_blocking(creds: &Credentials) -> Result<()> {
    let payload = serde_json::to_string(creds).context("encode credentials")?;
    entry()?
        .set_password(&payload)
        .map_err(|err| anyhow!("write keychain entry: {err}"))
}

pub async fn clear() -> Result<()> {
    tokio::task::spawn_blocking(clear_blocking)
        .await
        .context("join credentials clear task")?
}

fn clear_blocking() -> Result<()> {
    let entry = entry()?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(KeyringError::NoEntry) => Ok(()),
        Err(err) => Err(anyhow!("delete keychain entry: {err}")),
    }
}
