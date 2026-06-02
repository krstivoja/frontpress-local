//! PHP runtime management. Downloads prebuilt static PHP CLI binaries from
//! static-php.dev (no Homebrew dependency), one directory per fully-qualified
//! version, and resolves the latest patch for a given minor.

use crate::{net, paths, util};
use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use std::io::Write;
use std::path::PathBuf;

const BASE: &str = "https://dl.static-php.dev/static-php-cli/common";

/// Build-target architecture token used in static-php filenames.
#[cfg(target_arch = "aarch64")]
pub const ARCH: &str = "aarch64";
#[cfg(target_arch = "x86_64")]
pub const ARCH: &str = "x86_64";
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
pub const ARCH: &str = "aarch64";

fn download_url(version: &str) -> String {
    format!("{BASE}/php-{version}-cli-macos-{ARCH}.tar.gz")
}

/// Parse a static-php directory listing into the list of full versions
/// available for this architecture (e.g. "8.3.9"). Order is not guaranteed.
pub fn parse_listing(body: &str, arch: &str) -> Vec<String> {
    let suffix = format!("-cli-macos-{arch}.tar.gz");
    let mut out = Vec::new();
    for chunk in body.split("php-").skip(1) {
        if let Some(idx) = chunk.find(&suffix) {
            let ver = &chunk[..idx];
            if !ver.is_empty()
                && ver.contains('.')
                && ver.chars().all(|c| c.is_ascii_digit() || c == '.')
            {
                out.push(ver.to_string());
            }
        }
    }
    out.sort_by(|a, b| util::parse_version(b).cmp(&util::parse_version(a)));
    out.dedup();
    out
}

/// Fetch the remote catalogue of installable PHP versions, newest first.
pub async fn remote_versions() -> Result<Vec<String>> {
    let body = net::client()?
        .get(format!("{BASE}/"))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let versions = parse_listing(&body, ARCH);
    if versions.is_empty() {
        return Err(anyhow!("no PHP builds found in static-php listing"));
    }
    Ok(versions)
}

/// Highest patch release for a given "major.minor".
pub fn latest_patch(versions: &[String], minor: &str) -> Option<String> {
    versions
        .iter()
        .filter(|v| util::minor_of(v) == minor)
        .max_by(|a, b| util::parse_version(a).cmp(&util::parse_version(b)))
        .cloned()
}

/// Versions already downloaded (a `php` binary exists under php/<version>/).
pub fn installed_versions() -> Result<Vec<String>> {
    let root = paths::php_root()?;
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&root) {
        for e in entries.flatten() {
            if e.path().join("php").is_file() {
                if let Some(name) = e.file_name().to_str() {
                    out.push(name.to_string());
                }
            }
        }
    }
    out.sort_by(|a, b| util::parse_version(b).cmp(&util::parse_version(a)));
    Ok(out)
}

/// Ensure the given fully-qualified PHP version is installed locally,
/// downloading + extracting it if needed. `progress(done, total)` is called
/// during the download. Returns the path to the `php` binary.
pub async fn ensure_installed<F>(version: &str, progress: F) -> Result<PathBuf>
where
    F: Fn(u64, Option<u64>),
{
    let bin = paths::php_binary(version)?;
    if bin.is_file() {
        return Ok(bin);
    }

    // Download the tarball to a temp file alongside the target dir.
    let dir = paths::php_root()?.join(version);
    std::fs::create_dir_all(&dir)?;
    let tmp = dir.join("download.tar.gz");

    let resp = net::client()?
        .get(download_url(version))
        .send()
        .await?
        .error_for_status()
        .with_context(|| format!("PHP {version} not available for {ARCH}"))?;
    let total = resp.content_length();
    let mut stream = resp.bytes_stream();
    let mut file = std::fs::File::create(&tmp)?;
    let mut done = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        done += chunk.len() as u64;
        progress(done, total);
    }
    file.flush()?;
    drop(file);

    // Extract the single `php` entry on a blocking thread.
    let tmp2 = tmp.clone();
    let bin2 = bin.clone();
    tokio::task::spawn_blocking(move || extract_php(&tmp2, &bin2))
        .await
        .context("join extract task")??;

    let _ = std::fs::remove_file(&tmp);
    dequarantine(&bin);
    Ok(bin)
}

/// Pull the `php` binary out of a static-php tar.gz into `dest`, chmod +x.
fn extract_php(tarball: &PathBuf, dest: &PathBuf) -> Result<()> {
    use flate2::read::GzDecoder;
    let f = std::fs::File::open(tarball)?;
    let mut archive = tar::Archive::new(GzDecoder::new(f));
    for entry in archive.entries()? {
        let mut entry = entry?;
        let is_php = entry
            .path()?
            .file_name()
            .map(|n| n == "php")
            .unwrap_or(false);
        if is_php {
            entry.unpack(dest)?;
            set_executable(dest)?;
            return Ok(());
        }
    }
    Err(anyhow!("no `php` binary inside tarball"))
}

fn set_executable(path: &PathBuf) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

/// Strip the macOS quarantine xattr so Gatekeeper doesn't block the binary.
/// Best-effort: missing xattr / non-macOS is a no-op.
fn dequarantine(path: &PathBuf) {
    let _ = std::process::Command::new("xattr")
        .args(["-d", "com.apple.quarantine"])
        .arg(path)
        .output();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_listing_extracts_versions() {
        let html = r#"
            <a href="/static-php-cli/common/php-8.1.34-cli-macos-aarch64.tar.gz">x</a>
            <a href="/static-php-cli/common/php-8.3.9-cli-macos-aarch64.tar.gz">x</a>
            <a href="/static-php-cli/common/php-8.3.7-cli-macos-aarch64.tar.gz">x</a>
            <a href="/static-php-cli/common/php-8.2.21-cli-macos-x86_64.tar.gz">x</a>
            <a href="/static-php-cli/common/php-8.4.1-cli-macos-aarch64.tar.gz">x</a>
        "#;
        let v = parse_listing(html, "aarch64");
        assert_eq!(v.first().map(String::as_str), Some("8.4.1")); // newest first
        assert!(v.contains(&"8.1.34".to_string()));
        assert!(v.contains(&"8.3.9".to_string()));
        // x86_64-only entries excluded for aarch64
        assert!(!v.contains(&"8.2.21".to_string()));
    }

    #[test]
    fn latest_patch_picks_highest() {
        let v = vec![
            "8.3.9".to_string(),
            "8.3.7".to_string(),
            "8.3.10".to_string(),
            "8.1.34".to_string(),
        ];
        assert_eq!(latest_patch(&v, "8.3"), Some("8.3.10".to_string()));
        assert_eq!(latest_patch(&v, "8.1"), Some("8.1.34".to_string()));
        assert_eq!(latest_patch(&v, "8.9"), None);
    }
}
