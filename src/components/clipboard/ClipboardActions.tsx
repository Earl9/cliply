import { Clipboard, Copy, Pin, PinOff, Trash2, Type } from "lucide-react";
import { clsx } from "clsx";
import { getClipboardActionAvailability } from "@/lib/clipboardCapabilities";
import type { ClipboardActionKind, ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardActionsProps = {
  item: ClipboardItem;
  onAction: (action: ClipboardActionKind) => void;
};

export function ClipboardActions({ item, onAction }: ClipboardActionsProps) {
  const availability = getClipboardActionAvailability(item);
  const secondary: Array<{
    label: string;
    keys: string[];
    icon: typeof Copy;
    kind: ClipboardActionKind;
    disabled?: boolean;
    danger?: boolean;
  }> = [
    { label: "复制", keys: ["Ctrl", "C"], icon: Copy, kind: "copy", disabled: !availability.copy },
    {
    label: "无格式粘贴",
      keys: ["Shift", "Enter"],
      icon: Type,
      kind: "pastePlain",
      disabled: !availability.pastePlain,
    },
    {
      label: item.isPinned ? "取消固定" : "固定",
      keys: ["Ctrl", "P"],
      icon: item.isPinned ? PinOff : Pin,
      kind: "togglePin",
    },
  ];

  return (
    <footer className="cliply-action-bar flex h-14 shrink-0 items-center gap-1.5 border-t border-[color:var(--cliply-border)] px-3.5">
      {secondary.map((action) => {
        const Icon = action.icon;
        return (
          <button
            key={action.label}
            type="button"
            disabled={action.disabled}
            onClick={() => onAction(action.kind)}
            title={`${action.label} (${action.keys.join("+")})`}
            className={clsx(
              "cliply-action-button cliply-interactive flex h-8 min-w-0 shrink items-center gap-1.5 rounded-md px-2.5 text-[12px] text-[color:var(--cliply-body-text)]",
              "border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-surface-raised)] hover:border-[color:var(--cliply-border)] hover:bg-[color:var(--cliply-muted-bg)] hover:text-[color:var(--cliply-text)]",
              "focus-visible:outline focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-[color:var(--cliply-accent)]",
              "disabled:cursor-not-allowed disabled:text-[color:var(--cliply-disabled)] disabled:hover:bg-transparent",
            )}
          >
            <Icon className="size-3.5 shrink-0" />
            <span className="cliply-action-label truncate">{action.label}</span>
          </button>
        );
      })}

      <span aria-hidden="true" className="cliply-action-divider mx-1 h-4 w-px shrink-0 bg-[color:var(--cliply-border)]" />

      <button
        type="button"
        onClick={() => onAction("delete")}
        title="删除 (Del)"
        className={clsx(
          "cliply-action-button cliply-interactive flex size-8 shrink-0 items-center justify-center rounded-md border border-transparent text-[color:var(--cliply-muted)]",
          "hover:bg-[color:var(--cliply-danger-soft)] hover:text-[color:var(--cliply-danger)]",
          "focus-visible:outline focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-[color:var(--cliply-danger)]",
        )}
      >
        <Trash2 className="size-4" />
      </button>

      <button
        type="button"
        disabled={!availability.paste}
        onClick={() => onAction("paste")}
        data-primary="true"
        title="粘贴 (Enter)"
        className={clsx(
          "cliply-action-button ml-auto flex h-9 shrink-0 items-center gap-2 rounded-md px-4 text-[12px] font-semibold",
          "bg-[color:var(--cliply-primary)] text-[color:var(--cliply-primary-action-text)] hover:bg-[color:var(--cliply-primary-hover)] active:bg-[color:var(--cliply-primary-active)]",
          "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
          "disabled:cursor-not-allowed disabled:bg-[color:var(--cliply-disabled-bg)] disabled:text-[color:var(--cliply-disabled)]",
        )}
      >
        <Clipboard className="size-3.5 shrink-0" />
        <span>粘贴</span>
        <span className="cliply-primary-shortcut">Enter</span>
      </button>
    </footer>
  );
}
