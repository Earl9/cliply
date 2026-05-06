import { useEffect, useState } from "react";
import { Code2, ExternalLink, FileText, Image as ImageIcon, Shield } from "lucide-react";
import type { ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardPreviewProps = {
  item: ClipboardItem;
};

export function ClipboardPreview({ item }: ClipboardPreviewProps) {
  const [imageLoadFailed, setImageLoadFailed] = useState(false);

  useEffect(() => {
    setImageLoadFailed(false);
  }, [item.id]);

  if (item.sensitiveScore >= 50) {
    return (
      <div className="rounded-[12px] border border-amber-200 bg-amber-50 p-4">
        <div className="mb-2.5 flex items-center gap-2 text-[15px] font-semibold text-amber-800">
          <Shield className="size-4" />
          已隐藏敏感内容
        </div>
        <p className="rounded-[10px] border border-amber-200 bg-white px-3 py-2.5 text-[13px] leading-5 text-amber-800">
          这条记录被隐私规则标记，详情内容不会在本地保存。可以在设置中调整敏感内容过滤策略。
        </p>
      </div>
    );
  }

  if (item.type === "code") {
    const code = item.fullText ?? item.previewText;

    return (
      <div className="rounded-[12px] border border-[#e3e9f1] bg-[#fbfcfe] px-4 py-3">
        <div className="mb-2.5 flex items-center gap-2 text-[15px] font-semibold text-[color:var(--cliply-text)]">
          <Code2 className="size-4 text-[color:var(--cliply-accent-strong)]" />
          {item.title}
        </div>
        <pre className="cliply-code-font cliply-scrollbar max-h-[240px] overflow-auto rounded-[10px] border border-[#e3e9f1] bg-white px-4 py-3 text-[13px] leading-[1.55] text-[#1f2937]">
          <code>
            {code.split("\n").map((line, index) => (
              <span key={`${line}-${index}`} className="block">
                <span className="mr-3 inline-block w-8 select-none text-right text-[#9aa3b2]">
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
      <div className="rounded-[12px] border border-[#e3e9f1] bg-[#fbfcfe] p-4">
        <div className="mb-2.5 flex items-center gap-2 text-[15px] font-semibold text-[color:var(--cliply-text)]">
          <ExternalLink className="size-4 text-[color:var(--cliply-info)]" />
          链接预览
        </div>
        <div className="rounded-[10px] border border-[#e3e9f1] bg-white p-3">
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

    return (
      <div>
        <div className="mb-2.5 flex h-6 items-center gap-2 text-[15px] font-bold text-[color:var(--cliply-text)]">
          <ImageIcon className="size-4 text-[color:var(--cliply-amber)]" />
          {imageTitle(item)}
        </div>
        <div className="grid h-[240px] place-items-center overflow-hidden rounded-[12px] border border-[#dfe6ef] bg-[#f8fafc] bg-[linear-gradient(45deg,rgba(148,163,184,0.10)_25%,transparent_25%),linear-gradient(-45deg,rgba(148,163,184,0.10)_25%,transparent_25%),linear-gradient(45deg,transparent_75%,rgba(148,163,184,0.10)_75%),linear-gradient(-45deg,transparent_75%,rgba(148,163,184,0.10)_75%)] bg-[length:14px_14px] bg-[position:0_0,0_7px,7px_-7px,-7px_0] p-3 shadow-[0_6px_16px_rgba(15,23,42,0.035)]">
          {imageUrl && !imageLoadFailed ? (
            <img
              src={imageUrl}
              alt={item.imageAlt ?? item.title}
              className="max-h-full max-w-full rounded-lg object-contain"
              onError={() => setImageLoadFailed(true)}
            />
          ) : (
            <div className="grid place-items-center gap-2 text-[13px] font-medium text-amber-600">
              <ImageIcon className="size-8 text-amber-500" />
              <span>{imageUrl ? "图片加载失败" : "图片文件不可用"}</span>
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="rounded-[12px] border border-[#e3e9f1] bg-[#fbfcfe] px-4 py-3">
      <div className="mb-2.5 flex items-center gap-2 text-[15px] font-semibold text-[color:var(--cliply-text)]">
        <FileText className="size-4 text-slate-600" />
        {item.title}
      </div>
      <p className="cliply-scrollbar max-h-[190px] overflow-auto whitespace-pre-wrap rounded-[10px] border border-[#e3e9f1] bg-white px-4 py-3 text-sm leading-[1.55] text-[color:var(--cliply-body-text)]">
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
