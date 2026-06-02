import { useState } from "react";
import { api, SiteView } from "../api";
import { Modal } from "./Modal";

export function DuplicateModal({
  site,
  onClose,
  onDone,
}: {
  site: SiteView;
  onClose: () => void;
  onDone: () => void;
}) {
  const [name, setName] = useState(`${site.name} copy`);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    setBusy(true);
    setError(null);
    try {
      await api.duplicateSite(site.id, name);
      onDone();
    } catch (e) {
      setError(String(e));
      setBusy(false);
    }
  };

  return (
    <Modal title={`Duplicate “${site.name}”`} onClose={busy ? undefined : onClose}>
      {busy ? (
        <div className="progress-pane">
          <div className="spinner" />
          <p className="progress-msg">Copying site & starting…</p>
        </div>
      ) : (
        <>
          <label className="field">
            <span>New site name</span>
            <input
              autoFocus
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && name.trim()) submit();
              }}
            />
            <small className="muted">
              A full copy with its own folder, port, and fresh credentials.
            </small>
          </label>
          {error && <div className="banner error">{error}</div>}
          <div className="modal-actions">
            <button className="btn ghost" onClick={onClose}>
              Cancel
            </button>
            <button className="btn primary" disabled={!name.trim()} onClick={submit}>
              Duplicate
            </button>
          </div>
        </>
      )}
    </Modal>
  );
}
