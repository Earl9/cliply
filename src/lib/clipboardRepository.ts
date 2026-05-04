import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import type { ClipboardFilter, ClipboardItem, ClipboardItemType } from "@/lib/clipboardTypes";
import { mockClipboardItems } from "@/lib/mockClipboardItems";

type ClipboardItemDto = {
  id: string;
  itemType: ClipboardItemType;
  title: string;
  previewText: string;
  sourceApp: string;
  sourceWindow?: string | null;
  copiedAt: string;
  createdAt: string;
  relativeTime: string;
  sizeBytes: number;
  isPinned: boolean;
  sensitiveScore?: number;
  tags: string[];
  thumbnailPath?: string | null;
};

type ClipboardFormatDto = {
  formatName: string;
  mimeType?: string | null;
  dataKind: string;
  sizeBytes: number;
};

type ClipboardItemDetailDto = {
  item: ClipboardItemDto;
  fullText?: string | null;
  thumbnailPath?: string | null;
  imagePath?: string | null;
  imageWidth?: number | null;
  imageHeight?: number | null;
  formats: ClipboardFormatDto[];
};

type ListClipboardItemsOptions = {
  query: string;
  filter: ClipboardFilter;
};

export async function listClipboardItems({
  query,
  filter,
}: ListClipboardItemsOptions): Promise<ClipboardItem[]> {
  if (!isTauri()) {
    return listMockClipboardItems({ query, filter });
  }

  const itemType = filter !== "all" && filter !== "pinned" ? filter : null;
  const pinnedOnly = filter === "pinned";

  const items = await invokeWithMockFallback(
    () =>
      invoke<ClipboardItemDto[]>("list_clipboard_items", {
        query: query || null,
        itemType,
        pinnedOnly,
        limit: 100,
        offset: 0,
      }),
    () => listMockClipboardItems({ query, filter }).map(clipboardItemToDto),
  );

  return items.map(dtoToClipboardItem);
}

export async function getClipboardItemDetail(id: string): Promise<ClipboardItem | null> {
  if (!isTauri()) {
    return mockClipboardItems.find((item) => item.id === id) ?? null;
  }

  const detail = await invokeWithMockFallback(
    () => invoke<ClipboardItemDetailDto>("get_clipboard_item_detail", { id }),
    () => {
      const item = mockClipboardItems.find((mockItem) => mockItem.id === id);
      if (!item) {
        return null;
      }

      return {
        item: clipboardItemToDto(item),
        fullText: item.fullText ?? item.previewText,
        thumbnailPath: item.thumbnailUrl,
        imagePath: item.imageUrl,
        imageWidth: item.imageWidth,
        imageHeight: item.imageHeight,
        formats: item.formats.map((format) => ({
          formatName: format.formatName,
          mimeType: format.mimeType,
          dataKind: format.dataKind,
          sizeBytes: format.sizeBytes,
        })),
      } satisfies ClipboardItemDetailDto;
    },
  );

  if (!detail) {
    return null;
  }

  return detailDtoToClipboardItem(detail);
}

export async function togglePinClipboardItem(id: string): Promise<ClipboardItem | null> {
  if (!isTauri()) {
    return null;
  }

  const item = await invokeWithMockFallback(
    () => invoke<ClipboardItemDto>("toggle_pin_clipboard_item", { id }),
    () => null,
  );
  if (!item) {
    return null;
  }

  return dtoToClipboardItem(item);
}

export async function deleteClipboardItem(id: string): Promise<void> {
  if (!isTauri()) {
    return;
  }

  await invokeWithMockFallback(
    () => invoke<void>("delete_clipboard_item", { id }),
    () => undefined,
  );
}

export async function clearClipboardHistory(includePinned = false): Promise<void> {
  if (!isTauri()) {
    return;
  }

  await invokeWithMockFallback(
    () => invoke<void>("clear_clipboard_history", { includePinned }),
    () => undefined,
  );
}

export async function copyClipboardItem(id: string): Promise<void> {
  if (!isTauri()) {
    return;
  }

  await invoke<void>("copy_clipboard_item", { id });
}

export async function pasteClipboardItem(id: string): Promise<void> {
  if (!isTauri()) {
    return;
  }

  await invoke<void>("paste_clipboard_item", { id });
}

export async function pastePlainText(id: string): Promise<void> {
  if (!isTauri()) {
    return;
  }

  await invoke<void>("paste_plain_text", { id });
}

async function invokeWithMockFallback<T>(
  invokeCommand: () => Promise<T>,
  fallback: () => T,
): Promise<T> {
  try {
    return await invokeCommand();
  } catch (error) {
    console.warn("[cliply:tauri-fallback]", error);
    return fallback();
  }
}

function listMockClipboardItems({ query, filter }: ListClipboardItemsOptions) {
  const normalizedQuery = query.trim().toLowerCase();

  return mockClipboardItems.filter((item) => {
    const matchesFilter =
      filter === "all" || (filter === "pinned" ? item.isPinned : item.type === filter);

    if (!matchesFilter) {
      return false;
    }

    if (!normalizedQuery) {
      return true;
    }

    const haystack = [
      item.title,
      item.previewText,
      item.fullText,
      item.sourceApp,
      item.sourceWindow,
      item.tags.join(" "),
    ]
      .filter(Boolean)
      .join(" ")
      .toLowerCase();

    return haystack.includes(normalizedQuery);
  });
}

function detailDtoToClipboardItem(detail: ClipboardItemDetailDto): ClipboardItem {
  return {
    ...dtoToClipboardItem(detail.item),
    fullText: detail.fullText ?? detail.item.previewText,
    thumbnailUrl: toAssetUrl(detail.thumbnailPath),
    imageUrl: toAssetUrl(detail.imagePath ?? detail.thumbnailPath),
    imageWidth: detail.imageWidth ?? undefined,
    imageHeight: detail.imageHeight ?? undefined,
    formats: detail.formats.map((format, index) => ({
      id: `${detail.item.id}-format-${index}`,
      formatName: format.formatName,
      mimeType: format.mimeType ?? undefined,
      dataKind: toFormatKind(format.dataKind),
      sizeBytes: format.sizeBytes,
    })),
  };
}

function dtoToClipboardItem(item: ClipboardItemDto): ClipboardItem {
  return {
    id: item.id,
    type: item.itemType,
    title: item.title,
    previewText: item.previewText,
    sourceApp: item.sourceApp,
    sourceWindow: item.sourceWindow ?? undefined,
    copiedAt: item.copiedAt,
    createdAt: item.createdAt,
    sizeBytes: item.sizeBytes,
    isPinned: item.isPinned,
    sensitiveScore: item.sensitiveScore ?? 0,
    tags: item.tags ?? [],
    thumbnailUrl: toAssetUrl(item.thumbnailPath),
    formats: [],
  };
}

function clipboardItemToDto(item: ClipboardItem): ClipboardItemDto {
  return {
    id: item.id,
    itemType: item.type,
    title: item.title,
    previewText: item.previewText,
    sourceApp: item.sourceApp,
    sourceWindow: item.sourceWindow,
    copiedAt: item.copiedAt,
    createdAt: item.createdAt,
    relativeTime: "",
    sizeBytes: item.sizeBytes,
    isPinned: item.isPinned,
    sensitiveScore: item.sensitiveScore,
    tags: item.tags,
    thumbnailPath: item.thumbnailUrl,
  };
}

function toFormatKind(value: string) {
  if (
    value === "text" ||
    value === "html" ||
    value === "image_file" ||
    value === "binary_file" ||
    value === "external_ref"
  ) {
    return value;
  }

  return "external_ref";
}

function toAssetUrl(path?: string | null) {
  if (!path) {
    return undefined;
  }

  if (path.startsWith("data:") || path.startsWith("http://") || path.startsWith("https://")) {
    return path;
  }

  return isTauri() ? convertFileSrc(path) : path;
}
