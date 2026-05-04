import { ClipboardListItem } from "@/components/clipboard/ClipboardListItem";
import { EmptyState } from "@/components/clipboard/EmptyState";
import type { ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardListProps = {
  items: ClipboardItem[];
  totalCount: number;
  selectedId: string | null;
  query: string;
  onSelectItem: (id: string) => void;
  onTogglePin: (id: string) => void;
};

export function ClipboardList({
  items,
  totalCount,
  selectedId,
  query,
  onSelectItem,
  onTogglePin,
}: ClipboardListProps) {
  return (
    <section className="flex min-h-0 w-[42%] min-w-[290px] flex-col border-r border-[color:var(--cliply-border)]">
      <div className="flex h-11 shrink-0 items-center justify-between px-4 text-xs font-semibold uppercase tracking-normal text-[color:var(--cliply-muted)]">
        <span>History</span>
        <span>{items.length} shown</span>
      </div>
      <div className="cliply-scrollbar min-h-0 flex-1 overflow-auto px-3 pb-3">
        {items.length ? (
          <div className="space-y-2">
            {items.map((item) => (
              <ClipboardListItem
                key={item.id}
                item={item}
                selected={item.id === selectedId}
                onSelect={() => onSelectItem(item.id)}
                onTogglePin={() => onTogglePin(item.id)}
              />
            ))}
          </div>
        ) : (
          <EmptyState
            title={query ? "No matching content" : "No clipboard items yet"}
            description={
              query
                ? "Try another keyword or switch back to All."
                : "Copy text, links, code, or images and they will appear here."
            }
          />
        )}
      </div>
      <div className="shrink-0 border-t border-[color:var(--cliply-border)] px-4 py-2 text-xs text-[color:var(--cliply-muted)]">
        Total {totalCount} mock records
      </div>
    </section>
  );
}
