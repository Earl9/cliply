import { forwardRef } from "react";
import { Search, X } from "lucide-react";

type ClipboardSearchBarProps = {
  query: string;
  onQueryChange: (query: string) => void;
};

export const ClipboardSearchBar = forwardRef<HTMLInputElement, ClipboardSearchBarProps>(
  function ClipboardSearchBar({ query, onQueryChange }, ref) {
    return (
      <div className="cliply-search-shell flex h-10 min-w-0 flex-1 items-center gap-2.5 border border-[color:var(--cliply-border)] bg-[color:var(--cliply-input-bg)] px-3.5 focus-within:border-[color:var(--cliply-focus-border)]">
        <Search className="cliply-search-icon size-[15px] shrink-0 text-[color:var(--cliply-faint)]" strokeWidth={1.8} />
        <input
          ref={ref}
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          className="min-w-0 flex-1 border-0 bg-transparent text-[13px] text-[color:var(--cliply-text)] outline-none placeholder:text-[color:var(--cliply-placeholder)]"
          placeholder="搜索内容、来源或标签"
          aria-label="搜索剪贴板"
        />
        {query ? (
          <button
            type="button"
            aria-label="清空搜索"
            title="清空搜索"
            onClick={() => onQueryChange("")}
            className="cliply-interactive grid size-6 place-items-center text-[color:var(--cliply-muted)] hover:bg-[color:var(--cliply-muted-bg)] hover:text-[color:var(--cliply-text)]"
          >
            <X className="size-3.5" />
          </button>
        ) : (
          <span className="cliply-command-key" aria-hidden="true">Ctrl K</span>
        )}
      </div>
    );
  },
);
