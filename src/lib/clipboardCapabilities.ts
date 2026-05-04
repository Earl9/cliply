import type { ClipboardActionKind, ClipboardItem } from "@/lib/clipboardTypes";

export type ClipboardActionAvailability = Record<ClipboardActionKind, boolean>;

export function hasTextFallback(item: ClipboardItem) {
  if (item.sensitiveScore >= 50) {
    return false;
  }

  const hasStoredText = item.formats.some(
    (format) => format.dataKind === "text" || format.dataKind === "html",
  );
  const hasStoredImage = item.formats.some((format) => format.dataKind === "image_file");
  const hasLoadedText = Boolean(item.fullText?.trim());
  const hasPreviewFallback = item.type !== "image" && Boolean(item.previewText.trim());

  return hasStoredText || hasStoredImage || hasLoadedText || hasPreviewFallback;
}

export function canRunClipboardAction(kind: ClipboardActionKind, item: ClipboardItem) {
  if (kind === "togglePin" || kind === "delete") {
    return true;
  }

  if (kind === "pastePlain") {
    return item.type !== "image" && hasTextFallback(item);
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
