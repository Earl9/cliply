import { Clipboard } from "lucide-react";

type EmptyStateProps = {
  title?: string;
  description?: string;
};

export function EmptyState({
  title = "暂无剪贴板记录",
  description = "复制文本、链接或图片后，记录将显示在此处。",
}: EmptyStateProps) {
  return (
    <div className="grid min-h-48 place-items-center p-6 text-center">
      <div>
        <Clipboard className="mx-auto mb-3 size-6 text-[color:var(--cliply-faint)]" />
        <p className="text-sm font-medium text-[color:var(--cliply-text)]">{title}</p>
        <p className="mt-1 max-w-64 text-[13px] text-[color:var(--cliply-muted)]">
          {description}
        </p>
      </div>
    </div>
  );
}
