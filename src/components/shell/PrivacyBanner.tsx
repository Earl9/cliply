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
          ? "cliply-notice flex min-h-8 items-center justify-between gap-3 border-b border-[color:var(--cliply-danger)]/25 bg-[color:var(--cliply-danger-soft)] px-3 py-1.5 text-[12px] text-[color:var(--cliply-danger)]"
          : "cliply-notice flex min-h-8 items-center justify-between gap-3 border-b border-[color:var(--cliply-warning)]/25 bg-[color:var(--cliply-warning-soft)] px-3 py-1.5 text-[12px] text-[color:var(--cliply-warning)]"
      }
    >
      <div className="flex min-w-0 items-center gap-2">
        {tone === "error" ? (
          <AlertTriangle className="size-4 shrink-0" />
        ) : (
          <ShieldCheck className="size-4 shrink-0" />
        )}
        <span className="truncate">
          {errorMessage ?? "监听已暂停。暂停期间不会保存新的剪贴板内容。"}
        </span>
      </div>
      {monitoringPaused ? (
        <button
          type="button"
          onClick={onResumeMonitoring}
          className="shrink-0 px-2 py-1 text-[11.5px] font-semibold text-[color:var(--cliply-text)] hover:bg-[color:var(--cliply-muted-bg)] focus-visible:outline focus-visible:outline-1 focus-visible:outline-[color:var(--cliply-warning)]"
        >
          恢复监听
        </button>
      ) : null}
    </div>
  );
}
