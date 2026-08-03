import { Code2, FileText, Image, Link2, Pin } from "lucide-react";
import type { MouseEvent, ReactNode } from "react";
import { clsx } from "clsx";
import type { ClipboardItem, ClipboardItemType } from "@/lib/clipboardTypes";
import { formatRelativeTime } from "@/lib/formatTime";

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

/// Renders the preview in the shape of its own content: code stays monospaced,
/// links lead with the host and mute the path. Scanning a clipboard list is
/// mostly "which one was it" — typography does that work faster than a label.
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

export function ClipboardListItem({
  item,
  selected,
  onSelect,
  onTogglePin,
  onPaste,
  onContextMenu,
}: ClipboardListItemProps) {
  const Icon = iconByType[item.type] ?? FileText;

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
        "cliply-row group relative mx-1.5 grid h-[52px] cursor-default grid-cols-[28px_minmax(0,1fr)_auto] items-center gap-2.5 rounded-[6px] px-2.5 text-left",
        "focus-visible:outline focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-[color:var(--cliply-accent)]",
        selected
          ? "cliply-row-selected"
          : "hover:bg-[color:var(--cliply-muted-bg)]",
      )}
    >
      <span className="grid size-7 shrink-0 place-items-center">
        {item.type === "image" && item.thumbnailUrl ? (
          <img
            src={item.thumbnailUrl}
            alt={item.imageAlt ?? item.title}
            className="size-7 rounded-[4px] border border-[color:var(--cliply-border)] object-cover"
          />
        ) : (
          <Icon
            className={clsx(
              "size-4",
              selected
                ? "text-[color:var(--cliply-accent)]"
                : "text-[color:var(--cliply-faint)]",
            )}
          />
        )}
      </span>
      <span className="min-w-0">
        <span
          className={clsx(
            "block truncate leading-[18px] text-[color:var(--cliply-text)]",
            item.type === "code"
              ? "cliply-code-font text-[12.5px]"
              : "text-[13px]",
          )}
        >
          {renderPreview(item)}
        </span>
        <span className="cliply-caption mt-px flex min-w-0 items-center gap-1.5 text-[11px] leading-4 text-[color:var(--cliply-faint)]">
          <span className="truncate">{item.sourceApp}</span>
          <span aria-hidden="true">·</span>
          <span className="shrink-0">{formatRelativeTime(item.copiedAt)}</span>
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
        data-pinned={item.isPinned ? "true" : "false"}
        className={clsx(
          "cliply-pin-button cliply-interactive grid size-6 shrink-0 place-items-center rounded-[4px]",
          item.isPinned
            ? "text-[color:var(--cliply-accent)] opacity-100"
            : "text-[color:var(--cliply-muted)] opacity-0 hover:bg-[color:var(--cliply-border-soft)] focus-visible:opacity-100 group-hover:opacity-70",
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
