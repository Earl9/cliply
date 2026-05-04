import { MoreHorizontal } from "lucide-react";
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
};

export function ClipboardDetailPane({ item, onAction }: ClipboardDetailPaneProps) {
  return (
    <section className="flex min-w-0 flex-col overflow-hidden rounded-[14px] border border-[color:var(--cliply-border)] bg-[color:var(--cliply-card)] shadow-[var(--cliply-shadow-card)]">
      <div className="flex h-14 shrink-0 items-center justify-between border-b border-[color:var(--cliply-border)] px-5">
        <div>
          <h2 className="text-[15px] font-semibold text-[color:var(--cliply-text)]">
            {item ? `${typeLabel[item.type]} · ${item.sourceApp}` : "内容详情"}
          </h2>
        </div>
        <div className="flex items-center gap-2">
          {item?.isPinned ? <Badge tone="accent">已固定</Badge> : null}
          <IconButton label="更多">
            <MoreHorizontal className="size-4" />
          </IconButton>
        </div>
      </div>
      {item ? (
        <>
          <div className="cliply-scrollbar min-h-0 flex-1 overflow-auto p-5">
            <ClipboardPreview item={item} />
            <ClipboardMetadata item={item} />
          </div>
          <ClipboardActions item={item} onAction={onAction} />
        </>
      ) : (
        <div className="p-5">
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
