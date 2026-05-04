import { clsx } from "clsx";

type ShortcutKeyProps = {
  keys: string[];
  compact?: boolean;
};

export function ShortcutKey({ keys, compact }: ShortcutKeyProps) {
  return (
    <span className="inline-flex items-center gap-1 align-middle">
      {keys.map((key, index) => (
        <span key={`${key}-${index}`} className="inline-flex items-center gap-1">
          {index > 0 ? (
            <span className="text-[10px] text-[color:var(--cliply-faint)]">+</span>
          ) : null}
          <kbd
            className={clsx(
              "inline-flex min-w-5 items-center justify-center rounded border border-[color:var(--cliply-border)] bg-white/85 px-1.5 font-medium text-[color:var(--cliply-muted)] shadow-sm",
              compact ? "h-5 text-[10px]" : "h-6 text-[11px]",
            )}
          >
            {key}
          </kbd>
        </span>
      ))}
    </span>
  );
}
