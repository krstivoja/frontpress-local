//! Per-site admin password storage in the OS keychain (macOS Keychain via
//! keyring's apple-native backend). The plaintext password never touches
//! `sites.json`; we keep it here so the UI can reveal it and so the user can
//! sign in manually if they ever bypass the app.

use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE: &str = "FrontPress Local";

fn entry(site_id: &str) -> Result<Entry> {
    Entry::new(SERVICE, site_id).context("open keychain entry")
}

/// Store (or replace) the admin password for a site.
pub fn set_password(site_id: &str, password: &str) -> Result<()> {
    entry(site_id)?
        .set_password(password)
        .context("write keychain password")
}

/// Fetch the stored admin password, if any.
pub fn get_password(site_id: &str) -> Result<Option<String>> {
    match entry(site_id)?.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Remove a site's password (best-effort; missing entry is not an error).
pub fn delete_password(site_id: &str) -> Result<()> {
    match entry(site_id)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
