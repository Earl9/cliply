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
        "cliply-interactive grid size-8 place-items-center rounded-[6px] text-[color:var(--cliply-muted)]",
        "hover:bg-[color:var(--cliply-muted-bg)] hover:text-[color:var(--cliply-text)]",
        "focus-visible:outline focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-[color:var(--cliply-accent)]",
        "disabled:cursor-not-allowed disabled:text-[color:var(--cliply-disabled)] disabled:hover:bg-transparent",
        variant === "soft" && "bg-[color:var(--cliply-accent-soft)] text-[color:var(--cliply-accent)]",
        variant === "danger" && "hover:bg-[color:var(--cliply-danger-soft)] hover:text-[color:var(--cliply-danger)]",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
