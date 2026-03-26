import { useTranslation } from "react-i18next";
import { AlertCircle, ChevronUp } from "lucide-react";

interface TroubleshootingProps {
  onClose: () => void;
}

export default function Troubleshooting({ onClose }: TroubleshootingProps) {
  const { t } = useTranslation();

  return (
    <div className="glass-card p-4 border-obd-warning/30">
      <button
        onClick={onClose}
        className="flex items-center gap-2 w-full text-left"
      >
        <AlertCircle size={16} className="text-obd-warning" />
        <span className="text-sm font-semibold text-obd-warning flex-1">
          {t("connection.troubleshoot.title")}
        </span>
        <ChevronUp size={14} className="text-obd-text-muted" />
      </button>
      <ol className="mt-3 space-y-1.5 list-decimal list-inside">
        {[1, 2, 3, 4, 5, 6].map((n) => (
          <li key={n} className="text-xs text-obd-text-muted">
            {t(`connection.troubleshoot.tip${n}`)}
          </li>
        ))}
      </ol>
    </div>
  );
}
