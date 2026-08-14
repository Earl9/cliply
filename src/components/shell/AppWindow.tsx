import {
  lazy,
  Suspense,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type MouseEvent,
} from "react";
import { isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  Clipboard,
  ClipboardList as ClipboardListIcon,
  Copy,
  Database,
  ExternalLink,
  Eye,
  FileArchive,
  FileText,
  Filter,
  Image,
  Link2,
  PauseCircle,
  Pin,
  PlayCircle,
  Search,
  Settings,
  Trash2,
  Type,
  X,
} from "lucide-react";
import { ClipboardDetailPane } from "@/components/clipboard/ClipboardDetailPane";
import { ClipboardList } from "@/components/clipboard/ClipboardList";
import { ClipboardSearchBar } from "@/components/clipboard/ClipboardSearchBar";
import { ConfirmDialog } from "@/components/common/ConfirmDialog";
import { ContextMenu, type ContextMenuSection, type ContextMenuState } from "@/components/common/ContextMenu";
import { ImageViewer } from "@/components/common/ImageViewer";
import { GlobalToast, type ToastMessage } from "@/components/common/Toast";
import { FooterShortcuts } from "@/components/shell/FooterShortcuts";
import { PrivacyBanner } from "@/components/shell/PrivacyBanner";
import { TitleBar } from "@/components/shell/TitleBar";
import { getClipboardActionAvailability } from "@/lib/clipboardCapabilities";
import type {
  ClipboardActionKind,
  ClipboardFilter,
  ClipboardItem,
} from "@/lib/clipboardTypes";
import { checkCliplyUpdate } from "@/lib/updateService";
import { hideMainWindow, toggleAlwaysOnTop } from "@/lib/windowAdapter";
import {
  applyCliplyTheme,
  resolveCliplyThemeFromSettings,
  storeCliplyThemeName,
} from "@/theme/theme";
import { useClipboardStore } from "@/stores/clipboardStore";
import { useUiStore } from "@/stores/uiStore";

const SettingsDialog = lazy(() =>
  import("@/components/settings/SettingsDialog").then((module) => ({
    default: module.SettingsDialog,
  })),
);
const AboutDialog = lazy(() =>
  import("@/components/settings/AboutDialog").then((module) => ({
    default: module.AboutDialog,
  })),
);

function shouldAllowNativeContextMenu(target: EventTarget | null) {
  const selection = window.getSelection();
  if (selection && !selection.isCollapsed && selection.toString().length > 0) {
    return true;
  }

  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  );
}

function shouldRunAutoUpdateCheck(
  lastCheckedAt: string | null | undefined,
  interval: "manual" | "daily" | "weekly",
) {
  if (interval === "manual") {
    return false;
  }
  if (!lastCheckedAt) {
    return true;
  }

  const timestamp = Date.parse(lastCheckedAt);
  if (!Number.isFinite(timestamp)) {
    return true;
  }

  const intervalMs = interval === "weekly" ? 7 * 24 * 60 * 60 * 1000 : 24 * 60 * 60 * 1000;
  return Date.now() - timestamp >= intervalMs;
}

function DialogLoadingFallback({ label }: { label: string }) {
  return (
    <div className="cliply-overlay absolute inset-0 z-30 grid place-items-center bg-black/35 px-6">
      <div
        role="status"
        className="cliply-dialog rounded-[8px] border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] px-4 py-3 text-sm text-[color:var(--cliply-muted)] shadow-[var(--cliply-shadow-dialog)]"
      >
        {label}
      </div>
    </div>
  );
}

export function AppWindow() {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const {
    state,
    filteredItems,
    selectedItem,
    actionStatus,
    settings,
    dialogs,
    setQuery,
    setFilter,
    selectItem,
    runMockAction,
    togglePinItem,
    copyDebugPath,
    requestClearHistory,
    confirmClearHistory,
    setSettings,
    openSettings,
    openAbout,
    closeDialogs,
    clearActionStatus,
    toggleMonitoring,
    handleGlobalKeyDown,
  } = useClipboardStore();
  const { windowPinned, setWindowPinned } = useUiStore();
  const [contextMenu, setContextMenu] = useState<ContextMenuState>(null);
  const [imageViewerItem, setImageViewerItem] = useState<ClipboardItem | null>(null);
  const [systemPrefersDark, setSystemPrefersDark] = useState(() => getSystemPrefersDark());
  const settingsRef = useRef(settings);
  const runMockActionRef = useRef(runMockAction);

  useLayoutEffect(() => {
    runMockActionRef.current = runMockAction;
  }, [runMockAction]);

  const runClipboardAction = useCallback(
    (kind: ClipboardActionKind, itemId?: string) => {
      runMockActionRef.current(kind, itemId);
    },
    [],
  );

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key.toLowerCase() === "k" && event.ctrlKey) {
        event.preventDefault();
        searchInputRef.current?.focus();
        searchInputRef.current?.select();
        return;
      }

      if (event.key === "Escape") {
        if (imageViewerItem) {
          event.preventDefault();
          setImageViewerItem(null);
          return;
        }

        if (contextMenu) {
          event.preventDefault();
          setContextMenu(null);
          return;
        }

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
  }, [contextMenu, handleGlobalKeyDown, imageViewerItem, setQuery, state.query]);

  useEffect(() => {
    const blockDefaultContextMenu = (event: Event) => {
      if (shouldAllowNativeContextMenu(event.target)) {
        return;
      }

      event.preventDefault();
    };
    window.addEventListener("contextmenu", blockDefaultContextMenu);
    return () => window.removeEventListener("contextmenu", blockDefaultContextMenu);
  }, []);

  useEffect(() => {
    if (settings.focusSearchOnOpen) {
      searchInputRef.current?.focus();
    }
  }, [settings.focusSearchOnOpen]);

  useEffect(() => {
    const theme = resolveCliplyThemeFromSettings({
      ...settings,
      autoTheme: { ...settings.autoTheme, enabled: false },
      systemPrefersDark,
    });
    applyCliplyTheme(theme);
    storeCliplyThemeName(theme.name);
  }, [settings, systemPrefersDark]);

  useEffect(() => {
    const mediaQuery = window.matchMedia?.("(prefers-color-scheme: dark)");
    if (!mediaQuery) {
      return;
    }

    const updateSystemMode = () => setSystemPrefersDark(mediaQuery.matches);
    updateSystemMode();
    mediaQuery.addEventListener("change", updateSystemMode);
    return () => mediaQuery.removeEventListener("change", updateSystemMode);
  }, []);

  useEffect(() => {
    if (
      !settings.update.autoCheck ||
      !shouldRunAutoUpdateCheck(settings.update.lastCheckedAt, settings.update.checkInterval)
    ) {
      return;
    }

    const timeout = window.setTimeout(() => {
      const checkedAt = new Date().toISOString();
      const saveLastCheckedAt = async () => {
        const latestSettings = settingsRef.current;
        await setSettings({
          ...latestSettings,
          update: {
            ...latestSettings.update,
            lastCheckedAt: checkedAt,
          },
        });
      };

      void checkCliplyUpdate()
        .then(() => saveLastCheckedAt())
        .catch(() => {
          void saveLastCheckedAt().catch(() => {
            // Automatic update checks stay quiet; users can retry manually from About.
          });
        });
    }, 20_000);

    return () => window.clearTimeout(timeout);
  }, [setSettings, settings.update.autoCheck, settings.update.checkInterval, settings.update.lastCheckedAt]);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }

    let cancelled = false;
    let removeListeners: Array<() => void> = [];
    void Promise.all([
      listen("cliply-open-settings", openSettings),
      listen("cliply-open-about", openAbout),
      listen("cliply-open-clear-history", requestClearHistory),
    ]).then((listeners) => {
      if (cancelled) {
        listeners.forEach((unlisten) => unlisten());
      } else {
        removeListeners = listeners;
      }
    });

    return () => {
      cancelled = true;
      removeListeners.forEach((unlisten) => unlisten());
    };
  }, [openAbout, openSettings, requestClearHistory]);

  const onToggleWindowPin = useCallback(async () => {
    const nextPinned = !windowPinned;
    await toggleAlwaysOnTop(nextPinned);
    setWindowPinned(nextPinned);
  }, [setWindowPinned, windowPinned]);

  const pasteItem = useCallback(
    (id: string) => {
      runClipboardAction("paste", id);
    },
    [runClipboardAction],
  );

  const focusSearch = useCallback(() => {
    searchInputRef.current?.focus();
    searchInputRef.current?.select();
  }, []);

  const applyTypeFilter = useCallback(
    (nextFilter: ClipboardFilter) => {
      setFilter(nextFilter);
    },
    [setFilter],
  );

  const filterBySource = useCallback(
    (item: ClipboardItem) => {
      setFilter("all");
      setQuery(item.sourceApp);
    },
    [setFilter, setQuery],
  );

  const searchSelectedText = useCallback(
    (item: ClipboardItem) => {
      const nextQuery = item.tags[0] ?? item.sourceApp ?? item.title;
      setFilter("all");
      setQuery(nextQuery);
      focusSearch();
    },
    [focusSearch, setFilter, setQuery],
  );

  const openImageViewer = useCallback((item: ClipboardItem) => {
    if (item.type === "image" && (item.imageUrl || item.thumbnailUrl)) {
      setImageViewerItem(item);
    }
  }, []);

  const showContextMenu = useCallback(
    (event: MouseEvent<HTMLElement>, item: ClipboardItem | null) => {
      event.preventDefault();
      event.stopPropagation();

      if (item) {
        selectItem(item.id);
      }

      setContextMenu({
        x: event.clientX,
        y: event.clientY,
        sections: buildContextMenuSections({
          item,
          currentFilter: state.filter,
          monitoringPaused: settings.pauseMonitoring,
          windowPinned,
          compact: item !== null,
          onAction: (kind) => runClipboardAction(kind, item?.id),
          onFilter: applyTypeFilter,
          onFilterBySource: item ? () => filterBySource(item) : undefined,
          onSearchRelated: item ? () => searchSelectedText(item) : undefined,
          onOpenImage: item?.type === "image" ? () => openImageViewer(item) : undefined,
          onFocusSearch: focusSearch,
          onToggleWindowPin,
          onToggleMonitoring: toggleMonitoring,
          onOpenSettings: openSettings,
          onOpenAbout: openAbout,
          onClearHistory: requestClearHistory,
          onCopyDataDir: () => void copyDebugPath("dataDir"),
          onCopyLogPath: () => void copyDebugPath("logPath"),
          onCopyDatabasePath: () => void copyDebugPath("databasePath"),
          onCloseWindow: () => void hideMainWindow(),
        }),
      });
    },
    [
      applyTypeFilter,
      filterBySource,
      focusSearch,
      onToggleWindowPin,
      openImageViewer,
      openAbout,
      openSettings,
      requestClearHistory,
      copyDebugPath,
      runClipboardAction,
      searchSelectedText,
      selectItem,
      settings.pauseMonitoring,
      state.filter,
      toggleMonitoring,
      windowPinned,
    ],
  );

  const showAppContextMenu = useCallback(
    (event: MouseEvent<HTMLElement>) => {
      if (event.target !== event.currentTarget) {
        return;
      }

      showContextMenu(event, selectedItem);
    },
    [selectedItem, showContextMenu],
  );

  return (
    <main className="cliply-root" onContextMenu={showAppContextMenu}>
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
        <div className="cliply-app-shell min-h-0 flex-1">
          <section className="cliply-main-stage min-h-0 min-w-0">
            <header className="cliply-command-area shrink-0">
              <div className="cliply-command-row">
                <ClipboardSearchBar
                  ref={searchInputRef}
                  query={state.query}
                  onQueryChange={setQuery}
                />
                <span className="cliply-result-summary shrink-0 rounded-md border border-[color:var(--cliply-border-soft)] px-2.5 tabular-nums">
                  {resultSummary({
                    query: state.query,
                    filter: state.filter,
                    shownCount: filteredItems.length,
                    totalCount: state.items.length,
                  })}
                </span>
              </div>
            </header>

            <PrivacyBanner
              monitoringPaused={settings.pauseMonitoring}
              errorMessage={state.monitoringErrorMessage}
              onResumeMonitoring={toggleMonitoring}
            />

            <div className="cliply-workspace min-h-0 flex-1">
              <div className="cliply-workspace-grid grid h-full min-h-0 overflow-hidden">
                <ClipboardList
                  items={filteredItems}
                  selectedId={state.selectedId}
                  hasQuery={state.query.length > 0}
                  filter={state.filter}
                  onFilterChange={setFilter}
                  loading={state.loading}
                  errorMessage={state.listErrorMessage}
                  onSelectItem={selectItem}
                  onTogglePin={togglePinItem}
                  onPasteItem={pasteItem}
                  onItemContextMenu={showContextMenu}
                />
                <ClipboardDetailPane
                  item={selectedItem}
                  onAction={runClipboardAction}
                  onContextMenu={showContextMenu}
                  onOpenImage={openImageViewer}
                />
              </div>
            </div>
          </section>
        </div>
        <FooterShortcuts monitoringPaused={settings.pauseMonitoring} />
        <GlobalToast
          toast={!dialogs.settings && actionStatus ? actionStatusToToast(actionStatus) : null}
          onClose={clearActionStatus}
        />
        {dialogs.settings ? (
          <Suspense fallback={<DialogLoadingFallback label="正在加载设置…" />}>
            <SettingsDialog
              open
              settings={settings}
              onClose={closeDialogs}
              onSave={setSettings}
              onClearHistory={requestClearHistory}
            />
          </Suspense>
        ) : null}
        {dialogs.about ? (
          <Suspense fallback={<DialogLoadingFallback label="正在加载关于信息…" />}>
            <AboutDialog open onClose={closeDialogs} />
          </Suspense>
        ) : null}
        <ConfirmDialog
          open={dialogs.clearHistory}
          title="清空剪贴板历史记录？"
          description="将删除全部未固定记录，并保留固定记录。此操作无法撤销。"
          confirmLabel="清空历史记录"
          danger
          onConfirm={confirmClearHistory}
          onClose={closeDialogs}
        />
        <ContextMenu menu={contextMenu} onClose={() => setContextMenu(null)} />
        <ImageViewer item={imageViewerItem} onClose={() => setImageViewerItem(null)} />
      </div>
    </main>
  );
}

type ContextMenuConfig = {
  item: ClipboardItem | null;
  currentFilter: ClipboardFilter;
  monitoringPaused: boolean;
  windowPinned: boolean;
  compact: boolean;
  onAction: (kind: "paste" | "copy" | "pastePlain" | "togglePin" | "delete") => void;
  onFilter: (filter: ClipboardFilter) => void;
  onFilterBySource?: () => void;
  onSearchRelated?: () => void;
  onOpenImage?: () => void;
  onFocusSearch: () => void;
  onToggleWindowPin: () => void;
  onToggleMonitoring: () => void;
  onOpenSettings: () => void;
  onOpenAbout: () => void;
  onClearHistory: () => void;
  onCopyDataDir: () => void;
  onCopyLogPath: () => void;
  onCopyDatabasePath: () => void;
  onCloseWindow: () => void;
};

function buildContextMenuSections(config: ContextMenuConfig): ContextMenuSection[] {
  const itemSections = config.item ? buildItemSections(config) : [];
  if (config.compact) {
    return [
      ...itemSections,
      {
        id: "quick-view",
        title: "查看",
        items: [
          {
            id: "focus-search",
            label: "搜索剪贴板",
            shortcut: "Ctrl K",
            icon: Search,
            onSelect: config.onFocusSearch,
          },
          filterItem("all", "全部记录", ClipboardListIcon, config),
          filterItem("pinned", "仅显示固定记录", Pin, config),
        ],
      },
    ];
  }

  return [
    ...itemSections,
    {
      id: "filter",
      title: "查看",
      items: [
        {
          id: "focus-search",
          label: "搜索剪贴板",
          shortcut: "Ctrl K",
          icon: Search,
          onSelect: config.onFocusSearch,
        },
        filterItem("all", "全部记录", ClipboardListIcon, config),
        filterItem("text", "仅显示文本", FileText, config),
        filterItem("link", "仅显示链接", Link2, config),
        filterItem("image", "仅显示图片", Image, config),
        filterItem("pinned", "仅显示固定记录", Pin, config),
      ],
    },
    {
      id: "window",
      title: "窗口",
      items: [
        {
          id: "toggle-pin-window",
          label: config.windowPinned ? "取消窗口置顶" : "置顶窗口",
          icon: Pin,
          checked: config.windowPinned,
          onSelect: config.onToggleWindowPin,
        },
        {
          id: "toggle-monitoring",
          label: config.monitoringPaused ? "恢复监听" : "暂停监听",
          icon: config.monitoringPaused ? PlayCircle : PauseCircle,
          onSelect: config.onToggleMonitoring,
        },
        {
          id: "settings",
          label: "打开设置",
          icon: Settings,
          onSelect: config.onOpenSettings,
        },
        {
          id: "about",
          label: "关于 Cliply",
          icon: Eye,
          onSelect: config.onOpenAbout,
        },
        {
          id: "close",
          label: "隐藏窗口",
          shortcut: "Esc",
          icon: X,
          onSelect: config.onCloseWindow,
        },
      ],
    },
    {
      id: "data",
      title: "数据",
      items: [
        {
          id: "copy-data-dir",
          label: "复制数据目录路径",
          icon: FileArchive,
          onSelect: config.onCopyDataDir,
        },
        {
          id: "copy-log-path",
          label: "复制日志路径",
          icon: FileText,
          onSelect: config.onCopyLogPath,
        },
        {
          id: "copy-database-path",
          label: "复制数据库路径",
          icon: Database,
          onSelect: config.onCopyDatabasePath,
        },
        {
          id: "clear-history",
          label: "清空未固定记录",
          icon: Database,
          danger: true,
          onSelect: config.onClearHistory,
        },
      ],
    },
  ];
}

function buildItemSections(config: ContextMenuConfig): ContextMenuSection[] {
  const item = config.item;
  if (!item) {
    return [];
  }

  const availability = getClipboardActionAvailability(item);

  return [
    {
      id: "item-primary",
      title: "记录",
      items: [
        ...(item.type === "image"
          ? [
              {
                id: "open-image",
                label: "查看图片",
                icon: Eye,
                disabled: !(item.imageUrl || item.thumbnailUrl),
                onSelect: config.onOpenImage,
              },
            ]
          : []),
        {
          id: "paste",
          label: "粘贴",
          shortcut: "Enter",
          icon: Clipboard,
          disabled: !availability.paste,
          onSelect: () => config.onAction("paste"),
        },
        {
          id: "copy",
          label: "复制到剪贴板",
          shortcut: "Ctrl C",
          icon: Copy,
          disabled: !availability.copy,
          onSelect: () => config.onAction("copy"),
        },
        {
          id: "paste-plain",
          label: "无格式粘贴",
          shortcut: "Shift Enter",
          icon: Type,
          disabled: !availability.pastePlain,
          onSelect: () => config.onAction("pastePlain"),
        },
      ],
    },
    {
      id: "item-organize",
      title: "整理",
      items: [
        {
          id: "toggle-pin",
          label: item.isPinned ? "取消固定" : "固定记录",
          shortcut: "Ctrl P",
          icon: Pin,
          checked: item.isPinned,
          onSelect: () => config.onAction("togglePin"),
        },
        {
          id: "filter-type",
          label: `仅显示同类型记录：${typeLabel[item.type]}`,
          icon: Filter,
          onSelect: () => config.onFilter(item.type),
        },
        {
          id: "filter-source",
          label: `仅显示来源：${item.sourceApp}`,
          icon: ExternalLink,
          onSelect: config.onFilterBySource,
        },
        {
          id: "search-related",
          label: "按标签或来源搜索",
          icon: Search,
          onSelect: config.onSearchRelated,
        },
        {
          id: "delete",
          label: "删除记录",
          shortcut: "Del",
          icon: Trash2,
          danger: true,
          onSelect: () => config.onAction("delete"),
        },
      ],
    },
  ];
}

function filterItem(
  filter: ClipboardFilter,
  label: string,
  icon: typeof ClipboardListIcon,
  config: ContextMenuConfig,
) {
  return {
    id: `filter-${filter}`,
    label,
    icon,
    checked: config.currentFilter === filter,
    onSelect: () => config.onFilter(filter),
  };
}

const typeLabel = {
  code: "代码",
  image: "图片",
  link: "链接",
  text: "文本",
} satisfies Record<ClipboardItem["type"], string>;

function getSystemPrefersDark() {
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;
}

function resultSummary({
  query,
  filter,
  shownCount,
  totalCount,
}: {
  query: string;
  filter: ClipboardFilter;
  shownCount: number;
  totalCount: number;
}) {
  if (query.trim()) {
    return `找到 ${shownCount} 条`;
  }

  if (filter !== "all") {
    return `${shownCount} 条`;
  }

  return `共 ${totalCount} 条`;
}

function actionStatusToToast(actionStatus: NonNullable<ReturnType<typeof useClipboardStore>["actionStatus"]>): ToastMessage {
  return {
    id: String(actionStatus.at),
    title: actionStatus.label,
    description: actionStatus.itemTitle,
    tone: actionStatus.tone ?? "success",
    at: actionStatus.at,
  };
}
