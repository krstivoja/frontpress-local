import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { api, PhpCatalog, SetupProgress } from "../api";
import { Modal } from "./Modal";

export function PhpSettingsModal({ onClose }: { onClose: () => void }) {
  const [catalog, setCatalog] = useState<PhpCatalog | null>(null);
  const [globalVersion, setGlobalVersion] = useState("");
  const [busyMinor, setBusyMinor] = useState<string | null>(null);
  const [progress, setProgress] = useState<SetupProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = async () => {
    try {
      const [cat, settings] = await Promise.all([
        api.availablePhp(),
        api.appStatus(),
      ]);
      setCatalog(cat);
      setGlobalVersion(settings.globalPhpVersion);
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
    <Modal title="PHP runtimes" onClose={onClose}>
      <p className="muted">
        Static PHP builds for {catalog?.arch ?? "your Mac"}. Minimum supported by
        FrontPress: PHP {catalog?.minPhp ?? "8.1"}. The default is used for new
        sites unless you pick a per-site version.
      </p>

      {error && <div className="banner error">{error}</div>}

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
    </Modal>
  );
}
