import { PillTabs } from "@/components/common/PillTabs";
import type { ClipboardFilter } from "@/lib/clipboardTypes";

type ClipboardFilterTabsProps = {
  filter: ClipboardFilter;
  onFilterChange: (filter: ClipboardFilter) => void;
};

export function ClipboardFilterTabs({ filter, onFilterChange }: ClipboardFilterTabsProps) {
  return (
    <PillTabs
      value={filter}
      onValueChange={onFilterChange}
      options={[
        { value: "all", label: "全部" },
        { value: "text", label: "文本" },
        { value: "link", label: "链接" },
        { value: "image", label: "图片" },
        { value: "code", label: "代码" },
        { value: "pinned", label: "固定" },
      ]}
    />
  );
}
