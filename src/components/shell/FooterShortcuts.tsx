import { Circle } from "lucide-react";

const shortcuts = [
  { keys: ["↑↓"], label: "选择" },
  { keys: ["Enter"], label: "粘贴" },
  { keys: ["Esc"], label: "关闭" },
];

type FooterShortcutsProps = {
  monitoringPaused?: boolean;
};

export function FooterShortcuts({ monitoringPaused = false }: FooterShortcutsProps) {
  return (
    <footer className="cliply-status-bar flex h-7 shrink-0 items-center justify-between gap-3 border-t border-[color:var(--cliply-border)] px-3">
      <div className="flex min-w-0 items-center gap-x-4 overflow-hidden">
        {shortcuts.map((shortcut) => (
          <span
            key={`${shortcut.keys.join("-")}-${shortcut.label}`}
            className="cliply-caption inline-flex shrink-0 items-center gap-1.5 text-[10px] text-[color:var(--cliply-faint)]"
          >
            <span className="cliply-status-key">{shortcut.keys.join("+")}</span>
            <span>{shortcut.label}</span>
          </span>
        ))}
      </div>
      <div className="cliply-caption hidden shrink-0 items-center gap-1.5 text-[10px] text-[color:var(--cliply-faint)] sm:flex">
        <Circle
          className={
            monitoringPaused
              ? "size-1.5 fill-[color:var(--cliply-warning)] text-[color:var(--cliply-warning)]"
              : "size-1.5 fill-[color:var(--cliply-success)] text-[color:var(--cliply-success)]"
          }
        />
        {monitoringPaused ? "剪贴板监听已暂停" : "剪贴板监听中"}
      </div>
    </footer>
  );
}
