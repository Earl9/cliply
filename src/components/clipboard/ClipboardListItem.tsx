import { Code2, FileText, Image, Link2, Pin, Shield } from "lucide-react";
import { clsx } from "clsx";
import type { ClipboardItem, ClipboardItemType } from "@/lib/clipboardTypes";
import { formatCopiedTime, formatRelativeTime } from "@/lib/formatTime";

type ClipboardListItemProps = {
  item: ClipboardItem;
  selected?: boolean;
  onSelect: () => void;
  onTogglePin: () => void;
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
}: ClipboardListItemProps) {
  const sensitive = item.sensitiveScore >= 50;
  const Icon = sensitive ? Shield : iconByType[item.type] ?? FileText;

  return (
    <article
      tabIndex={0}
      onClick={onSelect}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect();
        }
      }}
      className={clsx(
        "group grid h-[104px] w-full cursor-pointer grid-cols-[56px_minmax(0,1fr)_28px] items-center gap-4 rounded-[14px] border px-[18px] py-4 text-left transition duration-150 active:scale-[0.995]",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
        selected
          ? "border-[color:var(--cliply-accent)] bg-[color:var(--cliply-accent-50)] shadow-[var(--cliply-shadow-selected)]"
          : "border-[#e7ebf2] bg-white shadow-none hover:-translate-y-px hover:border-[#dde3ec] hover:shadow-[0_8px_20px_rgba(15,23,42,0.06)]",
      )}
    >
      <span
        className={clsx(
          "grid size-[52px] shrink-0 place-items-center rounded-xl border border-[#e7ebf2] bg-white",
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
            className="size-full rounded-[11px] object-cover"
          />
        ) : (
          <Icon className="size-5" />
        )}
      </span>
      <span className="min-w-0">
        <span className="block truncate text-sm leading-5 text-[color:var(--cliply-placeholder)]">
          {sensitive ? "隐私" : typeLabel[item.type]} · {item.sourceApp}
        </span>
        <span className="mt-1 block truncate text-[17px] font-semibold leading-6 text-[color:var(--cliply-text)]">
          {sensitive ? "已隐藏敏感内容" : item.previewText}
        </span>
        <span className="mt-1 flex min-w-0 items-center gap-2 text-sm leading-5 text-[color:var(--cliply-placeholder)]">
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
        className={clsx(
          "grid size-7 shrink-0 place-items-center rounded-lg text-[#9aa3b2] opacity-55 transition hover:bg-white hover:text-[color:var(--cliply-muted)] hover:opacity-100 group-hover:opacity-100",
          item.isPinned && "text-[color:var(--cliply-accent-strong)] opacity-100",
        )}
      >
        <Pin
          className={clsx(
            "size-[18px]",
            item.isPinned && "fill-[color:var(--cliply-accent-strong)]",
          )}
        />
      </button>
    </article>
  );
}
