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
    <footer className="grid shrink-0 grid-cols-[1.18fr_repeat(4,minmax(0,1fr))] gap-3 border-t border-[color:var(--cliply-border-soft)] bg-white px-6 py-3">
      {actions.map((action) => {
        const Icon = action.icon;
        return (
          <button
            key={action.label}
            type="button"
            disabled={action.disabled}
            onClick={() => onAction(action.kind)}
            className={clsx(
              "flex h-[62px] min-w-0 flex-col items-center justify-center gap-1 rounded-[13px] border text-[15px] font-semibold transition disabled:cursor-not-allowed disabled:border-[#eef1f5] disabled:bg-[#f8fafc] disabled:text-[#a0a8b5] disabled:opacity-100",
              "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[rgba(124,92,255,0.45)]",
              action.primary
                ? "border-transparent bg-[color:var(--cliply-accent-strong)] text-white shadow-[0_8px_18px_rgba(100,65,232,0.24)] hover:bg-[color:var(--cliply-accent-dark)]"
                : action.danger
                  ? "border-[#eef1f5] bg-[#fbfcfe] text-[#364152] hover:border-[#fecaca] hover:bg-[#fff5f5] hover:text-[color:var(--cliply-danger)]"
                  : "border-[#eef1f5] bg-[#fbfcfe] text-[#364152] hover:border-[color:var(--cliply-border-strong)] hover:bg-white hover:shadow-[0_4px_12px_rgba(15,23,42,0.045)]",
            )}
          >
            <span className="flex items-center gap-2">
              <Icon className={clsx(action.primary ? "size-[20px]" : "size-[18px]")} />
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
