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
    { label: "删除", keys: ["Del"], icon: Trash2, kind: "delete" },
  ];

  return (
    <div className="grid shrink-0 grid-cols-5 gap-3 border-t border-[color:var(--cliply-border)] p-5">
      {actions.map((action) => {
        const Icon = action.icon;
        return (
          <button
            key={action.label}
            type="button"
            disabled={action.disabled}
            onClick={() => onAction(action.kind)}
            className={clsx(
              "flex h-[72px] min-w-0 flex-col items-center justify-center gap-1 rounded-xl text-sm font-semibold transition disabled:cursor-not-allowed disabled:opacity-45",
              "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
              action.primary
                ? "bg-[color:var(--cliply-accent-strong)] text-white shadow-sm hover:bg-[#4932af]"
                : "border border-[color:var(--cliply-border)] bg-white text-[color:var(--cliply-text)] hover:bg-[#fafafb] hover:shadow-sm",
            )}
          >
            <Icon className="size-5" />
            <span>{action.label}</span>
            <ShortcutKey keys={action.keys} compact />
          </button>
        );
      })}
    </div>
  );
}
