import type { HTMLAttributes } from "react";
import { clsx } from "clsx";

export function Card({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={clsx(
        "rounded-lg border border-[color:var(--cliply-border)] bg-[color:var(--cliply-card)] shadow-sm",
        className,
      )}
      {...props}
    />
  );
}
