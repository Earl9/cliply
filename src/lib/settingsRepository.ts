import { invoke, isTauri } from "@tauri-apps/api/core";
import type { CliplySettings } from "@/stores/settingsStore";
import { defaultSettingsState } from "@/stores/settingsStore";

export type ShortcutCheck = {
  ok: boolean;
  normalized: string;
  display: string;
  reason: "available" | "current" | "invalid" | "cliply-conflict" | "system-conflict";
  message: string;
};

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

export async function checkGlobalShortcut(
  shortcut: string,
  currentShortcut?: string,
): Promise<ShortcutCheck> {
  if (!isTauri()) {
    return checkMockShortcut(shortcut, currentShortcut);
  }

  return invoke<ShortcutCheck>("check_global_shortcut", {
    shortcut,
    currentShortcut,
  });
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

function checkMockShortcut(shortcut: string, currentShortcut?: string): ShortcutCheck {
  const parsed = normalizeShortcut(shortcut);
  if (!parsed.ok) {
    return parsed;
  }

  const current = currentShortcut ? normalizeShortcut(currentShortcut) : null;
  if (current?.ok && current.normalized === parsed.normalized) {
    return {
      ...parsed,
      reason: "current",
      message: "当前快捷键可用",
    };
  }

  const reserved = new Set(["Control+Alt+Delete", "Control+Shift+Escape", "Alt+F4"]);
  if (reserved.has(parsed.normalized)) {
    return {
      ...parsed,
      ok: false,
      reason: "system-conflict",
      message: "该组合通常被系统占用，请换一个快捷键",
    };
  }

  return parsed;
}

function normalizeShortcut(shortcut: string): ShortcutCheck {
  const parts = shortcut
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean);

  if (!parts.length) {
    return invalidShortcut(shortcut, "请先按下一个快捷键组合");
  }

  const modifiers = {
    control: false,
    alt: false,
    shift: false,
    super: false,
  };
  let key: { normalized: string; display: string } | null = null;

  for (const part of parts) {
    const lower = part.toLowerCase();
    if (lower === "ctrl" || lower === "control") {
      modifiers.control = true;
      continue;
    }
    if (lower === "alt" || lower === "option") {
      modifiers.alt = true;
      continue;
    }
    if (lower === "shift") {
      modifiers.shift = true;
      continue;
    }
    if (lower === "win" || lower === "meta" || lower === "super" || lower === "cmd") {
      modifiers.super = true;
      continue;
    }
    if (key) {
      return invalidShortcut(shortcut, "快捷键只能包含一个主按键");
    }
    key = normalizeShortcutKey(part);
  }

  if (!modifiers.control && !modifiers.alt && !modifiers.super) {
    return invalidShortcut(shortcut, "全局快捷键需要包含 Ctrl、Alt 或 Win，避免误触");
  }

  if (!key) {
    return invalidShortcut(shortcut, "请按下一个非修饰键作为主按键");
  }

  const normalizedParts = [
    modifiers.control ? "Control" : null,
    modifiers.alt ? "Alt" : null,
    modifiers.shift ? "Shift" : null,
    modifiers.super ? "Super" : null,
    key.normalized,
  ].filter(Boolean);
  const displayParts = [
    modifiers.control ? "Ctrl" : null,
    modifiers.alt ? "Alt" : null,
    modifiers.shift ? "Shift" : null,
    modifiers.super ? "Win" : null,
    key.display,
  ].filter(Boolean);

  return {
    ok: true,
    normalized: normalizedParts.join("+"),
    display: displayParts.join("+"),
    reason: "available",
    message: "快捷键可用",
  };
}

function normalizeShortcutKey(key: string) {
  const lower = key.toLowerCase();
  const aliases: Record<string, { normalized: string; display: string }> = {
    esc: { normalized: "Escape", display: "Escape" },
    escape: { normalized: "Escape", display: "Escape" },
    space: { normalized: "Space", display: "Space" },
    delete: { normalized: "Delete", display: "Delete" },
    del: { normalized: "Delete", display: "Delete" },
    backspace: { normalized: "Backspace", display: "Backspace" },
    enter: { normalized: "Enter", display: "Enter" },
    tab: { normalized: "Tab", display: "Tab" },
    insert: { normalized: "Insert", display: "Insert" },
    home: { normalized: "Home", display: "Home" },
    end: { normalized: "End", display: "End" },
    pageup: { normalized: "PageUp", display: "PageUp" },
    pagedown: { normalized: "PageDown", display: "PageDown" },
    arrowup: { normalized: "ArrowUp", display: "ArrowUp" },
    arrowdown: { normalized: "ArrowDown", display: "ArrowDown" },
    arrowleft: { normalized: "ArrowLeft", display: "ArrowLeft" },
    arrowright: { normalized: "ArrowRight", display: "ArrowRight" },
  };

  if (aliases[lower]) {
    return aliases[lower];
  }

  if (/^f([1-9]|1[0-9]|2[0-4])$/i.test(key)) {
    const display = key.toUpperCase();
    return { normalized: display, display };
  }

  if (/^[a-z]$/i.test(key)) {
    const display = key.toUpperCase();
    return { normalized: `Key${display}`, display };
  }

  if (/^[0-9]$/.test(key)) {
    return { normalized: `Digit${key}`, display: key };
  }

  return { normalized: key, display: key };
}

function invalidShortcut(shortcut: string, message: string): ShortcutCheck {
  return {
    ok: false,
    normalized: "",
    display: shortcut.trim(),
    reason: "invalid",
    message,
  };
}
