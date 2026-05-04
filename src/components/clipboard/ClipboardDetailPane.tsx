import { ClipboardActions } from "@/components/clipboard/ClipboardActions";
import { ClipboardMetadata } from "@/components/clipboard/ClipboardMetadata";
import { ClipboardPreview } from "@/components/clipboard/ClipboardPreview";

export function ClipboardDetailPane() {
  return (
    <section className="flex min-w-0 flex-1 flex-col">
      <div className="flex h-11 shrink-0 items-center justify-between border-b border-[color:var(--cliply-border)] px-4">
        <div>
          <h2 className="text-sm font-semibold text-[color:var(--cliply-text)]">Preview</h2>
        </div>
        <span className="rounded-md bg-[color:var(--cliply-accent-soft)] px-2 py-1 text-xs font-medium text-[color:var(--cliply-accent-strong)]">
          Mock
        </span>
      </div>
      <div className="cliply-scrollbar min-h-0 flex-1 overflow-auto p-4">
        <ClipboardPreview />
        <ClipboardMetadata />
      </div>
      <ClipboardActions />
    </section>
  );
}
