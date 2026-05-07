import type { ButtonHTMLAttributes, ReactNode } from "react";
import { clsx } from "clsx";

type IconButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  label: string;
  children: ReactNode;
  variant?: "ghost" | "danger" | "soft";
};

export function IconButton({
  label,
  children,
  className,
  variant = "ghost",
  ...props
}: IconButtonProps) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      className={clsx(
        "grid size-8 place-items-center rounded-lg border border-transparent text-[color:var(--cliply-muted)] transition",
        "hover:bg-[color:var(--cliply-muted-bg)] hover:text-[color:var(--cliply-text)] active:bg-[color:var(--cliply-card)]",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-focus-ring)]",
        variant === "soft" && "bg-[color:var(--cliply-accent-50)] text-[color:var(--cliply-accent-strong)]",
        variant === "danger" && "hover:bg-[rgba(220,38,38,0.08)] hover:text-[color:var(--cliply-danger)]",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
