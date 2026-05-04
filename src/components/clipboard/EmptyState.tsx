import { Clipboard } from "lucide-react";

export function EmptyState() {
  return (
    <div className="grid min-h-48 place-items-center rounded-lg border border-dashed border-[color:var(--cliply-border)] bg-white/45 p-6 text-center">
      <div>
        <Clipboard className="mx-auto mb-3 size-8 text-[color:var(--cliply-faint)]" />
        <p className="text-sm font-semibold text-[color:var(--cliply-text)]">No clipboard items yet</p>
        <p className="mt-1 max-w-64 text-sm text-[color:var(--cliply-muted)]">
          Copy text, links, code, or images and they will appear here.
        </p>
      </div>
    </div>
  );
}
