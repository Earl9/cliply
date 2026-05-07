import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ClipboardActionKind,
  ClipboardActionStatus,
  ClipboardFilter,
  ClipboardItem,
} from "@/lib/clipboardTypes";
import { canRunClipboardAction } from "@/lib/clipboardCapabilities";
import { getCliplyDebugInfo } from "@/lib/debugInfo";
import {
  clearClipboardHistory,
  copyClipboardItem,
  deleteClipboardItem,
  getClipboardItemDetail,
  listClipboardItems,
  pasteClipboardItem,
  pastePlainText,
  togglePinClipboardItem,
} from "@/lib/clipboardRepository";
import {
  getCliplySettings,
  setMonitoringPaused,
  updateCliplySettings,
} from "@/lib/settingsRepository";
import { clampIndex } from "@/lib/keyboard";
import { mockClipboardItems } from "@/lib/mockClipboardItems";
import type { CliplySettings } from "@/stores/settingsStore";
import { defaultSettingsState } from "@/stores/settingsStore";

const actionLabels: Record<ClipboardActionKind, string> = {
  paste: "已粘贴",
  copy: "已复制到剪贴板",
  pastePlain: "已无格式粘贴",
  togglePin: "固定状态已更新",
  delete: "记录已删除",
};

const actionErrorLabels: Record<Extract<ClipboardActionKind, "paste" | "copy" | "pastePlain">, string> = {
  paste: "粘贴失败，内容已尽量复制到剪贴板",
  copy: "复制失败",
  pastePlain: "无格式粘贴失败",
};

type StoreErrorKind = "list" | "detail" | "settings";

const storeErrorLabels: Record<StoreErrorKind, string> = {
  list: "读取剪贴板历史失败，请稍后重试",
  detail: "读取记录详情失败",
  settings: "保存设置失败，请检查快捷键或本地配置",
};

const initialClipboardItems = isTauri() ? [] : mockClipboardItems;

function isEditableShortcutTarget(target: EventTarget | null) {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  );
}

function hasSelectedPageText() {
  const selection = window.getSelection();
  return Boolean(selection && !selection.isCollapsed && selection.toString().length > 0);
}

export function useClipboardStore() {
  const [allItems, setAllItems] = useState<ClipboardItem[]>(initialClipboardItems);
  const [visibleItems, setVisibleItems] = useState<ClipboardItem[]>(initialClipboardItems);
  const [query, setQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [filter, setFilter] = useState<ClipboardFilter>("all");
  const [selectedId, setSelectedId] = useState<string | null>(
    initialClipboardItems[0]?.id ?? null,
  );
  const [detail, setDetail] = useState<ClipboardItem | null>(initialClipboardItems[0] ?? null);
  const [loading, setLoading] = useState(false);
  const [actionStatus, setActionStatus] = useState<ClipboardActionStatus>(null);
  const [refreshToken, setRefreshToken] = useState(0);
  const [settings, setSettingsState] = useState<CliplySettings>(defaultSettingsState);
  const [listErrorMessage, setListErrorMessage] = useState<string | null>(null);
  const [monitoringErrorMessage, setMonitoringErrorMessage] = useState<string | null>(null);
  const [dialogs, setDialogs] = useState({
    settings: false,
    about: false,
    clearHistory: false,
  });

  useEffect(() => {
    let cancelled = false;

    getCliplySettings()
      .then((nextSettings) => {
        if (!cancelled) {
          setSettingsState(nextSettings);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setMonitoringErrorMessage(storeErrorLabels.settings);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const timeout = window.setTimeout(() => setDebouncedQuery(query), 150);
    return () => window.clearTimeout(timeout);
  }, [query]);

  useEffect(() => {
    let cancelled = false;

    listClipboardItems({ query: "", filter: "all", limit: settings.maxHistoryItems })
      .then((items) => {
        if (!cancelled) {
          setAllItems(items);
          setListErrorMessage(null);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setListErrorMessage(storeErrorLabels.list);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [refreshToken, settings.maxHistoryItems]);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);

    listClipboardItems({ query: debouncedQuery, filter, limit: settings.maxHistoryItems })
      .then((items) => {
        if (cancelled) {
          return;
        }

        setListErrorMessage(null);
        setVisibleItems(items);
        setSelectedId((currentSelectedId) => {
          if (items.some((item) => item.id === currentSelectedId)) {
            return currentSelectedId;
          }

          return items[0]?.id ?? null;
        });
      })
      .catch(() => {
        if (!cancelled) {
          setListErrorMessage(storeErrorLabels.list);
          setVisibleItems([]);
          setSelectedId(null);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [debouncedQuery, filter, refreshToken, settings.maxHistoryItems]);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }

    let unlisten: (() => void) | undefined;
    let unlistenSettings: (() => void) | undefined;
    let unlistenErrors: (() => void) | undefined;
    let cancelled = false;

    Promise.all([
      listen("clipboard-items-changed", () => {
        setRefreshToken((token) => token + 1);
      }),
      listen<CliplySettings>("cliply-settings-changed", (event) => {
        setSettingsState(event.payload);
      }),
      listen<string>("cliply-error", (event) => {
        setActionStatus({
          label: event.payload,
          itemTitle: "稍后会自动继续监听",
          at: Date.now(),
          tone: "error",
        });
      }),
    ])
      .then((cleanup) => {
        if (cancelled) {
          cleanup.forEach((unlisten) => unlisten());
          return;
        }

        [unlisten, unlistenSettings, unlistenErrors] = cleanup;
      })
      .catch(() => {});

    return () => {
      cancelled = true;
      unlisten?.();
      unlistenSettings?.();
      unlistenErrors?.();
    };
  }, []);

  useEffect(() => {
    if (!selectedId) {
      setDetail(null);
      return;
    }

    let cancelled = false;

    getClipboardItemDetail(selectedId)
      .then((item) => {
        if (!cancelled) {
          setDetail(item);
          setListErrorMessage(null);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setDetail(null);
          if (!isMissingClipboardItemError(error)) {
            setListErrorMessage(storeErrorLabels.detail);
          }
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedId]);

  useEffect(() => {
    if (!actionStatus) {
      return;
    }

    const timeout = window.setTimeout(
      () => setActionStatus(null),
      actionStatus.tone === "error" ? 3000 : 1400,
    );
    return () => window.clearTimeout(timeout);
  }, [actionStatus]);

  const counts = useMemo(
    () => ({
      all: allItems.length,
      text: allItems.filter((item) => item.type === "text").length,
      link: allItems.filter((item) => item.type === "link").length,
      image: allItems.filter((item) => item.type === "image").length,
      code: allItems.filter((item) => item.type === "code").length,
      pinned: allItems.filter((item) => item.isPinned).length,
    }),
    [allItems],
  );

  const selectedItem =
    detail ?? visibleItems.find((item) => item.id === selectedId) ?? visibleItems[0] ?? null;

  const selectItem = useCallback((id: string) => {
    setSelectedId(id);
  }, []);

  const patchPinnedState = useCallback((id: string, updatedItem: ClipboardItem | null) => {
    const patchItem = (item: ClipboardItem) =>
      item.id === id ? updatedItem ?? { ...item, isPinned: !item.isPinned } : item;

    setAllItems((items) => items.map(patchItem));
    setVisibleItems((items) => items.map(patchItem));
    setDetail((item) => (item?.id === id ? patchItem(item) : item));
  }, []);

  const refreshItems = useCallback(() => {
    setRefreshToken((token) => token + 1);
  }, []);

  const updatePinnedState = useCallback(
    async (id: string) => {
      try {
        const updatedItem = await togglePinClipboardItem(id);
        patchPinnedState(id, updatedItem);
        refreshItems();
        return true;
      } catch {
        setActionStatus({
          label: "数据库写入失败",
          itemTitle: id,
          at: Date.now(),
          tone: "error",
        });
        return false;
      }
    },
    [patchPinnedState, refreshItems],
  );

  const moveSelection = useCallback(
    (direction: 1 | -1) => {
      if (!visibleItems.length) {
        return;
      }

      const currentIndex = visibleItems.findIndex((item) => item.id === selectedId);
      const nextIndex = clampIndex(
        (currentIndex === -1 ? 0 : currentIndex) + direction,
        visibleItems.length,
      );
      setSelectedId(visibleItems[nextIndex].id);
    },
    [selectedId, visibleItems],
  );

  const removeItem = useCallback(
    async (id: string) => {
      const previousAllItems = allItems;
      const previousVisibleItems = visibleItems;
      const removedItem = visibleItems.find((item) => item.id === id) ?? selectedItem;
      const removedIndex = visibleItems.findIndex((item) => item.id === id);
      const nextItems = visibleItems.filter((item) => item.id !== id);
      const nextSelectedId =
        nextItems[Math.min(Math.max(removedIndex, 0), nextItems.length - 1)]?.id ?? null;

      setVisibleItems(nextItems);
      setAllItems((items) => items.filter((item) => item.id !== id));
      setSelectedId(nextSelectedId);
      setDetail((item) => (item?.id === id ? null : item));

      try {
        await deleteClipboardItem(id);
        refreshItems();
      } catch {
        setVisibleItems(previousVisibleItems);
        setAllItems(previousAllItems);
        setSelectedId(id);
        setDetail(removedItem ?? null);
        setActionStatus({
          label: "数据库写入失败",
          itemTitle: removedItem?.title ?? id,
          at: Date.now(),
          tone: "error",
        });
        return;
      }

      if (removedItem) {
        setActionStatus({
          label: actionLabels.delete,
          itemTitle: removedItem.title,
          at: Date.now(),
        });
      }
    },
    [allItems, refreshItems, selectedItem, visibleItems],
  );

  const findActionItem = useCallback(
    (id?: string) =>
      id
        ? (detail?.id === id ? detail : null) ??
          visibleItems.find((item) => item.id === id) ??
          allItems.find((item) => item.id === id) ??
          null
        : selectedItem,
    [allItems, detail, selectedItem, visibleItems],
  );

  const runClipboardAction = useCallback(
    (kind: ClipboardActionKind, itemId?: string) => {
      const item = findActionItem(itemId);
      if (!item) {
        return;
      }

      if (itemId && itemId !== selectedId) {
        setSelectedId(itemId);
      }

      if (!canRunClipboardAction(kind, item)) {
        setActionStatus({
          label: "当前记录没有可粘贴的文本",
          itemTitle: item.title,
          at: Date.now(),
          tone: "error",
        });
        return;
      }

      if (kind === "togglePin") {
        void updatePinnedState(item.id).then((ok) => {
          if (ok) {
            setActionStatus({
              label: actionLabels.togglePin,
              itemTitle: item.title,
              at: Date.now(),
            });
          }
        });
        return;
      }

      if (kind === "delete") {
        void removeItem(item.id);
        return;
      }

      const command =
        kind === "copy"
          ? copyClipboardItem(item.id)
          : kind === "pastePlain"
            ? pastePlainText(item.id)
            : kind === "paste"
              ? pasteClipboardItem(item.id)
              : Promise.resolve();

      void command
        .then(() => {
          setActionStatus({
            label: actionLabels[kind],
            itemTitle: item.title,
            at: Date.now(),
          });
          refreshItems();
        })
        .catch(() => {
          setActionStatus({
            label: actionErrorLabels[kind as "paste" | "copy" | "pastePlain"],
            itemTitle: item.title,
            at: Date.now(),
            tone: "error",
          });
        });
    },
    [findActionItem, refreshItems, removeItem, selectedId, updatePinnedState],
  );

  const togglePinItem = useCallback(
    (id: string) => {
      const item = visibleItems.find((currentItem) => currentItem.id === id);
      if (!item) {
        return;
      }

      void updatePinnedState(id).then((ok) => {
        if (ok) {
          setActionStatus({
            label: actionLabels.togglePin,
            itemTitle: item.title,
            at: Date.now(),
          });
        }
      });
    },
    [updatePinnedState, visibleItems],
  );

  const copyDebugPath = useCallback(async (pathKind: "dataDir" | "logPath" | "databasePath") => {
    try {
      const debugInfo = await getCliplyDebugInfo();
      const value = debugInfo[pathKind];
      await navigator.clipboard.writeText(value);
      const labels = {
        dataDir: "数据目录路径已复制",
        logPath: "日志路径已复制",
        databasePath: "数据库路径已复制",
      };
      setActionStatus({
        label: labels[pathKind],
        itemTitle: value,
        at: Date.now(),
      });
    } catch {
      setActionStatus({
        label: "路径复制失败",
        itemTitle: "请从设置或日志说明中查看",
        at: Date.now(),
        tone: "error",
      });
    }
  }, []);

  const handleGlobalKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const isEditableTarget = isEditableShortcutTarget(event.target);
      const isCopyShortcut = event.key.toLowerCase() === "c" && (event.ctrlKey || event.metaKey);

      if (
        (isCopyShortcut && (isEditableTarget || hasSelectedPageText())) ||
        dialogs.settings ||
        dialogs.about ||
        dialogs.clearHistory
      ) {
        return;
      }

      if (event.key === "ArrowDown") {
        event.preventDefault();
        moveSelection(1);
        return;
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        moveSelection(-1);
        return;
      }

      if (event.key.toLowerCase() === "p" && event.ctrlKey) {
        event.preventDefault();
        runClipboardAction("togglePin");
        return;
      }

      if (event.key === "Delete") {
        event.preventDefault();
        runClipboardAction("delete");
        return;
      }

      if (isCopyShortcut) {
        event.preventDefault();
        runClipboardAction("copy");
        return;
      }

      if (event.key === "Enter") {
        event.preventDefault();
        runClipboardAction(event.shiftKey ? "pastePlain" : "paste");
      }
    },
    [
      dialogs.about,
      dialogs.clearHistory,
      dialogs.settings,
      moveSelection,
      runClipboardAction,
    ],
  );

  const clearHistory = useCallback(() => {
    void clearClipboardHistory(false)
      .then(() => {
        setActionStatus({
          label: "历史已清空",
          itemTitle: "固定记录已保留",
          at: Date.now(),
        });
        refreshItems();
      })
      .catch(() => {
        setActionStatus({
          label: "数据库写入失败",
          itemTitle: "历史未清空",
          at: Date.now(),
          tone: "error",
        });
      });
  }, [refreshItems]);

  const closeDialogs = useCallback(() => {
    setDialogs({
      settings: false,
      about: false,
      clearHistory: false,
    });
  }, []);

  const clearActionStatus = useCallback(() => {
    setActionStatus(null);
  }, []);

  const openSettings = useCallback(() => {
    setActionStatus(null);
    setDialogs({
      settings: true,
      about: false,
      clearHistory: false,
    });
  }, []);

  const openAbout = useCallback(() => {
    setDialogs({
      settings: false,
      about: true,
      clearHistory: false,
    });
  }, []);

  const requestClearHistory = useCallback(() => {
    setDialogs({
      settings: false,
      about: false,
      clearHistory: true,
    });
  }, []);

  const confirmClearHistory = useCallback(() => {
    closeDialogs();
    clearHistory();
  }, [clearHistory, closeDialogs]);

  const setSettings = useCallback(async (nextSettings: CliplySettings) => {
    const previousSettings = settings;
    setSettingsState(nextSettings);
    try {
      const savedSettings = await updateCliplySettings(nextSettings);
      setSettingsState(savedSettings);
      setMonitoringErrorMessage(null);
    } catch (error) {
      const message = error instanceof Error ? error.message : storeErrorLabels.settings;
      setSettingsState(previousSettings);
      setMonitoringErrorMessage(message || "设置保存失败，请检查本地权限");
      throw new Error(message || storeErrorLabels.settings);
    }
  }, [settings]);

  const toggleMonitoring = useCallback(() => {
    const paused = !settings.pauseMonitoring;
    setSettingsState((current) => ({ ...current, pauseMonitoring: paused }));
    void setMonitoringPaused(paused)
      .then((savedSettings) => {
        setSettingsState(savedSettings);
        setActionStatus({
          label: savedSettings.pauseMonitoring ? "监听已暂停" : "监听已恢复",
          itemTitle: "剪贴板监听状态已更新",
          at: Date.now(),
        });
      })
      .catch(() => {
        setSettingsState((current) => ({ ...current, pauseMonitoring: !paused }));
        setActionStatus({
          label: "数据库写入失败",
          itemTitle: "监听状态未保存",
          at: Date.now(),
          tone: "error",
        });
      });
  }, [settings.pauseMonitoring]);

  return {
    state: {
      items: allItems,
      selectedId,
      query,
      filter,
      loading,
      detail,
      listErrorMessage,
      monitoringErrorMessage,
    },
    filteredItems: visibleItems,
    selectedItem,
    counts,
    actionStatus,
    settings,
    dialogs,
    setQuery,
    setFilter,
    selectItem,
    moveSelection,
    runMockAction: runClipboardAction,
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
  };
}

function isMissingClipboardItemError(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  return (
    message.includes("Query returned no rows") ||
    message.includes("clipboard item was not found")
  );
}
