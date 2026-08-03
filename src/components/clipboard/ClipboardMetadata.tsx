import type { ClipboardItem, ClipboardItemType } from "@/lib/clipboardTypes";
import { formatBytes, formatFullCopiedTime } from "@/lib/formatTime";

type ClipboardMetadataProps = {
  item: ClipboardItem;
};

const typeLabels: Record<ClipboardItemType, string> = {
  code: "代码",
  image: "图片",
  link: "链接",
  text: "文本",
};

export function ClipboardMetadata({ item }: ClipboardMetadataProps) {
  const imageDimensions =
    item.type === "image" && item.imageWidth && item.imageHeight
      ? `${item.imageWidth} × ${item.imageHeight}`
      : null;
  const metadata: Array<[string, string]> = [
    ["类型", typeLabels[item.type]],
    ["复制时间", formatFullCopiedTime(item.copiedAt)],
    ["大小", formatBytes(item.sizeBytes)],
    ["来源窗口", item.sourceWindow ?? "未知"],
  ];
  if (imageDimensions) {
    metadata.push(["尺寸", imageDimensions]);
  }

  return (
    <div className="mt-4 border-t border-[color:var(--cliply-border-soft)] pt-3.5">
      <dl className="grid grid-cols-[76px_minmax(0,1fr)] gap-x-3 gap-y-2">
        {metadata.map(([label, value]) => (
          <div key={label} className="contents">
            <dt className="cliply-caption text-[11px] leading-5 text-[color:var(--cliply-faint)]">
              {label}
            </dt>
            <dd className="min-w-0 truncate text-[12.5px] leading-5 text-[color:var(--cliply-body-text)]">
              {value}
            </dd>
          </div>
        ))}
      </dl>
      {item.tags.length ? (
        <div className="mt-3 flex flex-wrap gap-1.5">
          {item.tags.map((tag) => (
            <span
              key={tag}
              className="cliply-caption rounded-[4px] bg-[color:var(--cliply-muted-bg)] px-1.5 py-0.5 text-[11px] text-[color:var(--cliply-muted)]"
            >
              #{tag}
            </span>
          ))}
        </div>
      ) : null}
    </div>
  );
}
