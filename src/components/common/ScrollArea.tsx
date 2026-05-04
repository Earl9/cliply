import type { HTMLAttributes } from "react";
import { clsx } from "clsx";

export function ScrollArea({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return <div className={clsx("cliply-scrollbar min-h-0 overflow-auto", className)} {...props} />;
}
