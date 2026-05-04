import { useEffect, useRef } from "react";
import { clsx } from "clsx";
import { ClipboardDetailPane } from "@/components/clipboard/ClipboardDetailPane";
import { ClipboardFilterTabs } from "@/components/clipboard/ClipboardFilterTabs";
import { ClipboardList } from "@/components/clipboard/ClipboardList";
import { ClipboardSearchBar } from "@/components/clipboard/ClipboardSearchBar";
import { ConfirmDialog } from "@/components/common/ConfirmDialog";
import { AboutDialog } from "@/components/settings/AboutDialog";
import { SettingsDialog } from "@/components/settings/SettingsDialog";
import { FooterShortcuts } from "@/components/shell/FooterShortcuts";
import { PrivacyBanner } from "@/components/shell/PrivacyBanner";
import { TitleBar } from "@/components/shell/TitleBar";
import { hideMainWindow, toggleAlwaysOnTop } from "@/lib/windowAdapter";
import { useClipboardStore } from "@/stores/clipboardStore";
import { useUiStore } from "@/stores/uiStore";

export function AppWindow() {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const {
    state,
    filteredItems,
    selectedItem,
    counts,
    actionStatus,
    settings,
    dialogs,
    setQuery,
    setFilter,
    selectItem,
    runMockAction,
    togglePinItem,
    requestClearHistory,
    confirmClearHistory,
    setSettings,
    openSettings,
    openAbout,
    closeDialogs,
    toggleMonitoring,
    handleGlobalKeyDown,
  } = useClipboardStore();
  const { windowPinned, setWindowPinned } = useUiStore();

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
        } else {
          event.preventDefault();
          void hideMainWindow();
        }
        return;
      }

      handleGlobalKeyDown(event);
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [handleGlobalKeyDown, setQuery, state.query]);

  useEffect(() => {
    if (settings.focusSearchOnOpen) {
      searchInputRef.current?.focus();
    }
  }, [settings.focusSearchOnOpen]);

  useEffect(() => {
    const removeListeners: Array<() => void> = [];

    const registerTauriListeners = async () => {
      const { isTauri } = await import("@tauri-apps/api/core");
      if (!isTauri()) {
        return;
      }

      const { listen } = await import("@tauri-apps/api/event");
      removeListeners.push(await listen("cliply-open-settings", openSettings));
      removeListeners.push(await listen("cliply-open-about", openAbout));
      removeListeners.push(await listen("cliply-open-clear-history", requestClearHistory));
    };

    void registerTauriListeners();
    return () => removeListeners.forEach((unlisten) => unlisten());
  }, [openAbout, openSettings, requestClearHistory]);

  const onToggleWindowPin = async () => {
    const nextPinned = !windowPinned;
    await toggleAlwaysOnTop(nextPinned);
    setWindowPinned(nextPinned);
  };

  return (
    <main className="cliply-root">
      <div className="cliply-window cliply-window-enter relative flex flex-col">
        <TitleBar
          windowPinned={windowPinned}
          monitoringPaused={settings.pauseMonitoring}
          onToggleWindowPin={onToggleWindowPin}
          onOpenSettings={openSettings}
          onOpenAbout={openAbout}
          onClearHistory={requestClearHistory}
          onToggleMonitoring={toggleMonitoring}
        />
        <ClipboardSearchBar ref={searchInputRef} query={state.query} onQueryChange={setQuery} />
        <PrivacyBanner
          monitoringPaused={settings.pauseMonitoring}
          errorMessage={state.errorMessage}
          onResumeMonitoring={toggleMonitoring}
        />
        <ClipboardFilterTabs filter={state.filter} counts={counts} onFilterChange={setFilter} />
        <div className="grid min-h-0 flex-1 grid-cols-[minmax(390px,0.88fr)_minmax(520px,1.12fr)] gap-8 px-8 pb-6 pt-7">
          <ClipboardList
            items={filteredItems}
            totalCount={state.items.length}
            selectedId={state.selectedId}
            query={state.query}
            filter={state.filter}
            loading={state.loading}
            errorMessage={state.errorMessage}
            onSelectItem={selectItem}
            onTogglePin={togglePinItem}
          />
          <ClipboardDetailPane item={selectedItem} onAction={runMockAction} />
        </div>
        <FooterShortcuts monitoringPaused={settings.pauseMonitoring} />
        {actionStatus ? (
          <div
            className={clsx(
              "pointer-events-none absolute bottom-[70px] left-1/2 max-w-[min(520px,calc(100%-48px))] -translate-x-1/2 rounded-xl border px-4 py-2 text-sm font-medium shadow-lg",
              actionStatus.tone === "error"
                ? "border-rose-200 bg-rose-50 text-rose-700"
                : actionStatus.tone === "warning"
                  ? "border-amber-200 bg-amber-50 text-amber-800"
                : "border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] text-[color:var(--cliply-text)]",
            )}
          >
            {actionStatus.label}: {actionStatus.itemTitle}
          </div>
        ) : null}
        <SettingsDialog
          open={dialogs.settings}
          settings={settings}
          onClose={closeDialogs}
          onSave={setSettings}
          onClearHistory={requestClearHistory}
        />
        <AboutDialog open={dialogs.about} onClose={closeDialogs} />
        <ConfirmDialog
          open={dialogs.clearHistory}
          title="清空剪贴板历史？"
          description="将清空所有未固定记录，固定记录会保留。此操作不可撤销。"
          confirmLabel="清空历史"
          danger
          onConfirm={confirmClearHistory}
          onClose={closeDialogs}
        />
      </div>
    </main>
  );
}
