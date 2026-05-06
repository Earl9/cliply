// Cliply theme token draft
// 用法：
//   import { applyCliplyTheme, DEFAULT_THEME_NAME } from "@/theme/theme";
//   applyCliplyTheme(DEFAULT_THEME_NAME);

export type CliplyThemeName =
  | "purple-default"
  | "mint-green"
  | "lake-blue"
  | "teal-fresh"
  | "coral-orange"
  | "rose-violet";

export type CliplyThemeTokens = {
  name: CliplyThemeName;
  label: string;
  description: string;

  // Brand colors
  primary: string;
  primaryHover: string;
  primaryActive: string;
  primarySoft: string;
  primaryBorder: string;
  primaryText: string;

  // Neutral surfaces
  appBg: string;
  windowBg: string;
  panelBg: string;
  cardBg: string;
  inputBg: string;
  mutedBg: string;

  // Borders and rings
  border: string;
  borderStrong: string;
  divider: string;
  focusRing: string;

  // Text
  text: string;
  textSecondary: string;
  muted: string;
  placeholder: string;
  disabledText: string;

  // Semantic colors
  success: string;
  successSoft: string;
  warning: string;
  warningSoft: string;
  danger: string;
  dangerSoft: string;
  info: string;
  infoSoft: string;

  // Shadows
  shadowWindow: string;
  shadowPanel: string;
  shadowCardHover: string;
  shadowSelected: string;

  // Theme preview swatch color
  swatch: string;
};

export const DEFAULT_THEME_NAME: CliplyThemeName = "purple-default";

export const CLIPLY_THEME_STORAGE_KEY = "cliply.theme.name";

export const CLIPLY_THEMES: Record<CliplyThemeName, CliplyThemeTokens> = {
  "purple-default": {
    name: "purple-default",
    label: "默认紫",
    description: "Cliply 默认主题，现代、稳定、适合生产力工具。",

    primary: "#6D4CFF",
    primaryHover: "#5B3FE6",
    primaryActive: "#4D34C7",
    primarySoft: "#F3F0FF",
    primaryBorder: "#D9D0FF",
    primaryText: "#FFFFFF",

    appBg: "#F5F7FB",
    windowBg: "#F8FAFC",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#F3F5F8",

    border: "#E7EAF1",
    borderStrong: "#D8DEE8",
    divider: "#EEF1F5",
    focusRing: "rgba(109, 76, 255, 0.16)",

    text: "#1F2937",
    textSecondary: "#667085",
    muted: "#98A2B3",
    placeholder: "#7B8496",
    disabledText: "#B6BEC9",

    success: "#22C55E",
    successSoft: "#ECFDF3",
    warning: "#F59E0B",
    warningSoft: "#FFF7E6",
    danger: "#EF4444",
    dangerSoft: "#FEF2F2",
    info: "#2563EB",
    infoSoft: "#EFF6FF",

    shadowWindow: "0 24px 80px rgba(15, 23, 42, 0.16)",
    shadowPanel: "0 8px 24px rgba(15, 23, 42, 0.045)",
    shadowCardHover: "0 8px 20px rgba(15, 23, 42, 0.06)",
    shadowSelected: "0 0 0 1px #6D4CFF, 0 8px 22px rgba(109, 76, 255, 0.12)",

    swatch: "#6D4CFF",
  },

  "mint-green": {
    name: "mint-green",
    label: "薄荷绿",
    description: "清爽、轻盈，适合偏自然和安全感的界面。",

    primary: "#6FCF7B",
    primaryHover: "#5FBD6C",
    primaryActive: "#4EA85A",
    primarySoft: "#EDF9EF",
    primaryBorder: "#CFEFD5",
    primaryText: "#FFFFFF",

    appBg: "#F5F8F6",
    windowBg: "#F8FBF9",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#F3F7F4",

    border: "#E6EEE8",
    borderStrong: "#D5E2D8",
    divider: "#EEF4EF",
    focusRing: "rgba(111, 207, 123, 0.20)",

    text: "#1F2937",
    textSecondary: "#667085",
    muted: "#98A2B3",
    placeholder: "#7B8496",
    disabledText: "#B6BEC9",

    success: "#22C55E",
    successSoft: "#ECFDF3",
    warning: "#F59E0B",
    warningSoft: "#FFF7E6",
    danger: "#EF4444",
    dangerSoft: "#FEF2F2",
    info: "#2563EB",
    infoSoft: "#EFF6FF",

    shadowWindow: "0 24px 80px rgba(15, 23, 42, 0.15)",
    shadowPanel: "0 8px 24px rgba(15, 23, 42, 0.04)",
    shadowCardHover: "0 8px 20px rgba(15, 23, 42, 0.055)",
    shadowSelected: "0 0 0 1px #6FCF7B, 0 8px 22px rgba(111, 207, 123, 0.14)",

    swatch: "#6FCF7B",
  },

  "lake-blue": {
    name: "lake-blue",
    label: "湖蓝色",
    description: "克制、专业，偏系统工具气质。",

    primary: "#3B82F6",
    primaryHover: "#2F74E6",
    primaryActive: "#2563EB",
    primarySoft: "#EFF6FF",
    primaryBorder: "#CFE1FF",
    primaryText: "#FFFFFF",

    appBg: "#F4F7FB",
    windowBg: "#F8FAFC",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#F3F6FA",

    border: "#E5EAF1",
    borderStrong: "#D7E0EC",
    divider: "#EEF2F6",
    focusRing: "rgba(59, 130, 246, 0.16)",

    text: "#1F2937",
    textSecondary: "#667085",
    muted: "#98A2B3",
    placeholder: "#7B8496",
    disabledText: "#B6BEC9",

    success: "#22C55E",
    successSoft: "#ECFDF3",
    warning: "#F59E0B",
    warningSoft: "#FFF7E6",
    danger: "#EF4444",
    dangerSoft: "#FEF2F2",
    info: "#2563EB",
    infoSoft: "#EFF6FF",

    shadowWindow: "0 24px 80px rgba(15, 23, 42, 0.16)",
    shadowPanel: "0 8px 24px rgba(15, 23, 42, 0.045)",
    shadowCardHover: "0 8px 20px rgba(15, 23, 42, 0.06)",
    shadowSelected: "0 0 0 1px #3B82F6, 0 8px 22px rgba(59, 130, 246, 0.12)",

    swatch: "#3B82F6",
  },

  "teal-fresh": {
    name: "teal-fresh",
    label: "清爽青",
    description: "比绿色更科技、更稳，适合安全、隐私和工具场景。",

    primary: "#14B8A6",
    primaryHover: "#0FA595",
    primaryActive: "#0D9488",
    primarySoft: "#ECFDF8",
    primaryBorder: "#BFEFE7",
    primaryText: "#FFFFFF",

    appBg: "#F3F8F8",
    windowBg: "#F8FBFB",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#F2F7F7",

    border: "#E4EEEE",
    borderStrong: "#D2E2E2",
    divider: "#EDF4F4",
    focusRing: "rgba(20, 184, 166, 0.18)",

    text: "#1F2937",
    textSecondary: "#667085",
    muted: "#98A2B3",
    placeholder: "#7B8496",
    disabledText: "#B6BEC9",

    success: "#22C55E",
    successSoft: "#ECFDF3",
    warning: "#F59E0B",
    warningSoft: "#FFF7E6",
    danger: "#EF4444",
    dangerSoft: "#FEF2F2",
    info: "#2563EB",
    infoSoft: "#EFF6FF",

    shadowWindow: "0 24px 80px rgba(15, 23, 42, 0.15)",
    shadowPanel: "0 8px 24px rgba(15, 23, 42, 0.04)",
    shadowCardHover: "0 8px 20px rgba(15, 23, 42, 0.055)",
    shadowSelected: "0 0 0 1px #14B8A6, 0 8px 22px rgba(20, 184, 166, 0.13)",

    swatch: "#14B8A6",
  },

  "coral-orange": {
    name: "coral-orange",
    label: "珊瑚橙",
    description: "更活泼，适合个性化主题，不建议作为默认。",

    primary: "#FF7A59",
    primaryHover: "#F16847",
    primaryActive: "#DD5A3C",
    primarySoft: "#FFF3EF",
    primaryBorder: "#FFD8CC",
    primaryText: "#FFFFFF",

    appBg: "#FAF7F5",
    windowBg: "#FBF8F6",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#F7F2EF",

    border: "#EEE6E1",
    borderStrong: "#E2D4CC",
    divider: "#F2EAE5",
    focusRing: "rgba(255, 122, 89, 0.18)",

    text: "#1F2937",
    textSecondary: "#667085",
    muted: "#98A2B3",
    placeholder: "#7B8496",
    disabledText: "#B6BEC9",

    success: "#22C55E",
    successSoft: "#ECFDF3",
    warning: "#F59E0B",
    warningSoft: "#FFF7E6",
    danger: "#EF4444",
    dangerSoft: "#FEF2F2",
    info: "#2563EB",
    infoSoft: "#EFF6FF",

    shadowWindow: "0 24px 80px rgba(15, 23, 42, 0.15)",
    shadowPanel: "0 8px 24px rgba(15, 23, 42, 0.04)",
    shadowCardHover: "0 8px 20px rgba(15, 23, 42, 0.055)",
    shadowSelected: "0 0 0 1px #FF7A59, 0 8px 22px rgba(255, 122, 89, 0.13)",

    swatch: "#FF7A59",
  },

  "rose-violet": {
    name: "rose-violet",
    label: "玫紫色",
    description: "柔和、精致，设计感更强。",

    primary: "#E856B6",
    primaryHover: "#D946A7",
    primaryActive: "#C73696",
    primarySoft: "#FFF0FA",
    primaryBorder: "#F7CDE9",
    primaryText: "#FFFFFF",

    appBg: "#FAF6FA",
    windowBg: "#FCF8FC",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#F8F1F7",

    border: "#EEE5EE",
    borderStrong: "#E2D4E2",
    divider: "#F2EAF2",
    focusRing: "rgba(232, 86, 182, 0.17)",

    text: "#1F2937",
    textSecondary: "#667085",
    muted: "#98A2B3",
    placeholder: "#7B8496",
    disabledText: "#B6BEC9",

    success: "#22C55E",
    successSoft: "#ECFDF3",
    warning: "#F59E0B",
    warningSoft: "#FFF7E6",
    danger: "#EF4444",
    dangerSoft: "#FEF2F2",
    info: "#2563EB",
    infoSoft: "#EFF6FF",

    shadowWindow: "0 24px 80px rgba(15, 23, 42, 0.15)",
    shadowPanel: "0 8px 24px rgba(15, 23, 42, 0.04)",
    shadowCardHover: "0 8px 20px rgba(15, 23, 42, 0.055)",
    shadowSelected: "0 0 0 1px #E856B6, 0 8px 22px rgba(232, 86, 182, 0.13)",

    swatch: "#E856B6",
  },
};

export const CLIPLY_THEME_OPTIONS = Object.values(CLIPLY_THEMES);

// 推荐首屏只展示这 4 个，Coral / Rose 可放到“更多主题”里。
export const RECOMMENDED_THEME_NAMES: CliplyThemeName[] = [
  "purple-default",
  "mint-green",
  "lake-blue",
  "teal-fresh",
];

export function isCliplyThemeName(value: unknown): value is CliplyThemeName {
  return typeof value === "string" && value in CLIPLY_THEMES;
}

export function getCliplyTheme(name: CliplyThemeName): CliplyThemeTokens {
  return CLIPLY_THEMES[name] ?? CLIPLY_THEMES[DEFAULT_THEME_NAME];
}

export function getStoredCliplyThemeName(): CliplyThemeName {
  if (typeof window === "undefined") return DEFAULT_THEME_NAME;

  const stored = window.localStorage.getItem(CLIPLY_THEME_STORAGE_KEY);
  return isCliplyThemeName(stored) ? stored : DEFAULT_THEME_NAME;
}

export function storeCliplyThemeName(name: CliplyThemeName): void {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(CLIPLY_THEME_STORAGE_KEY, name);
}

export function resolveInitialCliplyTheme(): CliplyThemeTokens {
  return getCliplyTheme(getStoredCliplyThemeName());
}

export function cssVarsFromCliplyTheme(theme: CliplyThemeTokens): Record<string, string> {
  return {
    "--cliply-primary": theme.primary,
    "--cliply-primary-hover": theme.primaryHover,
    "--cliply-primary-active": theme.primaryActive,
    "--cliply-primary-soft": theme.primarySoft,
    "--cliply-primary-border": theme.primaryBorder,
    "--cliply-primary-text": theme.primaryText,

    "--cliply-app-bg": theme.appBg,
    "--cliply-window-bg": theme.windowBg,
    "--cliply-panel-bg": theme.panelBg,
    "--cliply-card-bg": theme.cardBg,
    "--cliply-input-bg": theme.inputBg,
    "--cliply-muted-bg": theme.mutedBg,

    "--cliply-border": theme.border,
    "--cliply-border-strong": theme.borderStrong,
    "--cliply-divider": theme.divider,
    "--cliply-focus-ring": theme.focusRing,

    "--cliply-text": theme.text,
    "--cliply-text-secondary": theme.textSecondary,
    "--cliply-muted": theme.muted,
    "--cliply-placeholder": theme.placeholder,
    "--cliply-disabled-text": theme.disabledText,

    "--cliply-success": theme.success,
    "--cliply-success-soft": theme.successSoft,
    "--cliply-warning": theme.warning,
    "--cliply-warning-soft": theme.warningSoft,
    "--cliply-danger": theme.danger,
    "--cliply-danger-soft": theme.dangerSoft,
    "--cliply-info": theme.info,
    "--cliply-info-soft": theme.infoSoft,

    "--cliply-shadow-window": theme.shadowWindow,
    "--cliply-shadow-panel": theme.shadowPanel,
    "--cliply-shadow-card-hover": theme.shadowCardHover,
    "--cliply-shadow-selected": theme.shadowSelected,

    // Backward-compatible variables used by the current UI.
    "--cliply-bg": theme.appBg,
    "--cliply-bg-soft": theme.windowBg,
    "--cliply-panel": theme.windowBg,
    "--cliply-panel-strong": theme.panelBg,
    "--cliply-card": theme.cardBg,
    "--cliply-card-solid": theme.cardBg,
    "--cliply-border-soft": theme.divider,
    "--cliply-disabled": theme.disabledText,
    "--cliply-accent": theme.primary,
    "--cliply-accent-strong": theme.primary,
    "--cliply-accent-dark": theme.primaryHover,
    "--cliply-accent-50": theme.primarySoft,
    "--cliply-accent-100": theme.primarySoft,
    "--cliply-accent-soft": theme.primarySoft,
    "--cliply-accent-border": theme.primaryBorder,
    "--cliply-body-text": theme.text,
    "--cliply-faint": theme.textSecondary,
    "--cliply-shadow": theme.shadowWindow,
    "--cliply-shadow-card": theme.shadowPanel,
  };
}

export function applyCliplyTheme(nameOrTheme: CliplyThemeName | CliplyThemeTokens): void {
  if (typeof document === "undefined") return;

  const theme = typeof nameOrTheme === "string" ? getCliplyTheme(nameOrTheme) : nameOrTheme;

  const root = document.documentElement;
  const vars = cssVarsFromCliplyTheme(theme);

  for (const [key, value] of Object.entries(vars)) {
    root.style.setProperty(key, value);
  }

  root.dataset.cliplyTheme = theme.name;
}

export function setCliplyTheme(name: CliplyThemeName): CliplyThemeTokens {
  const theme = getCliplyTheme(name);
  storeCliplyThemeName(name);
  applyCliplyTheme(theme);
  return theme;
}

export function initializeCliplyTheme(): CliplyThemeTokens {
  const theme = resolveInitialCliplyTheme();
  applyCliplyTheme(theme);
  return theme;
}

// 可选：用于设置页预览，不保存，仅临时应用。
export function previewCliplyTheme(name: CliplyThemeName): CliplyThemeTokens {
  const theme = getCliplyTheme(name);
  applyCliplyTheme(theme);
  return theme;
}

// 可选：取消预览，恢复已保存主题。
export function restoreStoredCliplyTheme(): CliplyThemeTokens {
  const theme = resolveInitialCliplyTheme();
  applyCliplyTheme(theme);
  return theme;
}
