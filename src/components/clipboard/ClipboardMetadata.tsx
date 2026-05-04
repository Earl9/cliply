import { Badge } from "@/components/common/Badge";
import type { ClipboardItem, ClipboardItemType } from "@/lib/clipboardTypes";
import { formatBytes, formatFullCopiedTime } from "@/lib/formatTime";

type ClipboardMetadataProps = {
  item: ClipboardItem;
};

const typeLabels: Record<ClipboardItemType, string> = {
  code: "Code",
  image: "Image",
  link: "Link",
  text: "Text",
};

export function ClipboardMetadata({ item }: ClipboardMetadataProps) {
  const metadata = [
    ["Source app", item.sourceApp],
    ["Window", item.sourceWindow ?? "Unknown"],
    ["Copied", formatFullCopiedTime(item.copiedAt)],
    ["Type", typeLabels[item.type]],
    ["Size", formatBytes(item.sizeBytes)],
    ["Pinned", item.isPinned ? "Yes" : "No"],
  ];

  return (
    <div className="mt-4 rounded-lg border border-[color:var(--cliply-border)] bg-white/64 p-4">
      <h3 className="mb-3 text-sm font-semibold text-[color:var(--cliply-text)]">Metadata</h3>
      <dl className="grid grid-cols-[120px_minmax(0,1fr)] gap-x-4 gap-y-2 text-sm">
        {metadata.map(([label, value]) => (
          <div key={label} className="contents">
            <dt className="text-[color:var(--cliply-muted)]">{label}</dt>
            <dd className="min-w-0 text-[color:var(--cliply-text)]">
              {label === "Type" ? <Badge tone="accent">{value}</Badge> : value}
            </dd>
          </div>
        ))}
      </dl>
      {item.tags.length ? (
        <div className="mt-4 flex flex-wrap gap-2">
          {item.tags.map((tag) => (
            <Badge key={tag}>#{tag}</Badge>
          ))}
        </div>
      ) : null}
    </div>
  );
}
