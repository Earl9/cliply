import { Badge } from "@/components/common/Badge";
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
  const metadata = [
    ["来源应用", item.sourceApp],
    ["复制时间", formatFullCopiedTime(item.copiedAt)],
    ["类型", typeLabels[item.type]],
    ["大小", formatBytes(item.sizeBytes)],
    ["来源窗口", item.sourceWindow ?? "未知"],
    ["固定状态", item.isPinned ? "已固定" : "未固定"],
    ["隐私评分", item.sensitiveScore > 0 ? `${item.sensitiveScore}` : "无"],
  ];
  const imageDimensions =
    item.type === "image" && item.imageWidth && item.imageHeight
      ? `${item.imageWidth} × ${item.imageHeight}`
      : null;

  return (
    <div className="mt-3 rounded-[12px] border border-[color:var(--cliply-border-soft)] bg-[#fbfcfe] px-4 py-3 shadow-[0_3px_10px_rgba(15,23,42,0.025)]">
      <h3 className="mb-3 text-sm font-semibold text-[color:var(--cliply-text)]">元信息</h3>
      <dl className="grid grid-cols-[96px_minmax(0,1fr)] gap-x-4 gap-y-2.5">
        {metadata.map(([label, value]) => (
          <div key={label} className="contents">
            <dt className="text-xs font-normal text-[color:var(--cliply-muted)]">{label}</dt>
            <dd className="min-w-0 truncate text-[13px] font-medium text-[color:var(--cliply-body-text)]">
              {label === "类型" ? (
                <Badge tone="accent" className="h-5 rounded-md px-1.5 text-xs">
                  {value}
                </Badge>
              ) : label === "固定状态" ? (
                <Badge
                  tone={item.isPinned ? "accent" : "neutral"}
                  className="h-5 rounded-md px-1.5 text-xs"
                >
                  {value}
                </Badge>
              ) : (
                value
              )}
            </dd>
          </div>
        ))}
      </dl>
      {imageDimensions ? (
        <div className="mt-3 rounded-lg bg-[#f5f7fa] px-3 py-2 text-xs text-[color:var(--cliply-muted)]">
          图片尺寸：<span className="font-medium text-[color:var(--cliply-text)]">{imageDimensions}</span>
        </div>
      ) : null}
      {item.tags.length ? (
        <div className="mt-3 flex flex-wrap gap-1.5">
          {item.tags.map((tag) => (
            <Badge key={tag}>#{tag}</Badge>
          ))}
        </div>
      ) : null}
    </div>
  );
}
