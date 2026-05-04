import { PillTabs } from "@/components/common/PillTabs";
import type { ClipboardFilter } from "@/lib/clipboardTypes";

type ClipboardFilterTabsProps = {
  filter: ClipboardFilter;
  counts: Record<ClipboardFilter, number>;
  onFilterChange: (filter: ClipboardFilter) => void;
};

export function ClipboardFilterTabs({ filter, counts, onFilterChange }: ClipboardFilterTabsProps) {
  return (
    <div className="px-7 pt-4">
      <PillTabs
        value={filter}
        onValueChange={onFilterChange}
        options={[
          { value: "all", label: "全部", count: counts.all },
          { value: "text", label: "文本", count: counts.text },
          { value: "link", label: "链接", count: counts.link },
          { value: "image", label: "图片", count: counts.image },
          { value: "code", label: "代码", count: counts.code },
          { value: "pinned", label: "固定", count: counts.pinned },
        ]}
      />
    </div>
  );
}
