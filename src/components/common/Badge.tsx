import type { ReactNode } from "react";
import { clsx } from "clsx";

type BadgeTone = "neutral" | "accent" | "teal" | "amber" | "rose";

type BadgeProps = {
  children: ReactNode;
  tone?: BadgeTone;
  className?: string;
};

const toneClass: Record<BadgeTone, string> = {
  neutral: "bg-[#f3f5f8] text-[color:var(--cliply-muted)]",
  accent: "bg-[color:var(--cliply-accent-soft)] text-[color:var(--cliply-accent-strong)]",
  teal: "bg-teal-50 text-teal-700",
  amber: "bg-amber-50 text-amber-700",
  rose: "bg-rose-50 text-rose-700",
};

export function Badge({ children, tone = "neutral", className }: BadgeProps) {
  return (
    <span
      className={clsx(
        "inline-flex h-6 items-center rounded-md px-2 text-xs font-medium",
        toneClass[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}
