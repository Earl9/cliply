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
    <div className="border-b border-[color:var(--cliply-border)] px-3 py-3">
      <div className="flex h-11 items-center gap-3 rounded-lg border border-[color:var(--cliply-border)] bg-white/80 px-3 shadow-sm">
        <Search className="size-4 shrink-0 text-[color:var(--cliply-muted)]" />
        <input
          ref={ref}
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          className="min-w-0 flex-1 border-0 bg-transparent text-sm text-[color:var(--cliply-text)] outline-none placeholder:text-[color:var(--cliply-faint)]"
          placeholder="Search clipboard, tags, apps..."
          aria-label="Search clipboard"
        />
        {query ? (
          <button
            type="button"
            aria-label="Clear search"
            title="Clear search"
            onClick={() => onQueryChange("")}
            className="grid size-6 place-items-center rounded text-[color:var(--cliply-muted)] transition hover:bg-slate-100 hover:text-[color:var(--cliply-text)]"
          >
            <X className="size-3.5" />
          </button>
        ) : (
          <ShortcutKey keys={["Ctrl", "K"]} compact />
        )}
      </div>
    </div>
  );
});
