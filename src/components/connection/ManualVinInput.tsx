import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";
import type { VehicleInfo } from "@/stores/connection";

const isValidVin = (vin: string): boolean => {
  if (vin.length !== 17) return false;
  const invalidChars = /[IOQ]/i;
  return !invalidChars.test(vin) && /^[A-Z0-9]+$/.test(vin);
};

interface ManualVinInputProps {
  value: string;
  onChange: (vin: string) => void;
  onVehicleUpdate?: (vehicle: VehicleInfo) => void;
  showToast: (message: string, type?: "success" | "error") => void;
}

export default function ManualVinInput({ value: manualVin, onChange: setManualVin, onVehicleUpdate, showToast }: ManualVinInputProps) {
  const { t } = useTranslation();

  const handleSubmit = async () => {
    if (isValidVin(manualVin)) {
      try {
        const info = await invoke<VehicleInfo>("set_manual_vin", { vin: manualVin });
        onVehicleUpdate?.(info);
        showToast(`${t("connection.vin")}: ${info.make || manualVin} ${info.year || ""}`);
      } catch (e) {
        showToast(String(e), "error");
      }
    } else if (manualVin.length > 0) {
      showToast(t("connection.vinInvalid", { count: manualVin.length }), "error");
    }
  };

  return (
    <div className="space-y-1.5 pt-2 border-t border-obd-border/30">
      <label className="text-xs text-obd-text-muted">{t("connection.manualVin")}</label>
      <div className="flex gap-2">
        <div className="flex-1">
          <input
            type="text"
            value={manualVin}
            onChange={(e) => setManualVin(e.target.value.toUpperCase())}
            placeholder="VF3LCBHZ6JS123456"
            maxLength={17}
            className={cn("input-field font-mono text-xs w-full", manualVin && !isValidVin(manualVin) && "border-obd-danger")}
          />
          <p className="text-xs text-obd-text-muted mt-1">
            {t("connection.vinLength", { current: manualVin.length })}
          </p>
          {manualVin && /[IOQ]/i.test(manualVin) && (
            <p className="text-xs text-obd-danger mt-1">{t("connection.vinInvalidChars")}</p>
          )}
        </div>
        <button
          onClick={handleSubmit}
          disabled={!isValidVin(manualVin)}
          className={cn("btn-ghost text-xs px-3", manualVin && !isValidVin(manualVin) && "opacity-50")}
        >{t("common.ok")}</button>
      </div>
    </div>
  );
}
