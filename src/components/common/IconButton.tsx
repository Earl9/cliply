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
        "grid size-9 place-items-center rounded-[10px] border border-transparent text-[color:var(--cliply-muted)] transition",
        "hover:bg-slate-900/[0.06] hover:text-[color:var(--cliply-text)] active:bg-slate-900/[0.10]",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[rgba(124,92,255,0.45)]",
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
