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
          ? "mx-7 mt-3 flex min-h-11 items-center justify-between gap-3 rounded-xl border border-rose-200 bg-rose-50 px-4 py-2 text-sm text-rose-700"
          : "mx-7 mt-3 flex min-h-11 items-center justify-between gap-3 rounded-xl border border-amber-200 bg-amber-50 px-4 py-2 text-sm text-amber-800"
      }
    >
      <div className="flex min-w-0 items-center gap-2">
        {tone === "error" ? (
          <AlertTriangle className="size-4 shrink-0" />
        ) : (
          <ShieldCheck className="size-4 shrink-0" />
        )}
        <span className="truncate font-medium">
          {errorMessage ?? "监听已暂停，新的复制内容暂时不会被保存。"}
        </span>
      </div>
      {monitoringPaused ? (
        <button
          type="button"
          onClick={onResumeMonitoring}
          className="shrink-0 rounded-lg bg-white/70 px-3 py-1.5 text-xs font-semibold transition hover:bg-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-amber-500"
        >
          恢复监听
        </button>
      ) : null}
    </div>
  );
}
