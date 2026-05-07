import { useEffect } from "react";
import { Maximize2, X } from "lucide-react";
import { IconButton } from "@/components/common/IconButton";
import type { ClipboardItem } from "@/lib/clipboardTypes";

type ImageViewerProps = {
  item: ClipboardItem | null;
  onClose: () => void;
};

export function ImageViewer({ item, onClose }: ImageViewerProps) {
  const imageUrl = item?.type === "image" ? (item.imageUrl ?? item.thumbnailUrl) : undefined;

  useEffect(() => {
    if (!imageUrl) {
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [imageUrl, onClose]);

  if (!item || !imageUrl) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-[60] grid grid-rows-[48px_1fr] bg-slate-950/72 backdrop-blur-sm"
      onPointerDown={onClose}
    >
      <header
        className="flex h-12 items-center justify-between border-b border-white/10 px-3 text-white"
        onPointerDown={(event) => event.stopPropagation()}
      >
        <div className="flex min-w-0 items-center gap-2">
          <span className="grid size-7 shrink-0 place-items-center rounded-lg bg-white/10 text-white">
            <Maximize2 className="size-4" />
          </span>
          <div className="min-w-0">
            <div className="truncate text-[13px] font-semibold">{item.title}</div>
            <div className="text-[11px] text-white/62">{imageTitle(item)}</div>
          </div>
        </div>
        <IconButton
          label="关闭图片"
          className="size-8 text-white/74 hover:bg-white/12 hover:text-white active:bg-white/16"
          onClick={onClose}
        >
          <X className="size-4" />
        </IconButton>
      </header>
      <div className="grid min-h-0 place-items-center p-4" onPointerDown={(event) => event.stopPropagation()}>
        <img
          src={imageUrl}
          alt={item.imageAlt ?? item.title}
          className="max-h-full max-w-full rounded-[10px] object-contain shadow-[0_20px_80px_rgba(0,0,0,0.35)]"
          draggable={false}
        />
      </div>
    </div>
  );
}

function imageTitle(item: ClipboardItem) {
  if (item.imageWidth && item.imageHeight) {
    return `${item.imageWidth} x ${item.imageHeight}`;
  }

  return item.sourceApp;
}
