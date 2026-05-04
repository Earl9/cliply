import { Clipboard } from "lucide-react";

type EmptyStateProps = {
  title?: string;
  description?: string;
};

export function EmptyState({
  title = "No clipboard items yet",
  description = "Copy text, links, code, or images and they will appear here.",
}: EmptyStateProps) {
  return (
    <div className="grid min-h-48 place-items-center rounded-lg border border-dashed border-[color:var(--cliply-border)] bg-white/45 p-6 text-center">
      <div>
        <Clipboard className="mx-auto mb-3 size-8 text-[color:var(--cliply-faint)]" />
        <p className="text-sm font-semibold text-[color:var(--cliply-text)]">{title}</p>
        <p className="mt-1 max-w-64 text-sm text-[color:var(--cliply-muted)]">
          {description}
        </p>
      </div>
    </div>
  );
}
