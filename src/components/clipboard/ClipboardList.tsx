import { memo, useCallback, useEffect, useLayoutEffect, useRef, useState, type MouseEvent } from "react";
import { ClipboardListItem } from "@/components/clipboard/ClipboardListItem";
import { ClipboardFilterTabs } from "@/components/clipboard/ClipboardFilterTabs";
import { EmptyState } from "@/components/clipboard/EmptyState";
import {
  OverlayScrollArea,
  type OverlayScrollMetrics,
} from "@/components/common/OverlayScrollArea";
import type { ClipboardFilter, ClipboardItem } from "@/lib/clipboardTypes";

const ROW_HEIGHT = 68;
const ROW_GAP = 8;
const ROW_STRIDE = ROW_HEIGHT + ROW_GAP;
const OVERSCAN_ROWS = 8;

type VirtualRange = {
  start: number;
  end: number;
};

type ClipboardListProps = {
  items: ClipboardItem[];
  selectedId: string | null;
  hasQuery: boolean;
  filter: ClipboardFilter;
  loading?: boolean;
  errorMessage?: string | null;
  onFilterChange: (filter: ClipboardFilter) => void;
  onSelectItem: (id: string) => void;
  onTogglePin: (id: string) => void;
  onPasteItem: (id: string) => void;
  onItemContextMenu: (event: MouseEvent<HTMLElement>, item: ClipboardItem) => void;
};

function calculateVirtualRange(
  itemCount: number,
  { clientHeight, scrollTop }: Pick<OverlayScrollMetrics, "clientHeight" | "scrollTop">,
): VirtualRange {
  if (itemCount === 0) {
    return { start: 0, end: 0 };
  }

  const viewportHeight = Math.max(clientHeight, ROW_STRIDE);
  const start = Math.max(0, Math.floor(scrollTop / ROW_STRIDE) - OVERSCAN_ROWS);
  const end = Math.min(
    itemCount,
    Math.ceil((scrollTop + viewportHeight) / ROW_STRIDE) + OVERSCAN_ROWS,
  );
  return { start, end };
}

function ClipboardListComponent({
  items,
  selectedId,
  hasQuery,
  filter,
  loading = false,
  errorMessage = null,
  onFilterChange,
  onSelectItem,
  onTogglePin,
  onPasteItem,
  onItemContextMenu,
}: ClipboardListProps) {
  const viewportRef = useRef<HTMLDivElement>(null);
  const initialVirtualRange = useRef<VirtualRange>({
    start: 0,
    end: Math.min(items.length, OVERSCAN_ROWS * 2 + 1),
  });
  const virtualRangeRef = useRef(initialVirtualRange.current);
  const [virtualRange, setVirtualRange] = useState<VirtualRange>(initialVirtualRange.current);
  const [relativeTimeNow, setRelativeTimeNow] = useState(() => Date.now());

  const updateVirtualRange = useCallback(
    (metrics: Pick<OverlayScrollMetrics, "clientHeight" | "scrollTop">) => {
      const nextRange = calculateVirtualRange(items.length, metrics);
      const currentRange = virtualRangeRef.current;
      if (currentRange.start === nextRange.start && currentRange.end === nextRange.end) {
        return;
      }

      virtualRangeRef.current = nextRange;
      setVirtualRange(nextRange);
    },
    [items.length],
  );

  useLayoutEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }

    updateVirtualRange({
      clientHeight: viewport.clientHeight,
      scrollTop: viewport.scrollTop,
    });
  }, [items.length, updateVirtualRange]);

  useLayoutEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport || !selectedId) {
      return;
    }

    const selectedIndex = items.findIndex((item) => item.id === selectedId);
    if (selectedIndex === -1) {
      return;
    }

    const rowTop = selectedIndex * ROW_STRIDE;
    const rowBottom = rowTop + ROW_STRIDE;
    const viewportTop = viewport.scrollTop;
    const viewportBottom = viewportTop + viewport.clientHeight;

    let nextScrollTop = viewportTop;
    if (rowTop < viewportTop) {
      nextScrollTop = rowTop;
    } else if (rowBottom > viewportBottom) {
      nextScrollTop = Math.max(0, rowBottom - viewport.clientHeight);
    }

    if (nextScrollTop !== viewportTop) {
      viewport.scrollTop = nextScrollTop;
      updateVirtualRange({
        clientHeight: viewport.clientHeight,
        scrollTop: nextScrollTop,
      });
    }
  }, [items, selectedId, updateVirtualRange]);

  useEffect(() => {
    if (!items.length) {
      return;
    }

    const interval = window.setInterval(() => setRelativeTimeNow(Date.now()), 60_000);
    return () => window.clearInterval(interval);
  }, [items.length]);

  const visibleItems = items.slice(virtualRange.start, virtualRange.end);

  return (
    <section className="cliply-list-pane grid min-h-0 min-w-0 grid-rows-[48px_minmax(0,1fr)] overflow-hidden border-r border-[color:var(--cliply-border)]">
      <header className="cliply-history-toolbar min-w-0 overflow-hidden px-2.5">
        <ClipboardFilterTabs filter={filter} onFilterChange={onFilterChange} />
      </header>
      <OverlayScrollArea
        scrollbarLabel="滚动历史记录"
        className="min-h-0"
        viewportClassName="cliply-history-list px-1.5 pb-2 pr-3"
        viewportRef={viewportRef}
        onViewportMetricsChange={updateVirtualRange}
      >
        {loading ? (
          <div className="space-y-1 pt-1">
            {Array.from({ length: 7 }).map((_, index) => (
              <div key={index} className="flex h-[68px] items-center gap-3 rounded-md px-3">
                <div className="size-7 shrink-0 animate-pulse rounded-md bg-[color:var(--cliply-muted-bg)]" />
                <div className="min-w-0 flex-1">
                  <div className="mb-2 h-3 w-4/5 animate-pulse rounded-sm bg-[color:var(--cliply-muted-bg)]" />
                  <div className="h-2.5 w-28 animate-pulse rounded-sm bg-[color:var(--cliply-muted-bg)]" />
                </div>
              </div>
            ))}
          </div>
        ) : errorMessage ? (
          <EmptyState title="无法读取历史记录" description={errorMessage} />
        ) : items.length ? (
          <div
            role="list"
            className="relative min-w-0"
            style={{ height: items.length * ROW_STRIDE }}
          >
            {visibleItems.map((item, offset) => {
              const index = virtualRange.start + offset;
              return (
                <ClipboardListItem
                  key={item.id}
                  item={item}
                  selected={item.id === selectedId}
                  relativeTimeNow={relativeTimeNow}
                  position={index + 1}
                  setSize={items.length}
                  virtualOffset={index * ROW_STRIDE}
                  onSelectItem={onSelectItem}
                  onTogglePin={onTogglePin}
                  onPasteItem={onPasteItem}
                  onItemContextMenu={onItemContextMenu}
                />
              );
            })}
          </div>
        ) : (
          <EmptyState
            title={hasQuery ? "未找到匹配的记录" : filter === "pinned" ? "暂无固定记录" : "暂无剪贴板记录"}
            description={
              hasQuery
                ? "请更改搜索条件，或切换到“全部”。"
                : filter === "pinned"
                  ? "可使用记录右侧的固定按钮添加固定记录。"
                  : "复制文本、链接或图片后，记录将显示在此处。"
            }
          />
        )}
      </OverlayScrollArea>
    </section>
  );
}

export const ClipboardList = memo(ClipboardListComponent);
