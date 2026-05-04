import { Circle } from "lucide-react";
import { ShortcutKey } from "@/components/common/ShortcutKey";

const shortcuts = [
  { keys: ["Enter"], label: "粘贴" },
  { keys: ["Shift", "Enter"], label: "无格式" },
  { keys: ["Esc"], label: "关闭" },
];

type FooterShortcutsProps = {
  monitoringPaused?: boolean;
};

export function FooterShortcuts({ monitoringPaused = false }: FooterShortcutsProps) {
  return (
    <footer className="flex h-14 shrink-0 items-center justify-between gap-3 border-t border-[color:var(--cliply-border)] bg-[#f8fafc]/95 px-8">
      <div className="flex min-w-0 flex-wrap items-center gap-x-4 gap-y-2">
        {shortcuts.map((shortcut) => (
          <span
            key={`${shortcut.keys.join("-")}-${shortcut.label}`}
            className="inline-flex items-center gap-2 text-sm font-medium text-[color:var(--cliply-muted)]"
          >
            <ShortcutKey keys={shortcut.keys} compact />
            <span>{shortcut.label}</span>
          </span>
        ))}
      </div>
      <div className="hidden shrink-0 items-center gap-2 text-[15px] font-medium text-[#4b5563] sm:flex">
        <Circle
          className={
            monitoringPaused
              ? "size-2.5 fill-[color:var(--cliply-warning)] text-[color:var(--cliply-warning)]"
              : "size-2.5 fill-[color:var(--cliply-success)] text-[color:var(--cliply-success)]"
          }
        />
        {monitoringPaused ? "监听已暂停" : "本地保存"}
      </div>
    </footer>
  );
}
