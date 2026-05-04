import { Code2, FileText, Image, Link2, Pin } from "lucide-react";
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
  const Icon = iconByType[item.type] ?? FileText;

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
        "flex min-h-[92px] w-full cursor-pointer items-start gap-3 rounded-xl border p-4 text-left transition duration-150 active:scale-[0.995]",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
        selected
          ? "border-[color:var(--cliply-accent)] bg-[color:var(--cliply-accent-soft)] shadow-[var(--cliply-shadow-selected)]"
          : "border-[color:var(--cliply-border)] bg-white/75 shadow-sm hover:bg-white hover:shadow-[var(--cliply-shadow-card)]",
      )}
    >
      <span
        className={clsx(
          "grid size-[52px] shrink-0 place-items-center rounded-xl border border-[color:var(--cliply-border)] bg-white",
          item.type === "code" && "bg-indigo-50 text-indigo-700",
          item.type === "link" && "bg-teal-50 text-teal-700",
          item.type === "text" && "bg-slate-100 text-slate-600",
          item.type === "image" && "bg-amber-50 text-amber-700",
        )}
      >
        {item.type === "image" && item.thumbnailUrl ? (
          <img
            src={item.thumbnailUrl}
            alt={item.imageAlt ?? item.title}
            className="size-full rounded-[11px] object-cover"
          />
        ) : (
          <Icon className="size-5" />
        )}
      </span>
      <span className="min-w-0 flex-1">
        <span className="flex items-center justify-between gap-2">
          <span className="truncate text-[13px] text-[color:var(--cliply-faint)]">
            {typeLabel[item.type]} · {item.sourceApp}
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
              "grid size-7 shrink-0 place-items-center rounded-lg text-[color:var(--cliply-faint)] transition hover:bg-white/80 hover:text-[color:var(--cliply-accent)]",
              item.isPinned && "text-[color:var(--cliply-accent)]",
            )}
          >
            <Pin
              className={clsx(
                "size-3.5",
                item.isPinned && "fill-[color:var(--cliply-accent)]",
              )}
            />
          </button>
        </span>
        <span className="mt-1 block truncate text-[15px] font-medium leading-6 text-[color:var(--cliply-text)]">
          {item.previewText}
        </span>
        <span className="mt-1 flex min-w-0 items-center gap-2 text-[13px] text-[color:var(--cliply-faint)]">
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
    </article>
  );
}
