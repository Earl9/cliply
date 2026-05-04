import { AlertTriangle, X } from "lucide-react";
import { IconButton } from "@/components/common/IconButton";

type ConfirmDialogProps = {
  open: boolean;
  title: string;
  description: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
  onConfirm: () => void;
  onClose: () => void;
};

export function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel = "确认",
  cancelLabel = "取消",
  danger,
  onConfirm,
  onClose,
}: ConfirmDialogProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="absolute inset-0 z-40 grid place-items-center bg-slate-900/18 px-6 backdrop-blur-sm">
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="cliply-confirm-title"
        className="w-full max-w-[420px] rounded-2xl border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] p-5 shadow-2xl"
      >
        <div className="flex items-start justify-between gap-4">
          <div className="flex min-w-0 items-start gap-3">
            <span className="grid size-10 shrink-0 place-items-center rounded-xl bg-amber-50 text-amber-600">
              <AlertTriangle className="size-5" />
            </span>
            <div className="min-w-0">
              <h2 id="cliply-confirm-title" className="text-[15px] font-semibold text-[color:var(--cliply-text)]">
                {title}
              </h2>
              <p className="mt-1 text-sm leading-6 text-[color:var(--cliply-muted)]">{description}</p>
            </div>
          </div>
          <IconButton label="关闭" onClick={onClose}>
            <X className="size-4" />
          </IconButton>
        </div>
        <div className="mt-5 flex justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
            className="h-10 rounded-xl border border-[color:var(--cliply-border)] bg-white px-4 text-sm font-semibold text-[color:var(--cliply-text)] transition hover:bg-[#fafafb]"
          >
            {cancelLabel}
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className={
              danger
                ? "h-10 rounded-xl bg-rose-600 px-4 text-sm font-semibold text-white transition hover:bg-rose-700"
                : "h-10 rounded-xl bg-[color:var(--cliply-accent-strong)] px-4 text-sm font-semibold text-white transition hover:bg-[#4932af]"
            }
          >
            {confirmLabel}
          </button>
        </div>
      </section>
    </div>
  );
}
