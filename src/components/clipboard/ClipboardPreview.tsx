import { useEffect, useState } from "react";
import { Code2, FileText, Image as ImageIcon, Link2 } from "lucide-react";
import type { ClipboardItem, ClipboardItemType } from "@/lib/clipboardTypes";

type ClipboardPreviewProps = {
  item: ClipboardItem;
  onOpenImage: (item: ClipboardItem) => void;
};

const previewLabels: Record<ClipboardItemType, string> = {
  code: "代码预览",
  image: "图片预览",
  link: "链接预览",
  text: "文本预览",
};

const previewIcons = {
  code: Code2,
  image: ImageIcon,
  link: Link2,
  text: FileText,
};

export function ClipboardPreview({ item, onOpenImage }: ClipboardPreviewProps) {
  const [imageLoadFailed, setImageLoadFailed] = useState(false);

  useEffect(() => {
    setImageLoadFailed(false);
  }, [item.id]);

  const PreviewIcon = previewIcons[item.type];

  const content = (() => {
    if (item.type === "code") {
      const code = item.fullText ?? item.previewText;

      return (
        <pre className="cliply-code-surface cliply-code-font cliply-scrollbar max-h-[320px] overflow-auto py-3.5 text-[12.5px] leading-[1.7] text-[color:var(--cliply-body-text)]">
          <code>
            {code.split("\n").map((line, index) => (
              <span key={`${line}-${index}`} className="block">
                <span className="mr-4 inline-block w-11 select-none border-r border-[color:var(--cliply-border-soft)] pr-3 text-right tabular-nums text-[color:var(--cliply-faint)]">
                  {index + 1}
                </span>
                <span>{line || " "}</span>
              </span>
            ))}
          </code>
        </pre>
      );
    }

    if (item.type === "link") {
      const url = item.fullText ?? item.previewText;
      const domain = getDomain(url);

      return (
        <div className="cliply-link-preview px-5 py-4">
          <div className="mb-1.5 text-[14px] font-semibold text-[color:var(--cliply-text)]">{domain}</div>
          <p className="break-all text-[13px] leading-5 text-[color:var(--cliply-accent-on-soft)]">{url}</p>
        </div>
      );
    }

    if (item.type === "image") {
      const imageUrl = item.imageUrl ?? item.thumbnailUrl;
      const canOpenImage = Boolean(imageUrl && !imageLoadFailed);

      return (
        <button
          type="button"
          disabled={!canOpenImage}
          aria-label="查看图片"
          onClick={() => {
            if (canOpenImage) {
              onOpenImage(item);
            }
          }}
          className="cliply-image-checker grid h-[300px] w-full place-items-center overflow-hidden p-2 text-left disabled:cursor-default"
        >
          {imageUrl && !imageLoadFailed ? (
            <img
              src={imageUrl}
              alt={item.imageAlt ?? item.title}
              className="max-h-full max-w-full cursor-zoom-in object-contain"
              onError={() => setImageLoadFailed(true)}
            />
          ) : (
            <div className="grid place-items-center gap-2 text-[13px] text-[color:var(--cliply-muted)]">
              <ImageIcon className="size-7" />
              <span>{imageUrl ? "图片加载失败" : "图片文件不可用"}</span>
            </div>
          )}
        </button>
      );
    }

    return (
      <p className="cliply-scrollbar max-h-[280px] min-h-28 overflow-auto whitespace-pre-wrap px-5 py-4 text-[13.5px] leading-[1.7] text-[color:var(--cliply-body-text)]">
        {item.fullText ?? item.previewText}
      </p>
    );
  })();

  return (
    <section className="cliply-preview-panel overflow-hidden rounded-[8px] border border-[color:var(--cliply-border)]">
      <header className="cliply-preview-header flex h-10 items-center justify-between gap-3 border-b border-[color:var(--cliply-border-soft)] px-3.5">
        <span className="flex min-w-0 items-center gap-2 text-[11.5px] font-semibold text-[color:var(--cliply-text)]">
          <PreviewIcon className="size-3.5 text-[color:var(--cliply-accent)]" />
          {previewLabels[item.type]}
        </span>
        <span className="cliply-caption truncate text-[10px] text-[color:var(--cliply-faint)]">
          {item.formats[0]?.formatName ?? "标准格式"}
        </span>
      </header>
      {content}
    </section>
  );
}

function getDomain(value: string) {
  try {
    return new URL(value).hostname;
  } catch {
    return value;
  }
}
