import { useEffect, useRef } from "react";
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

  return (
    <main className="grid min-h-screen place-items-center bg-[radial-gradient(circle_at_20%_15%,rgba(14,159,154,0.16),transparent_28%),radial-gradient(circle_at_82%_16%,rgba(101,84,246,0.20),transparent_30%),linear-gradient(135deg,#edf4fb_0%,#f8fbff_46%,#eaf0f7_100%)] p-4">
      <div className="relative flex h-[min(760px,calc(100vh-32px))] w-[min(1160px,calc(100vw-32px))] min-w-0 flex-col overflow-hidden rounded-2xl border border-white/75 bg-[color:var(--cliply-panel)] shadow-[var(--cliply-shadow)] backdrop-blur-2xl">
        <TitleBar />
        <ClipboardSearchBar ref={searchInputRef} query={state.query} onQueryChange={setQuery} />
        <ClipboardFilterTabs filter={state.filter} counts={counts} onFilterChange={setFilter} />
        <div className="flex min-h-0 flex-1">
          <ClipboardList
            items={filteredItems}
            totalCount={state.items.length}
            selectedId={state.selectedId}
            query={state.query}
            onSelectItem={selectItem}
            onTogglePin={togglePinItem}
          />
          <ClipboardDetailPane item={selectedItem} onAction={runMockAction} />
        </div>
        <FooterShortcuts />
        {actionStatus ? (
          <div className="pointer-events-none absolute bottom-16 left-1/2 -translate-x-1/2 rounded-lg border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] px-3 py-2 text-sm font-medium text-[color:var(--cliply-text)] shadow-lg">
            {actionStatus.label}: {actionStatus.itemTitle}
          </div>
        ) : null}
      </div>
    </main>
  );
}
