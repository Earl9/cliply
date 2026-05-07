import { Check, type LucideIcon } from "lucide-react";
import { clsx } from "clsx";
import { useEffect, useLayoutEffect, useRef, useState } from "react";

export type ContextMenuItem = {
  id: string;
  label: string;
  shortcut?: string;
  icon?: LucideIcon;
  disabled?: boolean;
  danger?: boolean;
  checked?: boolean;
  hint?: string;
  onSelect?: () => void;
};

export type ContextMenuSection = {
  id: string;
  title?: string;
  items: ContextMenuItem[];
};

export type ContextMenuState = {
  x: number;
  y: number;
  compact?: boolean;
  sections: ContextMenuSection[];
} | null;

type ContextMenuProps = {
  menu: ContextMenuState;
  onClose: () => void;
};

const MENU_WIDTH = 250;
const MENU_MARGIN = 10;
const MENU_GAP = 8;

export function ContextMenu({ menu, onClose }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);
  const [menuSize, setMenuSize] = useState({ width: MENU_WIDTH, height: 0 });

  useEffect(() => {
    if (!menu) {
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    const onResize = () => onClose();

    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("resize", onResize);
    window.addEventListener("blur", onClose);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("resize", onResize);
      window.removeEventListener("blur", onClose);
    };
  }, [menu, onClose]);

  useLayoutEffect(() => {
    if (!menu) {
      return;
    }

    const rect = menuRef.current?.getBoundingClientRect();
    if (!rect) {
      return;
    }

    setMenuSize({
      width: Math.ceil(rect.width),
      height: Math.ceil(rect.height),
    });
  }, [menu]);

  if (!menu) {
    return null;
  }

  const availableBelow = window.innerHeight - menu.y - MENU_MARGIN;
  const availableAbove = menu.y - MENU_MARGIN;
  const menuHeight = menuSize.height || estimateMenuHeight(menu.sections);
  const openUp = menuHeight > availableBelow && availableAbove > availableBelow;
  const preferredTop = openUp ? menu.y - menuHeight - MENU_GAP : menu.y + MENU_GAP;
  const top = Math.max(
    MENU_MARGIN,
    Math.min(preferredTop, window.innerHeight - menuHeight - MENU_MARGIN),
  );
  const left = Math.max(
    MENU_MARGIN,
    Math.min(menu.x, window.innerWidth - menuSize.width - MENU_MARGIN),
  );

  return (
    <div
      className="fixed inset-0 z-50"
      onContextMenu={(event) => {
        event.preventDefault();
        onClose();
      }}
      onPointerDown={onClose}
    >
      <div
        ref={menuRef}
        className="cliply-scrollbar absolute w-[250px] overflow-auto rounded-[12px] border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] py-1 shadow-[0_18px_48px_rgba(15,23,42,0.28)] ring-1 ring-[color:var(--cliply-border-soft)]"
        style={{
          left,
          top,
          maxHeight: `calc(100vh - ${MENU_MARGIN * 2}px)`,
        }}
        onPointerDown={(event) => event.stopPropagation()}
      >
        {menu.sections.map((section, sectionIndex) =>
          section.items.length ? (
            <div
              key={section.id}
              className={clsx(sectionIndex > 0 && "border-t border-[color:var(--cliply-border-soft)] py-1")}
            >
              {section.title ? (
                <div className="px-3 pb-1 pt-1.5 text-[11px] font-semibold uppercase tracking-normal text-[color:var(--cliply-faint)]">
                  {section.title}
                </div>
              ) : null}
              {section.items.map((item) => (
                <ContextMenuButton key={item.id} item={item} onClose={onClose} />
              ))}
            </div>
          ) : null,
        )}
      </div>
    </div>
  );
}

function estimateMenuHeight(sections: ContextMenuSection[]) {
  const sectionCount = sections.filter((section) => section.items.length).length;
  const itemCount = sections.reduce((count, section) => count + section.items.length, 0);
  const titleCount = sections.filter((section) => section.title && section.items.length).length;

  return 10 + itemCount * 34 + titleCount * 24 + Math.max(0, sectionCount - 1) * 9;
}

function ContextMenuButton({
  item,
  onClose,
}: {
  item: ContextMenuItem;
  onClose: () => void;
}) {
  const Icon = item.icon;

  return (
    <button
      type="button"
      disabled={item.disabled}
      title={item.hint}
      onClick={() => {
        if (item.disabled) {
          return;
        }

        onClose();
        item.onSelect?.();
      }}
      className={clsx(
        "grid h-[34px] w-full grid-cols-[22px_minmax(0,1fr)_auto] items-center gap-2 px-2.5 text-left text-[13px] font-medium transition",
        "disabled:cursor-not-allowed disabled:text-[color:var(--cliply-disabled)]",
        item.danger
          ? "text-[color:var(--cliply-text)] hover:bg-[color:var(--cliply-danger-soft)] hover:text-[color:var(--cliply-danger)]"
          : "text-[color:var(--cliply-text)] hover:bg-[color:var(--cliply-muted-bg)]",
      )}
    >
      <span className="grid size-6 place-items-center text-[color:var(--cliply-muted)]">
        {item.checked ? <Check className="size-4" /> : Icon ? <Icon className="size-4" /> : null}
      </span>
      <span className="min-w-0 truncate">{item.label}</span>
      {item.shortcut ? (
        <span className="rounded-md bg-[color:var(--cliply-muted-bg)] px-1.5 py-0.5 text-[11px] font-medium text-[color:var(--cliply-faint)]">
          {item.shortcut}
        </span>
      ) : null}
    </button>
  );
}
