import { useState } from "react";
import { api, SiteView } from "../api";
import { SiteCard } from "./SiteCard";

export function SiteList({
  sites,
  onChanged,
}: {
  sites: SiteView[];
  onChanged: () => void;
}) {
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = async (id: string, fn: () => Promise<unknown>) => {
    setBusy(id);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(null);
      onChanged();
    }
  };

  return (
    <div className="site-list">
      {error && <div className="banner error">{error}</div>}
      {sites.map((site) => (
        <SiteCard
          key={site.id}
          site={site}
          busy={busy === site.id}
          onPreview={() => run(site.id, () => api.openPreview(site.id))}
          onLogin={() => run(site.id, () => api.autoLogin(site.id))}
          onStart={() => run(site.id, () => api.startSite(site.id))}
          onStop={() => run(site.id, () => api.stopSite(site.id))}
          onReveal={() => run(site.id, () => api.revealInFinder(site.id))}
          onDelete={(deleteFiles) =>
            run(site.id, () => api.deleteSite(site.id, deleteFiles))
          }
        />
      ))}
    </div>
  );
}
