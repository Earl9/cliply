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
              "inline-flex min-w-5 items-center justify-center rounded-md border border-[color:var(--cliply-border)] bg-[#f4f5f7] px-2 font-medium text-[color:var(--cliply-muted)]",
              compact ? "h-6 text-xs" : "h-7 text-xs",
            )}
          >
            {key}
          </kbd>
        </span>
      ))}
    </span>
  );
}
