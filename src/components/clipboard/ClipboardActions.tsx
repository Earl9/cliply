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
    <footer className="grid shrink-0 grid-cols-[1.25fr_repeat(4,minmax(0,1fr))] gap-[14px] border-t border-[color:var(--cliply-border-soft)] bg-white/95 px-6 pb-5 pt-[18px]">
      {actions.map((action) => {
        const Icon = action.icon;
        return (
          <button
            key={action.label}
            type="button"
            disabled={action.disabled}
            onClick={() => onAction(action.kind)}
            className={clsx(
              "flex h-[78px] min-w-0 flex-col items-center justify-center gap-1 rounded-[14px] text-[17px] font-semibold transition disabled:cursor-not-allowed disabled:border-transparent disabled:bg-[#f8fafc] disabled:text-[#a0a8b5] disabled:opacity-100",
              "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[rgba(124,92,255,0.45)]",
              action.primary
                ? "bg-[color:var(--cliply-accent-strong)] text-white shadow-sm hover:bg-[color:var(--cliply-accent-dark)]"
                : action.danger
                  ? "border border-transparent bg-white text-[color:var(--cliply-body-text)] hover:border-[#fecaca] hover:bg-[#fef2f2] hover:text-[color:var(--cliply-danger)]"
                  : "border border-transparent bg-white text-[color:var(--cliply-body-text)] hover:border-[color:var(--cliply-border-strong)] hover:bg-[#f8fafc]",
            )}
          >
            <Icon className={clsx(action.primary ? "size-[22px]" : "size-5")} />
            <span>{action.label}</span>
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
