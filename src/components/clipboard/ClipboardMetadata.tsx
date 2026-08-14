import { AppWindow, Clock3, Database, FileType2, Maximize2, PanelsTopLeft } from "lucide-react";
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
  const metadata = [
    { label: "来源应用", value: item.sourceApp, icon: AppWindow },
    { label: "内容类型", value: typeLabels[item.type], icon: FileType2 },
    { label: "复制时间", value: formatFullCopiedTime(item.copiedAt), icon: Clock3 },
    { label: "数据大小", value: formatBytes(item.sizeBytes), icon: Database },
    { label: "来源窗口", value: item.sourceWindow ?? "未知", icon: PanelsTopLeft, wide: true },
  ];
  if (imageDimensions) {
    metadata.push({ label: "图片尺寸", value: imageDimensions, icon: Maximize2 });
  }

  return (
    <section className="cliply-metadata-section mt-4 overflow-hidden rounded-[8px] border border-[color:var(--cliply-border)]">
      <header className="cliply-metadata-header flex h-10 items-center justify-between gap-3 border-b border-[color:var(--cliply-border-soft)] px-3.5">
        <h3 className="text-[11.5px] font-semibold text-[color:var(--cliply-text)]">记录信息</h3>
        <span className="cliply-format-count cliply-caption rounded px-1.5 py-0.5 text-[10px] tabular-nums text-[color:var(--cliply-faint)]">
          {item.formats.length} 种格式
        </span>
      </header>
      <dl className="cliply-metadata-grid grid grid-cols-2">
        {metadata.map(({ label, value, icon: Icon, wide }) => (
          <div key={label} data-wide={wide ? "true" : "false"} className="cliply-metadata-row flex min-w-0 items-start gap-2.5 px-3.5 py-3">
            <span className="cliply-metadata-icon grid size-7 shrink-0 place-items-center rounded-md">
              <Icon className="size-3.5" />
            </span>
            <div className="min-w-0">
              <dt className="cliply-caption text-[10px] leading-4 text-[color:var(--cliply-faint)]">
                {label}
              </dt>
              <dd className="mt-0.5 min-w-0 truncate text-[12px] leading-5 text-[color:var(--cliply-body-text)]">
                {value}
              </dd>
            </div>
          </div>
        ))}
      </dl>
      {item.tags.length ? (
        <div className="cliply-tags flex flex-wrap gap-1.5 border-t border-[color:var(--cliply-border-soft)] px-3.5 py-2.5">
          {item.tags.map((tag) => <span className="cliply-caption rounded px-1.5 py-0.5 text-[10px] text-[color:var(--cliply-faint)]" key={tag}>#{tag}</span>)}
        </div>
      ) : null}
      <div className="cliply-format-list flex min-w-0 items-center gap-2 border-t border-[color:var(--cliply-border-soft)] px-3.5 py-2.5">
        <span className="cliply-caption shrink-0 text-[10px] text-[color:var(--cliply-faint)]">可用格式</span>
        <div className="flex min-w-0 flex-wrap gap-1.5">
          {item.formats.map((format) => (
            <span className="cliply-format-chip cliply-caption rounded px-1.5 py-0.5 text-[10px] text-[color:var(--cliply-faint)]" key={format.id}>
              {format.formatName}
            </span>
          ))}
        </div>
      </div>
    </section>
  );
}
