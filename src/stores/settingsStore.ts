export type SettingsState = {
  theme: "light" | "dark";
  globalShortcut: string;
  localOnly: boolean;
};

export const defaultSettingsState: SettingsState = {
  theme: "light",
  globalShortcut: "Ctrl+Shift+V",
  localOnly: true,
};
