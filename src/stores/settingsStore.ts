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
  theme: "light" | "dark";
  themeName: string;
  accentColor: string;
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
};
