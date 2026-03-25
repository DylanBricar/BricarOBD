import { X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

interface ToastProps {
  message: string;
  type: "success" | "error";
  onDismiss: () => void;
}

export function Toast({ message, type, onDismiss }: ToastProps) {
  const { t } = useTranslation();

  return (
    <div className={cn(
      "fixed bottom-4 right-4 max-w-md px-4 py-3 rounded-lg shadow-lg flex items-start gap-3 animate-slide-in z-50",
      type === "success" ? "bg-obd-success/90 text-white" : "bg-obd-danger/90 text-white"
    )}>
      <p className="text-xs flex-1 leading-relaxed break-all">{message}</p>
      <button onClick={onDismiss} className="flex-shrink-0 hover:opacity-70" aria-label={t("common.close")}>
        <X size={14} />
      </button>
    </div>
  );
}
