import { useCallback, useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { api, AppStatus } from "./api";
import { SiteList } from "./components/SiteList";
import { CreateSiteModal } from "./components/CreateSiteModal";
import { PhpSettingsModal } from "./components/PhpSettingsModal";
import { UpdateBanner } from "./components/UpdateBanner";
import { useUpdater } from "./lib/useUpdater";
import "./App.css";

function App() {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [phpOpen, setPhpOpen] = useState(false);
  const [version, setVersion] = useState("");
  const updater = useUpdater();

  useEffect(() => {
    getVersion().then(setVersion).catch(() => {});
  }, []);

  const refresh = useCallback(async () => {
    try {
      setStatus(await api.appStatus());
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    refresh();
    const t = setInterval(refresh, 4000); // keep running/stopped dots fresh
    return () => clearInterval(t);
  }, [refresh]);

  return (
    <div className="app">
      <UpdateBanner
        update={updater.update}
        status={updater.status}
        error={updater.error}
        install={updater.install}
        dismiss={updater.dismiss}
      />

      <header className="topbar">
        <div className="brand">
          <span className="brand-mark">F</span>
          <div>
            <h1>
              FrontPress Local{version ? <span className="ver"> v{version}</span> : null}
            </h1>
            <p className="subtitle">
              {status
                ? `${status.sites.length} site${status.sites.length === 1 ? "" : "s"} · PHP default ${
                    status.globalPhpVersion || "not set"
                  } · ${status.arch}`
                : "Loading…"}
            </p>
          </div>
        </div>
        <div className="topbar-actions">
          <button
            className="btn ghost"
            onClick={async () => {
              await api.stopAllSites();
              refresh();
            }}
            disabled={!status?.sites.some((s) => s.running)}
          >
            Stop all
          </button>
          <button className="btn ghost" onClick={() => setPhpOpen(true)}>
            PHP settings
          </button>
          <button className="btn primary" onClick={() => setCreating(true)}>
            + New site
          </button>
        </div>
      </header>

      {error && <div className="banner error">{error}</div>}

      <main className="content">
        {status && status.sites.length === 0 ? (
          <EmptyState onCreate={() => setCreating(true)} />
        ) : (
          status && <SiteList sites={status.sites} onChanged={refresh} />
        )}
      </main>

      {creating && (
        <CreateSiteModal
          minPhp={status?.minPhp ?? "8.1"}
          globalPhp={status?.globalPhpVersion ?? ""}
          onClose={() => setCreating(false)}
          onCreated={() => {
            setCreating(false);
            refresh();
          }}
        />
      )}

      {phpOpen && (
        <PhpSettingsModal
          onClose={() => {
            setPhpOpen(false);
            refresh();
          }}
        />
      )}
    </div>
  );
}

function EmptyState({ onCreate }: { onCreate: () => void }) {
  return (
    <div className="empty">
      <div className="empty-card">
        <div className="empty-mark">F</div>
        <h2>No sites yet</h2>
        <p>
          Create your first FrontPress Studio site. The latest release is pulled
          from GitHub and runs on a bundled PHP — no WordPress, no database.
        </p>
        <button className="btn primary lg" onClick={onCreate}>
          + Create a site
        </button>
      </div>
    </div>
  );
}

export default App;
