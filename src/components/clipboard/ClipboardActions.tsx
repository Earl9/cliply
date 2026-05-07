import { Clipboard, Copy, Pin, Trash2, Type } from "lucide-react";
import { clsx } from "clsx";
import { ShortcutKey } from "@/components/common/ShortcutKey";
import { getClipboardActionAvailability } from "@/lib/clipboardCapabilities";
import type { ClipboardActionKind, ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardActionsProps = {
  item: ClipboardItem;
  onAction: (action: ClipboardActionKind) => void;
};

export function ClipboardActions({ item, onAction }: ClipboardActionsProps) {
  const availability = getClipboardActionAvailability(item);
  const actions: Array<{
    label: string;
    keys: string[];
    icon: typeof Clipboard;
    primary?: boolean;
    kind: ClipboardActionKind;
    disabled?: boolean;
    danger?: boolean;
  }> = [
    {
      label: "粘贴",
      keys: ["Enter"],
      icon: Clipboard,
      primary: true,
      kind: "paste",
      disabled: !availability.paste,
    },
    { label: "复制", keys: ["Ctrl", "C"], icon: Copy, kind: "copy", disabled: !availability.copy },
    {
      label: "无格式",
      keys: ["Shift", "Enter"],
      icon: Type,
      kind: "pastePlain",
      disabled: !availability.pastePlain,
    },
    { label: item.isPinned ? "取消固定" : "固定", keys: ["Ctrl", "P"], icon: Pin, kind: "togglePin" },
    { label: "删除", keys: ["Del"], icon: Trash2, kind: "delete", danger: true },
  ];

  return (
    <footer className="grid shrink-0 grid-cols-[1.18fr_repeat(4,minmax(0,1fr))] gap-2 border-t border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-card)] px-4 py-2">
      {actions.map((action) => {
        const Icon = action.icon;
        return (
          <button
            key={action.label}
            type="button"
            disabled={action.disabled}
            onClick={() => onAction(action.kind)}
            className={clsx(
              "flex h-[50px] min-w-0 flex-col items-center justify-center gap-0.5 rounded-[10px] border text-[13px] font-semibold transition disabled:cursor-not-allowed disabled:border-[color:var(--cliply-border-soft)] disabled:bg-[color:var(--cliply-muted-bg)] disabled:text-[color:var(--cliply-disabled)] disabled:opacity-100",
              "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-focus-ring)]",
              action.primary
                ? "border-transparent bg-[color:var(--cliply-accent-strong)] text-white shadow-[0_6px_14px_rgba(100,65,232,0.22)] hover:bg-[color:var(--cliply-accent-dark)]"
                : action.danger
                  ? "border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-muted-bg)] text-[color:var(--cliply-text)] hover:border-[color:var(--cliply-danger)] hover:bg-[color:var(--cliply-danger-soft)] hover:text-[color:var(--cliply-danger)]"
                  : "border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-muted-bg)] text-[color:var(--cliply-text)] hover:border-[color:var(--cliply-border-strong)] hover:bg-[color:var(--cliply-card)] hover:shadow-[0_4px_12px_rgba(15,23,42,0.045)]",
            )}
          >
            <span className="flex items-center gap-1.5">
              <Icon className={clsx(action.primary ? "size-4" : "size-[15px]")} />
              <span>{action.label}</span>
            </span>
            <ShortcutKey
              keys={action.keys}
              compact
              tone={action.primary ? "onPrimary" : "default"}
            />
          </button>
        );
      })}
    </footer>
  );
}
