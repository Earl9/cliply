import {
  DEFAULT_AUTO_THEME_SETTINGS,
  type CliplyAutoThemeSettings,
} from "@/theme/theme";

export type CliplySettings = {
  maxHistoryItems: number;
  autoDeleteDays: number;
  pauseMonitoring: boolean;
  launchAtStartup: boolean;
  startMinimized: boolean;
  focusSearchOnOpen: boolean;
  closeAfterPaste: boolean;
  ignoreDuplicate: boolean;
  saveImages: boolean;
  saveHtml: boolean;
  saveSensitive: boolean;
  ignoreApps: string[];
  globalShortcut: string;
  theme: "light" | "dark" | "system";
  themeName: string;
  accentColor: string;
  autoTheme: CliplyAutoThemeSettings;
  imageSync: CliplyImageSyncSettings;
  update: CliplyUpdateSettings;
};

export type ImageSyncMode =
  | "metadata-only"
  | "compressed"
  | "original"
  | "original-with-preview";

export type CliplyImageSyncSettings = {
  mode: ImageSyncMode;
  maxDimension: number;
  quality: number;
  stripMetadata: boolean;
  maxImageSizeMB: number;
};

export type UpdateCheckInterval = "manual" | "daily" | "weekly";
export type UpdateChannel = "stable" | "beta";

export type CliplyUpdateSettings = {
  autoCheck: boolean;
  checkInterval: UpdateCheckInterval;
  channel: UpdateChannel;
  lastCheckedAt?: string | null;
  ignoredVersion?: string | null;
};

export type SettingsState = CliplySettings;

export const defaultSettingsState: CliplySettings = {
  maxHistoryItems: 1000,
  autoDeleteDays: 30,
  pauseMonitoring: false,
  launchAtStartup: false,
  startMinimized: false,
  focusSearchOnOpen: true,
  closeAfterPaste: true,
  ignoreDuplicate: true,
  saveImages: true,
  saveHtml: true,
  saveSensitive: false,
  ignoreApps: ["1Password", "Bitwarden", "KeePass", "KeePassXC", "Windows Credential Manager"],
  globalShortcut: "Ctrl+Shift+V",
  theme: "light",
  themeName: "purple-default",
  accentColor: "#6D4CFF",
  autoTheme: DEFAULT_AUTO_THEME_SETTINGS,
  imageSync: {
    mode: "metadata-only",
    maxDimension: 1920,
    quality: 80,
    stripMetadata: true,
    maxImageSizeMB: 25,
  },
  update: {
    autoCheck: true,
    checkInterval: "daily",
    channel: "beta",
    lastCheckedAt: null,
    ignoredVersion: null,
  },
};
