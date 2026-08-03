import { useEffect, useState } from "react";
import { Image as ImageIcon } from "lucide-react";
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
      <pre className="cliply-code-font cliply-scrollbar max-h-[300px] overflow-auto rounded-[6px] border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-input-bg)] px-3 py-2.5 text-[12.5px] leading-[1.6] text-[color:var(--cliply-body-text)]">
        <code>
          {code.split("\n").map((line, index) => (
            <span key={`${line}-${index}`} className="block">
              <span className="mr-3 inline-block w-7 select-none text-right text-[color:var(--cliply-faint)]">
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
      <div className="rounded-[6px] border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-input-bg)] px-3 py-2.5">
        <div className="mb-1 text-[13px] font-medium text-[color:var(--cliply-text)]">{domain}</div>
        <p className="break-all text-[12.5px] leading-5 text-[color:var(--cliply-muted)]">{url}</p>
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
        className="cliply-image-checker grid h-[320px] w-full place-items-center overflow-hidden rounded-[6px] border border-[color:var(--cliply-border)] p-2 text-left disabled:cursor-default"
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
    <p className="cliply-scrollbar max-h-[260px] overflow-auto whitespace-pre-wrap rounded-[6px] border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-input-bg)] px-3 py-2.5 text-[13px] leading-[1.6] text-[color:var(--cliply-body-text)]">
      {item.fullText ?? item.previewText}
    </p>
  );
}

function getDomain(value: string) {
  try {
    return new URL(value).hostname;
  } catch {
    return value;
  }
}
