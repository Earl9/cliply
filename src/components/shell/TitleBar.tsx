import { Maximize2, Minimize2, Minus, MoreHorizontal, Pin, Settings, X } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { clsx } from "clsx";
import cliplyLogo from "@/assets/cliply-logo-20.png";
import {
  hideMainWindow,
  isMainWindowMaximized,
  minimizeMainWindow,
  toggleMainWindowMaximize,
} from "@/lib/windowAdapter";

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
  const [maximized, setMaximized] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let cancelled = false;

    void isMainWindowMaximized()
      .then((nextMaximized) => {
        if (!cancelled) {
          setMaximized(nextMaximized);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setMaximized(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

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

  const toggleMaximize = async () => {
    try {
      setMaximized(await toggleMainWindowMaximize());
    } catch {
      setMaximized(await isMainWindowMaximized().catch(() => false));
    }
  };

  return (
    <header
      className="cliply-titlebar flex h-11 shrink-0 select-none items-center justify-between pl-3 pr-0"
      data-tauri-drag-region
      onDoubleClick={() => void toggleMaximize()}
    >
      <div className="flex min-w-0 items-center gap-2" data-tauri-drag-region>
        <img
          src={cliplyLogo}
          alt="Cliply"
          className="size-5 object-contain"
          draggable={false}
          data-tauri-drag-region
        />
        <div className="min-w-0" data-tauri-drag-region>
          <div
            className="truncate text-[12.5px] font-semibold text-[color:var(--cliply-text)]"
            data-tauri-drag-region
          >
            Cliply
          </div>
        </div>
      </div>

      <div
        className="flex items-center"
        onMouseDown={(event) => event.stopPropagation()}
        onDoubleClick={(event) => event.stopPropagation()}
      >
        <TitleBarButton
          label={windowPinned ? "取消置顶" : "置顶窗口"}
          active={windowPinned}
          onMouseDown={(event) => event.stopPropagation()}
          onClick={onToggleWindowPin}
        >
          <Pin className="size-4" />
        </TitleBarButton>
        <TitleBarButton
          label="设置"
          onMouseDown={(event) => event.stopPropagation()}
          onClick={onOpenSettings}
        >
          <Settings className="size-4" />
        </TitleBarButton>
        <div ref={menuRef} className="relative">
          <TitleBarButton
            label="更多"
            active={menuOpen}
            onMouseDown={(event) => event.stopPropagation()}
            onClick={() => setMenuOpen((open) => !open)}
          >
            <MoreHorizontal className="size-4" />
          </TitleBarButton>
          {menuOpen ? (
            <div className="absolute right-1 top-9 z-20 w-40 overflow-hidden rounded-[4px] border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] py-1 shadow-[var(--cliply-shadow-popover)]">
              <MenuButton onClick={() => runMenuAction(onToggleMonitoring)}>
                {monitoringPaused ? "恢复监听" : "暂停监听"}
              </MenuButton>
              <MenuButton onClick={() => runMenuAction(onClearHistory)}>清空历史记录</MenuButton>
              <MenuButton onClick={() => runMenuAction(onOpenAbout)}>关于 Cliply</MenuButton>
            </div>
          ) : null}
        </div>
        <TitleBarButton
          label="最小化"
          onMouseDown={(event) => event.stopPropagation()}
          onClick={() => void minimizeMainWindow()}
        >
          <Minus className="size-4" />
        </TitleBarButton>
        <TitleBarButton
          label={maximized ? "还原窗口" : "最大化"}
          active={maximized}
          onMouseDown={(event) => event.stopPropagation()}
          onClick={() => void toggleMaximize()}
        >
          {maximized ? <Minimize2 className="size-4" /> : <Maximize2 className="size-4" />}
        </TitleBarButton>
        <TitleBarButton
          label="关闭"
          variant="danger"
          onMouseDown={(event) => event.stopPropagation()}
          onClick={() => void hideMainWindow()}
        >
          <X className="size-4" />
        </TitleBarButton>
      </div>
    </header>
  );
}

function TitleBarButton({
  label,
  children,
  active = false,
  variant = "ghost",
  onMouseDown,
  onClick,
}: {
  label: string;
  children: ReactNode;
  active?: boolean;
  variant?: "ghost" | "danger";
  onMouseDown?: (event: React.MouseEvent<HTMLButtonElement>) => void;
  onClick?: () => void;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onMouseDown={onMouseDown}
      onClick={onClick}
      className={clsx(
        "cliply-titlebar-button cliply-interactive grid h-11 w-10 place-items-center text-[color:var(--cliply-muted)]",
        "hover:bg-[color:var(--cliply-muted-bg)] hover:text-[color:var(--cliply-text)]",
        "focus-visible:outline focus-visible:outline-1 focus-visible:-outline-offset-1 focus-visible:outline-[color:var(--cliply-accent)]",
        active && "text-[color:var(--cliply-accent)]",
        variant === "danger" && "hover:bg-[color:var(--cliply-danger)] hover:text-white",
      )}
    >
      {children}
    </button>
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
      className="h-8 w-full px-3 text-left text-[13px] text-[color:var(--cliply-text)] hover:bg-[color:var(--cliply-muted-bg)]"
    >
      {children}
    </button>
  );
}
