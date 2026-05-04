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
      <div className="px-8 pt-3">
        <div className="flex h-14 items-center gap-[14px] rounded-[14px] border border-[color:var(--cliply-border-strong)] bg-white px-5 shadow-[0_5px_16px_rgba(15,23,42,0.055)] transition focus-within:border-[#8f73ff] focus-within:shadow-[0_0_0_4px_rgba(124,92,255,0.14),0_5px_16px_rgba(15,23,42,0.055)]">
        <Search className="size-5 shrink-0 text-[color:var(--cliply-muted)]" />
        <input
          ref={ref}
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          className="min-w-0 flex-1 border-0 bg-transparent text-[17px] text-[color:var(--cliply-text)] outline-none placeholder:text-[color:var(--cliply-placeholder)]"
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
