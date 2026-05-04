import { Code2, ExternalLink, FileText, Image as ImageIcon } from "lucide-react";
import type { ClipboardItem } from "@/lib/clipboardTypes";

type ClipboardPreviewProps = {
  item: ClipboardItem;
};

export function ClipboardPreview({ item }: ClipboardPreviewProps) {
  if (item.type === "code") {
    const code = item.fullText ?? item.previewText;

    return (
      <div className="rounded-lg border border-[color:var(--cliply-border)] bg-white/78 p-4 shadow-sm">
        <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-[color:var(--cliply-text)]">
          <Code2 className="size-4 text-[color:var(--cliply-accent)]" />
          {item.title}
        </div>
        <pre className="overflow-auto rounded-md bg-[#172033] p-4 text-sm leading-6 text-slate-100">
          <code>
            {code.split("\n").map((line, index) => (
              <span key={`${line}-${index}`} className="block">
                <span className="mr-4 inline-block w-5 select-none text-right text-slate-500">
                  {index + 1}
                </span>
                {line || " "}
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
      <div className="rounded-lg border border-[color:var(--cliply-border)] bg-white/78 p-4 shadow-sm">
        <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-[color:var(--cliply-text)]">
          <ExternalLink className="size-4 text-[color:var(--cliply-teal)]" />
          {domain}
        </div>
        <div className="rounded-md border border-teal-100 bg-teal-50/70 p-4">
          <p className="break-all text-sm font-medium text-teal-800">{url}</p>
          <p className="mt-2 text-xs text-teal-700">Source: {item.sourceApp}</p>
        </div>
      </div>
    );
  }

  if (item.type === "image") {
    return (
      <div className="rounded-lg border border-[color:var(--cliply-border)] bg-white/78 p-4 shadow-sm">
        <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-[color:var(--cliply-text)]">
          <ImageIcon className="size-4 text-[color:var(--cliply-amber)]" />
          {item.title}
        </div>
        <div className="grid aspect-[16/10] place-items-center overflow-hidden rounded-md border border-amber-100 bg-amber-50/60">
          {item.thumbnailUrl ? (
            <img
              src={item.thumbnailUrl}
              alt={item.imageAlt ?? item.title}
              className="max-h-full max-w-full object-contain"
            />
          ) : (
            <ImageIcon className="size-10 text-amber-500" />
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-[color:var(--cliply-border)] bg-white/78 p-4 shadow-sm">
      <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-[color:var(--cliply-text)]">
        <FileText className="size-4 text-slate-600" />
        {item.title}
      </div>
      <p className="whitespace-pre-wrap rounded-md bg-slate-50 p-4 text-sm leading-6 text-[color:var(--cliply-text)]">
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
