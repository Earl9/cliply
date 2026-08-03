import { MoreHorizontal, Pin, ShieldAlert } from "lucide-react";
import type { MouseEvent } from "react";
import { IconButton } from "@/components/common/IconButton";
import { ClipboardActions } from "@/components/clipboard/ClipboardActions";
import { ClipboardMetadata } from "@/components/clipboard/ClipboardMetadata";
import { ClipboardPreview } from "@/components/clipboard/ClipboardPreview";
import { EmptyState } from "@/components/clipboard/EmptyState";
import type { ClipboardActionKind, ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardDetailPaneProps = {
  item: ClipboardItem | null;
  onAction: (action: ClipboardActionKind) => void;
  onContextMenu: (event: MouseEvent<HTMLElement>, item: ClipboardItem | null) => void;
  onOpenImage: (item: ClipboardItem) => void;
};

export function ClipboardDetailPane({
  item,
  onAction,
  onContextMenu,
  onOpenImage,
}: ClipboardDetailPaneProps) {
  return (
    <section
      className="cliply-detail-pane grid min-h-0 min-w-0 grid-rows-[44px_1fr_auto] border-l border-[color:var(--cliply-border)]"
      onContextMenu={(event) => onContextMenu(event, item)}
    >
      <header className="flex h-11 shrink-0 items-center justify-between gap-2 border-b border-[color:var(--cliply-border-soft)] px-3.5">
        <h2 className="cliply-title min-w-0 truncate text-[14px] font-semibold text-[color:var(--cliply-text)]">
          {item ? item.sourceApp : "内容详情"}
        </h2>
        <div className="flex shrink-0 items-center gap-1">
          {item?.isRedacted ? (
            <span
              title="敏感内容已隐藏"
              className="grid size-7 place-items-center text-[color:var(--cliply-warning)]"
            >
              <ShieldAlert className="size-4" />
            </span>
          ) : null}
          {item?.isPinned ? (
            <span
              title="已固定"
              className="grid size-7 place-items-center text-[color:var(--cliply-accent)]"
            >
              <Pin className="size-4 fill-current" />
            </span>
          ) : null}
          <IconButton
            label="更多"
            className="size-7"
            disabled={!item}
            onClick={(event) => onContextMenu(event, item)}
          >
            <MoreHorizontal className="size-4" />
          </IconButton>
        </div>
      </header>
      {item ? (
        <>
          <div className="cliply-scrollbar min-h-0 overflow-y-auto px-3.5 py-3.5">
            <ClipboardPreview item={item} onOpenImage={onOpenImage} />
            <ClipboardMetadata item={item} />
          </div>
          <ClipboardActions item={item} onAction={onAction} />
        </>
      ) : (
        <div className="min-h-0">
          <EmptyState title="没有选中内容" description="从左侧列表选择一条记录。" />
        </div>
      )}
    </section>
  );
}
