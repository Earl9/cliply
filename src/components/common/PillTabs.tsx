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
    <div className="cliply-scrollbar flex h-7 min-w-0 items-center gap-0.5 overflow-x-auto">
      {options.map((option) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            onClick={() => onValueChange(option.value)}
            className={clsx(
              "cliply-interactive inline-flex h-7 shrink-0 items-center justify-center gap-1.5 rounded-[6px] px-2.5 text-[12.5px] leading-none",
              "focus-visible:outline focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-[color:var(--cliply-accent)]",
              selected
                ? "cliply-pill-tab-active bg-[color:var(--cliply-accent-soft)] font-medium text-[color:var(--cliply-accent-on-soft)]"
                : "text-[color:var(--cliply-faint)] hover:bg-[color:var(--cliply-muted-bg)] hover:text-[color:var(--cliply-text)]",
            )}
          >
            <span>{option.label}</span>
            {typeof option.count === "number" ? (
              <span
                className={clsx(
                  "cliply-caption text-[11px] tabular-nums",
                  selected ? "opacity-75" : "opacity-60",
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
