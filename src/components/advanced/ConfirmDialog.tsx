import { AlertTriangle } from "lucide-react";
import type { AdvancedOperation } from "./types";

interface ConfirmDialogProps {
  show: boolean;
  pendingOperation: { type: "operation" | "raw"; op?: AdvancedOperation; cmd?: string } | null;
  onCancel: () => void;
  onConfirm: () => void;
  t: (key: string) => string;
}

export default function ConfirmDialog({
  show,
  pendingOperation,
  onCancel,
  onConfirm,
  t,
}: ConfirmDialogProps) {
  if (!show) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
      <div className="glass-card p-6 max-w-sm mx-4 space-y-4">
        <div className="flex items-start gap-3">
          <div className="w-10 h-10 rounded-lg bg-obd-warning/10 border border-obd-warning/20 flex items-center justify-center flex-shrink-0">
            <AlertTriangle className="text-obd-warning" size={20} />
          </div>
          <div className="flex-1">
            <h3 className="font-semibold text-obd-text">{t("advanced.confirmDialog.title")}</h3>
            <p className="text-xs text-obd-text-muted mt-1">{t("advanced.confirmDialog.message")}</p>
            {pendingOperation && (
              <p className="text-xs text-obd-accent mt-2 font-mono">
                {pendingOperation.type === "operation" ? pendingOperation.op?.name : pendingOperation.cmd}
              </p>
            )}
          </div>
        </div>
        <div className="flex gap-2 pt-2">
          <button
            onClick={onCancel}
            className="flex-1 btn-secondary px-3 py-2 text-xs font-medium"
          >
            {t("advanced.confirmDialog.cancel")}
          </button>
          <button
            onClick={onConfirm}
            className="flex-1 btn-danger px-3 py-2 text-xs font-medium"
          >
            {t("advanced.confirmDialog.confirm")}
          </button>
        </div>
      </div>
    </div>
  );
}
