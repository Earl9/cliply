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
    <div className="mt-[18px] rounded-[14px] border border-[color:var(--cliply-border-soft)] bg-[#fbfcfe] px-5 py-[18px]">
      <h3 className="mb-[14px] text-base font-semibold text-[color:var(--cliply-text)]">元信息</h3>
      <dl className="grid grid-cols-[120px_minmax(0,1fr)] gap-x-5 gap-y-[14px]">
        {metadata.map(([label, value]) => (
          <div key={label} className="contents">
            <dt className="text-sm font-normal text-[color:var(--cliply-muted)]">{label}</dt>
            <dd className="min-w-0 truncate text-[15px] font-medium text-[color:var(--cliply-body-text)]">
              {label === "类型" ? (
                <Badge tone="accent" className="h-6 rounded-[7px] px-2 text-[13px]">
                  {value}
                </Badge>
              ) : label === "固定状态" ? (
                <Badge
                  tone={item.isPinned ? "accent" : "neutral"}
                  className="h-6 rounded-[7px] px-2 text-[13px]"
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
        <div className="mt-4 rounded-lg bg-[#f3f5f8] px-3 py-2 text-sm text-[color:var(--cliply-muted)]">
          图片尺寸：<span className="font-medium text-[color:var(--cliply-text)]">{imageDimensions}</span>
        </div>
      ) : null}
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
