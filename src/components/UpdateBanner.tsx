import { useCallback, useEffect, useRef, useState } from "react";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { api } from "../api";

type Phase = "idle" | "available" | "downloading" | "ready" | "error";

/**
 * Checks for a new FrontPress Local release on launch (via the Tauri updater
 * endpoint) and offers a one-click install. No-ops silently when running a
 * dev build or when the updater isn't configured.
 */
export function UpdateBanner() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [phase, setPhase] = useState<Phase>("idle");
  const [error, setError] = useState<string | null>(null);
  const installing = useRef(false);

  const install = useCallback(async (u: Update) => {
    if (installing.current) return;
    installing.current = true;
    setPhase("downloading");
    setError(null);
    try {
      await u.downloadAndInstall();
      setPhase("ready");
      await relaunch();
    } catch (e) {
      setError(String(e));
      setPhase("error");
      installing.current = false;
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    check()
      .then(async (u) => {
        if (cancelled || !u) return;
        setUpdate(u);
        setPhase("available");
        // Test hook: when launched with FP_SELFTEST_UPDATE, apply immediately
        // so the full download → verify → install → relaunch loop can be
        // exercised without a click.
        if (await api.selftestUpdate().catch(() => false)) {
          install(u);
        }
      })
      .catch(() => {
        // Updater unavailable (dev build, offline, no endpoint) — stay quiet.
      });
    return () => {
      cancelled = true;
    };
  }, [install]);

  if (phase === "idle" || !update) return null;

  return (
    <div className="update-banner">
      <div className="update-text">
        <strong>Update available</strong>
        <span className="muted">
          FrontPress Local {update.version} is ready to install
          {update.body ? ` — ${update.body.slice(0, 80)}` : ""}
        </span>
        {error && <span className="update-error">{error}</span>}
      </div>
      <div className="update-actions">
        {phase === "downloading" ? (
          <span className="muted">Downloading…</span>
        ) : phase === "ready" ? (
          <span className="muted">Restarting…</span>
        ) : (
          <button className="btn primary tiny" onClick={() => install(update)}>
            Install &amp; restart
          </button>
        )}
      </div>
    </div>
  );
}
