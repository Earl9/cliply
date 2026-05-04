import { PillTabs } from "@/components/common/PillTabs";

export function ClipboardFilterTabs() {
  return (
    <div className="border-b border-[color:var(--cliply-border)]">
      <PillTabs
        value="all"
        onValueChange={() => undefined}
        options={[
          { value: "all", label: "All", count: 8 },
          { value: "text", label: "Text" },
          { value: "link", label: "Links" },
          { value: "image", label: "Images" },
          { value: "code", label: "Code" },
          { value: "pinned", label: "Pinned" },
        ]}
      />
    </div>
  );
}
