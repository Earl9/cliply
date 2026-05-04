import { useEffect, useRef } from "react";
import { clsx } from "clsx";
import { ClipboardDetailPane } from "@/components/clipboard/ClipboardDetailPane";
import { ClipboardFilterTabs } from "@/components/clipboard/ClipboardFilterTabs";
import { ClipboardList } from "@/components/clipboard/ClipboardList";
import { ClipboardSearchBar } from "@/components/clipboard/ClipboardSearchBar";
import { FooterShortcuts } from "@/components/shell/FooterShortcuts";
import { TitleBar } from "@/components/shell/TitleBar";
import { useClipboardStore } from "@/stores/clipboardStore";

export function AppWindow() {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const {
    state,
    filteredItems,
    selectedItem,
    counts,
    actionStatus,
    setQuery,
    setFilter,
    selectItem,
    runMockAction,
    togglePinItem,
    clearHistory,
    handleGlobalKeyDown,
  } = useClipboardStore();

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key.toLowerCase() === "k" && event.ctrlKey) {
        event.preventDefault();
        searchInputRef.current?.focus();
        searchInputRef.current?.select();
        return;
      }

      if (event.key === "Escape") {
        if (state.query) {
          event.preventDefault();
          setQuery("");
        }
        return;
      }

      handleGlobalKeyDown(event);
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [handleGlobalKeyDown, setQuery, state.query]);

  useEffect(() => {
    searchInputRef.current?.focus();
  }, []);

  return (
    <main className="grid min-h-screen place-items-center bg-[radial-gradient(circle_at_20%_10%,rgba(115,87,246,0.08),transparent_32%),radial-gradient(circle_at_80%_80%,rgba(37,99,235,0.08),transparent_30%),#eef2f8] p-4">
      <div className="relative flex h-[min(720px,calc(100vh-32px))] min-h-[min(600px,calc(100vh-32px))] w-[min(1080px,calc(100vw-32px))] min-w-[min(880px,calc(100vw-32px))] flex-col overflow-hidden rounded-[18px] border border-white/65 bg-[color:var(--cliply-panel)] shadow-[var(--cliply-shadow)] backdrop-blur-2xl">
        <TitleBar onClearHistory={clearHistory} />
        <ClipboardSearchBar ref={searchInputRef} query={state.query} onQueryChange={setQuery} />
        <ClipboardFilterTabs filter={state.filter} counts={counts} onFilterChange={setFilter} />
        <div className="grid min-h-0 flex-1 grid-cols-[minmax(360px,0.92fr)_minmax(420px,1.08fr)] gap-6 px-7 pb-4 pt-5">
          <ClipboardList
            items={filteredItems}
            totalCount={state.items.length}
            selectedId={state.selectedId}
            query={state.query}
            filter={state.filter}
            onSelectItem={selectItem}
            onTogglePin={togglePinItem}
          />
          <ClipboardDetailPane item={selectedItem} onAction={runMockAction} />
        </div>
        <FooterShortcuts />
        {actionStatus ? (
          <div
            className={clsx(
              "pointer-events-none absolute bottom-[70px] left-1/2 max-w-[min(520px,calc(100%-48px))] -translate-x-1/2 rounded-xl border px-4 py-2 text-sm font-medium shadow-lg",
              actionStatus.tone === "error"
                ? "border-rose-200 bg-rose-50 text-rose-700"
                : "border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] text-[color:var(--cliply-text)]",
            )}
          >
            {actionStatus.label}: {actionStatus.itemTitle}
          </div>
        ) : null}
      </div>
    </main>
  );
}
