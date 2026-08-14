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
    <div className="cliply-filter-tabs flex h-8 min-w-0 items-center gap-0.5 overflow-x-auto p-0.5" role="group" aria-label="筛选剪贴板记录">
      {options.map((option) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            data-value={option.value}
            aria-pressed={selected}
            onClick={() => onValueChange(option.value)}
            className={clsx(
              "cliply-filter-tab cliply-interactive relative inline-flex h-7 min-w-0 flex-1 items-center justify-center gap-1.5 rounded-[5px] px-1.5 text-[11.5px] leading-none",
              "focus-visible:z-10 focus-visible:outline focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-[color:var(--cliply-accent)]",
              selected
                ? "font-semibold text-[color:var(--cliply-text)]"
                : "text-[color:var(--cliply-faint)] hover:text-[color:var(--cliply-text)]",
            )}
          >
            <span>{option.label}</span>
            {typeof option.count === "number" ? (
              <span
                className={clsx(
                  "cliply-caption min-w-3 text-[10px] tabular-nums",
                  selected ? "text-[color:var(--cliply-accent)]" : "opacity-70",
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
