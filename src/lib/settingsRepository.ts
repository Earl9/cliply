import { invoke, isTauri } from "@tauri-apps/api/core";
import type { CliplySettings } from "@/stores/settingsStore";
import { defaultSettingsState } from "@/stores/settingsStore";

export async function getCliplySettings(): Promise<CliplySettings> {
  if (!isTauri()) {
    return readMockSettings();
  }

  return invokeWithMockFallback(
    () => invoke<CliplySettings>("get_cliply_settings"),
    readMockSettings,
  );
}

export async function updateCliplySettings(
  settings: CliplySettings,
): Promise<CliplySettings> {
  if (!isTauri()) {
    writeMockSettings(settings);
    return settings;
  }

  return invokeWithMockFallback(
    () => invoke<CliplySettings>("update_cliply_settings", { settings }),
    () => {
      writeMockSettings(settings);
      return settings;
    },
  );
}

export async function setMonitoringPaused(paused: boolean): Promise<CliplySettings> {
  if (!isTauri()) {
    const settings = { ...readMockSettings(), pauseMonitoring: paused };
    writeMockSettings(settings);
    return settings;
  }

  return invokeWithMockFallback(
    () => invoke<CliplySettings>("set_monitoring_paused", { paused }),
    () => {
      const settings = { ...readMockSettings(), pauseMonitoring: paused };
      writeMockSettings(settings);
      return settings;
    },
  );
}

async function invokeWithMockFallback<T>(
  invokeCommand: () => Promise<T>,
  fallback: () => T,
): Promise<T> {
  try {
    return await invokeCommand();
  } catch (error) {
    console.warn("[cliply:settings-fallback]", error);
    return fallback();
  }
}

function readMockSettings(): CliplySettings {
  try {
    const raw = window.localStorage.getItem("cliply.settings");
    if (!raw) {
      return defaultSettingsState;
    }

    return { ...defaultSettingsState, ...JSON.parse(raw) };
  } catch {
    return defaultSettingsState;
  }
}

function writeMockSettings(settings: CliplySettings) {
  window.localStorage.setItem("cliply.settings", JSON.stringify(settings));
}
