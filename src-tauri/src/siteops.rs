//! Filesystem operations for duplicate / backup / restore: recursive copy,
//! zip a directory, unzip an archive, and a small backup-metadata sidecar so
//! a restored site keeps its name, PHP version, etc.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;

/// Sidecar written into a backup zip (as `.fp-local.json`) so restore can
/// recover the site's metadata. Removed from disk right after zipping.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackupMeta {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub php_version: String,
    #[serde(default)]
    pub frontpress_version: String,
    #[serde(default)]
    pub admin_user: String,
}

pub const META_FILE: &str = ".fp-local.json";

/// Recursively copy `src` into `dst`, skipping symlinks (FrontPress recreates
/// the `assets` symlink itself on first request).
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            continue;
        }
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ft.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Zip the *contents* of `src_dir` (entries relative to it, no wrapper folder)
/// into `zip_path`. Symlinks and any entry whose file name is in `exclude`
/// are skipped.
pub fn zip_dir(src_dir: &Path, zip_path: &Path, exclude: &[&str]) -> Result<()> {
    let file = std::fs::File::create(zip_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts: zip::write::SimpleFileOptions =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut buf = Vec::new();
    add_to_zip(&mut zip, &opts, src_dir, src_dir, exclude, &mut buf)?;
    zip.finish()?;
    Ok(())
}

fn add_to_zip<W: Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    opts: &zip::write::SimpleFileOptions,
    base: &Path,
    dir: &Path,
    exclude: &[&str],
    buf: &mut Vec<u8>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            if exclude.contains(&name) {
                continue;
            }
        }
        let path = entry.path();
        let rel = path.strip_prefix(base)?;
        let name = rel.to_string_lossy().replace('\\', "/");
        if ft.is_dir() {
            zip.add_directory(format!("{name}/"), *opts)?;
            add_to_zip(zip, opts, base, &path, exclude, buf)?;
        } else {
            zip.start_file(name, *opts)?;
            buf.clear();
            std::fs::File::open(&path)?.read_to_end(buf)?;
            zip.write_all(buf)?;
        }
    }
    Ok(())
}

/// Extract every entry of `zip_path` into `dest` (no leading-folder stripping).
pub fn unzip(zip_path: &Path, dest: &Path) -> Result<()> {
    let f = std::fs::File::open(zip_path)?;
    let mut zip = zip::ZipArchive::new(f)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_string();
        if name.contains("..") {
            continue;
        }
        let out = dest.join(&name);
        if name.ends_with('/') {
            std::fs::create_dir_all(&out)?;
            continue;
        }
        if let Some(mode) = entry.unix_mode() {
            if mode & 0o170000 == 0o120000 {
                continue; // skip symlinks
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

/// Write the backup metadata sidecar into a site dir (called just before zip).
pub fn write_meta(site_dir: &Path, meta: &BackupMeta) -> Result<()> {
    let json = serde_json::to_string_pretty(meta)?;
    std::fs::write(site_dir.join(META_FILE), json)?;
    Ok(())
}

/// Read + remove the backup metadata sidecar from a restored site dir.
pub fn take_meta(site_dir: &Path) -> Option<BackupMeta> {
    let path = site_dir.join(META_FILE);
    let meta = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<BackupMeta>(&s).ok());
    let _ = std::fs::remove_file(&path);
    meta
}

/// Validate that an extracted directory looks like a FrontPress `site/` folder
/// (content / themes / config.json) — what our backups contain.
pub fn looks_like_site_folder(dir: &Path) -> Result<()> {
    if dir.join("config.json").is_file()
        || dir.join("content").is_dir()
        || dir.join("themes").is_dir()
    {
        Ok(())
    } else {
        Err(anyhow!("That zip doesn't look like a FrontPress site backup"))
    }
}
