import { invoke } from "@tauri-apps/api/core";

export interface SiteView {
  id: string;
  name: string;
  slug: string;
  path: string;
  port: number;
  phpVersion: string;
  frontpressVersion: string;
  adminUser: string;
  running: boolean;
  url: string;
}

export interface AppStatus {
  sites: SiteView[];
  globalPhpVersion: string;
  minPhp: string;
  arch: string;
  installedPhp: string[];
  editor: string;
  sitesDir: string;
}

export interface PhpOption {
  minor: string;
  latest: string;
  installed: boolean;
}

export interface PhpCatalog {
  arch: string;
  minPhp: string;
  installed: string[];
  options: PhpOption[];
}

export interface CreateSiteArgs {
  name: string;
  phpMode: "global" | "custom";
  phpMinor?: string;
}

export const api = {
  appStatus: () => invoke<AppStatus>("app_status"),
  availablePhp: () => invoke<PhpCatalog>("available_php"),
  installPhp: (minor: string) => invoke<string>("install_php", { minor }),
  setGlobalPhp: (minor: string) => invoke<string>("set_global_php", { minor }),
  createSite: (args: CreateSiteArgs) => invoke<SiteView>("create_site", { args }),
  startSite: (id: string) => invoke<SiteView>("start_site", { id }),
  stopSite: (id: string) => invoke<SiteView>("stop_site", { id }),
  stopAllSites: () => invoke<void>("stop_all_sites"),
  deleteSite: (id: string, deleteFiles: boolean) =>
    invoke<void>("delete_site", { id, deleteFiles }),
  duplicateSite: (id: string, name: string) =>
    invoke<SiteView>("duplicate_site", { id, name }),
  importSite: (folder: string) => invoke<SiteView>("import_site", { folder }),
  backupSite: (id: string, dest: string) =>
    invoke<void>("backup_site", { id, dest }),
  restoreIntoSite: (id: string, zipPath: string) =>
    invoke<SiteView>("restore_into_site", { id, zipPath }),
  openPreview: (id: string) => invoke<void>("open_preview", { id }),
  autoLogin: (id: string) => invoke<void>("auto_login", { id }),
  revealInFinder: (id: string) => invoke<void>("reveal_in_finder", { id }),
  openInEditor: (id: string) => invoke<void>("open_in_editor", { id }),
  listEditors: () => invoke<string[]>("list_editors"),
  setEditor: (editor: string) => invoke<void>("set_editor", { editor }),
  setSitesDir: (newDir: string) => invoke<AppStatus>("set_sites_dir", { newDir }),
  rescanSites: () => invoke<AppStatus>("rescan_sites"),
  selftestUpdate: () => invoke<boolean>("selftest_update"),
};

export interface SetupProgress {
  phase: string;
  message: string;
  pct: number | null;
}
