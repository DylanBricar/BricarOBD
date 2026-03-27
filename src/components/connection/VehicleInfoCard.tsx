import { Car, Fingerprint, Radio, KeyRound } from "lucide-react";
import InfoRow from "./InfoRow";
import type { VehicleInfo } from "@/stores/connection";

interface VehicleInfoCardProps {
  vehicle: VehicleInfo | null;
  status: string;
  t: (key: string) => string;
}

export default function VehicleInfoCard({
  vehicle,
  status,
  t,
}: VehicleInfoCardProps) {
  return (
    <div className="glass-card p-5 space-y-4">
      <h3 className="text-sm font-semibold text-obd-text-secondary uppercase tracking-wider">
        {t("connection.vehicle")}
      </h3>

      {vehicle ? (
        <div className="space-y-3">
          <div className="p-4 rounded-lg bg-obd-accent/5 border border-obd-accent/15">
            <div className="flex items-center gap-3">
              <Car size={24} className="text-obd-accent" />
              <div>
                <p className="font-semibold text-obd-text">
                  {vehicle.make} {vehicle.model}
                </p>
                <p className="text-xs text-obd-text-muted">{vehicle.year}</p>
              </div>
            </div>
          </div>
          <div className="space-y-2">
            <InfoRow icon={<Fingerprint size={14} />} label={t("connection.vin")} value={vehicle.vin} mono />
            <InfoRow icon={<Radio size={14} />} label={t("connection.protocol")} value={vehicle.protocol} />
            <InfoRow icon={<KeyRound size={14} />} label={t("connection.elmVersion")} value={vehicle.elmVersion ?? ""} />
          </div>
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-8 text-obd-text-muted">
          <Car size={48} strokeWidth={1} className="mb-3 opacity-20" />
          <p className="text-sm">{t("connection.disconnected")}</p>
        </div>
      )}

      {status === "error" && (
        <div className="p-3 rounded-lg bg-obd-danger/10 border border-obd-danger/20">
          <p className="text-xs text-obd-danger">{t("connection.errorMessage")}</p>
        </div>
      )}
    </div>
  );
}
