import { useCallback, useEffect, useRef, useState } from "react";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { listen } from "@tauri-apps/api/event";
import { api } from "../api";

export type UpdaterStatus =
  | "idle"
  | "checking"
  | "available"
  | "uptodate"
  | "downloading"
  | "ready"
  | "error";

/**
 * Single source of truth for the updater across the topbar button and the
 * banner. All feedback is surfaced via React state (the Tauri webview does
 * not implement window.alert/confirm, so those silently do nothing).
 */
export function useUpdater() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [status, setStatus] = useState<UpdaterStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const installing = useRef(false);

  const install = useCallback(async () => {
    if (!update || installing.current) return;
    installing.current = true;
    setStatus("downloading");
    setError(null);
    try {
      await update.downloadAndInstall();
      setStatus("ready");
      await relaunch();
    } catch (e) {
      setError(String(e));
      setStatus("error");
      installing.current = false;
    }
  }, [update]);

  const checkNow = useCallback(
    async (silent = false) => {
      setError(null);
      if (!silent) setStatus("checking");
      try {
        const u = await check();
        if (u) {
          setUpdate(u);
          setStatus("available");
          if (await api.selftestUpdate().catch(() => false)) {
            // test hook: auto-apply
            await u.downloadAndInstall();
            await relaunch();
          }
        } else {
          setStatus(silent ? "idle" : "uptodate");
        }
      } catch (e) {
        if (silent) {
          setStatus("idle");
        } else {
          setError(String(e));
          setStatus("error");
        }
      }
    },
    []
  );

  // Silent check on launch.
  useEffect(() => {
    checkNow(true);
  }, [checkNow]);

  // The native menu's "Check for Updates…" routes here.
  useEffect(() => {
    const un = listen("menu:check-updates", () => checkNow(false));
    return () => {
      un.then((f) => f());
    };
  }, [checkNow]);

  // Auto-dismiss the transient "up to date" / "error" states.
  useEffect(() => {
    if (status === "uptodate" || status === "error") {
      const t = setTimeout(() => setStatus("idle"), 5000);
      return () => clearTimeout(t);
    }
  }, [status]);

  const dismiss = useCallback(() => setStatus("idle"), []);

  return { update, status, error, checkNow, install, dismiss };
}
