import { ClipboardList, MoreHorizontal, Pin, Settings, X } from "lucide-react";
import { IconButton } from "@/components/common/IconButton";
import { hideMainWindow } from "@/lib/windowAdapter";

type TitleBarProps = {
  onClearHistory?: () => void;
};

export function TitleBar({ onClearHistory }: TitleBarProps) {
  return (
    <header className="flex h-16 shrink-0 items-center justify-between px-7">
      <div className="flex min-w-0 items-center gap-3">
        <div className="grid size-8 place-items-center rounded-[10px] bg-gradient-to-br from-[#9577ff] to-[#5b3fd7] text-white shadow-sm">
          <ClipboardList className="size-4" />
        </div>
        <div className="min-w-0">
          <div className="truncate text-xl font-semibold tracking-normal text-[color:var(--cliply-text)]">
            Cliply
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
        <IconButton label="More" onClick={onClearHistory}>
          <MoreHorizontal className="size-4" />
        </IconButton>
        <IconButton label="Hide Cliply" variant="danger" onClick={() => void hideMainWindow()}>
          <X className="size-4" />
        </IconButton>
      </div>
    </header>
  );
}
