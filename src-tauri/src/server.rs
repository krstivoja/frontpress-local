//! Per-site PHP dev servers. Each running site is a `php -S 127.0.0.1:<port>
//! router.php` child process tracked by site id, so we can stop it on demand
//! and on app exit.

use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct ServerManager {
    running: Mutex<HashMap<String, Child>>,
}

impl ServerManager {
    pub fn new() -> Self {
        ServerManager::default()
    }

    /// Spawn `php -S 127.0.0.1:<port> router.php` rooted at `webroot`.
    /// No-op (Ok) if the site is already running.
    pub fn start(
        &self,
        site_id: &str,
        php_bin: &Path,
        port: u16,
        webroot: &Path,
        log_file: &Path,
    ) -> Result<()> {
        if self.is_running(site_id) {
            return Ok(());
        }
        if !php_bin.is_file() {
            return Err(anyhow!("PHP binary missing: {}", php_bin.display()));
        }
        if !webroot.join("router.php").is_file() {
            return Err(anyhow!("router.php not found in {}", webroot.display()));
        }

        if let Some(parent) = log_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let log = std::fs::File::create(log_file).context("create server log")?;
        let log_err = log.try_clone().context("clone log handle")?;

        let addr = format!("127.0.0.1:{port}");
        let child = Command::new(php_bin)
            .current_dir(webroot)
            .args(["-S", &addr, "-t", &webroot.to_string_lossy(), "router.php"])
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(log_err))
            .spawn()
            .context("spawn php -S")?;

        self.running
            .lock()
            .map_err(|_| anyhow!("server lock poisoned"))?
            .insert(site_id.to_string(), child);

        wait_until_ready(port, Duration::from_secs(8))?;
        Ok(())
    }

    /// Stop a running site. Ok if it wasn't running.
    pub fn stop(&self, site_id: &str) -> Result<()> {
        let mut guard = self
            .running
            .lock()
            .map_err(|_| anyhow!("server lock poisoned"))?;
        if let Some(mut child) = guard.remove(site_id) {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    /// True if the site's child process is still alive. Reaps it if it exited.
    pub fn is_running(&self, site_id: &str) -> bool {
        let mut guard = match self.running.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        if let Some(child) = guard.get_mut(site_id) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    guard.remove(site_id);
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Kill every running server (called on app exit).
    pub fn stop_all(&self) {
        if let Ok(mut guard) = self.running.lock() {
            for (_, mut child) in guard.drain() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}

/// Block until the dev server accepts a TCP connection, or time out.
fn wait_until_ready(port: u16, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    let addr = format!("127.0.0.1:{port}");
    loop {
        if TcpStream::connect_timeout(
            &addr.parse().context("parse loopback addr")?,
            Duration::from_millis(300),
        )
        .is_ok()
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(anyhow!("PHP server did not become ready on port {port}"));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
