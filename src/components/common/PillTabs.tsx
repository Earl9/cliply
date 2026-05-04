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
    <div className="flex min-w-0 items-center gap-1.5 overflow-x-auto px-3 py-2">
      {options.map((option) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            onClick={() => onValueChange(option.value)}
            className={clsx(
              "inline-flex h-8 shrink-0 items-center gap-1.5 rounded-md px-3 text-sm font-medium transition",
              "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
              selected
                ? "bg-[color:var(--cliply-accent)] text-white shadow-sm"
                : "text-[color:var(--cliply-muted)] hover:bg-white/70 hover:text-[color:var(--cliply-text)]",
            )}
          >
            <span>{option.label}</span>
            {typeof option.count === "number" ? (
              <span
                className={clsx(
                  "rounded px-1.5 text-[11px]",
                  selected ? "bg-white/18 text-white" : "bg-slate-100 text-slate-500",
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
