import { Code2, FileText, Image, Link2, Pin, Shield } from "lucide-react";
import type { MouseEvent } from "react";
import { clsx } from "clsx";
import type { ClipboardItem, ClipboardItemType } from "@/lib/clipboardTypes";
import { formatCopiedTime, formatRelativeTime } from "@/lib/formatTime";

type ClipboardListItemProps = {
  item: ClipboardItem;
  selected?: boolean;
  onSelect: () => void;
  onTogglePin: () => void;
  onPaste: () => void;
  onContextMenu: (event: MouseEvent<HTMLElement>) => void;
};

const iconByType: Record<ClipboardItemType, typeof FileText> = {
  code: Code2,
  link: Link2,
  text: FileText,
  image: Image,
};

const typeLabel: Record<ClipboardItemType, string> = {
  code: "代码",
  link: "链接",
  text: "文本",
  image: "图片",
};

export function ClipboardListItem({
  item,
  selected,
  onSelect,
  onTogglePin,
  onPaste,
  onContextMenu,
}: ClipboardListItemProps) {
  const sensitive = Boolean(item.isRedacted);
  const Icon = sensitive ? Shield : iconByType[item.type] ?? FileText;

  return (
    <article
      tabIndex={0}
      onClick={onSelect}
      onDoubleClick={(event) => {
        event.preventDefault();
        onPaste();
      }}
      onContextMenu={onContextMenu}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect();
        }
      }}
      className={clsx(
        "group grid h-[78px] w-full cursor-pointer grid-cols-[44px_minmax(0,1fr)_24px] items-center gap-3 rounded-[12px] border px-3 py-2.5 text-left transition duration-150 active:scale-[0.995]",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
        selected
          ? "border-[color:var(--cliply-accent)] bg-[color:var(--cliply-accent-50)] shadow-[var(--cliply-shadow-selected)]"
          : "border-[#e3e9f1] bg-white shadow-none hover:-translate-y-px hover:border-[#d5deea] hover:shadow-[var(--cliply-shadow-card-hover)]",
      )}
    >
      <span
        className={clsx(
          "grid size-[42px] shrink-0 place-items-center overflow-hidden rounded-[10px] border border-[#e3e9f1] bg-white",
          item.type === "code" && "bg-indigo-50 text-indigo-700",
          item.type === "link" && "bg-teal-50 text-teal-700",
          item.type === "text" && "bg-slate-100 text-slate-600",
          item.type === "image" && "bg-amber-50 text-amber-700",
          sensitive && "bg-amber-50 text-amber-700",
        )}
      >
        {item.type === "image" && item.thumbnailUrl && !sensitive ? (
          <img
            src={item.thumbnailUrl}
            alt={item.imageAlt ?? item.title}
            className="size-full rounded-[9px] object-contain"
          />
        ) : (
          <Icon className="size-4" />
        )}
      </span>
      <span className="min-w-0">
        <span className="block truncate text-xs font-medium leading-4 text-[color:var(--cliply-faint)]">
          {sensitive ? "隐私" : typeLabel[item.type]} · {item.sourceApp}
        </span>
        <span className="mt-0.5 block truncate text-[15px] font-semibold leading-5 text-[color:var(--cliply-text)]">
          {sensitive ? "已隐藏敏感内容" : item.previewText}
        </span>
        <span className="mt-0.5 flex min-w-0 items-center gap-1.5 text-xs leading-4 text-[color:var(--cliply-faint)]">
          <span>{formatCopiedTime(item.copiedAt)}</span>
          <span>·</span>
          <span>{formatRelativeTime(item.copiedAt)}</span>
          {item.tags.slice(0, 1).map((tag) => (
            <span key={tag} className="truncate">
              · #{tag}
            </span>
          ))}
        </span>
      </span>
      <button
        type="button"
        aria-label={item.isPinned ? "取消固定" : "固定"}
        title={item.isPinned ? "取消固定" : "固定"}
        onClick={(event) => {
          event.stopPropagation();
          onTogglePin();
        }}
        onDoubleClick={(event) => {
          event.preventDefault();
          event.stopPropagation();
        }}
        className={clsx(
          "grid size-6 shrink-0 place-items-center rounded-md text-[#a5afbd] opacity-30 transition hover:bg-white hover:text-[color:var(--cliply-muted)] hover:opacity-100 group-hover:opacity-80",
          item.isPinned && "text-[color:var(--cliply-accent-strong)] opacity-100",
        )}
      >
        <Pin
          className={clsx(
            "size-4",
            item.isPinned && "fill-[color:var(--cliply-accent-strong)]",
          )}
        />
      </button>
    </article>
  );
}
