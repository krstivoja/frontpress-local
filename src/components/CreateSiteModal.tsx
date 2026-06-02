import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, PhpOption, SetupProgress } from "../api";
import { Modal } from "./Modal";

export function CreateSiteModal({
  minPhp,
  globalPhp,
  onClose,
  onCreated,
}: {
  minPhp: string;
  globalPhp: string;
  onClose: () => void;
  onCreated: () => void;
}) {
  const [name, setName] = useState("");
  const [phpMode, setPhpMode] = useState<"global" | "custom">("global");
  const [options, setOptions] = useState<PhpOption[]>([]);
  const [phpMinor, setPhpMinor] = useState("");
  const [creating, setCreating] = useState(false);
  const [progress, setProgress] = useState<SetupProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .availablePhp()
      .then((cat) => {
        setOptions(cat.options);
        const def =
          cat.options.find((o) => o.installed)?.minor ??
          cat.options[0]?.minor ??
          "";
        setPhpMinor(def);
      })
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    const un = listen<SetupProgress>("setup-progress", (e) => setProgress(e.payload));
    return () => {
      un.then((f) => f());
    };
  }, []);

  const submit = async () => {
    setCreating(true);
    setError(null);
    setProgress(null);
    try {
      await api.createSite({
        name,
        phpMode,
        phpMinor: phpMode === "custom" ? phpMinor : undefined,
      });
      onCreated();
    } catch (e) {
      setError(String(e));
      setCreating(false);
    }
  };

  return (
    <Modal title="New FrontPress site" onClose={creating ? undefined : onClose}>
      {!creating ? (
        <>
          <label className="field">
            <span>Site name</span>
            <input
              autoFocus
              placeholder="My new site"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && name.trim()) submit();
              }}
            />
            <small className="muted">
              Folder: ~/FrontPress Sites/{slugify(name) || "…"}
            </small>
          </label>

          <div className="field">
            <span>PHP version</span>
            <div className="radio-row">
              <label className={phpMode === "global" ? "radio sel" : "radio"}>
                <input
                  type="radio"
                  checked={phpMode === "global"}
                  onChange={() => setPhpMode("global")}
                />
                Use default{globalPhp ? ` (${globalPhp})` : ""}
              </label>
              <label className={phpMode === "custom" ? "radio sel" : "radio"}>
                <input
                  type="radio"
                  checked={phpMode === "custom"}
                  onChange={() => setPhpMode("custom")}
                />
                Per-site
              </label>
            </div>
            {phpMode === "custom" && (
              <select value={phpMinor} onChange={(e) => setPhpMinor(e.target.value)}>
                {options.map((o) => (
                  <option key={o.minor} value={o.minor}>
                    PHP {o.minor} (latest {o.latest})
                    {o.installed ? " · installed" : ""}
                  </option>
                ))}
              </select>
            )}
            <small className="muted">Minimum supported: PHP {minPhp}</small>
          </div>

          {error && <div className="banner error">{error}</div>}

          <div className="modal-actions">
            <button className="btn ghost" onClick={onClose}>
              Cancel
            </button>
            <button className="btn primary" disabled={!name.trim()} onClick={submit}>
              Create site
            </button>
          </div>
        </>
      ) : (
        <div className="progress-pane">
          <div className="spinner" />
          <p className="progress-msg">{progress?.message ?? "Starting…"}</p>
          <div className="progressbar">
            <div
              className="progressbar-fill"
              style={{
                width:
                  progress?.pct != null ? `${Math.min(100, progress.pct)}%` : "100%",
                opacity: progress?.pct != null ? 1 : 0.4,
              }}
            />
          </div>
          {error && <div className="banner error">{error}</div>}
        </div>
      )}
    </Modal>
  );
}

function slugify(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}
