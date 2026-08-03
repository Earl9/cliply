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
    <div className="cliply-overlay absolute inset-0 z-40 grid place-items-center bg-black/35 px-6">
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="cliply-confirm-title"
        className="cliply-dialog w-full max-w-[400px] rounded-[10px] border border-[color:var(--cliply-border)] bg-[color:var(--cliply-panel-strong)] p-4 shadow-[var(--cliply-shadow-dialog)]"
      >
        <div className="flex items-start justify-between gap-4">
          <div className="flex min-w-0 items-start gap-2.5">
            <AlertTriangle className="mt-0.5 size-5 shrink-0 text-[color:var(--cliply-warning)]" />
            <div className="min-w-0">
              <h2
                id="cliply-confirm-title"
                className="cliply-title text-[15px] font-semibold text-[color:var(--cliply-text)]"
              >
                {title}
              </h2>
              <p className="mt-1 text-[13px] leading-5 text-[color:var(--cliply-muted)]">{description}</p>
            </div>
          </div>
          <IconButton label="关闭" onClick={onClose}>
            <X className="size-4" />
          </IconButton>
        </div>
        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="cliply-interactive h-8 rounded-[6px] border border-[color:var(--cliply-border)] px-3 text-[13px] font-medium text-[color:var(--cliply-text)] hover:bg-[color:var(--cliply-muted-bg)]"
          >
            {cancelLabel}
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className={
              danger
                ? "cliply-interactive h-8 rounded-[6px] bg-[color:var(--cliply-danger)] px-3 text-[13px] font-medium text-white hover:opacity-90"
                : "cliply-interactive h-8 rounded-[6px] bg-[color:var(--cliply-accent)] px-3 text-[13px] font-medium text-[color:var(--cliply-primary-text)] hover:bg-[color:var(--cliply-accent-dark)]"
            }
          >
            {confirmLabel}
          </button>
        </div>
      </section>
    </div>
  );
}
