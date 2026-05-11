import { ExternalLink, X } from "lucide-react";
import { useEffect, useState } from "react";
import cliplyLogo from "@/assets/cliply-logo.png";
import { Badge } from "@/components/common/Badge";
import { IconButton } from "@/components/common/IconButton";
import { getCliplyDebugInfo, type CliplyDebugInfo } from "@/lib/debugInfo";
import { CLIPLY_GITHUB_PAGE_URL, openCliplyGitHubPage } from "@/lib/updateService";

type AboutDialogProps = {
  open: boolean;
  onClose: () => void;
};

export function AboutDialog({ open, onClose }: AboutDialogProps) {
  const [debugInfo, setDebugInfo] = useState<CliplyDebugInfo | null>(null);

  useEffect(() => {
    if (!open) {
      return;
    }

    let cancelled = false;
    void getCliplyDebugInfo()
      .then((info) => {
        if (!cancelled) {
          setDebugInfo(info);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setDebugInfo(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [open]);

  if (!open) {
    return null;
  }

  return (
    <div className="absolute inset-0 z-30 grid place-items-center bg-black/30 px-6 backdrop-blur-sm">
      <div className="absolute inset-0" aria-hidden="true" data-tauri-drag-region />
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="cliply-about-title"
        className="relative z-10 w-full max-w-[440px] rounded-2xl border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] p-5 shadow-2xl"
      >
        <div className="flex justify-end">
          <IconButton label="关闭关于" onClick={onClose}>
            <X className="size-4" />
          </IconButton>
        </div>
        <div className="grid justify-items-center text-center">
          <img
            src={cliplyLogo}
            alt="Cliply"
            className="mb-4 size-16 rounded-2xl object-contain shadow-sm"
            draggable={false}
          />
          <h2 id="cliply-about-title" className="text-xl font-semibold text-[color:var(--cliply-text)]">
            Cliply
          </h2>
          <p className="mt-2 max-w-[320px] text-sm leading-6 text-[color:var(--cliply-muted)]">
            本地优先、键盘优先的现代剪贴板管理器。当前版本专注 Windows 体验。
          </p>
          <div className="mt-4 flex flex-wrap justify-center gap-2">
            <Badge tone="accent">Tauri v2</Badge>
            <Badge tone="teal">SQLite</Badge>
            <Badge>Local-first</Badge>
          </div>
          <button
            type="button"
            onClick={() => void openCliplyGitHubPage()}
            className="mt-4 inline-flex max-w-full items-center gap-2 rounded-lg border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-card)] px-3 py-2 text-xs font-semibold text-[color:var(--cliply-text)] transition hover:border-[color:var(--cliply-border)] hover:bg-[color:var(--cliply-muted-bg)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[color:var(--cliply-focus-ring)]"
          >
            <ExternalLink className="size-4 shrink-0 text-[color:var(--cliply-accent)]" />
            <span className="cliply-code-font min-w-0 truncate">{CLIPLY_GITHUB_PAGE_URL}</span>
          </button>
          <div className="mt-5 w-full rounded-xl border border-[color:var(--cliply-border-soft)] bg-[color:var(--cliply-card)] p-3 text-left">
            <div className="mb-2 text-sm font-semibold text-[color:var(--cliply-text)]">Debug</div>
            <dl className="grid gap-2 text-xs leading-5 text-[color:var(--cliply-muted)]">
              <div>
                <dt className="font-medium text-[color:var(--cliply-body-text)]">版本</dt>
                <dd className="cursor-text select-text break-all">{debugInfo?.appVersion ?? "正在读取..."}</dd>
              </div>
              <div>
                <dt className="font-medium text-[color:var(--cliply-body-text)]">日志文件</dt>
                <dd className="cursor-text select-text break-all">{debugInfo?.logPath ?? "正在读取..."}</dd>
              </div>
              <div>
                <dt className="font-medium text-[color:var(--cliply-body-text)]">数据库文件</dt>
                <dd className="cursor-text select-text break-all">{debugInfo?.databasePath ?? "正在读取..."}</dd>
              </div>
              <div>
                <dt className="font-medium text-[color:var(--cliply-body-text)]">历史记录数量</dt>
                <dd>{debugInfo?.historyCount ?? "正在读取..."}</dd>
              </div>
            </dl>
          </div>
        </div>
      </section>
    </div>
  );
}
