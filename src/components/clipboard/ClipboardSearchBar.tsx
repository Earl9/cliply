import { forwardRef } from "react";
import { Search, X } from "lucide-react";
import { ShortcutKey } from "@/components/common/ShortcutKey";

type ClipboardSearchBarProps = {
  query: string;
  onQueryChange: (query: string) => void;
};

export const ClipboardSearchBar = forwardRef<HTMLInputElement, ClipboardSearchBarProps>(
  function ClipboardSearchBar({ query, onQueryChange }, ref) {
    return (
      <div className="px-7">
        <div className="flex h-12 items-center gap-3 rounded-xl border border-[color:var(--cliply-border)] bg-white/70 px-4 shadow-sm transition focus-within:border-[color:var(--cliply-accent)] focus-within:bg-white focus-within:ring-2 focus-within:ring-[rgba(115,87,246,0.18)]">
        <Search className="size-4 shrink-0 text-[color:var(--cliply-muted)]" />
        <input
          ref={ref}
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          className="min-w-0 flex-1 border-0 bg-transparent text-[15px] text-[color:var(--cliply-text)] outline-none placeholder:text-[color:var(--cliply-faint)]"
          placeholder="搜索剪贴板、标签、应用..."
          aria-label="搜索剪贴板"
        />
        {query ? (
          <button
            type="button"
            aria-label="清空搜索"
            title="清空搜索"
            onClick={() => onQueryChange("")}
            className="grid size-7 place-items-center rounded-lg text-[color:var(--cliply-muted)] transition hover:bg-slate-100 hover:text-[color:var(--cliply-text)]"
          >
            <X className="size-3.5" />
          </button>
        ) : (
          <ShortcutKey keys={["Ctrl", "K"]} compact />
        )}
        </div>
      </div>
    );
  },
);
