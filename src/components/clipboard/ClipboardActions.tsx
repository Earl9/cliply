import { Clipboard, Copy, Pin, Type } from "lucide-react";
import { clsx } from "clsx";
import { ShortcutKey } from "@/components/common/ShortcutKey";
import type { ClipboardActionKind, ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardActionsProps = {
  item: ClipboardItem;
  onAction: (action: ClipboardActionKind) => void;
};

export function ClipboardActions({ item, onAction }: ClipboardActionsProps) {
  const actions: Array<{
    label: string;
    keys: string[];
    icon: typeof Clipboard;
    primary?: boolean;
    kind: ClipboardActionKind;
    disabled?: boolean;
  }> = [
    { label: "Paste", keys: ["Enter"], icon: Clipboard, primary: true, kind: "paste" },
    { label: "Copy", keys: ["Ctrl", "C"], icon: Copy, kind: "copy" },
    {
      label: "Plain",
      keys: ["Shift", "Enter"],
      icon: Type,
      kind: "pastePlain",
      disabled: item.type === "image" && !item.fullText,
    },
    { label: item.isPinned ? "Unpin" : "Pin", keys: ["Ctrl", "P"], icon: Pin, kind: "togglePin" },
  ];

  return (
    <div className="flex shrink-0 flex-wrap items-center gap-2 border-t border-[color:var(--cliply-border)] px-4 py-3">
      {actions.map((action) => {
        const Icon = action.icon;
        return (
          <button
            key={action.label}
            type="button"
            disabled={action.disabled}
            onClick={() => onAction(action.kind)}
            className={clsx(
              "transition disabled:cursor-not-allowed disabled:opacity-45",
              action.primary
                ? "inline-flex h-9 items-center gap-2 rounded-md bg-[color:var(--cliply-accent)] px-3 text-sm font-semibold text-white shadow-sm transition hover:bg-[color:var(--cliply-accent-strong)]"
                : "inline-flex h-9 items-center gap-2 rounded-md border border-[color:var(--cliply-border)] bg-white/72 px-3 text-sm font-medium text-[color:var(--cliply-text)] hover:bg-white",
            )}
          >
            <Icon className="size-4" />
            <span>{action.label}</span>
            <ShortcutKey keys={action.keys} compact />
          </button>
        );
      })}
    </div>
  );
}
