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
    <div className="flex h-10 min-w-0 items-center gap-4 overflow-x-auto">
      {options.map((option) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            onClick={() => onValueChange(option.value)}
            className={clsx(
              "inline-flex h-10 min-w-28 shrink-0 items-center justify-center gap-2 rounded-xl border px-[22px] text-[16px] font-medium transition",
              "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
              selected
                ? "border-[color:var(--cliply-accent-border)] bg-[color:var(--cliply-accent-50)] text-[color:var(--cliply-accent-strong)] shadow-[0_4px_12px_rgba(124,92,255,0.10)]"
                : "border-transparent bg-white/60 text-[#596275] hover:border-[#e7ebf2] hover:bg-white",
            )}
          >
            <span>{option.label}</span>
            {typeof option.count === "number" ? (
              <span
                className={clsx(
                  "ml-0 inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-xs font-medium",
                  selected
                    ? "bg-white text-[color:var(--cliply-accent-strong)]"
                    : "bg-[#eef2f7] text-[#718096]",
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
