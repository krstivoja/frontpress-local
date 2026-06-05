# Managing PHP

FrontPress Local bundles PHP **for you** — there's no Homebrew or system PHP to set up. It downloads small, self‑contained **static PHP** builds and runs each site with PHP's built‑in dev server (`php -S`).

Open **Settings → PHP Manager** to manage runtimes.

## How it works

- On macOS, runtimes come from **[static‑php.dev](https://dl.static-php.dev/)** — a single `php` binary per version, with the extensions FrontPress needs already compiled in (mbstring, gd, curl, sqlite3, openssl, dom, fileinfo, zip, …).
- Each version is stored under `~/Library/Application Support/FrontPress Local/php/<version>/`.
- **Minimum supported:** PHP **8.1** (FrontPress Studio's requirement).

## The PHP Manager tab

You'll see a row per PHP minor (8.1, 8.2, 8.3, 8.4, …):

- **Download** — fetches the latest patch of that minor.
- **Make default** — sets it as the **global default** used for new sites. The current default is tagged.
- Installed versions are marked **installed**.

## Global default vs. per‑site

- The **global default** is what a new site uses unless you choose otherwise.
- When **creating a site**, you can pick **"Per‑site"** and choose a specific minor for just that site.
- A site remembers its exact version (e.g. `8.3.31`). If that version isn't installed on a machine yet, it's **downloaded automatically the first time the site starts** — handy when [syncing sites across machines](syncing.md).

## Architecture

The PHP Manager shows your Mac's architecture (`aarch64` for Apple Silicon, `x86_64` for Intel) and only offers builds for it.

---

Next: **[Editor →](editor.md)**
