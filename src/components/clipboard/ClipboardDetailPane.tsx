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
    <section className="grid min-h-0 min-w-0 grid-rows-[64px_1fr_auto] overflow-hidden rounded-[14px] border border-[color:var(--cliply-border)] bg-white shadow-[var(--cliply-shadow-card)] ring-1 ring-white/80">
      <header className="flex h-16 shrink-0 items-center justify-between border-b border-[color:var(--cliply-border-soft)] bg-white px-6">
        <div>
          <h2 className="text-lg font-semibold text-[color:var(--cliply-text)]">
            {item ? `${typeLabel[item.type]} · ${item.sourceApp}` : "内容详情"}
          </h2>
        </div>
        <div className="flex items-center gap-2">
          {item?.sensitiveScore && item.sensitiveScore >= 50 ? (
            <Badge tone="amber">隐私保护</Badge>
          ) : null}
          {item?.isPinned ? <Badge tone="accent">已固定</Badge> : null}
          <IconButton label="更多" className="size-8">
            <MoreHorizontal className="size-[18px]" />
          </IconButton>
        </div>
      </header>
      {item ? (
        <>
          <div className="cliply-scrollbar min-h-0 overflow-y-auto bg-white px-6 pb-4 pt-4">
            <ClipboardPreview item={item} />
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
