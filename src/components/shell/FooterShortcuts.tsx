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
    <footer className="flex h-8 shrink-0 items-center justify-between gap-3 border-t border-[color:var(--cliply-border-soft)] px-3">
      <div className="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1">
        {shortcuts.map((shortcut) => (
          <span
            key={`${shortcut.keys.join("-")}-${shortcut.label}`}
            className="cliply-caption inline-flex items-center gap-1 text-[11px] text-[color:var(--cliply-faint)]"
          >
            <ShortcutKey keys={shortcut.keys} compact />
            <span>{shortcut.label}</span>
          </span>
        ))}
      </div>
      <div className="cliply-caption hidden shrink-0 items-center gap-1.5 text-[11px] text-[color:var(--cliply-faint)] sm:flex">
        <Circle
          className={
            monitoringPaused
              ? "size-2 fill-[color:var(--cliply-warning)] text-[color:var(--cliply-warning)]"
              : "size-2 fill-[color:var(--cliply-success)] text-[color:var(--cliply-success)]"
          }
        />
        {monitoringPaused ? "监听已暂停" : "本地保存"}
      </div>
    </footer>
  );
}
