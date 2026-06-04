import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, PhpCatalog, SetupProgress } from "../api";
import { Modal } from "./Modal";

const CUSTOM = "__custom__";

export function SettingsModal({ onClose }: { onClose: () => void }) {
  const [catalog, setCatalog] = useState<PhpCatalog | null>(null);
  const [globalVersion, setGlobalVersion] = useState("");
  const [busyMinor, setBusyMinor] = useState<string | null>(null);
  const [progress, setProgress] = useState<SetupProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const [editors, setEditors] = useState<string[]>([]);
  const [editor, setEditor] = useState("");
  const [customEditor, setCustomEditor] = useState("");
  const [picker, setPicker] = useState(""); // current <select> value

  const load = async () => {
    try {
      const [cat, status, eds] = await Promise.all([
        api.availablePhp(),
        api.appStatus(),
        api.listEditors(),
      ]);
      setCatalog(cat);
      setGlobalVersion(status.globalPhpVersion);
      setEditors(eds);
      setEditor(status.editor);
      // If the saved editor isn't a detected one, treat it as custom.
      setPicker(status.editor && !eds.includes(status.editor) ? CUSTOM : status.editor);
      if (status.editor && !eds.includes(status.editor)) setCustomEditor(status.editor);
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    load();
    const un = listen<SetupProgress>("setup-progress", (e) => setProgress(e.payload));
    return () => {
      un.then((f) => f());
    };
  }, []);

  const saveEditor = async (val: string) => {
    try {
      await api.setEditor(val);
      setEditor(val);
    } catch (e) {
      setError(String(e));
    }
  };

  const onPick = (val: string) => {
    setPicker(val);
    if (val === CUSTOM) return; // wait for the custom input + Save
    saveEditor(val);
  };

  const install = async (minor: string) => {
    setBusyMinor(minor);
    setError(null);
    setProgress(null);
    try {
      await api.installPhp(minor);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyMinor(null);
      setProgress(null);
    }
  };

  const makeDefault = async (minor: string) => {
    setBusyMinor(minor);
    setError(null);
    try {
      const v = await api.setGlobalPhp(minor);
      setGlobalVersion(v);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusyMinor(null);
      setProgress(null);
    }
  };

  return (
    <Modal title="Settings" onClose={onClose}>
      {error && <div className="banner error">{error}</div>}

      {/* ── Editor ─────────────────────────────────────── */}
      <div className="field">
        <span>Editor</span>
        <select value={picker} onChange={(e) => onPick(e.target.value)}>
          <option value="">None</option>
          {editors.map((e) => (
            <option key={e} value={e}>
              {e}
            </option>
          ))}
          <option value={CUSTOM}>Other…</option>
        </select>
        {picker === CUSTOM && (
          <div className="row-inline">
            <input
              placeholder={
                "App name (e.g. Visual Studio Code) or command (e.g. code)"
              }
              value={customEditor}
              onChange={(e) => setCustomEditor(e.target.value)}
            />
            <button
              className="btn tiny primary"
              disabled={!customEditor.trim()}
              onClick={() => saveEditor(customEditor.trim())}
            >
              Save
            </button>
          </div>
        )}
        <small className="muted">
          Used by “Open in editor” on each site.
          {editor ? ` Currently: ${editor}.` : ""}
          {editors.length === 0
            ? " No editors detected — type an app name or command."
            : ""}
        </small>
      </div>

      {/* ── PHP runtimes ───────────────────────────────── */}
      <div className="field">
        <span>PHP runtimes</span>
        <small className="muted">
          Static PHP builds for {catalog?.arch ?? "your Mac"}. Minimum:{" "}
          PHP {catalog?.minPhp ?? "8.1"}. The default is used for new sites
          unless you pick a per-site version.
        </small>
        <div className="php-table">
          {!catalog && <div className="muted">Loading available versions…</div>}
          {catalog?.options.map((o) => {
            const isDefault = globalVersion === o.latest;
            const busy = busyMinor === o.minor;
            return (
              <div key={o.minor} className="php-row">
                <div className="php-ver">
                  <strong>PHP {o.minor}</strong>
                  <span className="muted">latest {o.latest}</span>
                  {isDefault && <span className="pill accent">default</span>}
                  {o.installed && !isDefault && <span className="pill">installed</span>}
                </div>
                <div className="php-row-actions">
                  {busy && progress ? (
                    <span className="muted">
                      {progress.message}
                      {progress.pct != null ? ` ${Math.round(progress.pct)}%` : ""}
                    </span>
                  ) : o.installed ? (
                    <button
                      className="btn tiny"
                      disabled={isDefault || busy}
                      onClick={() => makeDefault(o.minor)}
                    >
                      {isDefault ? "Default" : "Make default"}
                    </button>
                  ) : (
                    <button
                      className="btn tiny primary"
                      disabled={busy}
                      onClick={() => install(o.minor)}
                    >
                      Download
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </Modal>
  );
}
