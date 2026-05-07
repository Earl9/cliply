import { ClipboardListItem } from "@/components/clipboard/ClipboardListItem";
import { EmptyState } from "@/components/clipboard/EmptyState";
import type { ClipboardFilter, ClipboardItem } from "@/lib/clipboardTypes";
import type { MouseEvent } from "react";

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
  onPasteItem: (id: string) => void;
  onItemContextMenu: (event: MouseEvent<HTMLElement>, item: ClipboardItem) => void;
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
  onPasteItem,
  onItemContextMenu,
}: ClipboardListProps) {
  const footerText = getFooterText({ query, filter, shownCount: items.length, totalCount });

  return (
    <section className="flex min-h-0 flex-col overflow-hidden rounded-[12px] border border-[color:var(--cliply-border)] bg-white shadow-[var(--cliply-shadow-card)] ring-1 ring-white/80">
      <div className="flex h-11 shrink-0 items-center justify-between border-b border-[color:var(--cliply-border-soft)] bg-white px-4">
        <span className="text-[15px] font-semibold text-[color:var(--cliply-text)]">剪贴板历史</span>
        <span className="text-xs text-[color:var(--cliply-muted)]">{items.length} 条</span>
      </div>
      <div className="cliply-scrollbar min-h-0 flex-1 overflow-auto bg-white px-2.5 pt-2.5">
        {loading ? (
          <div className="space-y-2">
            {Array.from({ length: 5 }).map((_, index) => (
              <div
                key={index}
                className="h-[78px] animate-pulse rounded-[12px] border border-[#e3e9f1] bg-white p-3"
              >
                <div className="mb-2 h-2.5 w-24 rounded bg-slate-200" />
                <div className="mb-2 h-3.5 w-4/5 rounded bg-slate-200" />
                <div className="h-2.5 w-36 rounded bg-slate-100" />
              </div>
            ))}
          </div>
        ) : errorMessage ? (
          <EmptyState
            title="读取历史失败"
            description={errorMessage}
          />
        ) : items.length ? (
          <div className="space-y-2">
            {items.map((item) => (
              <ClipboardListItem
                key={item.id}
                item={item}
                selected={item.id === selectedId}
                onSelect={() => onSelectItem(item.id)}
                onTogglePin={() => onTogglePin(item.id)}
                onPaste={() => onPasteItem(item.id)}
                onContextMenu={(event) => onItemContextMenu(event, item)}
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
      <div className="flex h-9 shrink-0 items-center border-t border-[color:var(--cliply-border-soft)] bg-white px-4 text-xs text-[color:var(--cliply-muted)]">
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
