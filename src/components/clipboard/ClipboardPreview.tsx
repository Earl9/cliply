import { Code2, ExternalLink, FileText, Image as ImageIcon } from "lucide-react";
import type { ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardPreviewProps = {
  item: ClipboardItem;
};

export function ClipboardPreview({ item }: ClipboardPreviewProps) {
  if (item.type === "code") {
    const code = item.fullText ?? item.previewText;

    return (
      <div className="rounded-xl border border-[color:var(--cliply-border)] bg-white/80 p-5 shadow-sm">
        <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-[color:var(--cliply-text)]">
          <Code2 className="size-4 text-[color:var(--cliply-accent)]" />
          {item.title}
        </div>
        <pre className="cliply-code-font max-h-[260px] overflow-auto rounded-xl border border-[color:var(--cliply-border)] bg-[#fbfbfd] p-[18px] text-sm leading-[1.65] text-[#202634]">
          <code>
            {code.split("\n").map((line, index) => (
              <span key={`${line}-${index}`} className="block">
                <span className="mr-4 inline-block w-6 select-none text-right text-[#a8afbd]">
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
      <div className="rounded-xl border border-[color:var(--cliply-border)] bg-white/80 p-5 shadow-sm">
        <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-[color:var(--cliply-text)]">
          <ExternalLink className="size-4 text-[color:var(--cliply-info)]" />
          链接预览
        </div>
        <div className="rounded-xl border border-[color:var(--cliply-border)] bg-[#fafafb] p-[18px]">
          <div className="mb-2 flex items-center gap-2 text-[15px] font-semibold text-[color:var(--cliply-text)]">
            <ExternalLink className="size-4 text-[color:var(--cliply-info)]" />
            {domain}
          </div>
          <p className="break-all text-sm leading-6 text-[color:var(--cliply-muted)]">{url}</p>
        </div>
      </div>
    );
  }

  if (item.type === "image") {
    const imageUrl = item.imageUrl ?? item.thumbnailUrl;

    return (
      <div className="rounded-xl border border-[color:var(--cliply-border)] bg-white/80 p-5 shadow-sm">
        <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-[color:var(--cliply-text)]">
          <ImageIcon className="size-4 text-[color:var(--cliply-amber)]" />
          {item.title}
        </div>
        <div className="grid max-h-[260px] min-h-[220px] place-items-center overflow-hidden rounded-xl border border-[color:var(--cliply-border)] bg-[linear-gradient(45deg,#f4f5f7_25%,transparent_25%),linear-gradient(-45deg,#f4f5f7_25%,transparent_25%),linear-gradient(45deg,transparent_75%,#f4f5f7_75%),linear-gradient(-45deg,transparent_75%,#f4f5f7_75%)] bg-[length:20px_20px] bg-[position:0_0,0_10px,10px_-10px,-10px_0]">
          {imageUrl ? (
            <img
              src={imageUrl}
              alt={item.imageAlt ?? item.title}
              className="max-h-full max-w-full object-contain"
            />
          ) : (
            <div className="grid place-items-center gap-2 text-sm font-medium text-amber-600">
              <ImageIcon className="size-10 text-amber-500" />
              <span>图片文件不可用</span>
            </div>
          )}
        </div>
        {item.imageWidth && item.imageHeight ? (
          <p className="mt-3 text-xs font-medium text-[color:var(--cliply-muted)]">
            {item.imageWidth} x {item.imageHeight}
          </p>
        ) : null}
      </div>
    );
  }

  return (
    <div className="rounded-xl border border-[color:var(--cliply-border)] bg-white/80 p-5 shadow-sm">
      <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-[color:var(--cliply-text)]">
        <FileText className="size-4 text-slate-600" />
        {item.title}
      </div>
      <p className="max-h-[220px] overflow-auto whitespace-pre-wrap rounded-xl border border-[color:var(--cliply-border)] bg-[#fafafb] p-[18px] text-sm leading-[1.6] text-[color:var(--cliply-text)]">
        {item.fullText ?? item.previewText}
      </p>
    </div>
  );
}

function getDomain(value: string) {
  try {
    return new URL(value).hostname;
  } catch {
    return value;
  }
}
