import { MoreHorizontal } from "lucide-react";
import type { MouseEvent } from "react";
import { Badge } from "@/components/common/Badge";
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
      className="grid min-h-0 min-w-0 grid-rows-[50px_1fr_auto] overflow-hidden rounded-[12px] border border-[color:var(--cliply-border)] bg-[color:var(--cliply-card)] shadow-[var(--cliply-shadow-card)]"
      onContextMenu={(event) => onContextMenu(event, item)}
    >
      <header className="flex h-[50px] shrink-0 items-center justify-between border-b border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-card)] px-4">
        <div>
          <h2 className="text-base font-semibold text-[color:var(--cliply-text)]">
            {item ? `${typeLabel[item.type]} · ${item.sourceApp}` : "内容详情"}
          </h2>
        </div>
        <div className="flex items-center gap-2">
          {item?.isRedacted ? (
            <Badge tone="amber">隐私保护</Badge>
          ) : null}
          {item?.isPinned ? <Badge tone="accent">已固定</Badge> : null}
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
          <div className="cliply-scrollbar min-h-0 overflow-y-auto bg-[color:var(--cliply-card)] px-4 pb-3 pt-3">
            <ClipboardPreview item={item} onOpenImage={onOpenImage} />
            <ClipboardMetadata item={item} />
          </div>
          <ClipboardActions item={item} onAction={onAction} />
        </>
      ) : (
        <div className="min-h-0 p-5">
          <EmptyState title="没有选中内容" description="从左侧历史列表选择一条记录。" />
        </div>
      )}
    </section>
  );
}

const typeLabel = {
  code: "代码",
  image: "图片",
  link: "链接",
  text: "文本",
} satisfies Record<ClipboardItem["type"], string>;
