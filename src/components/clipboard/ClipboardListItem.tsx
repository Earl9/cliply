import { Code2, FileText, Image, Link2, Pin } from "lucide-react";
import { clsx } from "clsx";
import type { ClipboardItem, ClipboardItemType } from "@/lib/clipboardTypes";
import { formatRelativeTime } from "@/lib/formatTime";

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
  code: "Code",
  link: "Link",
  text: "Text",
  image: "Image",
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
        "flex w-full cursor-pointer items-start gap-3 rounded-lg border p-3 text-left transition",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
        selected
          ? "border-[color:var(--cliply-accent)] bg-[color:var(--cliply-accent-soft)] shadow-sm"
          : "border-transparent bg-white/52 hover:border-[color:var(--cliply-border)] hover:bg-white/78",
      )}
    >
      <span
        className={clsx(
          "grid size-9 shrink-0 place-items-center rounded-md",
          item.type === "code" && "bg-indigo-50 text-indigo-700",
          item.type === "link" && "bg-teal-50 text-teal-700",
          item.type === "text" && "bg-slate-100 text-slate-600",
          item.type === "image" && "bg-amber-50 text-amber-700",
        )}
      >
        <Icon className="size-4" />
      </span>
      <span className="min-w-0 flex-1">
        <span className="flex items-center justify-between gap-2">
          <span className="truncate text-sm font-semibold text-[color:var(--cliply-text)]">
            {item.sourceApp}
          </span>
          <span className="shrink-0 text-xs text-[color:var(--cliply-faint)]">
            {formatRelativeTime(item.copiedAt)}
          </span>
        </span>
        <span className="mt-1 line-clamp-2 text-sm leading-5 text-[color:var(--cliply-muted)]">
          {item.previewText}
        </span>
        <span className="mt-2 flex items-center justify-between gap-2">
          <span className="flex min-w-0 items-center gap-2">
          <span className="rounded bg-white/75 px-1.5 py-0.5 text-[11px] font-medium text-[color:var(--cliply-muted)]">
              {typeLabel[item.type]}
          </span>
            {item.tags.slice(0, 2).map((tag) => (
              <span key={tag} className="truncate text-[11px] text-[color:var(--cliply-faint)]">
                #{tag}
              </span>
            ))}
          </span>
          <button
            type="button"
            aria-label={item.isPinned ? "Unpin item" : "Pin item"}
            title={item.isPinned ? "Unpin item" : "Pin item"}
            onClick={(event) => {
              event.stopPropagation();
              onTogglePin();
            }}
            className={clsx(
              "grid size-6 shrink-0 place-items-center rounded text-[color:var(--cliply-faint)] transition hover:bg-white/80 hover:text-[color:var(--cliply-accent)]",
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
      </span>
    </article>
  );
}
