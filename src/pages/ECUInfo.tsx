import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Cpu, RefreshCw, ChevronRight, Send, Battery } from "lucide-react";
import { useState, useEffect, useMemo } from "react";
import type { EcuInfo } from "@/stores/vehicle";
import { cn } from "@/lib/utils";

interface ECUInfoProps {
  ecus: EcuInfo[];
  isScanning?: boolean;
  onScan: () => void;
}


export default function ECUInfo({ ecus, isScanning = false, onScan }: ECUInfoProps) {
  const { t } = useTranslation();
  const [selectedEcu, setSelectedEcu] = useState<string | null>(null);
  const [didResults, setDidResults] = useState<Record<string, string>>({});
  const [loadingDid, setLoadingDid] = useState<string | null>(null);
  const [batteryVoltage, setBatteryVoltage] = useState<number | null>(null);
  const selected = ecus.find((e) => e.address === selectedEcu);

  const sortedEcus = useMemo(() => [...ecus].sort((a, b) => a.address.localeCompare(b.address)), [ecus]);

  const didLabels = useMemo<Record<string, string>>(() => ({
    "F190": "VIN",
    "F187": t("ecu.partNumber"),
    "F18C": t("ecu.serialNumber"),
    "F189": t("ecu.softwareVersion"),
    "F191": t("ecu.hardwareVersion"),
    "F194": t("ecu.supplierIdentifier"),
    "F195": t("ecu.softwareVersionNumber"),
  }), [t]);

  useEffect(() => {
    setDidResults({});
  }, [selectedEcu]);

  // Fetch battery voltage from adapter (only when ECUs are present = connected)
  useEffect(() => {
    if (ecus.length === 0) {
      setBatteryVoltage(null);
      return;
    }
    const fetchVoltage = async () => {
      try {
        const v = await invoke<number | null>("get_battery_voltage");
        setBatteryVoltage(v);
      } catch { /* non-critical */ }
    };
    fetchVoltage();
    const interval = setInterval(fetchVoltage, 15000);
    return () => clearInterval(interval);
  }, [ecus.length]);

  return (
    <div className="p-6 space-y-4 animate-slide-in h-full flex flex-col">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
            <Cpu className="text-obd-accent" size={20} />
          </div>
          <div>
            <h2 className="text-lg font-semibold">{t("ecu.title")}</h2>
            <p className="text-xs text-obd-text-muted">{ecus.length} {ecus.length !== 1 ? t("ecu.modulesDetected") : t("ecu.moduleDetected")}</p>
          </div>
        </div>
        <button onClick={onScan} disabled={isScanning} className={cn("btn-accent flex items-center gap-1.5 text-xs", isScanning && "opacity-60")}>
          <RefreshCw size={14} className={cn(isScanning && "animate-spin")} />
          {t("common.refresh")}
        </button>
      </div>

      {batteryVoltage !== null && (
        <div className="glass-card p-4">
          <div className="flex items-center gap-3">
            <div className={cn("w-10 h-10 rounded-lg flex items-center justify-center",
              batteryVoltage >= 12.4 ? "bg-obd-success/10" : batteryVoltage >= 11.8 ? "bg-obd-warning/10" : "bg-obd-danger/10"
            )}>
              <Battery size={18} className={cn(
                batteryVoltage >= 12.4 ? "text-obd-success" : batteryVoltage >= 11.8 ? "text-obd-warning" : "text-obd-danger"
              )} />
            </div>
            <div>
              <p className={cn("text-sm font-medium",
                batteryVoltage >= 12.4 ? "text-obd-success" : batteryVoltage >= 11.8 ? "text-obd-warning" : "text-obd-danger"
              )}>
                {batteryVoltage.toFixed(1)} V
              </p>
              <p className="text-xs text-obd-text-muted">{t("connection.batteryVoltage")}</p>
            </div>
          </div>
        </div>
      )}

      <div className="flex flex-col md:flex-row gap-4 flex-1 min-h-0">
        {/* ECU List */}
        <div className="flex-1 glass-card overflow-y-auto">
          {ecus.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-obd-text-muted">
              <Cpu size={48} strokeWidth={1} className="mb-3 opacity-20" />
              <p className="text-sm">{t("ecu.noModulesDetected")}</p>
            </div>
          ) : (
            sortedEcus.map((ecu) => (
            <button
              key={ecu.address}
              onClick={() => setSelectedEcu(ecu.address === selectedEcu ? null : ecu.address)}
              className={cn(
                "data-row w-full text-left",
                selectedEcu === ecu.address && "bg-obd-accent/5 border-l-2 border-l-obd-accent"
              )}
            >
              <div className="w-10 h-10 rounded-lg bg-obd-accent/10 flex items-center justify-center mr-3">
                <Cpu size={18} className="text-obd-accent" />
              </div>
              <div className="flex-1">
                <p className="text-sm font-medium text-obd-text">{ecu.name}</p>
                <p className="text-[10px] text-obd-text-muted">{ecu.protocol}</p>
              </div>
              <span className="text-xs font-mono text-obd-text-muted mr-2">{ecu.address}</span>
              <ChevronRight size={14} className="text-obd-text-muted" />
            </button>
            ))
          )}
        </div>

        {/* ECU Detail Panel */}
        {selected && (
          <div className="w-full md:w-96 glass-card p-5 space-y-4 overflow-y-auto">
            <div className="flex items-center gap-3">
              <div className="w-12 h-12 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
                <Cpu className="text-obd-accent" size={24} />
              </div>
              <div>
                <h3 className="font-semibold text-obd-text">{selected.name}</h3>
                <p className="text-xs text-obd-text-muted font-mono">{selected.address}</p>
              </div>
            </div>

            {/* Protocol Info */}
            <div className="space-y-2">
              <h4 className="text-xs font-semibold text-obd-text-secondary uppercase tracking-wider">{t("ecu.protocol")}</h4>
              <div className="flex items-center gap-2 p-3 rounded-lg bg-white/[0.02]">
                <div className="w-8 h-8 rounded-md bg-obd-accent/10 flex items-center justify-center">
                  <Cpu size={16} className="text-obd-accent" />
                </div>
                <div>
                  <p className="text-xs font-medium text-obd-text">{selected.protocol}</p>
                  <p className="text-[10px] text-obd-text-muted">{t("ecu.canInterface")}</p>
                </div>
              </div>
            </div>

            {/* DIDs list */}
            {Object.entries(selected.dids).length > 0 && (
              <div className="space-y-3">
                <h4 className="text-xs font-semibold text-obd-text-secondary uppercase tracking-wider">{t("ecu.dataIdentifiers")}</h4>
                <div className="space-y-1.5">
                  {Object.entries(selected.dids).map(([did, val]) => (
                    <div key={did} className="flex items-center justify-between p-2.5 rounded-lg bg-white/[0.02]">
                      <div className="flex-1 min-w-0">
                        <p className="text-xs font-mono text-obd-accent">{didLabels[did] || did}</p>
                        <p className="text-xs text-obd-text truncate">{val}</p>
                      </div>
                      {didResults[did] && (
                        <p className={cn("text-[10px] font-mono mt-0.5 truncate", didResults[did].startsWith("Error") ? "text-obd-danger" : "text-obd-success")}>
                          {didResults[did].startsWith("[DEMO]")
                            ? t("common.success")
                            : didResults[did]}
                        </p>
                      )}
                      <button
                        onClick={async () => {
                          setLoadingDid(did);
                          try {
                            const result = await invoke<string>("read_did", { ecuAddress: selected.address, did });
                            setDidResults(prev => ({ ...prev, [did]: result }));
                          } catch (e) {
                            setDidResults(prev => ({ ...prev, [did]: `Error: ${e}` }));
                          }
                          setLoadingDid(null);
                        }}
                        disabled={loadingDid === did}
                        className={cn("ml-2 p-1.5 rounded-md bg-obd-accent/10 hover:bg-obd-accent/20 transition-colors flex-shrink-0", loadingDid === did && "opacity-60")}
                      >
                        <Send size={12} className={cn("text-obd-accent", loadingDid === did && "animate-spin")} />
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {Object.entries(selected.dids).length === 0 && (
              <div className="flex items-center justify-center py-6 text-obd-text-muted">
                <p className="text-xs">{t("ecu.noDidsAvailable")}</p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
