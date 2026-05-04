import type { ClipboardActionKind, ClipboardItem } from "@/lib/clipboardTypes";

export type ClipboardActionAvailability = Record<ClipboardActionKind, boolean>;

export function hasTextFallback(item: ClipboardItem) {
  const hasStoredText = item.formats.some(
    (format) => format.dataKind === "text" || format.dataKind === "html",
  );
  const hasLoadedText = Boolean(item.fullText?.trim());
  const hasPreviewFallback = item.type !== "image" && Boolean(item.previewText.trim());

  return hasStoredText || hasLoadedText || hasPreviewFallback;
}

export function canRunClipboardAction(kind: ClipboardActionKind, item: ClipboardItem) {
  if (kind === "togglePin" || kind === "delete") {
    return true;
  }

  return hasTextFallback(item);
}

export function getClipboardActionAvailability(
  item: ClipboardItem,
): ClipboardActionAvailability {
  return {
    paste: canRunClipboardAction("paste", item),
    copy: canRunClipboardAction("copy", item),
    pastePlain: canRunClipboardAction("pastePlain", item),
    togglePin: true,
    delete: true,
  };
}
