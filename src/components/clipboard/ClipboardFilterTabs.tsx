import { PillTabs } from "@/components/common/PillTabs";
import type { ClipboardFilter } from "@/lib/clipboardTypes";

type ClipboardFilterTabsProps = {
  filter: ClipboardFilter;
  counts: Record<ClipboardFilter, number>;
  onFilterChange: (filter: ClipboardFilter) => void;
};

export function ClipboardFilterTabs({ filter, counts, onFilterChange }: ClipboardFilterTabsProps) {
  return (
    <div className="border-b border-[color:var(--cliply-border)]">
      <PillTabs
        value={filter}
        onValueChange={onFilterChange}
        options={[
          { value: "all", label: "All", count: counts.all },
          { value: "text", label: "Text", count: counts.text },
          { value: "link", label: "Links", count: counts.link },
          { value: "image", label: "Images", count: counts.image },
          { value: "code", label: "Code", count: counts.code },
          { value: "pinned", label: "Pinned", count: counts.pinned },
        ]}
      />
    </div>
  );
}
