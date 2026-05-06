import { clsx } from "clsx";

type ShortcutKeyProps = {
  keys: string[];
  compact?: boolean;
  tone?: "default" | "onPrimary";
};

export function ShortcutKey({ keys, compact, tone = "default" }: ShortcutKeyProps) {
  return (
    <span className="inline-flex items-center gap-1 align-middle">
      {keys.map((key, index) => (
        <span key={`${key}-${index}`} className="inline-flex items-center gap-1">
          {index > 0 ? (
            <span className={clsx("text-xs", tone === "onPrimary" ? "text-white/65" : "text-[#a0a8b5]")}>+</span>
          ) : null}
          <kbd
            className={clsx(
              "inline-flex min-w-4 items-center justify-center rounded-md px-1.5 font-medium",
              compact ? "h-[18px] text-[11px]" : "h-6 text-xs",
              tone === "onPrimary"
                ? "border border-transparent bg-white/18 text-white"
                : "border border-[#e4e8ef] bg-[#f3f5f8] text-[color:var(--cliply-muted)]",
            )}
          >
            {key}
          </kbd>
        </span>
      ))}
    </span>
  );
}
