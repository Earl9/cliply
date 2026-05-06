import {
  useEffect,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type ReactNode,
} from "react";
import { BellOff, History, Keyboard, Shield, Sparkles, X } from "lucide-react";
import { clsx } from "clsx";
import { Badge } from "@/components/common/Badge";
import { IconButton } from "@/components/common/IconButton";
import {
  checkGlobalShortcut,
  type ShortcutCheck,
} from "@/lib/settingsRepository";
import {
  CLIPLY_THEME_OPTIONS,
  DEFAULT_THEME_NAME,
  applyCliplyTheme,
  getCliplyTheme,
  isCliplyThemeName,
  type CliplyThemeName,
} from "@/theme/theme";
import type { CliplySettings } from "@/stores/settingsStore";

type SettingsDialogProps = {
  open: boolean;
  settings: CliplySettings;
  onClose: () => void;
  onSave: (settings: CliplySettings) => void;
  onClearHistory: () => void;
};

export function SettingsDialog({
  open,
  settings,
  onClose,
  onSave,
  onClearHistory,
}: SettingsDialogProps) {
  const [draft, setDraft] = useState(settings);
  const [capturingShortcut, setCapturingShortcut] = useState(false);
  const [shortcutCheck, setShortcutCheck] = useState<ShortcutCheck | null>(null);
  const ignoreAppsText = draft.ignoreApps.join("\n");

  useEffect(() => {
    if (open) {
      setDraft(settings);
      setCapturingShortcut(false);
      setShortcutCheck(null);
    }
  }, [open, settings]);

  useEffect(() => {
    if (open) {
      applyCliplyTheme(getDraftThemeName(draft.themeName));
    }
  }, [draft.themeName, open]);

  useEffect(() => {
    if (!open) {
      return;
    }

    let cancelled = false;
    const timeout = window.setTimeout(() => {
      void checkGlobalShortcut(draft.globalShortcut, settings.globalShortcut)
        .then((result) => {
          if (cancelled) {
            return;
          }

          setShortcutCheck(result);
          if (result.ok && result.display && result.display !== draft.globalShortcut) {
            setDraft((current) => ({
              ...current,
              globalShortcut: result.display,
            }));
          }
        })
        .catch((error) => {
          if (cancelled) {
            return;
          }

          setShortcutCheck({
            ok: false,
            normalized: "",
            display: draft.globalShortcut,
            reason: "system-conflict",
            message: error instanceof Error ? error.message : "快捷键检测失败",
          });
        });
    }, 220);

    return () => {
      cancelled = true;
      window.clearTimeout(timeout);
    };
  }, [draft.globalShortcut, open, settings.globalShortcut]);

  if (!open) {
    return null;
  }

  const updateDraft = <K extends keyof CliplySettings>(key: K, value: CliplySettings[K]) => {
    setDraft((current) => ({ ...current, [key]: value }));
  };

  const cancelSettings = () => {
    applyCliplyTheme(getDraftThemeName(settings.themeName));
    setCapturingShortcut(false);
    onClose();
  };

  const saveDisabled = !shortcutCheck?.ok;

  return (
    <div className="absolute inset-0 z-30 grid place-items-center bg-slate-900/18 px-6 backdrop-blur-sm">
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="cliply-settings-title"
        className="flex max-h-[calc(100%-40px)] w-full max-w-[760px] flex-col overflow-hidden rounded-xl border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] shadow-2xl"
      >
        <header className="flex h-12 shrink-0 items-center justify-between border-b border-[color:var(--cliply-border)] px-4">
          <div>
            <h2 id="cliply-settings-title" className="text-[15px] font-semibold text-[color:var(--cliply-text)]">
              设置
            </h2>
            <p className="mt-0.5 text-xs font-medium text-[color:var(--cliply-muted)]">
              本地优先，Windows MVP
            </p>
          </div>
          <IconButton label="关闭设置" onClick={cancelSettings}>
            <X className="size-4" />
          </IconButton>
        </header>

        <div className="cliply-scrollbar min-h-0 flex-1 overflow-auto p-4">
          <div className="grid gap-3 md:grid-cols-2">
            <SettingSection icon={Keyboard} title="快捷键">
              <ShortcutRecorder
                value={draft.globalShortcut}
                check={shortcutCheck}
                capturing={capturingShortcut}
                onStartCapture={() => setCapturingShortcut(true)}
                onStopCapture={() => setCapturingShortcut(false)}
                onChange={(value) => updateDraft("globalShortcut", value)}
              />
              <div className="grid grid-cols-2 gap-2 text-xs font-medium text-[color:var(--cliply-muted)]">
                <ShortcutRow label="粘贴" value="Enter" />
                <ShortcutRow label="无格式" value="Shift + Enter" />
                <ShortcutRow label="固定" value="Ctrl + P" />
                <ShortcutRow label="删除" value="Delete" />
              </div>
            </SettingSection>

            <SettingSection icon={BellOff} title="通用">
              <ToggleRow
                label="暂停监听"
                checked={draft.pauseMonitoring}
                onChange={(value) => updateDraft("pauseMonitoring", value)}
              />
              <ToggleRow
                label="开机自启"
                checked={draft.launchAtStartup}
                onChange={(value) => updateDraft("launchAtStartup", value)}
              />
              <ToggleRow
                label="启动时最小化到托盘"
                checked={draft.startMinimized}
                onChange={(value) => updateDraft("startMinimized", value)}
              />
              <ToggleRow
                label="打开后自动聚焦搜索框"
                checked={draft.focusSearchOnOpen}
                onChange={(value) => updateDraft("focusSearchOnOpen", value)}
              />
              <ToggleRow
                label="粘贴后自动关闭窗口"
                checked={draft.closeAfterPaste}
                onChange={(value) => updateDraft("closeAfterPaste", value)}
              />
            </SettingSection>

            <SettingSection icon={Shield} title="隐私">
              <p className="rounded-lg bg-amber-50 px-3 py-2 text-xs leading-5 text-amber-800">
                默认不保存私钥、API token、银行卡号等高风险内容；疑似验证码只保存隐藏占位。
              </p>
              <ToggleRow
                label="启用敏感内容过滤"
                checked={!draft.saveSensitive}
                onChange={(value) => updateDraft("saveSensitive", !value)}
              />
              <ToggleRow
                label="保存图片"
                checked={draft.saveImages}
                onChange={(value) => updateDraft("saveImages", value)}
              />
              <ToggleRow
                label="保存 HTML 富文本"
                checked={draft.saveHtml}
                onChange={(value) => updateDraft("saveHtml", value)}
              />
              <label className="grid gap-2 text-sm font-medium text-[color:var(--cliply-muted)]">
                忽略应用列表
                <textarea
                  value={ignoreAppsText}
                  onChange={(event) =>
                    updateDraft(
                      "ignoreApps",
                      event.target.value
                        .split("\n")
                        .map((value) => value.trim())
                        .filter(Boolean),
                    )
                  }
                  rows={4}
                  className="cliply-scrollbar resize-none rounded-xl border border-[color:var(--cliply-border)] bg-white px-3 py-2 text-sm text-[color:var(--cliply-text)] outline-none focus:border-[color:var(--cliply-accent)]"
                />
              </label>
            </SettingSection>

            <SettingSection icon={History} title="历史记录">
              <NumberRow
                label="最大历史条数"
                value={draft.maxHistoryItems}
                min={50}
                max={10000}
                onChange={(value) => updateDraft("maxHistoryItems", value)}
              />
              <NumberRow
                label="自动清理天数"
                value={draft.autoDeleteDays}
                min={1}
                max={365}
                onChange={(value) => updateDraft("autoDeleteDays", value)}
              />
              <ToggleRow
                label="忽略重复内容"
                checked={draft.ignoreDuplicate}
                onChange={(value) => updateDraft("ignoreDuplicate", value)}
              />
              <button
                type="button"
                onClick={onClearHistory}
                className="h-10 rounded-xl border border-rose-200 bg-rose-50 px-3 text-left text-sm font-semibold text-rose-700 transition hover:bg-rose-100"
              >
                清空未固定历史
              </button>
            </SettingSection>

            <SettingSection icon={Sparkles} title="外观">
              <div className="flex items-center justify-between gap-3">
                <span className="text-sm font-medium text-[color:var(--cliply-muted)]">主题</span>
                <Badge tone="accent">{getCliplyTheme(getDraftThemeName(draft.themeName)).label}</Badge>
              </div>
              <ThemePicker
                value={getDraftThemeName(draft.themeName)}
                onChange={(value) => {
                  const theme = getCliplyTheme(value);
                  updateDraft("themeName", value);
                  updateDraft("accentColor", theme.primary);
                }}
              />
            </SettingSection>
          </div>
        </div>

        <footer className="flex h-12 shrink-0 items-center justify-end gap-2 border-t border-[color:var(--cliply-border)] px-4">
          <button
            type="button"
            onClick={cancelSettings}
            className="h-8 rounded-lg border border-[color:var(--cliply-border)] bg-white px-3 text-[13px] font-semibold text-[color:var(--cliply-text)] transition hover:bg-[#fafafb]"
          >
            取消
          </button>
          <button
            type="button"
            disabled={saveDisabled}
            onClick={() => {
              setCapturingShortcut(false);
              onSave(draft);
            }}
            className="h-8 rounded-lg bg-[color:var(--cliply-accent-strong)] px-3 text-[13px] font-semibold text-white transition hover:bg-[color:var(--cliply-accent-dark)] disabled:cursor-not-allowed disabled:bg-[#d8dee8] disabled:text-[#7b8496]"
          >
            保存设置
          </button>
        </footer>
      </section>
    </div>
  );
}

type SectionIcon = typeof Keyboard;

function SettingSection({
  icon: Icon,
  title,
  children,
}: {
  icon: SectionIcon;
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="rounded-lg border border-[color:var(--cliply-border)] bg-white/72 p-3">
      <div className="mb-3 flex items-center gap-2 text-[13px] font-semibold text-[color:var(--cliply-text)]">
        <Icon className="size-4 text-[color:var(--cliply-accent)]" />
        {title}
      </div>
      <div className="grid gap-2.5">{children}</div>
    </section>
  );
}

function ShortcutRecorder({
  value,
  check,
  capturing,
  onStartCapture,
  onStopCapture,
  onChange,
}: {
  value: string;
  check: ShortcutCheck | null;
  capturing: boolean;
  onStartCapture: () => void;
  onStopCapture: () => void;
  onChange: (value: string) => void;
}) {
  const statusTone = check?.ok ? "ok" : check ? "error" : "pending";
  const statusLabel = capturing
    ? "录制中"
    : statusTone === "ok"
      ? "可用"
      : statusTone === "error"
        ? "不可用"
        : "检测中";

  const handleKeyDown = (event: ReactKeyboardEvent<HTMLButtonElement>) => {
    if (!capturing) {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        event.stopPropagation();
        onStartCapture();
      }
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      onStopCapture();
      return;
    }

    if (event.key === "Backspace" || event.key === "Delete") {
      onChange("");
      onStopCapture();
      return;
    }

    const shortcut = shortcutFromKeyboardEvent(event);
    if (shortcut === null) {
      return;
    }

    onChange(shortcut);
    onStopCapture();
  };

  return (
    <div className="grid gap-2 text-sm font-medium text-[color:var(--cliply-muted)]">
      <div className="flex items-center justify-between gap-3">
        <span>打开 Cliply</span>
        <span
          className={clsx(
            "rounded-full px-2 py-0.5 text-[11px] font-semibold",
            capturing && "bg-[color:var(--cliply-accent-50)] text-[color:var(--cliply-accent-strong)]",
            !capturing && statusTone === "ok" && "bg-emerald-50 text-emerald-700",
            !capturing && statusTone === "error" && "bg-rose-50 text-rose-700",
            !capturing && statusTone === "pending" && "bg-[#f3f5f8] text-[color:var(--cliply-muted)]",
          )}
        >
          {statusLabel}
        </span>
      </div>
      <button
        type="button"
        onClick={onStartCapture}
        onBlur={onStopCapture}
        onKeyDown={handleKeyDown}
        className={clsx(
          "flex h-11 items-center justify-between rounded-xl border bg-white px-3 text-left text-sm font-semibold outline-none transition",
          capturing
            ? "border-[color:var(--cliply-accent)] text-[color:var(--cliply-accent-strong)] shadow-[0_0_0_4px_var(--cliply-focus-ring)]"
            : check?.ok
              ? "border-emerald-200 text-[color:var(--cliply-text)] hover:border-[color:var(--cliply-border-strong)]"
              : check
                ? "border-rose-200 text-[color:var(--cliply-text)] hover:border-rose-300"
                : "border-[color:var(--cliply-border)] text-[color:var(--cliply-text)] hover:border-[color:var(--cliply-border-strong)]",
        )}
      >
        <span className="cliply-code-font truncate">
          {capturing ? "按下新的快捷键..." : value || "点击录制快捷键"}
        </span>
        <span className="ml-3 shrink-0 text-xs font-medium text-[color:var(--cliply-muted)]">
          {capturing ? "Esc 取消" : "点击修改"}
        </span>
      </button>
      <p
        className={clsx(
          "min-h-5 text-xs leading-5",
          check?.ok ? "text-emerald-700" : check ? "text-rose-700" : "text-[color:var(--cliply-muted)]",
        )}
      >
        {capturing
          ? "需要包含 Ctrl、Alt 或 Win；按 Backspace/Delete 可清空。"
          : check?.message ?? "正在检测快捷键是否可用..."}
      </p>
    </div>
  );
}

function ToggleRow({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="flex h-8 items-center justify-between gap-3 text-[13px] font-medium text-[color:var(--cliply-muted)]">
      {label}
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        className="size-4 accent-[color:var(--cliply-accent)]"
      />
    </label>
  );
}

function NumberRow({
  label,
  value,
  min,
  max,
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="flex items-center justify-between gap-3 text-sm font-medium text-[color:var(--cliply-muted)]">
      {label}
      <input
        type="number"
        min={min}
        max={max}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        className="h-8 w-24 rounded-lg border border-[color:var(--cliply-border)] bg-white px-2.5 text-right text-[13px] font-semibold text-[color:var(--cliply-text)] outline-none focus:border-[color:var(--cliply-accent)]"
      />
    </label>
  );
}

function ShortcutRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-2 rounded-lg bg-[#fafafb] px-3 py-2">
      <span>{label}</span>
      <span className="cliply-code-font text-[color:var(--cliply-text)]">{value}</span>
    </div>
  );
}

function ThemePicker({
  value,
  onChange,
}: {
  value: CliplyThemeName;
  onChange: (value: CliplyThemeName) => void;
}) {
  return (
    <div className="grid gap-2">
      <div className="flex items-center justify-between gap-3">
        <span className="text-sm font-medium text-[color:var(--cliply-muted)]">主题方案</span>
        <span className="cliply-code-font text-xs font-medium text-[color:var(--cliply-muted)]">
          {value}
        </span>
      </div>
      <div className="grid grid-cols-2 gap-2">
        {CLIPLY_THEME_OPTIONS.map((theme) => {
          const selected = theme.name === value;
          return (
            <button
              key={theme.name}
              type="button"
              onClick={() => onChange(theme.name)}
              className={clsx(
                "flex min-h-[58px] items-center gap-3 rounded-lg border bg-white px-3 py-2 text-left transition",
                selected
                  ? "border-[color:var(--cliply-accent)] bg-[color:var(--cliply-accent-50)]"
                  : "border-[color:var(--cliply-border)] hover:border-[color:var(--cliply-border-strong)]",
              )}
            >
              <span
                className="size-5 shrink-0 rounded-full"
                style={{ backgroundColor: theme.swatch }}
              />
              <span className="min-w-0">
                <span className="block text-[13px] font-semibold text-[color:var(--cliply-text)]">
                  {theme.label}
                </span>
                <span className="line-clamp-1 block text-xs text-[color:var(--cliply-muted)]">
                  {theme.description}
                </span>
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

function getDraftThemeName(value: string): CliplyThemeName {
  return isCliplyThemeName(value) ? value : DEFAULT_THEME_NAME;
}

function shortcutFromKeyboardEvent(event: ReactKeyboardEvent<HTMLButtonElement>) {
  if (event.key === "Tab") {
    return null;
  }

  const key = normalizeShortcutKey(event.key, event.code);
  if (!key) {
    return null;
  }

  return [
    event.ctrlKey ? "Ctrl" : null,
    event.altKey ? "Alt" : null,
    event.shiftKey ? "Shift" : null,
    event.metaKey ? "Win" : null,
    key,
  ]
    .filter(Boolean)
    .join("+");
}

function normalizeShortcutKey(key: string, code: string) {
  if (["Control", "Shift", "Alt", "Meta"].includes(key)) {
    return null;
  }

  if (/^Key[A-Z]$/.test(code)) {
    return code.slice(3);
  }

  if (/^Digit[0-9]$/.test(code)) {
    return code.slice(5);
  }

  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(key)) {
    return key;
  }

  if (key.length === 1) {
    return key.toUpperCase();
  }

  const knownKeys: Record<string, string> = {
    ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",
    ArrowUp: "ArrowUp",
    Backspace: "Backspace",
    Delete: "Delete",
    End: "End",
    Enter: "Enter",
    Escape: "Escape",
    Home: "Home",
    Insert: "Insert",
    PageDown: "PageDown",
    PageUp: "PageUp",
    Space: "Space",
    Tab: "Tab",
  };

  return knownKeys[key] ?? null;
}
