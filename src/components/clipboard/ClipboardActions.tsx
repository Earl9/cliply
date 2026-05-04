import { Clipboard, Copy, Pin, Type } from "lucide-react";
import { ShortcutKey } from "@/components/common/ShortcutKey";

const actions = [
  { label: "Paste", keys: ["Enter"], icon: Clipboard, primary: true },
  { label: "Copy", keys: ["Ctrl", "C"], icon: Copy },
  { label: "Plain", keys: ["Shift", "Enter"], icon: Type },
  { label: "Pin", keys: ["Ctrl", "P"], icon: Pin },
];

export function ClipboardActions() {
  return (
    <div className="flex shrink-0 flex-wrap items-center gap-2 border-t border-[color:var(--cliply-border)] px-4 py-3">
      {actions.map((action) => {
        const Icon = action.icon;
        return (
          <button
            key={action.label}
            type="button"
            className={
              action.primary
                ? "inline-flex h-9 items-center gap-2 rounded-md bg-[color:var(--cliply-accent)] px-3 text-sm font-semibold text-white shadow-sm transition hover:bg-[color:var(--cliply-accent-strong)]"
                : "inline-flex h-9 items-center gap-2 rounded-md border border-[color:var(--cliply-border)] bg-white/72 px-3 text-sm font-medium text-[color:var(--cliply-text)] transition hover:bg-white"
            }
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
