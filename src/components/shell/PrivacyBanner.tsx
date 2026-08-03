import { AlertTriangle, ShieldCheck } from "lucide-react";

type PrivacyBannerProps = {
  monitoringPaused: boolean;
  errorMessage: string | null;
  onResumeMonitoring: () => void;
};

export function PrivacyBanner({
  monitoringPaused,
  errorMessage,
  onResumeMonitoring,
}: PrivacyBannerProps) {
  if (!monitoringPaused && !errorMessage) {
    return null;
  }

  const tone = errorMessage ? "error" : "warning";

  return (
    <div
      className={
        tone === "error"
          ? "mx-3 mt-2 flex min-h-9 items-center justify-between gap-3 rounded-[6px] border border-[color:var(--cliply-danger)]/30 bg-[color:var(--cliply-danger-soft)] px-3 py-1.5 text-[13px] text-[color:var(--cliply-danger)]"
          : "mx-3 mt-2 flex min-h-9 items-center justify-between gap-3 rounded-[6px] border border-[color:var(--cliply-warning)]/30 bg-[color:var(--cliply-warning-soft)] px-3 py-1.5 text-[13px] text-[color:var(--cliply-warning)]"
      }
    >
      <div className="flex min-w-0 items-center gap-2">
        {tone === "error" ? (
          <AlertTriangle className="size-4 shrink-0" />
        ) : (
          <ShieldCheck className="size-4 shrink-0" />
        )}
        <span className="truncate">
          {errorMessage ?? "监听已暂停，新的复制内容暂时不会被保存。"}
        </span>
      </div>
      {monitoringPaused ? (
        <button
          type="button"
          onClick={onResumeMonitoring}
          className="shrink-0 rounded-[4px] border border-[color:var(--cliply-border)] bg-[color:var(--cliply-card)] px-2.5 py-1 text-xs font-medium text-[color:var(--cliply-text)] hover:bg-[color:var(--cliply-muted-bg)] focus-visible:outline focus-visible:outline-1 focus-visible:outline-[color:var(--cliply-warning)]"
        >
          恢复监听
        </button>
      ) : null}
    </div>
  );
}
