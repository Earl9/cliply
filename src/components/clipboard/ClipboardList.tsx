import { ClipboardListItem } from "@/components/clipboard/ClipboardListItem";
import { EmptyState } from "@/components/clipboard/EmptyState";
import type { ClipboardFilter, ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardListProps = {
  items: ClipboardItem[];
  totalCount: number;
  selectedId: string | null;
  query: string;
  filter: ClipboardFilter;
  loading?: boolean;
  errorMessage?: string | null;
  onSelectItem: (id: string) => void;
  onTogglePin: (id: string) => void;
};

export function ClipboardList({
  items,
  totalCount,
  selectedId,
  query,
  filter,
  loading = false,
  errorMessage = null,
  onSelectItem,
  onTogglePin,
}: ClipboardListProps) {
  const footerText = getFooterText({ query, filter, shownCount: items.length, totalCount });

  return (
    <section className="flex min-h-0 flex-col overflow-hidden rounded-[14px] border border-[color:var(--cliply-border)] bg-white shadow-[var(--cliply-shadow-card)] ring-1 ring-white/80">
      <div className="flex h-14 shrink-0 items-center justify-between border-b border-[color:var(--cliply-border-soft)] bg-white px-6">
        <span className="text-[17px] font-semibold text-[color:var(--cliply-text)]">剪贴板历史</span>
        <span className="text-sm text-[color:var(--cliply-muted)]">{items.length} 条</span>
      </div>
      <div className="cliply-scrollbar min-h-0 flex-1 overflow-auto bg-white px-[14px] pt-[14px]">
        {loading ? (
          <div className="space-y-2.5">
            {Array.from({ length: 5 }).map((_, index) => (
              <div
                key={index}
                className="h-[104px] animate-pulse rounded-[14px] border border-[#e3e9f1] bg-white p-4"
              >
                <div className="mb-3 h-3 w-28 rounded bg-slate-200" />
                <div className="mb-3 h-4 w-4/5 rounded bg-slate-200" />
                <div className="h-3 w-40 rounded bg-slate-100" />
              </div>
            ))}
          </div>
        ) : errorMessage ? (
          <EmptyState
            title="读取历史失败"
            description={errorMessage}
          />
        ) : items.length ? (
          <div className="space-y-2.5">
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
            title={query ? "没有找到匹配内容" : filter === "pinned" ? "还没有固定内容" : "还没有剪贴板记录"}
            description={
              query
                ? "试试换个关键词，或者切换到“全部”。"
                : filter === "pinned"
                  ? "点击记录右侧的图钉，可以把常用内容固定在这里。"
                  : "复制一段文字、链接或图片后，它会出现在这里。"
            }
          />
        )}
      </div>
      <div className="flex h-11 shrink-0 items-center border-t border-[color:var(--cliply-border-soft)] bg-white px-6 text-sm text-[color:var(--cliply-muted)]">
        {footerText}
      </div>
    </section>
  );
}

function getFooterText({
  query,
  filter,
  shownCount,
  totalCount,
}: {
  query: string;
  filter: ClipboardFilter;
  shownCount: number;
  totalCount: number;
}) {
  if (query.trim()) {
    return `找到 ${shownCount} 条匹配结果`;
  }

  if (filter === "pinned") {
    return `共 ${shownCount} 条固定记录`;
  }

  return `共 ${totalCount} 条记录`;
}
