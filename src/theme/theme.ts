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
  primaryBorderSelected?: string;
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

export const DEFAULT_THEME_NAME: CliplyThemeName = "purple-default";

export const CLIPLY_THEME_STORAGE_KEY = "cliply.theme.name";

export const DEFAULT_AUTO_THEME_SETTINGS: CliplyAutoThemeSettings = {
  enabled: false,
  source: "system-accent",
  intensity: "normal",
  applyScope: "accent-only",
};

const AUTO_THEME_FALLBACK_COLORS: Record<CliplyAutoThemeSource, string> = {
  "system-accent": "#6D4CFF",
  wallpaper: "#3B82F6",
};

let themeTransitionResetTimer: number | undefined;

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
    description: "比绿色更科技、更稳，适合安全和工具场景。",

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
    primaryText: readableTextForColor(rgb),
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
    primaryText: "#FFFFFF",
    swatch: darkAccent,
    appBg: "#0B1120",
    windowBg: "#0F172A",
    panelBg: "#111C2E",
    cardBg: "#152238",
    inputBg: "#101A2D",
    mutedBg: "#111D31",
    border: "rgba(148, 163, 184, 0.18)",
    borderStrong: "rgba(148, 163, 184, 0.28)",
    divider: "rgba(148, 163, 184, 0.12)",
    text: "#F8FAFC",
    bodyText: "#E5E7EB",
    textSecondary: "#CBD5E1",
    muted: "#94A3B8",
    placeholder: "#64748B",
    disabledText: "#64748B",
    focusBorder: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.65)`,
    focusRing: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.18)`,
    successSoft: "rgba(34, 197, 94, 0.14)",
    warningSoft: "rgba(245, 158, 11, 0.16)",
    dangerSoft: "rgba(239, 68, 68, 0.16)",
    infoSoft: "rgba(37, 99, 235, 0.18)",
    shadowWindow: "0 24px 80px rgba(0, 0, 0, 0.48)",
    shadowPanel: "0 14px 36px rgba(0, 0, 0, 0.28)",
    shadowCardHover: "0 12px 28px rgba(0, 0, 0, 0.32)",
    shadowSelected: `0 0 0 1px rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.28)`,
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
    primaryText: "#FFFFFF",
    focusBorder: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.65)`,
    focusRing: `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.18)`,
    shadowSelected: `0 0 0 1px rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, 0.28)`,
    swatch: darkAccent,
  };
}

function normalizeDefaultDarkAccent(accent: string) {
  const normalized = normalizeHexColor(accent);
  if (!normalized) {
    return "#7C5CFF";
  }
  return normalized === "#6D4CFF" ? "#7C5CFF" : normalized;
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

function readableTextForColor(rgb: { r: number; g: number; b: number }) {
  const luminance = relativeLuminance(rgb);
  return luminance > 0.72 ? "#1F2937" : "#FFFFFF";
}

function relativeLuminance(rgb: { r: number; g: number; b: number }) {
  return (0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b) / 255;
}

function clampChannel(value: number) {
  return Math.min(255, Math.max(0, value));
}
