import { invoke, isTauri } from "@tauri-apps/api/core";

export type CliplyDebugInfo = {
  logPath: string;
  databasePath: string;
};

export async function getCliplyDebugInfo(): Promise<CliplyDebugInfo> {
  if (!isTauri()) {
    return {
      logPath: "%APPDATA%\\com.cliply.app\\cliply.log",
      databasePath: "%APPDATA%\\com.cliply.app\\cliply.db",
    };
  }

  return invoke<CliplyDebugInfo>("get_debug_info");
}
