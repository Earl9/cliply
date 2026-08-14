import { Code2, FileText, Image, Link2, Pin } from "lucide-react";
import { memo, type MouseEvent, type ReactNode } from "react";
import { clsx } from "clsx";
import type { ClipboardItem, ClipboardItemType } from "@/lib/clipboardTypes";
import { formatRelativeTime } from "@/lib/formatTime";

type ClipboardListItemProps = {
  item: ClipboardItem;
  selected?: boolean;
  relativeTimeNow: number;
  position: number;
  setSize: number;
  virtualOffset: number;
  onSelectItem: (id: string) => void;
  onTogglePin: (id: string) => void;
  onPasteItem: (id: string) => void;
  onItemContextMenu: (event: MouseEvent<HTMLElement>, item: ClipboardItem) => void;
};

const iconByType: Record<ClipboardItemType, typeof FileText> = {
  code: Code2,
  link: Link2,
  text: FileText,
  image: Image,
};

function renderPreview(item: ClipboardItem): ReactNode {
  const preview = item.previewText;

  if (item.type === "link") {
    const { host, rest } = splitUrl(preview);
    return (
      <>
        <span className="text-[color:var(--cliply-text)]">{host}</span>
        {rest ? <span className="text-[color:var(--cliply-faint)]">{rest}</span> : null}
      </>
    );
  }

  return preview;
}

function splitUrl(value: string) {
  try {
    const url = new URL(value);
    const rest = `${url.pathname}${url.search}`.replace(/\/$/, "");
    return { host: url.host, rest };
  } catch {
    return { host: value, rest: "" };
  }
}

function ClipboardListItemComponent({
  item,
  selected,
  relativeTimeNow,
  position,
  setSize,
  virtualOffset,
  onSelectItem,
  onTogglePin,
  onPasteItem,
  onItemContextMenu,
}: ClipboardListItemProps) {
  const Icon = iconByType[item.type] ?? FileText;

  return (
    <article
      role="listitem"
      tabIndex={0}
      aria-posinset={position}
      aria-setsize={setSize}
      style={{
        position: "absolute",
        top: virtualOffset,
        right: 0,
        left: 0,
      }}
      onClick={() => onSelectItem(item.id)}
      onDoubleClick={(event) => {
        event.preventDefault();
        onPasteItem(item.id);
      }}
      onContextMenu={(event) => onItemContextMenu(event, item)}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelectItem(item.id);
        }
      }}
      className={clsx(
        "cliply-row group relative my-1 grid h-[68px] cursor-default grid-cols-[28px_minmax(0,1fr)_26px] items-center gap-3 rounded-[7px] border border-transparent px-3 text-left",
        "focus-visible:outline focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
        selected
          ? "cliply-row-selected"
          : "hover:border-[color:var(--cliply-border-soft)] hover:bg-[color:var(--cliply-surface-raised)]",
      )}
    >
      <span className="cliply-type-icon grid size-7 shrink-0 place-items-center rounded-md" data-type={item.type}>
        {item.type === "image" && item.thumbnailUrl ? (
          <img
            src={item.thumbnailUrl}
            alt={item.imageAlt ?? item.title}
            className="size-7 rounded-[5px] border border-[color:var(--cliply-border)] object-cover"
          />
        ) : (
          <Icon className="size-4" />
        )}
      </span>
      <span className="min-w-0">
        <span
          className={clsx(
            "block truncate font-normal leading-[18px] text-[color:var(--cliply-text)]",
            item.type === "code"
              ? "cliply-code-font text-[12.5px]"
              : "text-[13px]",
          )}
        >
          {renderPreview(item)}
        </span>
        <span className="cliply-caption mt-1.5 flex min-w-0 items-center gap-1.5 text-[10.5px] leading-4 text-[color:var(--cliply-faint)]">
          <span className="truncate">{item.sourceApp}</span>
          <span aria-hidden="true">·</span>
          <span className="shrink-0">{formatRelativeTime(item.copiedAt, relativeTimeNow)}</span>
        </span>
      </span>
      <button
        type="button"
        aria-label={item.isPinned ? "取消固定" : "固定"}
        title={item.isPinned ? "取消固定" : "固定"}
        onClick={(event) => {
          event.stopPropagation();
          onTogglePin(item.id);
        }}
        onDoubleClick={(event) => {
          event.preventDefault();
          event.stopPropagation();
        }}
        data-pinned={item.isPinned ? "true" : "false"}
        className={clsx(
          "cliply-pin-button cliply-interactive grid size-6 shrink-0 place-items-center rounded-[5px]",
          item.isPinned
            ? "text-[color:var(--cliply-accent)] opacity-100"
            : "text-[color:var(--cliply-muted)] opacity-0 hover:bg-[color:var(--cliply-muted-bg)] focus-visible:opacity-100 group-hover:opacity-70",
        )}
      >
        <Pin
          className={clsx(
            "size-3.5",
            item.isPinned && "fill-[color:var(--cliply-accent)]",
          )}
        />
      </button>
    </article>
  );
}

export const ClipboardListItem = memo(ClipboardListItemComponent);
