# Creating & managing websites

## Create a new site

1. Click **+ New site**.
2. Enter a **name** (the folder/slug is derived from it).
3. Choose a **PHP version** — either the **global default** or a **per‑site** version (see [Managing PHP](php.md)).
4. Click **Create site**.

The app then:

- downloads the **latest FrontPress Studio release**,
- ensures the chosen **PHP runtime** is installed,
- and **starts the site online** automatically.

New sites are created in your [Sites location](syncing.md) (default `~/FrontPress Sites/<slug>/`).

### Default credentials

A new site uses FrontPress Studio's shipped defaults:

- **Username:** `fpsadmin`
- **Password:** `fpspass`

Change them from inside the site's admin (Settings → password). FrontPress requires the **current** password to set a new one — which is why the app uses a known default rather than a random one.

> The app doesn't generate a `config.php`; FrontPress runs on its bundled `sample.config.php` defaults and writes `config.php` itself the first time you change a credential.

## The site card

Each site in the list shows its name, **PHP** and **FP** (FrontPress) versions, and URL, plus:

| Control | What it does |
|---|---|
| **● toggle** (left) | Start/stop the site. **Green** = running (hover → red square to stop). **Grey with an ✕** = stopped (click to start). |
| **Preview** | Opens the **public site** in your browser. |
| **Login** | Opens **`/admin`**, already signed in (no password prompt — see *Auto‑login* below). |
| **⋯ menu** | Open in editor · Back up… · Restore from backup… · Reveal in Finder · Delete… |

**Stop all** (header) stops every running site at once.

### Auto‑login

The **Login** button signs you in without typing a password. The app drops a tiny one‑shot bridge file into the site and uses a single‑use token to establish an authenticated admin session on the site's own origin, then redirects to `/admin`. (It works even before `config.php` exists.)

## Duplicate a site

**⋯ → Duplicate…**, enter a name for the copy.

It makes a **full copy** of the site into a new folder with its **own port** and the default credentials, then starts it. Great for spinning up a variant or a sandbox.

## Back up a site

**⋯ → Back up…** opens a save dialog and writes a `.zip`.

A backup contains **only the site's `site/` folder** — your **content, themes, `config.json`, and uploads**. The framework (the re‑downloadable FrontPress code) is *not* included, so backups are small and portable. The login token and the regenerable `cache/` are skipped.

This is the same `site/` content you'd drop into any FrontPress install, so a backup is also a clean way to **hand a site's content to someone**.

## Restore a site

**⋯ → Restore from backup…** opens a panel where you **drop a `.zip`** (or click to browse), then confirm.

Restore is **per‑site and in‑place**: it stops the site, swaps **only its `site/` folder** for the backup contents, and restarts it — keeping the site's name, port, and framework. It's safe: the backup is validated and unpacked first, the current content is held aside, and it rolls back if anything fails.

## Open a site's folder in your editor

**⋯ → Open in editor** opens the site's **`site/`** folder (content, themes, config) in your chosen editor — not the whole framework. Set the editor first in [Settings → Editor](editor.md).

## Reveal in Finder

**⋯ → Reveal in Finder** opens the site's folder in Finder.

## Delete a site

**⋯ → Delete…** asks for confirmation, then **permanently deletes the site and all its files**. This can't be undone.

---

Next: **[Managing PHP →](php.md)**
