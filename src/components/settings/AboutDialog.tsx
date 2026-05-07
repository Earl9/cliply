import { X } from "lucide-react";
import { useEffect, useState } from "react";
import cliplyLogo from "@/assets/cliply-logo.png";
import { Badge } from "@/components/common/Badge";
import { IconButton } from "@/components/common/IconButton";
import { getCliplyDebugInfo, type CliplyDebugInfo } from "@/lib/debugInfo";

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
    <div className="absolute inset-0 z-30 grid place-items-center bg-slate-900/18 px-6 backdrop-blur-sm">
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="cliply-about-title"
        className="w-full max-w-[440px] rounded-2xl border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] p-5 shadow-2xl"
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
            本地优先、键盘优先的现代剪贴板管理器。当前版本面向 Windows MVP。
          </p>
          <div className="mt-4 flex flex-wrap justify-center gap-2">
            <Badge tone="accent">Tauri v2</Badge>
            <Badge tone="teal">SQLite</Badge>
            <Badge>Local-first</Badge>
          </div>
          <div className="mt-5 w-full rounded-xl border border-[color:var(--cliply-border-soft)] bg-[#fbfcfe] p-3 text-left">
            <div className="mb-2 text-sm font-semibold text-[color:var(--cliply-text)]">Debug</div>
            <dl className="grid gap-2 text-xs leading-5 text-[color:var(--cliply-muted)]">
              <div>
                <dt className="font-medium text-[color:var(--cliply-body-text)]">日志文件</dt>
                <dd className="break-all">{debugInfo?.logPath ?? "正在读取..."}</dd>
              </div>
              <div>
                <dt className="font-medium text-[color:var(--cliply-body-text)]">数据库文件</dt>
                <dd className="break-all">{debugInfo?.databasePath ?? "正在读取..."}</dd>
              </div>
            </dl>
          </div>
        </div>
      </section>
    </div>
  );
}
