import type { ReactNode } from "react";
import { clsx } from "clsx";

type BadgeTone = "neutral" | "accent" | "teal" | "amber" | "rose";

type BadgeProps = {
  children: ReactNode;
  tone?: BadgeTone;
  className?: string;
};

const toneClass: Record<BadgeTone, string> = {
  neutral: "bg-[color:var(--cliply-muted-bg)] text-[color:var(--cliply-muted)]",
  accent: "bg-[color:var(--cliply-accent-soft)] text-[color:var(--cliply-accent-strong)]",
  teal: "bg-[color:var(--cliply-info-soft)] text-[color:var(--cliply-info)]",
  amber: "bg-[color:var(--cliply-warning-soft)] text-[color:var(--cliply-warning)]",
  rose: "bg-[color:var(--cliply-danger-soft)] text-[color:var(--cliply-danger)]",
};

export function Badge({ children, tone = "neutral", className }: BadgeProps) {
  return (
    <span
      data-tone={tone}
      className={clsx(
        "cliply-badge inline-flex h-6 items-center rounded-md px-2 text-xs font-medium",
        toneClass[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}
