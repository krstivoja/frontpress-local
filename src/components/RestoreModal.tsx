import { useEffect, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { Modal } from "./Modal";

export function RestoreModal({
  onClose,
  onDone,
}: {
  onClose: () => void;
  onDone: () => void;
}) {
  const [busy, setBusy] = useState(false);
  const [hover, setHover] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const restore = async (path: string) => {
    if (!path.toLowerCase().endsWith(".zip")) {
      setError("Please drop a .zip backup file.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await api.restoreSite(path);
      onDone();
    } catch (e) {
      setError(String(e));
      setBusy(false);
    }
  };

  // Native file drops are delivered by Tauri (the browser's drop event has no
  // real paths), so we subscribe to the webview drag-drop event while open.
  useEffect(() => {
    const un = getCurrentWebview().onDragDropEvent((e) => {
      if (e.payload.type === "over" || e.payload.type === "enter") {
        setHover(true);
      } else if (e.payload.type === "leave") {
        setHover(false);
      } else if (e.payload.type === "drop") {
        setHover(false);
        const p = e.payload.paths?.[0];
        if (p && !busy) restore(p);
      }
    });
    return () => {
      un.then((f) => f());
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [busy]);

  const browse = async () => {
    const file = await open({
      multiple: false,
      filters: [{ name: "Backup zip", extensions: ["zip"] }],
    });
    if (typeof file === "string") restore(file);
  };

  return (
    <Modal title="Restore from backup" onClose={busy ? undefined : onClose}>
      {busy ? (
        <div className="progress-pane">
          <div className="spinner" />
          <p className="progress-msg">Restoring & starting…</p>
        </div>
      ) : (
        <>
          <div className={`dropzone ${hover ? "is-hover" : ""}`} onClick={browse}>
            <div className="dropzone-icon">⬇</div>
            <strong>Drop a backup .zip here</strong>
            <span className="muted">or click to choose a file</span>
          </div>
          <small className="muted">
            Restores into a new site with its own folder and port. The original
            stays untouched.
          </small>
          {error && <div className="banner error">{error}</div>}
          <div className="modal-actions">
            <button className="btn ghost" onClick={onClose}>
              Cancel
            </button>
          </div>
        </>
      )}
    </Modal>
  );
}
