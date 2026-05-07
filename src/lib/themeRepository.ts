import { invoke, isTauri } from "@tauri-apps/api/core";

export type SystemThemeColorInfo = {
  systemAccent?: string | null;
  source: string;
  status: "ok" | "fallback" | "error" | string;
  message: string;
};

export async function getSystemThemeColors(): Promise<SystemThemeColorInfo> {
  if (!isTauri()) {
    return readMockSystemThemeColors();
  }

  return invoke<SystemThemeColorInfo>("get_system_theme_colors");
}

function readMockSystemThemeColors(): SystemThemeColorInfo {
  return {
    systemAccent: window.localStorage.getItem("cliply.autoTheme.mockSystemAccentColor"),
    source: "browser-mock",
    status: "fallback",
    message: "浏览器预览模式使用 localStorage mock 颜色。",
  };
}
