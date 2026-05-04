export type ClipboardItemType = "text" | "link" | "image" | "code";

export type ClipboardFilter = "all" | ClipboardItemType | "pinned";

export type ClipboardFormatKind = "text" | "html" | "image_file" | "binary_file" | "external_ref";

export type ClipboardFormat = {
  id: string;
  formatName: string;
  mimeType?: string;
  dataKind: ClipboardFormatKind;
  sizeBytes: number;
};

export type ClipboardItem = {
  id: string;
  type: ClipboardItemType;
  title: string;
  previewText: string;
  fullText?: string;
  sourceApp: string;
  sourceWindow?: string;
  copiedAt: string;
  createdAt: string;
  sizeBytes: number;
  isPinned: boolean;
  sensitiveScore: number;
  tags: string[];
  thumbnailUrl?: string;
  imageUrl?: string;
  imageWidth?: number;
  imageHeight?: number;
  imageAlt?: string;
  formats: ClipboardFormat[];
};

export type ClipboardItemDetail = ClipboardItem;

export type ClipboardActionKind = "paste" | "copy" | "pastePlain" | "togglePin" | "delete";

export type ClipboardActionStatus = {
  label: string;
  itemTitle: string;
  at: number;
  tone?: "success" | "warning" | "error";
} | null;

export type ClipboardState = {
  items: ClipboardItem[];
  selectedId: string | null;
  query: string;
  filter: ClipboardFilter;
  loading: boolean;
  detail: ClipboardItemDetail | null;
  errorMessage: string | null;
};
