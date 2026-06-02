import { useState } from "react";
import { SiteView } from "../api";

export function SiteCard({
  site,
  busy,
  onPreview,
  onLogin,
  onStart,
  onStop,
  onReveal,
  onDelete,
}: {
  site: SiteView;
  busy: boolean;
  onPreview: () => void;
  onLogin: () => void;
  onStart: () => void;
  onStop: () => void;
  onReveal: () => void;
  onDelete: (deleteFiles: boolean) => void;
}) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  return (
    <div className={`site-card ${site.running ? "is-running" : ""}`}>
      <div className="site-main">
        <div className="site-head">
          <span className={`dot ${site.running ? "on" : "off"}`} />
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
          <span className="muted">· {site.slug}</span>
        </div>
      </div>

      <div className="site-actions">
        <button className="btn" disabled={busy} onClick={onPreview} title="Open the public site">
          Preview
        </button>
        <button className="btn accent" disabled={busy} onClick={onLogin} title="Open the admin, already signed in">
          Log in
        </button>
        {site.running ? (
          <button className="btn ghost" disabled={busy} onClick={onStop}>
            Stop
          </button>
        ) : (
          <button className="btn ghost" disabled={busy} onClick={onStart}>
            Start
          </button>
        )}
        <div className="menu-wrap">
          <button className="btn icon" onClick={() => setMenuOpen((v) => !v)}>
            ⋯
          </button>
          {menuOpen && (
            <div className="menu" onMouseLeave={() => setMenuOpen(false)}>
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
    </div>
  );
}
