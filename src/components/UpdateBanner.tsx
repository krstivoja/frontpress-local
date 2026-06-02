import { Update } from "@tauri-apps/plugin-updater";
import { UpdaterStatus } from "../lib/useUpdater";

/**
 * Presentational update banner. Renders whatever the shared updater state is,
 * so both the launch check and the "Check for updates" button surface here.
 */
export function UpdateBanner({
  update,
  status,
  error,
  install,
  dismiss,
}: {
  update: Update | null;
  status: UpdaterStatus;
  error: string | null;
  install: () => void;
  dismiss: () => void;
}) {
  if (status === "idle") return null;

  const highlight =
    status === "available" || status === "downloading" || status === "ready";

  return (
    <div className={`update-banner${highlight ? " is-available" : ""}`}>
      <div className="update-text">
        {status === "checking" && <strong>Checking for updates…</strong>}
        {status === "uptodate" && (
          <strong>You're on the latest version.</strong>
        )}
        {status === "error" && (
          <>
            <strong>Update check failed</strong>
            {error && <span className="update-error">{error}</span>}
          </>
        )}
        {(status === "available" ||
          status === "downloading" ||
          status === "ready") && (
          <>
            <strong>Update available</strong>
            <span className="muted">
              FrontPress Local {update?.version} is ready to install
              {update?.body ? ` — ${update.body.slice(0, 80)}` : ""}
            </span>
            {error && <span className="update-error">{error}</span>}
          </>
        )}
      </div>
      <div className="update-actions">
        {status === "downloading" ? (
          <span className="muted">Downloading…</span>
        ) : status === "ready" ? (
          <span className="muted">Restarting…</span>
        ) : status === "available" ? (
          <button className="btn primary tiny" onClick={install}>
            Install &amp; restart
          </button>
        ) : status === "uptodate" || status === "error" ? (
          <button className="btn ghost tiny" onClick={dismiss}>
            Dismiss
          </button>
        ) : null}
      </div>
    </div>
  );
}
