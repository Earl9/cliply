import { Database } from "lucide-react";
import { ShortcutKey } from "@/components/common/ShortcutKey";

const shortcuts = [
  { keys: ["Enter"], label: "Paste" },
  { keys: ["Shift", "Enter"], label: "Paste plain" },
  { keys: ["Ctrl", "P"], label: "Pin" },
  { keys: ["Esc"], label: "Close" },
];

export function FooterShortcuts() {
  return (
    <footer className="flex min-h-12 shrink-0 items-center justify-between gap-3 border-t border-[color:var(--cliply-border)] px-4 py-2">
      <div className="flex min-w-0 flex-wrap items-center gap-x-4 gap-y-2">
        {shortcuts.map((shortcut) => (
          <span
            key={`${shortcut.keys.join("-")}-${shortcut.label}`}
            className="inline-flex items-center gap-2 text-xs text-[color:var(--cliply-muted)]"
          >
            <ShortcutKey keys={shortcut.keys} compact />
            <span>{shortcut.label}</span>
          </span>
        ))}
      </div>
      <div className="hidden shrink-0 items-center gap-2 text-xs font-medium text-[color:var(--cliply-muted)] sm:flex">
        <Database className="size-3.5 text-[color:var(--cliply-teal)]" />
        Local storage
      </div>
    </footer>
  );
}
