// Cliply theme token draft
// 用法：
//   import { applyCliplyTheme, DEFAULT_THEME_NAME } from "@/theme/theme";
//   applyCliplyTheme(DEFAULT_THEME_NAME);

export type CliplyThemeName =
  | "coral-pulse"
  | "system-blue"
  | "lake-blue"
  | "indigo-spark"
  | "purple-default"
  | "magenta-pop"
  | "rose-violet"
  | "coral-orange"
  | "amber-glow"
  | "lime-punch"
  | "mint-green"
  | "teal-fresh";

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
  primaryBorderSelected?: string;
  /// Text drawn on top of a `primary` fill. Vivid accents are often too light
  /// for white, so each theme states what actually passes contrast.
  primaryText: string;
  /// Accent darkened enough to be readable as *text* on `primarySoft`.
  /// Separate from `primaryHover`, which is a hover background — for light
  /// accents the two requirements pull in opposite directions.
  primaryOnSoft?: string;

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
  focusBorder?: string;
  focusRing: string;

  // Text
  text: string;
  bodyText?: string;
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

export type CliplyThemeMode = "light" | "dark" | "system";
export type CliplyResolvedThemeMode = "light" | "dark";
export type CliplyAutoThemeSource = "system-accent" | "wallpaper";
export type CliplyAutoThemeIntensity = "soft" | "normal" | "vivid";
export type CliplyAutoThemeApplyScope = "accent-only" | "full-theme";

export type CliplyAutoThemeSettings = {
  enabled: boolean;
  source: CliplyAutoThemeSource;
  intensity: CliplyAutoThemeIntensity;
  applyScope: CliplyAutoThemeApplyScope;
};

export type CliplyAutoThemeColorSources = {
  systemAccent?: string | null;
};

export const DEFAULT_THEME_NAME: CliplyThemeName = "coral-pulse";

export const CLIPLY_THEME_STORAGE_KEY = "cliply.theme.name";

export const DEFAULT_AUTO_THEME_SETTINGS: CliplyAutoThemeSettings = {
  enabled: false,
  source: "system-accent",
  intensity: "normal",
  applyScope: "accent-only",
};

const AUTO_THEME_FALLBACK_COLORS: Record<CliplyAutoThemeSource, string> = {
  "system-accent": "#FF6257",
  wallpaper: "#22B793",
};

let themeTransitionResetTimer: number | undefined;

export const CLIPLY_THEMES: Record<CliplyThemeName, CliplyThemeTokens> = {
  "coral-pulse": {
    name: "coral-pulse",
    label: "珊瑚红",
    description: "使用珊瑚红作为默认强调色。",

    primary: "#FF6257",
    primaryHover: "#FF7066",
    primaryActive: "#F75A50",
    primarySoft: "#FFEFEE",
    primaryBorder: "#FFD8D5",
    primaryText: "#14161A",
    primaryOnSoft: "#B8473F",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(255, 98, 87, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#FF6257",
    infoSoft: "#FFEFEE",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#FF6257",
  },

  "system-blue": {
    name: "system-blue",
    label: "系统蓝",
    description: "使用 Windows 系统蓝作为强调色。",

    primary: "#2F69FA",
    primaryHover: "#295CDC",
    primaryActive: "#2552C3",
    primarySoft: "#ECF2FF",
    primaryBorder: "#CBDAFE",
    primaryText: "#FFFFFF",
    primaryOnSoft: "#2B61E6",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(47, 105, 250, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#2F69FA",
    infoSoft: "#ECF2FF",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#2F69FA",
  },

  "lake-blue": {
    name: "lake-blue",
    label: "深蓝",
    description: "使用深蓝色作为强调色。",

    primary: "#1D5FD6",
    primaryHover: "#1A54BC",
    primaryActive: "#174AA7",
    primarySoft: "#EDF2FC",
    primaryBorder: "#C7D7F5",
    primaryText: "#FFFFFF",
    primaryOnSoft: "#1D5FD6",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(29, 95, 214, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#1D5FD6",
    infoSoft: "#EDF2FC",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#1D5FD6",
  },

  "indigo-spark": {
    name: "indigo-spark",
    label: "靛蓝",
    description: "使用靛蓝色作为强调色。",

    primary: "#4F46E5",
    primaryHover: "#463ECA",
    primaryActive: "#3E37B3",
    primarySoft: "#F1F0FD",
    primaryBorder: "#D3D1F9",
    primaryText: "#FFFFFF",
    primaryOnSoft: "#4F46E5",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(79, 70, 229, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#4F46E5",
    infoSoft: "#F1F0FD",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#4F46E5",
  },

  "purple-default": {
    name: "purple-default",
    label: "紫色",
    description: "使用紫色作为强调色。",

    primary: "#6D4CFF",
    primaryHover: "#6043E0",
    primaryActive: "#553BC7",
    primarySoft: "#F2EFFF",
    primaryBorder: "#DBD2FF",
    primaryText: "#FFFFFF",
    primaryOnSoft: "#6B4AFA",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(109, 76, 255, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#6D4CFF",
    infoSoft: "#F2EFFF",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#6D4CFF",
  },

  "magenta-pop": {
    name: "magenta-pop",
    label: "洋红",
    description: "使用洋红色作为强调色。",

    primary: "#D6218C",
    primaryHover: "#BC1D7B",
    primaryActive: "#A71A6D",
    primarySoft: "#FCEDF6",
    primaryBorder: "#F5C8E2",
    primaryText: "#FFFFFF",
    primaryOnSoft: "#C91F84",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(214, 33, 140, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#D6218C",
    infoSoft: "#FCEDF6",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#D6218C",
  },

  "rose-violet": {
    name: "rose-violet",
    label: "玫红",
    description: "使用玫红色作为强调色。",

    primary: "#DB2777",
    primaryHover: "#C12269",
    primaryActive: "#AB1E5D",
    primarySoft: "#FCEEF4",
    primaryBorder: "#F6C9DD",
    primaryText: "#FFFFFF",
    primaryOnSoft: "#C9246D",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(219, 39, 119, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#DB2777",
    infoSoft: "#FCEEF4",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#DB2777",
  },

  "coral-orange": {
    name: "coral-orange",
    label: "橙红",
    description: "使用橙红色作为强调色。",

    primary: "#E8552D",
    primaryHover: "#EA633E",
    primaryActive: "#DF522B",
    primarySoft: "#FDEEEA",
    primaryBorder: "#F9D5CB",
    primaryText: "#14161A",
    primaryOnSoft: "#BA4424",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(232, 85, 45, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#E8552D",
    infoSoft: "#FDEEEA",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#E8552D",
  },

  "amber-glow": {
    name: "amber-glow",
    label: "琥珀色",
    description: "使用琥珀色作为强调色。",

    primary: "#C2820A",
    primaryHover: "#C78C1E",
    primaryActive: "#BA7D0A",
    primarySoft: "#F9F3E7",
    primaryBorder: "#F0E0C2",
    primaryText: "#14161A",
    primaryOnSoft: "#936308",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(194, 130, 10, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#C2820A",
    infoSoft: "#F9F3E7",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#C2820A",
  },

  "lime-punch": {
    name: "lime-punch",
    label: "青柠绿",
    description: "使用青柠绿色作为强调色。",

    primary: "#4E9F0D",
    primaryHover: "#5CA720",
    primaryActive: "#4B990C",
    primarySoft: "#EDF5E7",
    primaryBorder: "#D3E7C3",
    primaryText: "#14161A",
    primaryOnSoft: "#3D7C0A",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(78, 159, 13, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#4E9F0D",
    infoSoft: "#EDF5E7",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#4E9F0D",
  },

  "mint-green": {
    name: "mint-green",
    label: "薄荷绿",
    description: "使用薄荷绿色作为强调色。",

    primary: "#1BA36B",
    primaryHover: "#2DAA77",
    primaryActive: "#1A9C67",
    primarySoft: "#E8F6F0",
    primaryBorder: "#C6E8DA",
    primaryText: "#14161A",
    primaryOnSoft: "#157C51",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(27, 163, 107, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#1BA36B",
    infoSoft: "#E8F6F0",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#1BA36B",
  },

  "teal-fresh": {
    name: "teal-fresh",
    label: "青绿色",
    description: "使用青绿色作为强调色。",

    primary: "#0D9488",
    primaryHover: "#209D92",
    primaryActive: "#0C8E83",
    primarySoft: "#E7F4F3",
    primaryBorder: "#C3E4E1",
    primaryText: "#14161A",
    primaryOnSoft: "#0B7970",

    appBg: "#F5F9FD",
    windowBg: "#F5F9FD",
    panelBg: "#FFFFFF",
    cardBg: "#FFFFFF",
    inputBg: "#FFFFFF",
    mutedBg: "#EFF5FA",

    border: "#E2EAF2",
    borderStrong: "#CEDBE8",
    divider: "#EDF3F8",
    focusRing: "rgba(13, 148, 136, 0.15)",

    text: "#1B2734",
    bodyText: "#2C3A49",
    textSecondary: "#5F6F80",
    muted: "#64748B",
    placeholder: "#7C8B9C",
    disabledText: "#B9C5D1",

    success: "#168F73",
    successSoft: "#E8F7F2",
    warning: "#B45309",
    warningSoft: "#FFF7E6",
    danger: "#DC2626",
    dangerSoft: "#FEF2F2",
    info: "#0D9488",
    infoSoft: "#E7F4F3",

    shadowWindow:
      "0 12px 32px rgba(27, 39, 52, 0.14), 0 2px 6px rgba(27, 39, 52, 0.07)",
    shadowPanel: "0 1px 2px rgba(27, 39, 52, 0.04)",
    shadowCardHover: "0 1px 2px rgba(27, 39, 52, 0.06)",
    shadowSelected: "none",

    swatch: "#0D9488",
  },
};

export const CLIPLY_THEME_OPTIONS = Object.values(CLIPLY_THEMES);

// 推荐首屏展示这几个，其余在“更多主题”里。
export const RECOMMENDED_THEME_NAMES: CliplyThemeName[] = [
  "coral-pulse",
  "mint-green",
  "lake-blue",
  "magenta-pop",
];

export function isCliplyThemeName(value: unknown): value is CliplyThemeName {
  return typeof value === "string" && value in CLIPLY_THEMES;
}

export function getCliplyTheme(name: CliplyThemeName): CliplyThemeTokens {
  return CLIPLY_THEMES[name] ?? CLIPLY_THEMES[DEFAULT_THEME_NAME];
}

export function getCliplyThemeWithAccent(
  name: CliplyThemeName,
  accentColor?: string | null,
  mode: CliplyResolvedThemeMode = "light",
): CliplyThemeTokens {
  const theme = getCliplyThemeForMode(name, mode);
  const accent = normalizeHexColor(accentColor);

  if (!accent || accent.toLowerCase() === theme.primary.toLowerCase()) {
    return theme;
  }

  const rgb = hexToRgb(accent);
  if (!rgb) {
    return theme;
  }

  if (mode === "dark") {
    return withDarkAccent(theme, accent);
  }

  return {
    ...theme,
    primary: accent,
    primaryHover: mixHex(accent, "#000000", 0.12),
    primaryActive: mixHex(accent, "#000000", 0.22),
    primarySoft: mixHex(accent, "#FFFFFF", 0.9),
    primaryBorder: mixHex(accent, "#FFFFFF", 0.72),
    primaryText: readableTextForAccent(accent),
    primaryOnSoft: readableAccentOnLightSurface(accent),
    focusRing: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.18)`,
    shadowSelected: `0 0 0 1px ${accent}, 0 8px 22px rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.13)`,
    swatch: accent,
  };
}

export function getAutoThemeColor(
  autoTheme?: Partial<CliplyAutoThemeSettings> | null,
  colorSources?: CliplyAutoThemeColorSources | null,
): string {
  const settings = normalizeAutoThemeSettings(autoTheme);
  const sourceColor = readAutoThemeSourceColor(settings.source, colorSources);
  return adjustAutoThemeColor(sourceColor, settings.intensity);
}

export function getCliplyThemeWithAutoTheme(
  name: CliplyThemeName,
  autoTheme?: Partial<CliplyAutoThemeSettings> | null,
  colorSources?: CliplyAutoThemeColorSources | null,
  mode: CliplyResolvedThemeMode = "light",
): CliplyThemeTokens {
  const settings = normalizeAutoThemeSettings(autoTheme);
  if (!settings.enabled) {
    return getCliplyThemeForMode(name, mode);
  }

  const accent = getAutoThemeColor(settings, colorSources);
  if (settings.applyScope === "accent-only") {
    return getCliplyThemeWithAccent(name, accent, mode);
  }

  const accentTheme = getCliplyThemeWithAccent(name, accent, mode);
  if (mode === "dark") {
    return {
      ...accentTheme,
      appBg: mixHex(accent, "#0B1120", 0.94),
      windowBg: mixHex(accent, "#0F172A", 0.95),
      panelBg: mixHex(accent, "#111C2E", 0.95),
      cardBg: mixHex(accent, "#152238", 0.94),
      mutedBg: mixHex(accent, "#111D31", 0.9),
      border: mixHex(accent, "#334155", 0.86),
      borderStrong: mixHex(accent, "#475569", 0.8),
      divider: mixHex(accent, "#1E293B", 0.88),
    };
  }

  return {
    ...accentTheme,
    appBg: mixHex(accent, "#FFFFFF", 0.96),
    windowBg: mixHex(accent, "#FFFFFF", 0.975),
    mutedBg: mixHex(accent, "#FFFFFF", 0.93),
    border: mixHex(accent, "#E7EAF1", 0.88),
    borderStrong: mixHex(accent, "#D8DEE8", 0.82),
    divider: mixHex(accent, "#EEF1F5", 0.9),
  };
}

export function resolveCliplyThemeFromSettings(settings: {
  theme?: string | null;
  themeName?: string | null;
  accentColor?: string | null;
  autoTheme?: Partial<CliplyAutoThemeSettings> | null;
  autoThemeColorSources?: CliplyAutoThemeColorSources | null;
  systemPrefersDark?: boolean | null;
}): CliplyThemeTokens {
  const themeName = isCliplyThemeName(settings.themeName)
    ? settings.themeName
    : DEFAULT_THEME_NAME;
  const autoTheme = normalizeAutoThemeSettings(settings.autoTheme);
  const mode = resolveThemeMode(settings.theme, settings.systemPrefersDark);

  if (autoTheme.enabled) {
    return getCliplyThemeWithAutoTheme(
      themeName,
      autoTheme,
      settings.autoThemeColorSources,
      mode,
    );
  }

  return getCliplyThemeWithAccent(themeName, settings.accentColor, mode);
}

export function normalizeAutoThemeSettings(
  value?: Partial<CliplyAutoThemeSettings> | null,
): CliplyAutoThemeSettings {
  return {
    enabled: Boolean(value?.enabled),
    source:
      value?.source === "system-accent" || value?.source === "wallpaper"
        ? value.source
        : DEFAULT_AUTO_THEME_SETTINGS.source,
    intensity:
      value?.intensity === "soft" ||
      value?.intensity === "normal" ||
      value?.intensity === "vivid"
        ? value.intensity
        : DEFAULT_AUTO_THEME_SETTINGS.intensity,
    applyScope:
      value?.applyScope === "full-theme" || value?.applyScope === "accent-only"
        ? value.applyScope
        : DEFAULT_AUTO_THEME_SETTINGS.applyScope,
  };
}

export function resolveThemeMode(
  mode?: string | null,
  systemPrefersDark?: boolean | null,
): CliplyResolvedThemeMode {
  if (mode === "dark") {
    return "dark";
  }
  if (mode === "system") {
    return systemPrefersDark ? "dark" : "light";
  }
  return "light";
}

export function getCliplyThemeForMode(
  name: CliplyThemeName,
  mode: CliplyResolvedThemeMode = "light",
): CliplyThemeTokens {
  const theme = getCliplyTheme(name);
  return mode === "dark" ? createDarkThemeTokens(theme) : theme;
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
    "--cliply-primary-border-selected": theme.primaryBorderSelected ?? theme.primaryBorder,
    "--cliply-primary-text": theme.primaryText,
    "--cliply-primary-action-text": theme.primaryText,
    "--cliply-accent-on-soft": theme.primaryOnSoft ?? theme.primaryHover,

    "--cliply-app-bg": theme.appBg,
    "--cliply-window-bg": theme.windowBg,
    "--cliply-panel-bg": theme.panelBg,
    "--cliply-card-bg": theme.cardBg,
    "--cliply-input-bg": theme.inputBg,
    "--cliply-muted-bg": theme.mutedBg,

    "--cliply-border": theme.border,
    "--cliply-border-strong": theme.borderStrong,
    "--cliply-divider": theme.divider,
    "--cliply-focus-border": theme.focusBorder ?? theme.primary,
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
    "--cliply-accent-border-selected": theme.primaryBorderSelected ?? theme.primaryBorder,
    "--cliply-body-text": theme.bodyText ?? theme.text,
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
  suppressThemeTransitions(root);

  for (const [key, value] of Object.entries(vars)) {
    root.style.setProperty(key, value);
  }

  root.dataset.cliplyTheme = theme.name;
  root.dataset.theme = isDarkTheme(theme) ? "dark" : "light";
  root.style.colorScheme = isDarkTheme(theme) ? "dark" : "light";
}

function suppressThemeTransitions(root: HTMLElement) {
  if (typeof window === "undefined") {
    return;
  }

  root.classList.add("cliply-theme-applying");
  if (themeTransitionResetTimer !== undefined) {
    window.clearTimeout(themeTransitionResetTimer);
  }

  themeTransitionResetTimer = window.setTimeout(() => {
    root.classList.remove("cliply-theme-applying");
    themeTransitionResetTimer = undefined;
  }, 120);
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

function normalizeHexColor(value?: string | null) {
  if (!value) {
    return null;
  }

  const trimmed = value.trim();
  const shortMatch = /^#([0-9a-f]{3})$/i.exec(trimmed);
  if (shortMatch) {
    const [r, g, b] = shortMatch[1].split("");
    return `#${r}${r}${g}${g}${b}${b}`.toUpperCase();
  }

  return /^#[0-9a-f]{6}$/i.test(trimmed) ? trimmed.toUpperCase() : null;
}

function readAutoThemeSourceColor(
  source: CliplyAutoThemeSource,
  colorSources?: CliplyAutoThemeColorSources | null,
) {
  const sourceColor =
    source === "system-accent" ? normalizeHexColor(colorSources?.systemAccent) : null;
  if (sourceColor) {
    return sourceColor;
  }

  if (typeof window !== "undefined") {
    const key = "cliply.autoTheme.mockSystemAccentColor";
    const mockColor = normalizeHexColor(window.localStorage.getItem(key));
    if (mockColor) {
      return mockColor;
    }
  }

  return AUTO_THEME_FALLBACK_COLORS[source];
}

function createDarkThemeTokens(theme: CliplyThemeTokens): CliplyThemeTokens {
  const darkAccent = normalizeDefaultDarkAccent(theme.primary);
  const rgb = hexToRgb(darkAccent) ?? { r: 124, g: 92, b: 255 };
  const isDefaultPurple = darkAccent === "#7C5CFF";
  const isBrandCoral = theme.name === "coral-pulse";

  return {
    ...theme,
    primary: darkAccent,
    primaryHover: isBrandCoral
      ? "#FF7066"
      : isDefaultPurple
        ? "#8B6DFF"
        : mixHex(darkAccent, "#FFFFFF", 0.08),
    primaryActive: isBrandCoral
      ? "#F75A50"
      : isDefaultPurple
        ? "#6D4CFF"
        : mixHex(darkAccent, "#000000", 0.12),
    primarySoft: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.16)`,
    primaryBorder: isDefaultPurple
      ? "rgba(167, 139, 250, 0.55)"
      : `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.55)`,
    primaryBorderSelected: isDefaultPurple
      ? "rgba(167, 139, 250, 0.75)"
      : `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.72)`,
    // Dark accents are light, so filled controls take dark text (Fluent).
    primaryText: "#10151A",
    primaryOnSoft: darkAccent,
    swatch: darkAccent,
    appBg: "#141719",
    windowBg: "#17191C",
    panelBg: "#1E2125",
    cardBg: "#1E2125",
    inputBg: "#1A1D21",
    mutedBg: "#25292E",
    border: "rgba(255, 255, 255, 0.09)",
    borderStrong: "rgba(255, 255, 255, 0.16)",
    divider: "rgba(255, 255, 255, 0.055)",
    text: "#F2F4F6",
    bodyText: "#DFE3E8",
    textSecondary: "#9BA5B1",
    muted: "#79838F",
    placeholder: "#79838F",
    disabledText: "#5D666F",
    focusBorder: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.6)`,
    focusRing: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.16)`,
    success: "#3DD0AC",
    successSoft: "rgba(61, 208, 172, 0.14)",
    warningSoft: "rgba(245, 158, 11, 0.16)",
    dangerSoft: "rgba(239, 68, 68, 0.16)",
    infoSoft: "rgba(37, 99, 235, 0.18)",
    shadowWindow:
      "0 12px 32px rgba(0, 0, 0, 0.5), 0 2px 6px rgba(0, 0, 0, 0.3)",
    shadowPanel: "0 1px 2px rgba(0, 0, 0, 0.28)",
    shadowCardHover: "0 1px 2px rgba(0, 0, 0, 0.34)",
    shadowSelected: "none",
  };
}

function withDarkAccent(theme: CliplyThemeTokens, accent: string): CliplyThemeTokens {
  const darkAccent = normalizeDefaultDarkAccent(accent);
  const rgb = hexToRgb(darkAccent);
  if (!rgb) {
    return theme;
  }

  const isDefaultPurple = darkAccent === "#7C5CFF";
  return {
    ...theme,
    primary: darkAccent,
    primaryHover: isDefaultPurple ? "#8B6DFF" : mixHex(darkAccent, "#FFFFFF", 0.08),
    primaryActive: isDefaultPurple ? "#6D4CFF" : mixHex(darkAccent, "#000000", 0.12),
    primarySoft: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.16)`,
    primaryBorder: isDefaultPurple
      ? "rgba(167, 139, 250, 0.55)"
      : `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.55)`,
    primaryBorderSelected: isDefaultPurple
      ? "rgba(167, 139, 250, 0.75)"
      : `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.72)`,
    primaryText: readableTextForAccent(darkAccent),
    focusBorder: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.65)`,
    focusRing: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.18)`,
    shadowSelected: `0 0 0 1px rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.28)`,
    swatch: darkAccent,
  };
}

/// True WCAG relative luminance (gamma-corrected), unlike the cheap weighted
/// average used for rough accent-tone checks.
function wcagLuminance(rgb: { r: number; g: number; b: number }) {
  const channel = (value: number) => {
    const c = value / 255;
    return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  };
  return (
    0.2126 * channel(rgb.r) + 0.7152 * channel(rgb.g) + 0.0722 * channel(rgb.b)
  );
}

function contrastRatio(a: string, b: string) {
  const rgbA = hexToRgb(a);
  const rgbB = hexToRgb(b);
  if (!rgbA || !rgbB) {
    return 1;
  }
  const lumA = wcagLuminance(rgbA);
  const lumB = wcagLuminance(rgbB);
  return (Math.max(lumA, lumB) + 0.05) / (Math.min(lumA, lumB) + 0.05);
}

// Reference surface the dark-mode accent has to stay legible against: the dark
// card colour, warmed slightly to stand in for the translucent accent wash.
const DARK_SURFACE_REFERENCE = "#262A30";

function normalizeDefaultDarkAccent(accent: string) {
  const normalized = normalizeHexColor(accent);
  if (!normalized) {
    return "#5FA8EE";
  }

  // On dark surfaces the accent is both a filled-button background (with dark
  // text) and an on-surface text colour, so lighten any hue until it clears
  // both. Hardcoding a couple of hues left every other theme broken in dark.
  if (contrastRatio(normalized, DARK_SURFACE_REFERENCE) >= 4.8) {
    return normalized;
  }
  for (let amount = 0.04; amount <= 0.9; amount += 0.04) {
    const candidate = mixHex(normalized, "#FFFFFF", amount);
    if (contrastRatio(candidate, DARK_SURFACE_REFERENCE) >= 4.8) {
      return candidate;
    }
  }
  return mixHex(normalized, "#FFFFFF", 0.9);
}

function isDarkTheme(theme: CliplyThemeTokens) {
  const rgb = hexToRgb(theme.windowBg);
  if (!rgb) {
    return false;
  }
  return relativeLuminance(rgb) < 0.24;
}

function adjustAutoThemeColor(
  color: string,
  intensity: CliplyAutoThemeIntensity,
) {
  const safeColor = clampAccentLuminance(color);
  if (intensity === "soft") {
    return mixHex(safeColor, "#FFFFFF", 0.18);
  }
  if (intensity === "vivid") {
    return saturateHex(mixHex(safeColor, "#000000", 0.04), 0.18);
  }
  return safeColor;
}

function clampAccentLuminance(hex: string) {
  const rgb = hexToRgb(hex);
  if (!rgb) {
    return AUTO_THEME_FALLBACK_COLORS["system-accent"];
  }

  const luminance = relativeLuminance(rgb);
  if (luminance < 0.18) {
    return mixHex(hex, "#FFFFFF", 0.26);
  }
  if (luminance > 0.78) {
    return mixHex(hex, "#000000", 0.28);
  }
  return normalizeHexColor(hex) ?? AUTO_THEME_FALLBACK_COLORS["system-accent"];
}

function saturateHex(hex: string, amount: number) {
  const rgb = hexToRgb(hex);
  if (!rgb) {
    return hex;
  }

  const average = (rgb.r + rgb.g + rgb.b) / 3;
  const channel = (value: number) =>
    clampChannel(Math.round(value + (value - average) * amount))
      .toString(16)
      .padStart(2, "0");

  return `#${channel(rgb.r)}${channel(rgb.g)}${channel(rgb.b)}`.toUpperCase();
}

function hexToRgb(hex: string) {
  const normalized = normalizeHexColor(hex);
  if (!normalized) {
    return null;
  }

  return {
    r: Number.parseInt(normalized.slice(1, 3), 16),
    g: Number.parseInt(normalized.slice(3, 5), 16),
    b: Number.parseInt(normalized.slice(5, 7), 16),
  };
}

function mixHex(from: string, to: string, amount: number) {
  const fromRgb = hexToRgb(from);
  const toRgb = hexToRgb(to);
  if (!fromRgb || !toRgb) {
    return from;
  }

  const mixChannel = (start: number, end: number) =>
    Math.round(start + (end - start) * amount)
      .toString(16)
      .padStart(2, "0");

  return `#${mixChannel(fromRgb.r, toRgb.r)}${mixChannel(fromRgb.g, toRgb.g)}${mixChannel(
    fromRgb.b,
    toRgb.b,
  )}`.toUpperCase();
}

function readableTextForAccent(accent: string) {
  const lightContrast = contrastRatio(accent, "#FFFFFF");
  const darkContrast = contrastRatio(accent, "#14161A");
  return lightContrast >= darkContrast ? "#FFFFFF" : "#14161A";
}

function readableAccentOnLightSurface(accent: string) {
  if (contrastRatio(accent, "#FFFFFF") >= 4.5) {
    return accent;
  }

  for (let amount = 0.04; amount <= 0.8; amount += 0.04) {
    const candidate = mixHex(accent, "#000000", amount);
    if (contrastRatio(candidate, "#FFFFFF") >= 4.5) {
      return candidate;
    }
  }

  return mixHex(accent, "#000000", 0.8);
}

function relativeLuminance(rgb: { r: number; g: number; b: number }) {
  return (0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b) / 255;
}

function clampChannel(value: number) {
  return Math.min(255, Math.max(0, value));
}
