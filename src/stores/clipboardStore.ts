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

  const updatePinnedState = useCallback(
    async (id: string) => {
      const updatedItem = await togglePinClipboardItem(id);
      patchPinnedState(id, updatedItem);
    },
    [patchPinnedState],
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

  const runMockAction = useCallback(
    (kind: ClipboardActionKind) => {
      const selected = selectedItem;
      if (!selected) {
        return;
      }

      if (kind === "togglePin") {
        void updatePinnedState(selected.id);
      }

      setActionStatus({
        label: actionLabels[kind],
        itemTitle: selected.title,
        at: Date.now(),
      });
    },
    [selectedItem, updatePinnedState],
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

      if (event.key === "Enter") {
        event.preventDefault();
        runMockAction(event.shiftKey ? "pastePlain" : "paste");
      }
    },
    [moveSelection, runMockAction],
  );

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
    handleGlobalKeyDown,
  };
}
