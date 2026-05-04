import { Search } from "lucide-react";
import { ShortcutKey } from "@/components/common/ShortcutKey";

export function ClipboardSearchBar() {
  return (
    <div className="border-b border-[color:var(--cliply-border)] px-3 py-3">
      <div className="flex h-11 items-center gap-3 rounded-lg border border-[color:var(--cliply-border)] bg-white/80 px-3 shadow-sm">
        <Search className="size-4 shrink-0 text-[color:var(--cliply-muted)]" />
        <input
          className="min-w-0 flex-1 border-0 bg-transparent text-sm text-[color:var(--cliply-text)] outline-none placeholder:text-[color:var(--cliply-faint)]"
          placeholder="Search clipboard, tags, apps..."
          aria-label="Search clipboard"
        />
        <ShortcutKey keys={["Ctrl", "K"]} compact />
      </div>
    </div>
  );
}
