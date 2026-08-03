import type { ReactNode } from "react";
import { clsx } from "clsx";

type BadgeTone = "neutral" | "accent" | "teal" | "amber" | "rose";

type BadgeProps = {
  children: ReactNode;
  tone?: BadgeTone;
  className?: string;
};

// Quiet status labels: tone lives in the text color only, no filled chip.
const toneClass: Record<BadgeTone, string> = {
  neutral: "text-[color:var(--cliply-muted)]",
  accent: "text-[color:var(--cliply-accent)]",
  teal: "text-[color:var(--cliply-info)]",
  amber: "text-[color:var(--cliply-warning)]",
  rose: "text-[color:var(--cliply-danger)]",
};

export function Badge({ children, tone = "neutral", className }: BadgeProps) {
  return (
    <span
      data-tone={tone}
      className={clsx(
        "cliply-badge inline-flex items-center text-xs",
        toneClass[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}
