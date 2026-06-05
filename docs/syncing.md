# Sites location & sync

Because FrontPress is a **flat‑file** CMS, a site is just a folder on disk — no database to dump, no services to migrate. FrontPress Local turns that into a superpower: keep your sites in **Dropbox, Google Drive, iCloud, or any synced folder**, and they travel with you, sync between machines, and are easy to **import** and **share**.

## Where sites live

By default, sites are stored in:

```
~/FrontPress Sites/<slug>/
```

Each `<slug>/` folder is a complete FrontPress install: the framework **plus** its `site/` content folder.

The current location is shown in **Settings → Location**.

## Change the location (and migrate your sites)

**Settings → Location → Change…**, then pick a folder (e.g. a folder inside your Dropbox).

The app will:

1. **Stop** all running sites,
2. **copy** every site to the new folder,
3. switch the app to the new location and **remove the originals**,
4. and create new sites there from now on.

It's safe by design: everything is **copied first**, and only after every copy succeeds does it switch paths and delete the originals. If any copy fails, it **rolls back** and nothing changes.

> Tip: try it with a small test folder first to get comfortable. Your existing sites *are* moved.

## Sync across machines (Dropbox / Google Drive)

Once your Sites location is a synced folder, the **whole site folders** sync automatically. To bring those sites into FrontPress Local on a **second machine**:

1. Make sure the same Dropbox/Drive folder is synced locally on that machine.
2. Install FrontPress Local → **Settings → Location → Change…** → point it at that folder.
3. The app **discovers** every FrontPress site in the folder and lists them.

Discovery also runs **on every launch**, and you can trigger it manually with **Settings → Location → "Scan folder for sites."**

What each machine handles **locally** (and what it doesn't):

- ✅ **Files** (content, themes, config, uploads) — synced via Dropbox/Drive.
- ↪️ **Ports** — assigned per machine automatically (no conflicts).
- ↪️ **PHP runtime** — not synced (it's a binary in app data); the right version is **downloaded automatically** the first time a site starts on that machine.
- 🪪 **Identity** — each site carries a small `.frontpress-local.json` file (name, PHP/FP version) so any machine recognizes it.

## Import a single site

Use the header **Import** button to add **one** existing FrontPress site folder from anywhere on disk:

1. **Import → choose folder.**
2. If the folder is a FrontPress site (it has `router.php`), it's **added in place** (registered where it is — not copied), given a port, and started.

This is the quick way to add a site that lives outside your Sites location, or a single folder a teammate shared with you.

> **Import** = add one folder in place. **Scan folder for sites** = pull in *all* sites found in your Sites location at once.

## Share a site

Two easy options, thanks to flat files:

- **Whole site:** zip the site's `<slug>/` folder (or just share it via your synced drive) and let the other person **Import** it.
- **Just the content:** use **⋯ → Back up…** to export the site's `site/` folder, and the other person **Restore**s it into their copy. See [websites](websites.md#back-up-a-site).

## Caveats (real‑world cloud sync)

Syncing live site folders is great for moving between **your own** machines and for sharing — but keep these in mind:

- **Not live collaboration.** If two people edit the **same** site at the same time, Dropbox/Drive will create "conflicted copy" files. Coordinate who's editing.
- **Don't run the same site on two machines at once** against the live synced files.
- **`cache/` and the `assets` symlink** can cause some sync churn. FrontPress re‑creates the `assets` symlink on first request, and the cache is regenerable, so this self‑heals — but it's noise on the sync.
- Large numbers of small framework files can make initial sync slow.

---

← Back to the **[README](../README.md)**
