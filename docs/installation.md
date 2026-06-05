# Installation

FrontPress Local is a native macOS app. (Windows support is wired up but not shipped yet.)

## 1. Download

Open the **[Releases page](https://github.com/krstivoja/frontpress-local/releases/latest)** and download the `.dmg`:

- `FrontPress.Local_<version>_universal.dmg` — a universal build that runs on both **Apple Silicon** and **Intel** Macs.

Open the DMG and drag **FrontPress Local** into your **Applications** folder.

## 2. First launch — the "Open Anyway" step

The first time you open the app, macOS will say it *"can't be opened because Apple cannot check it for malware,"* or that it *"was blocked to protect your Mac."*

This is expected. Apple requires a **paid annual Developer certificate** to sign and notarize apps, and this project isn't signed yet. The app is safe; macOS just can't verify the developer.

To open it:

1. Try to open the app once (it gets blocked).
2. Go to **System Settings → Privacy & Security**.
3. Scroll to the **Security** section — you'll see *"FrontPress Local was blocked…"* with an **Open Anyway** button. Click it.
4. Confirm with **Open**.

You only need to do this **once per version**.

<img width="1312" height="1486" alt="Security" src="https://github.com/user-attachments/assets/ede64b57-7f43-4218-907f-131f5e6da974" />

> Prefer the terminal? `xattr -dr com.apple.quarantine "/Applications/FrontPress Local.app"` also clears the block.

Once the project has enough funding/community support to cover an Apple Developer membership, the app will be **notarized** and this step will disappear.

## 3. Updates

The app **checks for updates on launch** and via the menu bar (**FrontPress Local → Check for Updates…**). When a new version is available you'll see an amber **"Update available"** bar at the top of the window — click **Install & restart**.

> Note: because the app isn't notarized yet, an auto‑downloaded update may also need the one‑time **Open Anyway** until signing is in place.

## Where things are stored

- **Your sites:** `~/FrontPress Sites/` by default (configurable — see [Sites location & sync](syncing.md)).
- **App data:** `~/Library/Application Support/FrontPress Local/` — the downloaded PHP runtimes, the site list (`sites.json`), and per‑site server logs.

---

Next: **[Creating & managing websites →](websites.md)**
