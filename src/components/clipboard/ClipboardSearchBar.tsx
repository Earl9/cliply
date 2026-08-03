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
      <div className="cliply-search-shell cliply-interactive flex h-9 items-center gap-2.5 rounded-[6px] border border-[color:var(--cliply-border)] bg-[color:var(--cliply-input-bg)] px-3 focus-within:border-[color:var(--cliply-accent)]">
        <Search className="size-4 shrink-0 text-[color:var(--cliply-faint)]" />
        <input
          ref={ref}
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          className="min-w-0 flex-1 border-0 bg-transparent text-[13px] text-[color:var(--cliply-text)] outline-none placeholder:text-[color:var(--cliply-placeholder)]"
          placeholder="搜索剪贴板、标签、应用..."
          aria-label="搜索剪贴板"
        />
        {query ? (
          <button
            type="button"
            aria-label="清空搜索"
            title="清空搜索"
            onClick={() => onQueryChange("")}
            className="cliply-interactive grid size-5 place-items-center rounded-[4px] text-[color:var(--cliply-muted)] hover:bg-[color:var(--cliply-muted-bg)] hover:text-[color:var(--cliply-text)]"
          >
            <X className="size-3" />
          </button>
        ) : (
          <ShortcutKey keys={["Ctrl", "K"]} compact />
        )}
      </div>
    );
  },
);
