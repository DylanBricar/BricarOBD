import { Plug, Smartphone } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ConnectionStatus } from "@/stores/connection";

interface ConnectionTypeSelectorProps {
  connectionType: "usb" | "wifi" | "usb_android";
  onTypeChange: (type: "usb" | "wifi" | "usb_android") => void;
  isConnected: boolean;
  isAndroid: boolean;
  status: ConnectionStatus;
  t: (key: string) => string;
}

export default function ConnectionTypeSelector({
  connectionType,
  onTypeChange,
  isConnected,
  isAndroid,
  t,
}: Omit<ConnectionTypeSelectorProps, 'status'>) {
  return (
    <div className="flex gap-2">
      <button
        onClick={() => onTypeChange("usb")}
        disabled={isConnected}
        className={cn(
          "flex-1 px-3 py-2 rounded-lg text-xs font-medium transition-all border",
          connectionType === "usb"
            ? "bg-obd-accent text-white border-obd-accent"
            : "bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40",
          isConnected && "opacity-50 cursor-not-allowed"
        )}
      >
        {t("connection.usb")}
      </button>
      <button
        onClick={() => onTypeChange("wifi")}
        disabled={isConnected}
        className={cn(
          "flex-1 px-3 py-2 rounded-lg text-xs font-medium transition-all border",
          connectionType === "wifi"
            ? "bg-obd-accent text-white border-obd-accent"
            : "bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40",
          isConnected && "opacity-50 cursor-not-allowed"
        )}
      >
        <Smartphone size={14} className="inline mr-1" />
        {t("connection.wifi")}
      </button>
      {isAndroid && (
        <button
          onClick={() => onTypeChange("usb_android")}
          disabled={isConnected}
          className={cn(
            "flex-1 px-3 py-2 rounded-lg text-xs font-medium transition-all border",
            connectionType === "usb_android"
              ? "bg-obd-accent text-white border-obd-accent"
              : "bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40",
            isConnected && "opacity-50 cursor-not-allowed"
          )}
        >
          <Plug size={14} className="inline mr-1" />
          {t("connection.usb")}
        </button>
      )}
    </div>
  );
}
