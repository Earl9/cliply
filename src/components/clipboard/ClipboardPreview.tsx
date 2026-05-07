import { useEffect, useState } from "react";
import { Code2, ExternalLink, FileText, Image as ImageIcon } from "lucide-react";
import type { ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardPreviewProps = {
  item: ClipboardItem;
  onOpenImage: (item: ClipboardItem) => void;
};

export function ClipboardPreview({ item, onOpenImage }: ClipboardPreviewProps) {
  const [imageLoadFailed, setImageLoadFailed] = useState(false);

  useEffect(() => {
    setImageLoadFailed(false);
  }, [item.id]);

  if (item.type === "code") {
    const code = item.fullText ?? item.previewText;

    return (
      <div className="rounded-[12px] border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-card)] px-4 py-3">
        <div className="mb-2.5 flex items-center gap-2 text-[15px] font-semibold text-[color:var(--cliply-text)]">
          <Code2 className="size-4 text-[color:var(--cliply-accent-strong)]" />
          {item.title}
        </div>
        <pre className="cliply-code-font cliply-scrollbar max-h-[240px] overflow-auto rounded-[10px] border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-input-bg)] px-4 py-3 text-[13px] leading-[1.55] text-[color:var(--cliply-body-text)]">
          <code>
            {code.split("\n").map((line, index) => (
              <span key={`${line}-${index}`} className="block">
                <span className="mr-3 inline-block w-8 select-none text-right text-[color:var(--cliply-faint)]">
                  {index + 1}
                </span>
                <span>{line || " "}</span>
              </span>
            ))}
          </code>
        </pre>
      </div>
    );
  }

  if (item.type === "link") {
    const url = item.fullText ?? item.previewText;
    const domain = getDomain(url);

    return (
      <div className="rounded-[12px] border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-card)] p-4">
        <div className="mb-2.5 flex items-center gap-2 text-[15px] font-semibold text-[color:var(--cliply-text)]">
          <ExternalLink className="size-4 text-[color:var(--cliply-info)]" />
          链接预览
        </div>
        <div className="rounded-[10px] border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-input-bg)] p-3">
          <div className="mb-1.5 flex items-center gap-2 text-[15px] font-semibold text-[color:var(--cliply-text)]">
            <ExternalLink className="size-4 text-[color:var(--cliply-info)]" />
            {domain}
          </div>
          <p className="line-clamp-2 break-all text-[13px] leading-5 text-[color:var(--cliply-muted)]">{url}</p>
        </div>
      </div>
    );
  }

  if (item.type === "image") {
    const imageUrl = item.imageUrl ?? item.thumbnailUrl;
    const canOpenImage = Boolean(imageUrl && !imageLoadFailed);

    return (
      <div>
        <div className="mb-2.5 flex h-6 items-center gap-2 text-[15px] font-bold text-[color:var(--cliply-text)]">
          <ImageIcon className="size-4 text-[color:var(--cliply-amber)]" />
          {imageTitle(item)}
        </div>
        <button
          type="button"
          disabled={!canOpenImage}
          aria-label="查看图片"
          onClick={() => {
            if (canOpenImage) {
              onOpenImage(item);
            }
          }}
          className="cliply-image-checker grid h-[320px] w-full place-items-center overflow-hidden rounded-[12px] border border-[color:var(--cliply-border)] p-3 text-left shadow-[0_6px_16px_rgba(15,23,42,0.055)] transition hover:border-[color:var(--cliply-primary-border)] hover:shadow-[var(--cliply-shadow-card-hover)] disabled:cursor-default disabled:hover:border-[color:var(--cliply-border)] disabled:hover:shadow-[0_6px_16px_rgba(15,23,42,0.055)]"
        >
          {imageUrl && !imageLoadFailed ? (
            <img
              src={imageUrl}
              alt={item.imageAlt ?? item.title}
              className="max-h-full max-w-full cursor-zoom-in rounded-lg object-contain"
              onError={() => setImageLoadFailed(true)}
            />
          ) : (
            <div className="grid place-items-center gap-2 text-[13px] font-medium text-amber-600">
              <ImageIcon className="size-8 text-[color:var(--cliply-warning)]" />
              <span>{imageUrl ? "图片加载失败" : "图片文件不可用"}</span>
            </div>
          )}
        </button>
      </div>
    );
  }

  return (
    <div className="rounded-[12px] border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-card)] px-4 py-3">
      <div className="mb-2.5 flex items-center gap-2 text-[15px] font-semibold text-[color:var(--cliply-text)]">
        <FileText className="size-4 text-[color:var(--cliply-muted)]" />
        {item.title}
      </div>
      <p className="cliply-scrollbar max-h-[190px] overflow-auto whitespace-pre-wrap rounded-[10px] border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-input-bg)] px-4 py-3 text-sm leading-[1.55] text-[color:var(--cliply-body-text)]">
        {item.fullText ?? item.previewText}
      </p>
    </div>
  );
}

function imageTitle(item: ClipboardItem) {
  if (item.imageWidth && item.imageHeight) {
    return `图片 ${item.imageWidth} × ${item.imageHeight}`;
  }

  return item.title;
}

function getDomain(value: string) {
  try {
    return new URL(value).hostname;
  } catch {
    return value;
  }
}
