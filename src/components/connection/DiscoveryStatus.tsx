import { AlertTriangle, Loader2, Trash2 } from "lucide-react";
import type { ConnectionStatus, VehicleInfo } from "@/stores/connection";

interface DiscoveryStatusProps {
  discoveryProgress: number;
  isDiscoveryComplete: boolean;
  status: ConnectionStatus;
  vehicle: VehicleInfo | null;
  hasVinCache: boolean;
  isConnected: boolean;
  onClearCache: () => Promise<void>;
  t: (key: string) => string;
}

export default function DiscoveryStatus({
  discoveryProgress,
  isDiscoveryComplete,
  status,
  vehicle,
  hasVinCache,
  onClearCache,
  t,
}: Omit<DiscoveryStatusProps, 'isConnected'>) {
  return (
    <>
      {/* Discovery progress */}
      {status === "connected" && vehicle && vehicle.vin && !isDiscoveryComplete && (
        <div className="flex items-start gap-3 p-3 rounded-lg bg-obd-accent/10 border border-obd-accent/30">
          <Loader2 className="w-4 h-4 text-obd-accent mt-0.5 flex-shrink-0 animate-spin" />
          <div className="flex-1">
            <p className="text-xs text-obd-accent font-medium">{t("connection.analyzing")}</p>
            <div className="mt-2 w-full h-1.5 bg-obd-border/30 rounded-full overflow-hidden">
              <div className="h-full bg-obd-accent transition-all" style={{ width: `${discoveryProgress}%` }} />
            </div>
          </div>
        </div>
      )}

      {/* VIN required alert */}
      {status === "connected" && vehicle && !vehicle.vin && (
        <div className="flex items-start gap-3 p-3 rounded-lg bg-obd-danger/10 border border-obd-danger/30">
          <AlertTriangle className="w-4 h-4 text-obd-danger mt-0.5 flex-shrink-0" />
          <p className="text-xs text-obd-danger leading-relaxed">
            {t("connection.vinRequired")}
          </p>
        </div>
      )}

      {/* Cache clear button */}
      {hasVinCache && status === "connected" && (
        <button
          onClick={onClearCache}
          className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-obd-border/20 text-obd-text-muted border border-obd-border/30 hover:bg-obd-border/40 text-xs"
        >
          <Trash2 size={14} />
          {t("connection.clearCache")}
        </button>
      )}
    </>
  );
}
