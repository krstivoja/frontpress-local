//! Install FrontPress Studio into a site directory from a GitHub release,
//! generate its `config.php` with app-managed admin credentials, and drop in
//! the one-shot auto-login bridge used by the desktop app.

use crate::net;
use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use std::io::Write;
use std::path::{Path, PathBuf};

const RELEASES_API: &str =
    "https://api.github.com/repos/krstivoja/frontpress-studio/releases/latest";

pub struct Release {
    pub version: String,
    pub zip_url: String,
}

/// Resolve the latest FrontPress Studio release and its `.zip` asset URL.
pub async fn latest_release() -> Result<Release> {
    let json: serde_json::Value = net::client()?
        .get(RELEASES_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let version = json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("release has no tag_name"))?
        .trim_start_matches('v')
        .to_string();

    let zip_url = json
        .get("assets")
        .and_then(|a| a.as_array())
        .and_then(|assets| {
            assets.iter().find_map(|asset| {
                let name = asset.get("name").and_then(|n| n.as_str())?;
                if name.ends_with(".zip") {
                    asset
                        .get("browser_download_url")
                        .and_then(|u| u.as_str())
                        .map(String::from)
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| anyhow!("release has no .zip asset"))?;

    Ok(Release { version, zip_url })
}

/// Download the release zip and extract it into `site_dir` (stripping the
/// single top-level `frontpress-studio-<version>/` folder). Returns nothing;
/// the caller already knows the version.
pub async fn install_into<F>(zip_url: &str, site_dir: &Path, progress: F) -> Result<()>
where
    F: Fn(u64, Option<u64>),
{
    std::fs::create_dir_all(site_dir)?;
    let tmp = site_dir.join(".frontpress-download.zip");

    let resp = net::client()?
        .get(zip_url)
        .send()
        .await?
        .error_for_status()
        .context("download FrontPress release")?;
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

    let tmp2 = tmp.clone();
    let dir2 = site_dir.to_path_buf();
    tokio::task::spawn_blocking(move || extract_zip_stripped(&tmp2, &dir2))
        .await
        .context("join unzip task")??;

    let _ = std::fs::remove_file(&tmp);
    Ok(())
}

/// Strip the leading path component ("frontpress-studio-x.y.z/") from a zip
/// entry name. Returns None for the top dir itself or unsafe (`..`) names.
fn strip_top(name: &str) -> Option<&str> {
    if name.contains("..") {
        return None;
    }
    let rest = name.split_once('/').map(|(_, r)| r)?;
    if rest.is_empty() {
        None
    } else {
        Some(rest)
    }
}

fn extract_zip_stripped(zip_path: &Path, dest: &Path) -> Result<()> {
    let f = std::fs::File::open(zip_path)?;
    let mut zip = zip::ZipArchive::new(f)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_string();
        let is_dir = name.ends_with('/');
        let rel = match strip_top(&name) {
            Some(r) => r,
            None => continue,
        };
        let out = dest.join(rel);

        if is_dir {
            std::fs::create_dir_all(&out)?;
            continue;
        }
        // Skip symlinks (e.g. the `assets` link). bootstrap.php recreates the
        // symlink on first request; an extracted text file would break that.
        if let Some(mode) = entry.unix_mode() {
            if mode & 0o170000 == 0o120000 {
                continue;
            }
        }
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut outfile = std::fs::File::create(&out)?;
        std::io::copy(&mut entry, &mut outfile)?;

        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&out, std::fs::Permissions::from_mode(mode))?;
        }
    }
    Ok(())
}

/// Build a `config.php` from the install's `sample.config.php`, substituting
/// the admin username and bcrypt password hash.
pub fn render_config_php(sample: &str, user: &str, pass_hash: &str) -> String {
    let user_line = format!("define('FPS_ADMIN_USER', '{}');", esc(user));
    let hash_line = format!("define('FPS_ADMIN_PASS_HASH', '{}');", esc(pass_hash));
    let mut out = String::with_capacity(sample.len());
    for line in sample.lines() {
        let t = line.trim_start();
        if t.starts_with("define('FPS_ADMIN_USER'") {
            out.push_str(&user_line);
        } else if t.starts_with("define('FPS_ADMIN_PASS_HASH'") {
            out.push_str(&hash_line);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

/// Escape a value for single-quoted PHP string context.
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Write `config.php` next to `sample.config.php` for the freshly installed
/// site, using the given username and bcrypt hash.
pub fn write_config(site_dir: &Path, user: &str, pass_hash: &str) -> Result<()> {
    let sample = std::fs::read_to_string(site_dir.join("sample.config.php"))
        .context("read sample.config.php")?;
    let config = render_config_php(&sample, user, pass_hash);
    std::fs::write(site_dir.join("config.php"), config)?;
    Ok(())
}

/// The one-shot auto-login bridge dropped into each site's webroot. See
/// `commands::auto_login` for how the token is issued.
pub const LOGIN_HELPER: &str = include_str!("../resources/fp-local-login.php");

/// Install the auto-login helper into the site webroot.
pub fn write_login_helper(site_dir: &Path) -> Result<()> {
    std::fs::write(site_dir.join("fp-local-login.php"), LOGIN_HELPER)?;
    Ok(())
}

/// Compute the bcrypt hash FrontPress's `password_verify()` will accept.
pub fn bcrypt_hash(password: &str) -> Result<String> {
    bcrypt::hash(password, 12).context("bcrypt hash")
}

/// Where the auto-login token file lives (consumed by the helper).
pub fn login_token_path(site_dir: &Path) -> PathBuf {
    site_dir.join("site").join(".fp-local-login")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_top_handles_paths() {
        assert_eq!(strip_top("frontpress-studio-0.4.1/router.php"), Some("router.php"));
        assert_eq!(
            strip_top("frontpress-studio-0.4.1/cms/lib/Env.php"),
            Some("cms/lib/Env.php")
        );
        assert_eq!(strip_top("frontpress-studio-0.4.1/"), None);
        assert_eq!(strip_top("frontpress-studio-0.4.1/../evil"), None);
    }

    #[test]
    fn render_config_substitutes_creds() {
        let sample = "<?php\n\
            defined('FRONTPRESS_BOOT') || exit;\n\
            define('FPS_ADMIN_USER',      getenv('FPS_ADMIN_USER')      ?: 'fpsadmin');\n\
            define('FPS_ADMIN_PASS_HASH', '$2y$12$old');\n\
            define('FPS_APP_ENV', 'dev');\n";
        let out = render_config_php(sample, "marko", "$2y$12$NEWHASH");
        assert!(out.contains("define('FPS_ADMIN_USER', 'marko');"));
        assert!(out.contains("define('FPS_ADMIN_PASS_HASH', '$2y$12$NEWHASH');"));
        assert!(!out.contains("fpsadmin"));
        assert!(!out.contains("$2y$12$old"));
        // untouched lines survive
        assert!(out.contains("define('FPS_APP_ENV', 'dev');"));
        assert!(out.contains("defined('FRONTPRESS_BOOT') || exit;"));
    }

    #[test]
    fn bcrypt_roundtrips() {
        let h = bcrypt_hash("correct horse").unwrap();
        assert!(bcrypt::verify("correct horse", &h).unwrap());
        assert!(!bcrypt::verify("wrong", &h).unwrap());
    }

    #[test]
    fn esc_quotes() {
        assert_eq!(esc("a'b"), "a\\'b");
        assert_eq!(esc("a\\b"), "a\\\\b");
    }
}
