import { Circle, MoreHorizontal, Pin, Settings, X } from "lucide-react";
import { IconButton } from "@/components/common/IconButton";
import { hideMainWindow } from "@/lib/windowAdapter";

export function TitleBar() {
  return (
    <header className="flex h-14 shrink-0 items-center justify-between border-b border-[color:var(--cliply-border)] px-4">
      <div className="flex min-w-0 items-center gap-3">
        <div className="grid size-8 place-items-center rounded-lg bg-[color:var(--cliply-accent)] text-sm font-semibold text-white shadow-sm">
          C
        </div>
        <div className="min-w-0">
          <div className="truncate text-sm font-semibold tracking-normal text-[color:var(--cliply-text)]">
            Cliply
          </div>
          <div className="flex items-center gap-1.5 text-xs text-[color:var(--cliply-muted)]">
            <Circle className="size-2 fill-[color:var(--cliply-teal)] text-[color:var(--cliply-teal)]" />
            Local-first clipboard
          </div>
        </div>
      </div>

      <div className="flex items-center gap-1">
        <IconButton label="Pin window">
          <Pin className="size-4" />
        </IconButton>
        <IconButton label="Settings">
          <Settings className="size-4" />
        </IconButton>
        <IconButton label="More">
          <MoreHorizontal className="size-4" />
        </IconButton>
        <IconButton label="Hide Cliply" variant="danger" onClick={() => void hideMainWindow()}>
          <X className="size-4" />
        </IconButton>
      </div>
    </header>
  );
}
