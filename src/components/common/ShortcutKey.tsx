import { clsx } from "clsx";

type ShortcutKeyProps = {
  keys: string[];
  compact?: boolean;
  tone?: "default" | "onPrimary";
};

export function ShortcutKey({ keys, compact, tone = "default" }: ShortcutKeyProps) {
  return (
    <span className="inline-flex shrink-0 items-center gap-0.5 whitespace-nowrap align-middle">
      {keys.map((key, index) => (
        <span key={`${key}-${index}`} className="inline-flex shrink-0 items-center gap-0.5">
          {index > 0 ? (
            <span className={clsx("text-[11px]", tone === "onPrimary" ? "opacity-60" : "text-[color:var(--cliply-faint)]")}>+</span>
          ) : null}
          <kbd
            className={clsx(
              "inline-flex min-w-4 shrink-0 items-center justify-center rounded-[3px] px-1 font-sans",
              compact ? "h-4 text-[11px]" : "h-5 text-xs",
              tone === "onPrimary"
                // Inherit the button's own text colour: bright accents use dark
                // text, so a hard-coded white would be unreadable there.
                ? "opacity-75"
                : "border border-[color:var(--cliply-border)] text-[color:var(--cliply-faint)]",
            )}
          >
            {key}
          </kbd>
        </span>
      ))}
    </span>
  );
}
