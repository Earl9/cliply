import { ClipboardList, MoreHorizontal, Pin, Settings, X } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { IconButton } from "@/components/common/IconButton";
import { hideMainWindow, toggleMainWindowMaximize } from "@/lib/windowAdapter";

type TitleBarProps = {
  windowPinned: boolean;
  monitoringPaused: boolean;
  onToggleWindowPin: () => void;
  onOpenSettings: () => void;
  onOpenAbout: () => void;
  onClearHistory: () => void;
  onToggleMonitoring: () => void;
};

export function TitleBar({
  windowPinned,
  monitoringPaused,
  onToggleWindowPin,
  onOpenSettings,
  onOpenAbout,
  onClearHistory,
  onToggleMonitoring,
}: TitleBarProps) {
  const [menuOpen, setMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!menuOpen) {
      return;
    }

    const onPointerDown = (event: PointerEvent) => {
      if (!menuRef.current?.contains(event.target as Node)) {
        setMenuOpen(false);
      }
    };

    window.addEventListener("pointerdown", onPointerDown);
    return () => window.removeEventListener("pointerdown", onPointerDown);
  }, [menuOpen]);

  const runMenuAction = (action: () => void) => {
    setMenuOpen(false);
    action();
  };

  return (
    <header
      className="flex h-12 shrink-0 select-none items-center justify-between px-5"
      data-tauri-drag-region
      onDoubleClick={() => void toggleMainWindowMaximize()}
    >
      <div className="flex min-w-0 items-center gap-3" data-tauri-drag-region>
        <div
          className="grid size-8 place-items-center rounded-[10px] bg-[color:var(--cliply-accent-strong)] text-white shadow-sm"
          data-tauri-drag-region
        >
          <ClipboardList className="size-4" />
        </div>
        <div className="min-w-0" data-tauri-drag-region>
          <div
            className="truncate text-xl font-semibold tracking-normal text-[color:var(--cliply-text)]"
            data-tauri-drag-region
          >
            Cliply
          </div>
        </div>
      </div>

      <div
        className="flex items-center gap-2"
        onMouseDown={(event) => event.stopPropagation()}
        onDoubleClick={(event) => event.stopPropagation()}
      >
        <IconButton
          label={windowPinned ? "取消置顶" : "置顶窗口"}
          variant={windowPinned ? "soft" : "ghost"}
          onMouseDown={(event) => event.stopPropagation()}
          onClick={onToggleWindowPin}
        >
          <Pin className="size-4" />
        </IconButton>
        <IconButton
          label="设置"
          onMouseDown={(event) => event.stopPropagation()}
          onClick={onOpenSettings}
        >
          <Settings className="size-4" />
        </IconButton>
        <div ref={menuRef} className="relative">
          <IconButton
            label="更多"
            onMouseDown={(event) => event.stopPropagation()}
            onClick={() => setMenuOpen((open) => !open)}
          >
            <MoreHorizontal className="size-4" />
          </IconButton>
          {menuOpen ? (
            <div className="absolute right-0 top-10 z-20 w-44 overflow-hidden rounded-xl border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] p-1 shadow-xl">
              <MenuButton onClick={() => runMenuAction(onToggleMonitoring)}>
                {monitoringPaused ? "恢复监听" : "暂停监听"}
              </MenuButton>
              <MenuButton onClick={() => runMenuAction(onClearHistory)}>清空历史</MenuButton>
              <MenuButton onClick={() => runMenuAction(onOpenAbout)}>关于 Cliply</MenuButton>
            </div>
          ) : null}
        </div>
        <IconButton
          label="关闭"
          variant="danger"
          onMouseDown={(event) => event.stopPropagation()}
          onClick={() => void hideMainWindow()}
        >
          <X className="size-4" />
        </IconButton>
      </div>
    </header>
  );
}

function MenuButton({
  children,
  onClick,
}: {
  children: ReactNode;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="h-8 w-full rounded-lg px-3 text-left text-[13px] font-medium text-[color:var(--cliply-text)] transition hover:bg-slate-900/[0.06]"
    >
      {children}
    </button>
  );
}
