import { useState } from "react";
import { save, message } from "@tauri-apps/plugin-dialog";
import { api, SiteView } from "../api";
import { DuplicateModal } from "./DuplicateModal";

export function SiteCard({
  site,
  busy,
  onPreview,
  onLogin,
  onStart,
  onStop,
  onReveal,
  onDelete,
  onChanged,
}: {
  site: SiteView;
  busy: boolean;
  onPreview: () => void;
  onLogin: () => void;
  onStart: () => void;
  onStop: () => void;
  onReveal: () => void;
  onDelete: (deleteFiles: boolean) => void;
  onChanged: () => void;
}) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [duplicating, setDuplicating] = useState(false);

  const backup = async () => {
    setMenuOpen(false);
    try {
      const dest = await save({
        defaultPath: `${site.slug}-backup.zip`,
        filters: [{ name: "Backup zip", extensions: ["zip"] }],
      });
      if (!dest) return;
      await api.backupSite(site.id, dest);
      await message("Backup saved.", { title: "FrontPress Local", kind: "info" });
    } catch (e) {
      await message(String(e), { title: "Backup failed", kind: "error" });
    }
  };

  return (
    <div className="site-card">
      <button
        className={`site-toggle ${site.running ? "running" : "stopped"}`}
        disabled={busy}
        onClick={site.running ? onStop : onStart}
        title={busy ? "Working…" : site.running ? "Stop site" : "Start site"}
        aria-label={site.running ? "Stop site" : "Start site"}
      />

      <div className="site-main">
        <div className="site-head">
          <h3 className="site-name">{site.name}</h3>
          <span className="pill">PHP {site.phpVersion}</span>
          <span className="pill subtle">FP {site.frontpressVersion}</span>
        </div>
        <div className="site-meta">
          <a
            className="link"
            onClick={(e) => {
              e.preventDefault();
              onPreview();
            }}
            href={site.url}
          >
            {site.url}
          </a>
        </div>
      </div>

      <div className="site-actions">
        <button className="btn" disabled={busy} onClick={onPreview} title="Open the public site">
          Preview
        </button>
        <button className="btn accent" disabled={busy} onClick={onLogin} title="Open the admin, already signed in">
          Login
        </button>
        <div className="menu-wrap">
          <button className="btn icon" onClick={() => setMenuOpen((v) => !v)}>
            ⋯
          </button>
          {menuOpen && (
            <div className="menu" onMouseLeave={() => setMenuOpen(false)}>
              <button
                onClick={() => {
                  setMenuOpen(false);
                  setDuplicating(true);
                }}
              >
                Duplicate…
              </button>
              <button onClick={backup}>Back up…</button>
              <button
                onClick={() => {
                  setMenuOpen(false);
                  onReveal();
                }}
              >
                Reveal in Finder
              </button>
              <button
                className="danger"
                onClick={() => {
                  setMenuOpen(false);
                  setConfirmDelete(true);
                }}
              >
                Delete…
              </button>
            </div>
          )}
        </div>
      </div>

      {confirmDelete && (
        <div className="confirm">
          <p>
            Delete <strong>{site.name}</strong>?
          </p>
          <div className="confirm-actions">
            <button className="btn ghost" onClick={() => setConfirmDelete(false)}>
              Cancel
            </button>
            <button
              className="btn"
              onClick={() => {
                setConfirmDelete(false);
                onDelete(false);
              }}
            >
              Remove from list only
            </button>
            <button
              className="btn danger"
              onClick={() => {
                setConfirmDelete(false);
                onDelete(true);
              }}
            >
              Delete files too
            </button>
          </div>
        </div>
      )}

      {duplicating && (
        <DuplicateModal
          site={site}
          onClose={() => setDuplicating(false)}
          onDone={() => {
            setDuplicating(false);
            onChanged();
          }}
        />
      )}
    </div>
  );
}
