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
        "grid size-8 place-items-center rounded-md border border-transparent text-[color:var(--cliply-muted)] transition",
        "hover:border-[color:var(--cliply-border)] hover:bg-white/80 hover:text-[color:var(--cliply-text)]",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
        variant === "soft" && "bg-white/70 text-[color:var(--cliply-accent)]",
        variant === "danger" && "hover:bg-rose-50 hover:text-rose-600",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
