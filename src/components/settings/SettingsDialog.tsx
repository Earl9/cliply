import { useEffect, useState, type ReactNode } from "react";
import { BellOff, History, Keyboard, Shield, Sparkles, X } from "lucide-react";
import { Badge } from "@/components/common/Badge";
import { IconButton } from "@/components/common/IconButton";
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
  const ignoreAppsText = draft.ignoreApps.join("\n");

  useEffect(() => {
    if (open) {
      setDraft(settings);
    }
  }, [open, settings]);

  if (!open) {
    return null;
  }

  const updateDraft = <K extends keyof CliplySettings>(key: K, value: CliplySettings[K]) => {
    setDraft((current) => ({ ...current, [key]: value }));
  };

  return (
    <div className="absolute inset-0 z-30 grid place-items-center bg-slate-900/18 px-6 backdrop-blur-sm">
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="cliply-settings-title"
        className="flex max-h-[calc(100%-48px)] w-full max-w-[820px] flex-col overflow-hidden rounded-2xl border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] shadow-2xl"
      >
        <header className="flex h-16 shrink-0 items-center justify-between border-b border-[color:var(--cliply-border)] px-5">
          <div>
            <h2 id="cliply-settings-title" className="text-[15px] font-semibold text-[color:var(--cliply-text)]">
              设置
            </h2>
            <p className="mt-0.5 text-xs font-medium text-[color:var(--cliply-muted)]">
              本地优先，Windows MVP
            </p>
          </div>
          <IconButton label="关闭设置" onClick={onClose}>
            <X className="size-4" />
          </IconButton>
        </header>

        <div className="cliply-scrollbar min-h-0 flex-1 overflow-auto p-5">
          <div className="grid gap-4 md:grid-cols-2">
            <SettingSection icon={Keyboard} title="快捷键">
              <label className="grid gap-2 text-sm font-medium text-[color:var(--cliply-muted)]">
                打开 Cliply
                <input
                  value={draft.globalShortcut}
                  onChange={(event) => updateDraft("globalShortcut", event.target.value)}
                  className="h-10 rounded-xl border border-[color:var(--cliply-border)] bg-white px-3 text-sm font-semibold text-[color:var(--cliply-text)] outline-none focus:border-[color:var(--cliply-accent)]"
                />
              </label>
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
                <Badge tone="accent">浅色</Badge>
              </div>
            </SettingSection>
          </div>
        </div>

        <footer className="flex h-16 shrink-0 items-center justify-end gap-3 border-t border-[color:var(--cliply-border)] px-5">
          <button
            type="button"
            onClick={onClose}
            className="h-10 rounded-xl border border-[color:var(--cliply-border)] bg-white px-4 text-sm font-semibold text-[color:var(--cliply-text)] transition hover:bg-[#fafafb]"
          >
            取消
          </button>
          <button
            type="button"
            onClick={() => onSave(draft)}
            className="h-10 rounded-xl bg-[color:var(--cliply-accent-strong)] px-4 text-sm font-semibold text-white transition hover:bg-[#4932af]"
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
    <section className="rounded-xl border border-[color:var(--cliply-border)] bg-white/72 p-4">
      <div className="mb-4 flex items-center gap-2 text-sm font-semibold text-[color:var(--cliply-text)]">
        <Icon className="size-4 text-[color:var(--cliply-accent)]" />
        {title}
      </div>
      <div className="grid gap-3">{children}</div>
    </section>
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
    <label className="flex h-10 items-center justify-between gap-3 text-sm font-medium text-[color:var(--cliply-muted)]">
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
        className="h-10 w-28 rounded-xl border border-[color:var(--cliply-border)] bg-white px-3 text-right text-sm font-semibold text-[color:var(--cliply-text)] outline-none focus:border-[color:var(--cliply-accent)]"
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
