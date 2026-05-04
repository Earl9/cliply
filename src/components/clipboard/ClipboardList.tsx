import { ClipboardListItem } from "@/components/clipboard/ClipboardListItem";

const staticItems = [
  {
    type: "Code",
    app: "Visual Studio Code",
    text: "const user = await getProfile(session.userId);",
    time: "Just now",
    pinned: true,
  },
  {
    type: "Link",
    app: "Chrome",
    text: "https://github.com/tauri-apps/tauri",
    time: "1 min ago",
    pinned: false,
  },
  {
    type: "Text",
    app: "Notepad",
    text: "Windows MVP first, keep platform adapters clean.",
    time: "8 min ago",
    pinned: false,
  },
  {
    type: "Image",
    app: "Snipping Tool",
    text: "Screenshot 1160 x 760",
    time: "18 min ago",
    pinned: true,
  },
];

export function ClipboardList() {
  return (
    <section className="flex min-h-0 w-[42%] min-w-[290px] flex-col border-r border-[color:var(--cliply-border)]">
      <div className="flex h-11 shrink-0 items-center justify-between px-4 text-xs font-semibold uppercase tracking-normal text-[color:var(--cliply-muted)]">
        <span>History</span>
        <span>{staticItems.length} items</span>
      </div>
      <div className="cliply-scrollbar min-h-0 flex-1 space-y-2 overflow-auto px-3 pb-3">
        {staticItems.map((item, index) => (
          <ClipboardListItem key={`${item.app}-${item.text}`} item={item} selected={index === 0} />
        ))}
      </div>
    </section>
  );
}
