import { Code2, FileText, Image, Link2, Pin } from "lucide-react";
import { clsx } from "clsx";

type StaticItem = {
  type: string;
  app: string;
  text: string;
  time: string;
  pinned: boolean;
};

type ClipboardListItemProps = {
  item: StaticItem;
  selected?: boolean;
};

const iconByType: Record<string, typeof FileText> = {
  Code: Code2,
  Link: Link2,
  Text: FileText,
  Image,
};

export function ClipboardListItem({ item, selected }: ClipboardListItemProps) {
  const Icon = iconByType[item.type] ?? FileText;

  return (
    <button
      type="button"
      className={clsx(
        "flex w-full items-start gap-3 rounded-lg border p-3 text-left transition",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
        selected
          ? "border-[color:var(--cliply-accent)] bg-[color:var(--cliply-accent-soft)] shadow-sm"
          : "border-transparent bg-white/52 hover:border-[color:var(--cliply-border)] hover:bg-white/78",
      )}
    >
      <span
        className={clsx(
          "grid size-9 shrink-0 place-items-center rounded-md",
          item.type === "Code" && "bg-indigo-50 text-indigo-700",
          item.type === "Link" && "bg-teal-50 text-teal-700",
          item.type === "Text" && "bg-slate-100 text-slate-600",
          item.type === "Image" && "bg-amber-50 text-amber-700",
        )}
      >
        <Icon className="size-4" />
      </span>
      <span className="min-w-0 flex-1">
        <span className="flex items-center justify-between gap-2">
          <span className="truncate text-sm font-semibold text-[color:var(--cliply-text)]">
            {item.app}
          </span>
          <span className="shrink-0 text-xs text-[color:var(--cliply-faint)]">{item.time}</span>
        </span>
        <span className="mt-1 line-clamp-2 text-sm leading-5 text-[color:var(--cliply-muted)]">
          {item.text}
        </span>
        <span className="mt-2 flex items-center gap-2">
          <span className="rounded bg-white/75 px-1.5 py-0.5 text-[11px] font-medium text-[color:var(--cliply-muted)]">
            {item.type}
          </span>
          {item.pinned ? <Pin className="size-3.5 fill-[color:var(--cliply-accent)] text-[color:var(--cliply-accent)]" /> : null}
        </span>
      </span>
    </button>
  );
}
