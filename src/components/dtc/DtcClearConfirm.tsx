import { AlertTriangle } from "lucide-react";

interface DtcClearConfirmProps {
  t: (key: string) => string;
  onConfirm: () => void;
  onCancel: () => void;
}

export default function DtcClearConfirm({ t, onConfirm, onCancel }: DtcClearConfirmProps) {
  return (
    <div
      className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
      onClick={onCancel}
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="dtc-confirm-title"
    >
      <div className="glass-card p-6 max-w-md w-full mx-4 space-y-4" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-full bg-obd-danger/20 flex items-center justify-center">
            <AlertTriangle className="text-obd-danger" size={20} />
          </div>
          <h3 id="dtc-confirm-title" className="text-lg font-semibold">{t("dtc.confirmClear")}</h3>
        </div>
        <p className="text-sm text-obd-text-secondary">{t("dtc.confirmClearMsg")}</p>
        <div className="flex gap-3 justify-end">
          <button onClick={onCancel} className="btn-ghost">
            {t("common.cancel")}
          </button>
          <button onClick={onConfirm} className="btn-danger">
            {t("common.confirm")}
          </button>
        </div>
      </div>
    </div>
  );
}
