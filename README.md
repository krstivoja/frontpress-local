# FrontPress Local

Run [FrontPress Studio](https://github.com/krstivoja/frontpress-studio) sites on your Mac — no WordPress, no database, no Docker. A small native app (Tauri + Rust) that downloads PHP for you, serves each site on its own local port, and gives you one‑click preview and admin login.

Because FrontPress is a **flat‑file** CMS, a "site" is just a folder. That means sites are trivially **portable, syncable (Dropbox / Google Drive), importable, and shareable** — which the app leans into.

<img width="2184" height="1664" alt="Screenshot 2026-06-05 at 11 45 19" src="https://github.com/user-attachments/assets/6ece664e-0f14-4803-b151-f26f563fa245" />

### Watch video to see FrontPress Local in action

[![Watch the video](https://img.youtube.com/vi/zPbcGmOjzIA/maxresdefault.jpg)](https://youtu.be/zPbcGmOjzIA)

---

## Download

Grab the latest `.dmg` from the **[Releases page](https://github.com/krstivoja/frontpress-local/releases/latest)**, drag the app to Applications, and open it. (macOS only for now.)

The first launch needs a one‑time **"Open Anyway"** in System Settings (the app isn't notarized yet) — see [Installation](docs/installation.md).

---

## Documentation

- 📦 **[Installation](docs/installation.md)** — download, the first‑run security step, updates
- 🌐 **[Creating & managing websites](docs/websites.md)** — new site, preview, login, start/stop, **duplicate**, **back up**, **restore**, delete, open in editor
- 🐘 **[Managing PHP](docs/php.md)** — installing PHP versions, the global default, per‑site versions
- ✏️ **[Editor](docs/editor.md)** — pick a favorite editor and open a site's folder in it
- ☁️ **[Sites location & sync](docs/syncing.md)** — change where sites live, move them to Dropbox / Google Drive, **import** & **auto‑discover**, share with teammates

---

## At a glance

- **Create a site** and it's online instantly — FrontPress Studio's latest release + a PHP runtime are downloaded automatically.
- **Preview** opens the public site; **Login** drops you straight into `/admin`, already signed in.
- **Duplicate / Back up / Restore** each site from its `⋯` menu.
- **Settings** has three tabs: **PHP Manager**, **Editor**, and **Location**.
- Point **Location** at a Dropbox/Drive folder to keep your sites in sync across machines.

Default credentials for a new site are **`fpsadmin` / `fpspass`** — change them from the site's admin.
