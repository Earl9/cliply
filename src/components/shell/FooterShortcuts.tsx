import { Circle } from "lucide-react";
import { ShortcutKey } from "@/components/common/ShortcutKey";

const shortcuts = [
  { keys: ["Enter"], label: "粘贴" },
  { keys: ["Shift", "Enter"], label: "无格式" },
  { keys: ["Ctrl", "P"], label: "固定" },
  { keys: ["Esc"], label: "关闭" },
];

export function FooterShortcuts() {
  return (
    <footer className="flex h-14 shrink-0 items-center justify-between gap-3 border-t border-[color:var(--cliply-border)] px-7">
      <div className="flex min-w-0 flex-wrap items-center gap-x-4 gap-y-2">
        {shortcuts.map((shortcut) => (
          <span
            key={`${shortcut.keys.join("-")}-${shortcut.label}`}
            className="inline-flex items-center gap-2 text-[13px] font-medium text-[color:var(--cliply-muted)]"
          >
            <ShortcutKey keys={shortcut.keys} compact />
            <span>{shortcut.label}</span>
          </span>
        ))}
      </div>
      <div className="hidden shrink-0 items-center gap-2 text-[13px] font-medium text-[color:var(--cliply-muted)] sm:flex">
        <Circle className="size-2.5 fill-[color:var(--cliply-success)] text-[color:var(--cliply-success)]" />
        本地保存
      </div>
    </footer>
  );
}
