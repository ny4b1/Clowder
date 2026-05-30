use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow};
use keyring_core::Entry;

static KEYRING_INIT: OnceLock<std::result::Result<(), String>> = OnceLock::new();

pub fn entry(service: &str, account: &str) -> Result<Entry> {
    ensure_native_store()?;
    Entry::new(service, account).context("open keychain entry")
}

fn ensure_native_store() -> Result<()> {
    match KEYRING_INIT
        .get_or_init(|| keyring::use_native_store(false).map_err(|err| err.to_string()))
    {
        Ok(()) => Ok(()),
        Err(err) => Err(anyhow!("initialize native keychain store: {err}")),
    }
}
