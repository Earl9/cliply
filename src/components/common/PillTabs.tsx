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
    <div className="flex h-8 min-w-0 items-center gap-2 overflow-x-auto">
      {options.map((option) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            onClick={() => onValueChange(option.value)}
            className={clsx(
              "inline-flex h-8 min-w-20 shrink-0 items-center justify-center gap-1.5 rounded-[10px] border px-3 text-sm font-semibold leading-none transition",
              "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
              selected
                ? "border-[color:var(--cliply-accent-border)] bg-[color:var(--cliply-accent-50)] text-[color:var(--cliply-accent-strong)] shadow-none"
                : "border-transparent bg-transparent text-[color:var(--cliply-muted)] hover:border-[color:var(--cliply-border)] hover:bg-[color:var(--cliply-muted-bg)] hover:text-[color:var(--cliply-text)]",
            )}
          >
            <span>{option.label}</span>
            {typeof option.count === "number" ? (
              <span
                className={clsx(
                  "inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-semibold leading-none",
                  selected
                    ? "bg-[color:var(--cliply-muted-bg)] text-[color:var(--cliply-accent-strong)]"
                    : "bg-[color:var(--cliply-muted-bg)] text-[color:var(--cliply-muted)]",
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
