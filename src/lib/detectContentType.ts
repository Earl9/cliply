import type { ClipboardItemType } from "@/lib/clipboardTypes";

export function detectContentType(value: string): ClipboardItemType {
  const trimmed = value.trim();

  if (/^(https?|mailto|file):/i.test(trimmed)) {
    return "link";
  }

  const codePattern =
    /\b(import|export|function|const|let|class|interface|SELECT|FROM|WHERE)\b|=>|[{};]/;

  if (codePattern.test(trimmed)) {
    return "code";
  }

  return "text";
}
