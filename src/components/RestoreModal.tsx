import { useEffect, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { api, SiteView } from "../api";
import { Modal } from "./Modal";

/** Per-site, in-place restore: replace THIS site's files with a backup zip. */
export function RestoreModal({
  site,
  onClose,
  onDone,
}: {
  site: SiteView;
  onClose: () => void;
  onDone: () => void;
}) {
  const [picked, setPicked] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [hover, setHover] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const choose = (path: string) => {
    if (!path.toLowerCase().endsWith(".zip")) {
      setError("Please choose a .zip backup file.");
      return;
    }
    setError(null);
    setPicked(path);
  };

  const restore = async () => {
    if (!picked) return;
    setBusy(true);
    setError(null);
    try {
      await api.restoreIntoSite(site.id, picked);
      onDone();
    } catch (e) {
      setError(String(e));
      setBusy(false);
    }
  };

  // Native file drops come through Tauri's webview event, not the DOM.
  useEffect(() => {
    const un = getCurrentWebview().onDragDropEvent((e) => {
      if (e.payload.type === "over" || e.payload.type === "enter") setHover(true);
      else if (e.payload.type === "leave") setHover(false);
      else if (e.payload.type === "drop") {
        setHover(false);
        const p = e.payload.paths?.[0];
        if (p && !busy) choose(p);
      }
    });
    return () => {
      un.then((f) => f());
    };
  }, [busy]);

  const browse = async () => {
    const file = await open({
      multiple: false,
      filters: [{ name: "Backup zip", extensions: ["zip"] }],
    });
    if (typeof file === "string") choose(file);
  };

  const fileName = picked?.split("/").pop() ?? "";

  return (
    <Modal title={`Restore “${site.name}”`} onClose={busy ? undefined : onClose}>
      {busy ? (
        <div className="progress-pane">
          <div className="spinner" />
          <p className="progress-msg">Restoring & starting…</p>
        </div>
      ) : !picked ? (
        <>
          <div className={`dropzone ${hover ? "is-hover" : ""}`} onClick={browse}>
            <div className="dropzone-icon">⬇</div>
            <strong>Drop a backup .zip for this site</strong>
            <span className="muted">or click to choose a file</span>
          </div>
          <small className="muted">
            This replaces <strong>{site.name}</strong>'s current files with the
            backup. The site keeps its address ({site.url}).
          </small>
          {error && <div className="banner error">{error}</div>}
          <div className="modal-actions">
            <button className="btn ghost" onClick={onClose}>
              Cancel
            </button>
          </div>
        </>
      ) : (
        <>
          <div className="confirm-restore">
            <p>
              Replace <strong>{site.name}</strong>'s files with{" "}
              <code>{fileName}</code>?
            </p>
            <small className="muted">
              The current files are swapped out and the site is restarted. This
              can't be undone.
            </small>
          </div>
          {error && <div className="banner error">{error}</div>}
          <div className="modal-actions">
            <button className="btn ghost" onClick={() => setPicked(null)}>
              Choose another
            </button>
            <button className="btn danger" onClick={restore}>
              Restore &amp; replace
            </button>
          </div>
        </>
      )}
    </Modal>
  );
}
