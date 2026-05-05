import { invoke, isTauri } from "@tauri-apps/api/core";
import type { CliplySettings } from "@/stores/settingsStore";
import { defaultSettingsState } from "@/stores/settingsStore";

export async function getCliplySettings(): Promise<CliplySettings> {
  if (!isTauri()) {
    return readMockSettings();
  }

  return invoke<CliplySettings>("get_cliply_settings");
}

export async function updateCliplySettings(
  settings: CliplySettings,
): Promise<CliplySettings> {
  if (!isTauri()) {
    writeMockSettings(settings);
    return settings;
  }

  return invoke<CliplySettings>("update_cliply_settings", { settings });
}

export async function setMonitoringPaused(paused: boolean): Promise<CliplySettings> {
  if (!isTauri()) {
    const settings = { ...readMockSettings(), pauseMonitoring: paused };
    writeMockSettings(settings);
    return settings;
  }

  return invoke<CliplySettings>("set_monitoring_paused", { paused });
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
