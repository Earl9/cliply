import { clsx } from "clsx";

export type PillTabOption<T extends string> = {
  value: T;
  label: string;
  count?: number;
};

type PillTabsProps<T extends string> = {
  options: PillTabOption<T>[];
  value: T;
  onValueChange: (value: T) => void;
};

export function PillTabs<T extends string>({ options, value, onValueChange }: PillTabsProps<T>) {
  return (
    <div className="flex h-10 min-w-0 items-center gap-3 overflow-x-auto">
      {options.map((option) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            onClick={() => onValueChange(option.value)}
            className={clsx(
              "inline-flex h-9 shrink-0 items-center gap-2 rounded-[10px] border px-5 text-sm font-medium transition",
              "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
              selected
                ? "border-[color:var(--cliply-accent-border)] bg-[color:var(--cliply-accent-soft)] text-[color:var(--cliply-accent-strong)] shadow-sm"
                : "border-[color:var(--cliply-border)] bg-white/55 text-[color:var(--cliply-muted)] hover:bg-white hover:text-[color:var(--cliply-text)]",
            )}
          >
            <span>{option.label}</span>
            {typeof option.count === "number" ? (
              <span
                className={clsx(
                  "rounded-md px-1.5 text-[11px]",
                  selected
                    ? "bg-white/70 text-[color:var(--cliply-accent-strong)]"
                    : "bg-slate-100 text-slate-500",
                )}
              >
                {option.count}
              </span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}
