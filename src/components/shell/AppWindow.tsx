import { ClipboardDetailPane } from "@/components/clipboard/ClipboardDetailPane";
import { ClipboardFilterTabs } from "@/components/clipboard/ClipboardFilterTabs";
import { ClipboardList } from "@/components/clipboard/ClipboardList";
import { ClipboardSearchBar } from "@/components/clipboard/ClipboardSearchBar";
import { FooterShortcuts } from "@/components/shell/FooterShortcuts";
import { TitleBar } from "@/components/shell/TitleBar";

export function AppWindow() {
  return (
    <main className="grid min-h-screen place-items-center bg-[radial-gradient(circle_at_20%_15%,rgba(14,159,154,0.16),transparent_28%),radial-gradient(circle_at_82%_16%,rgba(101,84,246,0.20),transparent_30%),linear-gradient(135deg,#edf4fb_0%,#f8fbff_46%,#eaf0f7_100%)] p-4">
      <div className="flex h-[min(760px,calc(100vh-32px))] w-[min(1160px,calc(100vw-32px))] min-w-0 flex-col overflow-hidden rounded-2xl border border-white/75 bg-[color:var(--cliply-panel)] shadow-[var(--cliply-shadow)] backdrop-blur-2xl">
        <TitleBar />
        <ClipboardSearchBar />
        <ClipboardFilterTabs />
        <div className="flex min-h-0 flex-1">
          <ClipboardList />
          <ClipboardDetailPane />
        </div>
        <FooterShortcuts />
      </div>
    </main>
  );
}
