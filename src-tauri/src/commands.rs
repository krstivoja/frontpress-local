//! Tauri command surface: everything the React UI calls. Each command returns
//! `Result<_, String>` so errors surface as readable strings in the frontend.

use crate::server::ServerManager;
use crate::store::{now_secs, Settings, Site, Store};
use crate::{frontpress, keychain, paths, php, util};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

pub struct AppState {
    pub store: Mutex<Store>,
    pub servers: ServerManager,
}

impl AppState {
    pub fn load() -> Self {
        AppState {
            store: Mutex::new(Store::load().unwrap_or_default()),
            servers: ServerManager::new(),
        }
    }
}

type CmdResult<T> = Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// ── DTOs ────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteView {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub path: String,
    pub port: u16,
    pub php_version: String,
    pub frontpress_version: String,
    pub admin_user: String,
    pub running: bool,
    pub url: String,
}

fn view_of(site: &Site, servers: &ServerManager) -> SiteView {
    SiteView {
        id: site.id.clone(),
        name: site.name.clone(),
        slug: site.slug.clone(),
        path: site.path.clone(),
        port: site.port,
        php_version: site.php_version.clone(),
        frontpress_version: site.frontpress_version.clone(),
        admin_user: site.admin_user.clone(),
        running: servers.is_running(&site.id),
        url: format!("http://localhost:{}/", site.port),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub sites: Vec<SiteView>,
    pub global_php_version: String,
    pub min_php: String,
    pub arch: String,
    pub installed_php: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpOption {
    pub minor: String,
    pub latest: String,
    pub installed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpCatalog {
    pub arch: String,
    pub min_php: String,
    pub installed: Vec<String>,
    pub options: Vec<PhpOption>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSiteArgs {
    pub name: String,
    /// "global" | "custom"
    pub php_mode: String,
    /// e.g. "8.3" — required when php_mode == "custom"
    pub php_minor: Option<String>,
}

#[derive(Clone, Serialize)]
struct Progress {
    phase: String,
    message: String,
    pct: Option<f64>,
}

fn emit_progress(app: &AppHandle, phase: &str, message: &str, done: u64, total: Option<u64>) {
    let pct = total.and_then(|t| {
        if t > 0 {
            Some((done as f64 / t as f64) * 100.0)
        } else {
            None
        }
    });
    let _ = app.emit(
        "setup-progress",
        Progress {
            phase: phase.to_string(),
            message: message.to_string(),
            pct,
        },
    );
}

// ── Status / catalogue ──────────────────────────────────────────────────────

#[tauri::command]
pub fn app_status(state: State<'_, AppState>) -> CmdResult<AppStatus> {
    let store = state.store.lock().map_err(err)?;
    let sites = store
        .sites
        .iter()
        .map(|s| view_of(s, &state.servers))
        .collect();
    Ok(AppStatus {
        sites,
        global_php_version: store.settings.global_php_version.clone(),
        min_php: store.settings.min_php.clone(),
        arch: php::ARCH.to_string(),
        installed_php: php::installed_versions().unwrap_or_default(),
    })
}

#[tauri::command]
pub async fn available_php(state: State<'_, AppState>) -> CmdResult<PhpCatalog> {
    let min_php = state
        .store
        .lock()
        .map_err(err)?
        .settings
        .min_php
        .clone();

    let remote = php::remote_versions().await.map_err(err)?;
    let installed = php::installed_versions().unwrap_or_default();

    // One option per supported minor (>= min_php), newest patch.
    let mut seen = Vec::<String>::new();
    let mut options = Vec::new();
    for v in &remote {
        let minor = util::minor_of(v);
        if seen.contains(&minor) || !util::version_at_least(&minor, &min_php) {
            continue;
        }
        if let Some(latest) = php::latest_patch(&remote, &minor) {
            options.push(PhpOption {
                installed: installed.contains(&latest),
                minor: minor.clone(),
                latest,
            });
            seen.push(minor);
        }
    }
    Ok(PhpCatalog {
        arch: php::ARCH.to_string(),
        min_php,
        installed,
        options,
    })
}

/// Download a PHP runtime for a given minor (resolving the latest patch).
/// Returns the resolved full version.
#[tauri::command]
pub async fn install_php(app: AppHandle, minor: String) -> CmdResult<String> {
    let remote = php::remote_versions().await.map_err(err)?;
    let version = php::latest_patch(&remote, &minor)
        .ok_or_else(|| format!("No PHP {minor} build available"))?;
    let app2 = app.clone();
    let v2 = version.clone();
    php::ensure_installed(&version, move |done, total| {
        emit_progress(&app2, "php", &format!("Downloading PHP {v2}"), done, total);
    })
    .await
    .map_err(err)?;
    Ok(version)
}

/// Set the default PHP for new sites (resolves + installs the latest patch).
#[tauri::command]
pub async fn set_global_php(
    app: AppHandle,
    state: State<'_, AppState>,
    minor: String,
) -> CmdResult<String> {
    let version = install_php(app, minor).await?;
    let mut store = state.store.lock().map_err(err)?;
    store.settings.global_php_version = version.clone();
    store.save().map_err(err)?;
    Ok(version)
}

// ── Site lifecycle ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn create_site(
    app: AppHandle,
    state: State<'_, AppState>,
    args: CreateSiteArgs,
) -> CmdResult<SiteView> {
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err("Please enter a site name.".into());
    }
    let slug = util::slugify(&name);

    // Snapshot what we need without holding the lock across awaits.
    let (port, min_php, global_php) = {
        let store = state.store.lock().map_err(err)?;
        if !store.slug_available(&slug) {
            return Err(format!("A site named “{slug}” already exists."));
        }
        (
            store.next_free_port(),
            store.settings.min_php.clone(),
            store.settings.global_php_version.clone(),
        )
    };

    // Resolve the PHP version to run.
    let php_version = match args.php_mode.as_str() {
        "custom" => {
            let minor = args
                .php_minor
                .clone()
                .ok_or("Please choose a PHP version.")?;
            if !util::version_at_least(&minor, &min_php) {
                return Err(format!(
                    "PHP {minor} is below FrontPress's minimum ({min_php})."
                ));
            }
            let remote = php::remote_versions().await.map_err(err)?;
            php::latest_patch(&remote, &minor)
                .ok_or_else(|| format!("No PHP {minor} build available"))?
        }
        _ => {
            if !global_php.is_empty() {
                global_php
            } else {
                let remote = php::remote_versions().await.map_err(err)?;
                php::latest_patch(&remote, crate::store::PREFERRED_PHP_MINOR)
                    .ok_or("No default PHP build available")?
            }
        }
    };
    if !util::version_at_least(&php_version, &min_php) {
        return Err(format!(
            "Selected PHP {php_version} is below the minimum {min_php}."
        ));
    }

    // 1. PHP runtime.
    let app_php = app.clone();
    let vlabel = php_version.clone();
    php::ensure_installed(&php_version, move |done, total| {
        emit_progress(&app_php, "php", &format!("Preparing PHP {vlabel}"), done, total);
    })
    .await
    .map_err(err)?;

    // 2. Site directory.
    let site_dir = paths::default_sites_parent().map_err(err)?.join(&slug);
    if site_dir.exists() {
        return Err(format!("Directory already exists: {}", site_dir.display()));
    }

    // 3. FrontPress release.
    emit_progress(&app, "frontpress", "Finding latest FrontPress Studio…", 0, None);
    let release = frontpress::latest_release().await.map_err(err)?;
    let fp_version = release.version.clone();
    let app_fp = app.clone();
    let label_version = fp_version.clone();
    frontpress::install_into(&release.zip_url, &site_dir, move |done, total| {
        emit_progress(
            &app_fp,
            "frontpress",
            &format!("Downloading FrontPress Studio {label_version}"),
            done,
            total,
        );
    })
    .await
    .map_err(|e| {
        let _ = std::fs::remove_dir_all(&site_dir);
        err(e)
    })?;

    // 4. Credentials + config.php + login bridge.
    emit_progress(&app, "config", "Configuring site…", 0, None);
    let admin_user = "admin".to_string();
    let password = util::random_password();
    let hash = frontpress::bcrypt_hash(&password).map_err(err)?;
    frontpress::write_config(&site_dir, &admin_user, &hash).map_err(err)?;
    frontpress::write_login_helper(&site_dir).map_err(err)?;

    let id = util::random_id();
    keychain::set_password(&id, &password).map_err(err)?;

    // 5. Persist.
    let site = Site {
        id: id.clone(),
        name,
        slug,
        path: site_dir.to_string_lossy().to_string(),
        port,
        php_version,
        frontpress_version: fp_version,
        admin_user,
        created_at: now_secs(),
    };
    let view = {
        let mut store = state.store.lock().map_err(err)?;
        // If global PHP was unset, adopt what we just resolved.
        if store.settings.global_php_version.is_empty() {
            store.settings.global_php_version = site.php_version.clone();
        }
        store.sites.push(site.clone());
        store.save().map_err(err)?;
        view_of(&site, &state.servers)
    };
    emit_progress(&app, "done", "Site ready", 100, Some(100));
    Ok(view)
}

#[tauri::command]
pub fn start_site(state: State<'_, AppState>, id: String) -> CmdResult<SiteView> {
    let site = {
        let store = state.store.lock().map_err(err)?;
        store.site(&id).cloned().ok_or("Unknown site")?
    };
    start_internal(&state, &site)?;
    Ok(view_of(&site, &state.servers))
}

#[tauri::command]
pub fn stop_site(state: State<'_, AppState>, id: String) -> CmdResult<SiteView> {
    state.servers.stop(&id).map_err(err)?;
    let store = state.store.lock().map_err(err)?;
    let site = store.site(&id).cloned().ok_or("Unknown site")?;
    Ok(view_of(&site, &state.servers))
}

fn start_internal(state: &State<'_, AppState>, site: &Site) -> CmdResult<()> {
    let php_bin = paths::php_binary(&site.php_version).map_err(err)?;
    if !php_bin.is_file() {
        return Err(format!(
            "PHP {} isn't installed. Re-install it from PHP settings.",
            site.php_version
        ));
    }
    let webroot = PathBuf::from(&site.path);
    let log = paths::app_data_dir()
        .map_err(err)?
        .join("logs")
        .join(format!("{}.log", site.id));
    state
        .servers
        .start(&site.id, &php_bin, site.port, &webroot, &log)
        .map_err(err)
}

#[tauri::command]
pub fn delete_site(
    state: State<'_, AppState>,
    id: String,
    delete_files: bool,
) -> CmdResult<()> {
    state.servers.stop(&id).map_err(err)?;
    let removed = {
        let mut store = state.store.lock().map_err(err)?;
        let removed = store.remove_site(&id);
        store.save().map_err(err)?;
        removed
    };
    let _ = keychain::delete_password(&id);
    if let Some(site) = removed {
        if delete_files {
            let _ = std::fs::remove_dir_all(&site.path);
        }
    }
    Ok(())
}

// ── Browser actions ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn open_preview(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let site = {
        let store = state.store.lock().map_err(err)?;
        store.site(&id).cloned().ok_or("Unknown site")?
    };
    start_internal(&state, &site)?;
    open_url(&format!("http://localhost:{}/", site.port))
}

#[tauri::command]
pub fn auto_login(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let site = {
        let store = state.store.lock().map_err(err)?;
        store.site(&id).cloned().ok_or("Unknown site")?
    };
    start_internal(&state, &site)?;

    // Issue a single-use token the bridge will consume.
    let token = util::random_token();
    let token_path = frontpress::login_token_path(&PathBuf::from(&site.path));
    if let Some(parent) = token_path.parent() {
        std::fs::create_dir_all(parent).map_err(err)?;
    }
    std::fs::write(&token_path, &token).map_err(err)?;

    open_url(&format!(
        "http://localhost:{}/fp-local-login.php?token={}",
        site.port, token
    ))
}

#[tauri::command]
pub fn reveal_in_finder(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    let store = state.store.lock().map_err(err)?;
    let site = store.site(&id).cloned().ok_or("Unknown site")?;
    std::process::Command::new("open")
        .args(["-R", &site.path])
        .spawn()
        .map_err(err)?;
    Ok(())
}

fn open_url(url: &str) -> CmdResult<()> {
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .map_err(err)?;
    Ok(())
}

/// Re-read the on-disk settings into a struct the UI can show.
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> CmdResult<Settings> {
    Ok(state.store.lock().map_err(err)?.settings.clone())
}

/// Test-only: true when launched with FP_SELFTEST_UPDATE set, so the update
/// banner can auto-apply during an automated end-to-end updater test.
#[tauri::command]
pub fn selftest_update() -> bool {
    std::env::var("FP_SELFTEST_UPDATE").is_ok()
}
