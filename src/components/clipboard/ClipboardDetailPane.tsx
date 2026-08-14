import { Code2, FileText, Image, Link2, MoreHorizontal, Pin, ShieldAlert } from "lucide-react";
import { memo, type MouseEvent } from "react";
import { IconButton } from "@/components/common/IconButton";
import { ClipboardActions } from "@/components/clipboard/ClipboardActions";
import { ClipboardMetadata } from "@/components/clipboard/ClipboardMetadata";
import { ClipboardPreview } from "@/components/clipboard/ClipboardPreview";
import { EmptyState } from "@/components/clipboard/EmptyState";
import { OverlayScrollArea } from "@/components/common/OverlayScrollArea";
import type { ClipboardActionKind, ClipboardItem } from "@/lib/clipboardTypes";
import { formatRelativeTime } from "@/lib/formatTime";

const iconByType = {
  code: Code2,
  image: Image,
  link: Link2,
  text: FileText,
};

const typeLabels = {
  code: "代码",
  image: "图片",
  link: "链接",
  text: "文本",
};

type ClipboardDetailPaneProps = {
  item: ClipboardItem | null;
  onAction: (action: ClipboardActionKind) => void;
  onContextMenu: (event: MouseEvent<HTMLElement>, item: ClipboardItem | null) => void;
  onOpenImage: (item: ClipboardItem) => void;
};

function ClipboardDetailPaneComponent({
  item,
  onAction,
  onContextMenu,
  onOpenImage,
}: ClipboardDetailPaneProps) {
  const TypeIcon = item ? iconByType[item.type] : FileText;

  return (
    <section
      className="cliply-detail-pane grid min-h-0 min-w-0 grid-rows-[48px_minmax(0,1fr)_auto]"
      onContextMenu={(event) => onContextMenu(event, item)}
    >
      <header className="cliply-detail-header flex h-12 shrink-0 items-center justify-between gap-3 px-4">
        <div className="flex min-w-0 items-center gap-3">
          <span className="cliply-detail-type-icon grid size-7 shrink-0 place-items-center rounded-md">
            <TypeIcon className="size-4" />
          </span>
          <div className="min-w-0">
            <h2 className="min-w-0 truncate text-[13.5px] font-semibold text-[color:var(--cliply-text)]">
              {item ? item.title : "内容详情"}
            </h2>
            {item ? (
              <p className="cliply-caption mt-0.5 truncate text-[10.5px] text-[color:var(--cliply-faint)]">
                {typeLabels[item.type]} · {item.sourceApp} · {formatRelativeTime(item.copiedAt)}
              </p>
            ) : null}
          </div>
        </div>
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
            className="size-8"
            disabled={!item}
            onClick={(event) => onContextMenu(event, item)}
          >
            <MoreHorizontal className="size-4" />
          </IconButton>
        </div>
      </header>
      {item ? (
        <>
          <OverlayScrollArea
            scrollbarLabel="滚动详情内容"
            className="min-h-0"
            viewportClassName="cliply-detail-content p-5 pr-6"
          >
            <ClipboardPreview item={item} onOpenImage={onOpenImage} />
            <ClipboardMetadata item={item} />
          </OverlayScrollArea>
          <ClipboardActions item={item} onAction={onAction} />
        </>
      ) : (
        <div className="min-h-0">
          <EmptyState title="未选择记录" description="请从记录列表中选择一项。" />
        </div>
      )}
    </section>
  );
}

export const ClipboardDetailPane = memo(ClipboardDetailPaneComponent);
