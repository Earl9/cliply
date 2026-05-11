import { invoke, isTauri } from "@tauri-apps/api/core";

export type CliplyDebugInfo = {
  appVersion: string;
  logPath: string;
  logDir: string;
  databasePath: string;
  dataDir: string;
  databaseSizeBytes: number;
  historyCount: number;
  lastSyncedAt?: string | null;
  lastSyncStatus?: string | null;
  lastSyncError?: string | null;
  recentError?: string | null;
};

export async function getCliplyDebugInfo(): Promise<CliplyDebugInfo> {
  if (!isTauri()) {
    return {
      appVersion: "0.4.1-beta.4",
      logPath: "%APPDATA%\\com.cliply.app\\cliply.log",
      logDir: "%APPDATA%\\com.cliply.app",
      databasePath: "%APPDATA%\\com.cliply.app\\cliply.db",
      dataDir: "%APPDATA%\\com.cliply.app",
      databaseSizeBytes: 0,
      historyCount: 4,
      lastSyncedAt: null,
      lastSyncStatus: "not configured",
      lastSyncError: null,
      recentError: null,
    };
  }

  return invoke<CliplyDebugInfo>("get_debug_info");
}

export async function openCliplyLogDirectory(): Promise<void> {
  if (!isTauri()) {
    return;
  }

  await invoke<void>("open_log_directory");
}
