import { Maximize2, Minimize2, Minus, MoreHorizontal, Pin, Settings, X } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { clsx } from "clsx";
import cliplyLogo from "@/assets/cliply-logo.png";
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
      className="flex h-12 shrink-0 select-none items-center justify-between px-5"
      data-tauri-drag-region
      onDoubleClick={() => void toggleMaximize()}
    >
      <div className="flex min-w-0 items-center gap-3" data-tauri-drag-region>
        <img
          src={cliplyLogo}
          alt="Cliply"
          className="size-8 rounded-[10px] object-contain shadow-sm"
          draggable={false}
          data-tauri-drag-region
        />
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
        className="flex items-center gap-1 rounded-xl border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-muted-bg)] p-0.5"
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
            <div className="absolute right-0 top-10 z-20 w-44 overflow-hidden rounded-xl border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] p-1.5 shadow-[0_18px_44px_rgba(15,23,42,0.16)]">
              <MenuButton onClick={() => runMenuAction(onToggleMonitoring)}>
                {monitoringPaused ? "恢复监听" : "暂停监听"}
              </MenuButton>
              <MenuButton onClick={() => runMenuAction(onClearHistory)}>清空历史</MenuButton>
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
        "grid size-8 place-items-center rounded-[10px] border border-transparent text-[color:var(--cliply-muted)] transition",
        "hover:border-[color:var(--cliply-border)] hover:bg-[color:var(--cliply-card)] hover:text-[color:var(--cliply-text)] hover:shadow-[0_8px_18px_rgba(15,23,42,0.08)]",
        "active:scale-95 active:bg-[color:var(--cliply-muted-bg)]",
        "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-accent)]",
        active &&
          "border-[color:var(--cliply-border)] bg-[color:var(--cliply-accent-50)] text-[color:var(--cliply-accent-strong)] shadow-[0_8px_18px_rgba(15,23,42,0.06)]",
        variant === "danger" &&
          "hover:border-transparent hover:bg-[color:var(--cliply-danger-soft)] hover:text-[color:var(--cliply-danger)]",
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
      className="h-8 w-full rounded-lg px-3 text-left text-[13px] font-semibold text-[color:var(--cliply-text)] transition hover:bg-[color:var(--cliply-muted-bg)]"
    >
      {children}
    </button>
  );
}
