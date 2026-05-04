import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ClipboardActionKind,
  ClipboardActionStatus,
  ClipboardFilter,
  ClipboardItem,
} from "@/lib/clipboardTypes";
import {
  clearClipboardHistory,
  deleteClipboardItem,
  getClipboardItemDetail,
  listClipboardItems,
  togglePinClipboardItem,
} from "@/lib/clipboardRepository";
import { clampIndex } from "@/lib/keyboard";
import { mockClipboardItems } from "@/lib/mockClipboardItems";

const actionLabels: Record<ClipboardActionKind, string> = {
  paste: "已模拟粘贴",
  copy: "已模拟复制",
  pastePlain: "已模拟无格式粘贴",
  togglePin: "固定状态已更新",
  delete: "记录已删除",
};

export function useClipboardStore() {
  const [allItems, setAllItems] = useState<ClipboardItem[]>(mockClipboardItems);
  const [visibleItems, setVisibleItems] = useState<ClipboardItem[]>(mockClipboardItems);
  const [query, setQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const [filter, setFilter] = useState<ClipboardFilter>("all");
  const [selectedId, setSelectedId] = useState<string | null>(mockClipboardItems[0]?.id ?? null);
  const [detail, setDetail] = useState<ClipboardItem | null>(mockClipboardItems[0] ?? null);
  const [loading, setLoading] = useState(false);
  const [actionStatus, setActionStatus] = useState<ClipboardActionStatus>(null);
  const [refreshToken, setRefreshToken] = useState(0);

  useEffect(() => {
    const timeout = window.setTimeout(() => setDebouncedQuery(query), 150);
    return () => window.clearTimeout(timeout);
  }, [query]);

  useEffect(() => {
    let cancelled = false;

    listClipboardItems({ query: "", filter: "all" }).then((items) => {
      if (!cancelled) {
        setAllItems(items);
      }
    });

    return () => {
      cancelled = true;
    };
  }, [refreshToken]);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);

    listClipboardItems({ query: debouncedQuery, filter })
      .then((items) => {
        if (cancelled) {
          return;
        }

        setVisibleItems(items);
        setSelectedId((currentSelectedId) => {
          if (items.some((item) => item.id === currentSelectedId)) {
            return currentSelectedId;
          }

          return items[0]?.id ?? null;
        });
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [debouncedQuery, filter, refreshToken]);

  useEffect(() => {
    if (!isTauri()) {
      return;
    }

    let unlisten: (() => void) | undefined;
    let cancelled = false;

    listen("clipboard-items-changed", () => {
      setRefreshToken((token) => token + 1);
    })
      .then((cleanup) => {
        if (cancelled) {
          cleanup();
          return;
        }

        unlisten = cleanup;
      })
      .catch(() => {});

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    if (!selectedId) {
      setDetail(null);
      return;
    }

    let cancelled = false;

    getClipboardItemDetail(selectedId).then((item) => {
      if (!cancelled) {
        setDetail(item);
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

    const timeout = window.setTimeout(() => setActionStatus(null), 1400);
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
      const updatedItem = await togglePinClipboardItem(id);
      patchPinnedState(id, updatedItem);
      refreshItems();
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
      const removedItem = visibleItems.find((item) => item.id === id) ?? selectedItem;
      const removedIndex = visibleItems.findIndex((item) => item.id === id);
      const nextItems = visibleItems.filter((item) => item.id !== id);
      const nextSelectedId =
        nextItems[Math.min(Math.max(removedIndex, 0), nextItems.length - 1)]?.id ?? null;

      setVisibleItems(nextItems);
      setAllItems((items) => items.filter((item) => item.id !== id));
      setSelectedId(nextSelectedId);
      setDetail((item) => (item?.id === id ? null : item));

      await deleteClipboardItem(id);
      refreshItems();

      if (removedItem) {
        setActionStatus({
          label: actionLabels.delete,
          itemTitle: removedItem.title,
          at: Date.now(),
        });
      }
    },
    [refreshItems, selectedItem, visibleItems],
  );

  const runMockAction = useCallback(
    (kind: ClipboardActionKind) => {
      const selected = selectedItem;
      if (!selected) {
        return;
      }

      if (kind === "togglePin") {
        void updatePinnedState(selected.id);
      }

      if (kind === "delete") {
        void removeItem(selected.id);
        return;
      }

      setActionStatus({
        label: actionLabels[kind],
        itemTitle: selected.title,
        at: Date.now(),
      });
    },
    [removeItem, selectedItem, updatePinnedState],
  );

  const togglePinItem = useCallback(
    (id: string) => {
      const item = visibleItems.find((currentItem) => currentItem.id === id);
      if (!item) {
        return;
      }

      void updatePinnedState(id);
      setActionStatus({
        label: actionLabels.togglePin,
        itemTitle: item.title,
        at: Date.now(),
      });
    },
    [updatePinnedState, visibleItems],
  );

  const handleGlobalKeyDown = useCallback(
    (event: KeyboardEvent) => {
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
        runMockAction("togglePin");
        return;
      }

      if (event.key === "Delete") {
        event.preventDefault();
        runMockAction("delete");
        return;
      }

      if (event.key === "Enter") {
        event.preventDefault();
        runMockAction(event.shiftKey ? "pastePlain" : "paste");
      }
    },
    [moveSelection, runMockAction],
  );

  const clearHistory = useCallback(() => {
    const shouldClear = window.confirm("清空未固定的剪贴板历史？固定记录会保留。");
    if (!shouldClear) {
      return;
    }

    void clearClipboardHistory(false).then(() => {
      setActionStatus({
        label: "历史已清空",
        itemTitle: "固定记录已保留",
        at: Date.now(),
      });
      refreshItems();
    });
  }, [refreshItems]);

  return {
    state: {
      items: allItems,
      selectedId,
      query,
      filter,
      loading,
      detail,
    },
    filteredItems: visibleItems,
    selectedItem,
    counts,
    actionStatus,
    setQuery,
    setFilter,
    selectItem,
    moveSelection,
    runMockAction,
    togglePinItem,
    clearHistory,
    handleGlobalKeyDown,
  };
}
