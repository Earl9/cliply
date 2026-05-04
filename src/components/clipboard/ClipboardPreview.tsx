import { Code2, ExternalLink, FileText, Image as ImageIcon, Shield } from "lucide-react";
import type { ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardPreviewProps = {
  item: ClipboardItem;
};

export function ClipboardPreview({ item }: ClipboardPreviewProps) {
  if (item.sensitiveScore >= 50) {
    return (
      <div className="rounded-[14px] border border-amber-200 bg-amber-50 p-5">
        <div className="mb-3 flex items-center gap-2 text-[17px] font-semibold text-amber-800">
          <Shield className="size-5" />
          已隐藏敏感内容
        </div>
        <p className="rounded-[14px] border border-amber-200 bg-white p-[18px] text-sm leading-6 text-amber-800">
          这条记录被隐私规则标记，详情内容不会在本地保存。可以在设置中调整敏感内容过滤策略。
        </p>
      </div>
    );
  }

  if (item.type === "code") {
    const code = item.fullText ?? item.previewText;

    return (
      <div className="rounded-[14px] border border-[#e7ebf2] bg-[#fbfcfe] px-5 py-[18px]">
        <div className="mb-3 flex items-center gap-2 text-[17px] font-semibold text-[color:var(--cliply-text)]">
          <Code2 className="size-5 text-[color:var(--cliply-accent-strong)]" />
          {item.title}
        </div>
        <pre className="cliply-code-font cliply-scrollbar max-h-[320px] overflow-auto rounded-[14px] border border-[#e7ebf2] bg-white px-5 py-[18px] text-sm leading-[1.7] text-[#1f2937]">
          <code>
            {code.split("\n").map((line, index) => (
              <span key={`${line}-${index}`} className="block">
                <span className="mr-4 inline-block w-9 select-none text-right text-[#9aa3b2]">
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
      <div className="rounded-[14px] border border-[#e7ebf2] bg-[#fbfcfe] p-5">
        <div className="mb-3 flex items-center gap-2 text-[17px] font-semibold text-[color:var(--cliply-text)]">
          <ExternalLink className="size-4 text-[color:var(--cliply-info)]" />
          链接预览
        </div>
        <div className="rounded-[14px] border border-[#e7ebf2] bg-white p-5">
          <div className="mb-2 flex items-center gap-2 text-[17px] font-semibold text-[color:var(--cliply-text)]">
            <ExternalLink className="size-5 text-[color:var(--cliply-info)]" />
            {domain}
          </div>
          <p className="line-clamp-2 break-all text-sm leading-6 text-[color:var(--cliply-muted)]">{url}</p>
        </div>
      </div>
    );
  }

  if (item.type === "image") {
    const imageUrl = item.imageUrl ?? item.thumbnailUrl;

    return (
      <div className="rounded-[14px] border border-[#e7ebf2] bg-white p-5">
        <div className="mb-[14px] flex h-8 items-center gap-2 text-[17px] font-semibold text-[color:var(--cliply-text)]">
          <ImageIcon className="size-5 text-[color:var(--cliply-amber)]" />
          {imageTitle(item)}
        </div>
        <div className="grid h-[320px] place-items-center overflow-hidden rounded-[14px] border border-[#e7ebf2] bg-[#f7f9fc] bg-[linear-gradient(45deg,rgba(148,163,184,0.12)_25%,transparent_25%),linear-gradient(-45deg,rgba(148,163,184,0.12)_25%,transparent_25%),linear-gradient(45deg,transparent_75%,rgba(148,163,184,0.12)_75%),linear-gradient(-45deg,transparent_75%,rgba(148,163,184,0.12)_75%)] bg-[length:16px_16px] bg-[position:0_0,0_8px,8px_-8px,-8px_0] p-4">
          {imageUrl ? (
            <img
              src={imageUrl}
              alt={item.imageAlt ?? item.title}
              className="max-h-full max-w-full rounded-[10px] object-contain"
            />
          ) : (
            <div className="grid place-items-center gap-2 text-sm font-medium text-amber-600">
              <ImageIcon className="size-10 text-amber-500" />
              <span>图片文件不可用</span>
            </div>
          )}
        </div>
        {item.imageWidth && item.imageHeight ? (
          <p className="mt-3 text-sm font-medium text-[color:var(--cliply-muted)]">
            {item.imageWidth} × {item.imageHeight}
          </p>
        ) : null}
      </div>
    );
  }

  return (
    <div className="rounded-[14px] border border-[#e7ebf2] bg-[#fbfcfe] p-5">
      <div className="mb-3 flex items-center gap-2 text-[17px] font-semibold text-[color:var(--cliply-text)]">
        <FileText className="size-5 text-slate-600" />
        {item.title}
      </div>
      <p className="cliply-scrollbar max-h-[320px] overflow-auto whitespace-pre-wrap rounded-[14px] border border-[#e7ebf2] bg-white px-5 py-[18px] text-sm leading-[1.7] text-[color:var(--cliply-body-text)]">
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
