use anyhow::{Context, Result, anyhow};
use keyring_core::Error as KeyringError;

use crate::e621::Credentials;

const SERVICE: &str = "com.nyabi.clowder";
const ACCOUNT: &str = "default";

fn entry() -> Result<keyring_core::Entry> {
    crate::keychain::entry(SERVICE, ACCOUNT)
}

pub fn load() -> Result<Option<Credentials>> {
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

pub fn save(creds: &Credentials) -> Result<()> {
    let payload = serde_json::to_string(creds).context("encode credentials")?;
    entry()?
        .set_password(&payload)
        .map_err(|err| anyhow!("write keychain entry: {err}"))
}

pub fn clear() -> Result<()> {
    let entry = entry()?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(KeyringError::NoEntry) => Ok(()),
        Err(err) => Err(anyhow!("delete keychain entry: {err}")),
    }
}
