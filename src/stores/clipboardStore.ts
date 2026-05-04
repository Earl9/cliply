import { useCallback, useEffect, useMemo, useState } from "react";
import type {
  ClipboardActionKind,
  ClipboardActionStatus,
  ClipboardFilter,
  ClipboardItem,
} from "@/lib/clipboardTypes";
import { clampIndex, isEditableElement } from "@/lib/keyboard";
import { mockClipboardItems } from "@/lib/mockClipboardItems";

const actionLabels: Record<ClipboardActionKind, string> = {
  paste: "Paste mocked",
  copy: "Copy mocked",
  pastePlain: "Plain paste mocked",
  togglePin: "Pin updated",
};

export function useClipboardStore() {
  const [items, setItems] = useState<ClipboardItem[]>(mockClipboardItems);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<ClipboardFilter>("all");
  const [selectedId, setSelectedId] = useState<string | null>(mockClipboardItems[0]?.id ?? null);
  const [actionStatus, setActionStatus] = useState<ClipboardActionStatus>(null);

  const filteredItems = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();

    return items.filter((item) => {
      const matchesFilter =
        filter === "all" ||
        (filter === "pinned" ? item.isPinned : item.type === filter);

      if (!matchesFilter) {
        return false;
      }

      if (!normalizedQuery) {
        return true;
      }

      const haystack = [
        item.title,
        item.previewText,
        item.fullText,
        item.sourceApp,
        item.sourceWindow,
        item.tags.join(" "),
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase();

      return haystack.includes(normalizedQuery);
    });
  }, [filter, items, query]);

  const counts = useMemo(
    () => ({
      all: items.length,
      text: items.filter((item) => item.type === "text").length,
      link: items.filter((item) => item.type === "link").length,
      image: items.filter((item) => item.type === "image").length,
      code: items.filter((item) => item.type === "code").length,
      pinned: items.filter((item) => item.isPinned).length,
    }),
    [items],
  );

  const selectedItem = useMemo(
    () => filteredItems.find((item) => item.id === selectedId) ?? filteredItems[0] ?? null,
    [filteredItems, selectedId],
  );

  useEffect(() => {
    if (!filteredItems.length) {
      setSelectedId(null);
      return;
    }

    if (!selectedId || !filteredItems.some((item) => item.id === selectedId)) {
      setSelectedId(filteredItems[0].id);
    }
  }, [filteredItems, selectedId]);

  useEffect(() => {
    if (!actionStatus) {
      return;
    }

    const timeout = window.setTimeout(() => setActionStatus(null), 1400);
    return () => window.clearTimeout(timeout);
  }, [actionStatus]);

  const selectItem = useCallback((id: string) => {
    setSelectedId(id);
  }, []);

  const moveSelection = useCallback(
    (direction: 1 | -1) => {
      if (!filteredItems.length) {
        return;
      }

      const currentIndex = filteredItems.findIndex((item) => item.id === selectedId);
      const nextIndex = clampIndex(
        (currentIndex === -1 ? 0 : currentIndex) + direction,
        filteredItems.length,
      );
      setSelectedId(filteredItems[nextIndex].id);
    },
    [filteredItems, selectedId],
  );

  const runMockAction = useCallback(
    (kind: ClipboardActionKind) => {
      const selected = selectedItem;
      if (!selected) {
        return;
      }

      if (kind === "togglePin") {
        setItems((currentItems) =>
          currentItems.map((item) =>
            item.id === selected.id ? { ...item, isPinned: !item.isPinned } : item,
          ),
        );
      }

      setActionStatus({
        label: actionLabels[kind],
        itemTitle: selected.title,
        at: Date.now(),
      });
    },
    [selectedItem],
  );

  const togglePinItem = useCallback(
    (id: string) => {
      const item = items.find((currentItem) => currentItem.id === id);
      if (!item) {
        return;
      }

      setItems((currentItems) =>
        currentItems.map((currentItem) =>
          currentItem.id === id
            ? { ...currentItem, isPinned: !currentItem.isPinned }
            : currentItem,
        ),
      );
      setActionStatus({
        label: actionLabels.togglePin,
        itemTitle: item.title,
        at: Date.now(),
      });
    },
    [items],
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
      items,
      selectedId,
      query,
      filter,
      loading: false,
      detail: selectedItem,
    },
    filteredItems,
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
