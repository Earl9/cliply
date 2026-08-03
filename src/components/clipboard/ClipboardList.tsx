import { ClipboardListItem } from "@/components/clipboard/ClipboardListItem";
import { EmptyState } from "@/components/clipboard/EmptyState";
import type { ClipboardFilter, ClipboardItem } from "@/lib/clipboardTypes";
import type { MouseEvent } from "react";

type ClipboardListProps = {
  items: ClipboardItem[];
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
  return (
    <section className="cliply-scrollbar min-h-0 overflow-y-auto py-1.5">
      {loading ? (
        <div>
          {Array.from({ length: 7 }).map((_, index) => (
            <div key={index} className="mx-1.5 flex h-[52px] items-center gap-2.5 px-2.5">
              <div className="size-7 shrink-0 animate-pulse rounded-[4px] bg-[color:var(--cliply-muted-bg)]" />
              <div className="min-w-0 flex-1">
                <div className="mb-1.5 h-3 w-4/5 animate-pulse rounded bg-[color:var(--cliply-muted-bg)]" />
                <div className="h-2.5 w-28 animate-pulse rounded bg-[color:var(--cliply-muted-bg)]" />
              </div>
            </div>
          ))}
        </div>
      ) : errorMessage ? (
        <EmptyState title="读取历史失败" description={errorMessage} />
      ) : items.length ? (
        items.map((item) => (
          <ClipboardListItem
            key={item.id}
            item={item}
            selected={item.id === selectedId}
            onSelect={() => onSelectItem(item.id)}
            onTogglePin={() => onTogglePin(item.id)}
            onPaste={() => onPasteItem(item.id)}
            onContextMenu={(event) => onItemContextMenu(event, item)}
          />
        ))
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
    </section>
  );
}
