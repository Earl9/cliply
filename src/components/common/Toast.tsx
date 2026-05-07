import { useEffect, useMemo, useRef, useState } from "react";
import { AlertCircle, CheckCircle2, Info, X } from "lucide-react";
import { clsx } from "clsx";

export type ToastTone = "success" | "warning" | "error";

export type ToastMessage = {
  id?: string;
  title: string;
  description?: string;
  tone?: ToastTone;
  at?: number;
  durationMs?: number;
};

type ToastPresenterProps = {
  toast: ToastMessage | null;
  contextual?: boolean;
  onClose?: () => void;
};

const TOAST_EXIT_MS = 200;

export function GlobalToast({ toast, onClose }: ToastPresenterProps) {
  return (
    <div className="pointer-events-none absolute bottom-[72px] right-6 z-40 flex w-[340px] max-w-[calc(100%-48px)] justify-end">
      <ToastPresenter toast={toast} onClose={onClose} />
    </div>
  );
}

export function ContextualToast({ toast, onClose }: ToastPresenterProps) {
  return <ToastPresenter toast={toast} contextual onClose={onClose} />;
}

function ToastPresenter({ toast, contextual = false, onClose }: ToastPresenterProps) {
  const [renderedToast, setRenderedToast] = useState<ToastMessage | null>(toast);
  const [phase, setPhase] = useState<"open" | "closing">("open");
  const onCloseRef = useRef(onClose);
  const closeTimerRef = useRef<number | null>(null);

  const toastKey = useMemo(() => {
    if (!toast) {
      return null;
    }

    return toast.id ?? String(toast.at ?? `${toast.title}:${toast.description ?? ""}`);
  }, [toast]);

  useEffect(() => {
    onCloseRef.current = onClose;
  }, [onClose]);

  useEffect(() => {
    if (closeTimerRef.current !== null) {
      window.clearTimeout(closeTimerRef.current);
      closeTimerRef.current = null;
    }

    if (!toast) {
      setRenderedToast(null);
      setPhase("open");
      return;
    }

    setRenderedToast(toast);
    setPhase("open");

    const durationMs =
      toast.durationMs ?? (toast.tone === "error" ? 5600 : 2400);
    const timeout = window.setTimeout(() => {
      dismissToast();
    }, durationMs);

    return () => window.clearTimeout(timeout);
  }, [toastKey]);

  const dismissToast = () => {
    if (closeTimerRef.current !== null) {
      return;
    }

    setPhase("closing");
    closeTimerRef.current = window.setTimeout(() => {
      closeTimerRef.current = null;
      setRenderedToast(null);
      onCloseRef.current?.();
    }, TOAST_EXIT_MS);
  };

  if (!renderedToast) {
    return null;
  }

  return (
    <ToastCard
      toast={renderedToast}
      phase={phase}
      contextual={contextual}
      onClose={renderedToast.tone === "error" ? dismissToast : undefined}
    />
  );
}

function ToastCard({
  toast,
  phase,
  contextual,
  onClose,
}: {
  toast: ToastMessage;
  phase: "open" | "closing";
  contextual: boolean;
  onClose?: () => void;
}) {
  const tone = toast.tone ?? "success";
  const Icon =
    tone === "error" ? AlertCircle : tone === "warning" ? Info : CheckCircle2;

  return (
    <div
      className={clsx(
        "cliply-toast pointer-events-auto grid min-w-[280px] max-w-[340px] grid-cols-[auto_minmax(0,1fr)_auto] gap-3 rounded-2xl px-4 py-3.5 text-left",
        contextual ? "w-[300px]" : "w-full",
      )}
      data-phase={phase}
      data-tone={tone}
      role={tone === "error" ? "alert" : "status"}
    >
      <span
        className={clsx(
          "mt-0.5 grid size-6 place-items-center rounded-full",
          tone === "success" &&
            "bg-[color:var(--cliply-success-soft)] text-[color:var(--cliply-success)]",
          tone === "warning" &&
            "bg-[color:var(--cliply-warning-soft)] text-[color:var(--cliply-warning)]",
          tone === "error" &&
            "bg-[color:var(--cliply-danger-soft)] text-[color:var(--cliply-danger)]",
        )}
      >
        <Icon className="size-4" />
      </span>
      <span className="min-w-0">
        <span className="block truncate text-[13px] font-semibold text-[color:var(--cliply-text)]">
          {toast.title}
        </span>
        {toast.description ? (
          <span className="mt-0.5 block max-h-10 overflow-hidden break-words text-xs font-medium leading-5 text-[color:var(--cliply-muted)]">
            {toast.description}
          </span>
        ) : null}
      </span>
      {onClose ? (
        <button
          type="button"
          aria-label="关闭提示"
          onClick={onClose}
          className="-mr-1 -mt-1 grid size-7 place-items-center rounded-lg text-[color:var(--cliply-muted)] transition hover:bg-[color:var(--cliply-muted-bg)] hover:text-[color:var(--cliply-text)]"
        >
          <X className="size-4" />
        </button>
      ) : (
        <span className="size-7" aria-hidden="true" />
      )}
    </div>
  );
}
