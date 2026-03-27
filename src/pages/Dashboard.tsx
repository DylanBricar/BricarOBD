import { useTranslation } from "react-i18next";
import { LayoutDashboard, Circle, Thermometer, Gauge, Fuel, Battery, Wind, Zap, Settings } from "lucide-react";
import { useState, useMemo, useCallback } from "react";
import CircularGauge from "@/components/gauges/CircularGauge";
import LiveChart from "@/components/charts/LiveChart";
import type { PidValue } from "@/stores/vehicle";
import { convertValue, useUnitSystem } from "@/lib/units";

interface DashboardProps {
  pidData: Map<number, PidValue>;
}

/** Standard OBD-II PID codes used in dashboard gauges */
const PID = {
  ENGINE_LOAD: 0x04,
  COOLANT_TEMP: 0x05,
  RPM: 0x0c,
  SPEED: 0x0d,
  INTAKE_AIR_TEMP: 0x0f,
  MAF_RATE: 0x10,
  FUEL_LEVEL: 0x2f,
  CONTROL_MODULE_VOLTAGE: 0x42,
} as const;

const pidIconMap: Record<number, React.ReactNode> = {
  [PID.ENGINE_LOAD]: <Gauge size={16} />,
  [PID.COOLANT_TEMP]: <Thermometer size={16} />,
  [PID.RPM]: <Zap size={16} />,
  [PID.SPEED]: <Wind size={16} />,
  [PID.INTAKE_AIR_TEMP]: <Thermometer size={16} />,
  [PID.MAF_RATE]: <Wind size={16} />,
  [PID.FUEL_LEVEL]: <Fuel size={16} />,
  [PID.CONTROL_MODULE_VOLTAGE]: <Battery size={16} />,
};

const pidColorMap: Record<number, string> = {
  [PID.ENGINE_LOAD]: "text-obd-warning",
  [PID.COOLANT_TEMP]: "text-obd-info",
  [PID.RPM]: "text-obd-accent",
  [PID.SPEED]: "text-obd-accent",
  [PID.INTAKE_AIR_TEMP]: "text-obd-info",
  [PID.MAF_RATE]: "text-obd-warning",
  [PID.FUEL_LEVEL]: "text-obd-accent",
  [PID.CONTROL_MODULE_VOLTAGE]: "text-obd-success",
};

const DEFAULT_GAUGES = [PID.RPM, PID.SPEED, PID.COOLANT_TEMP, PID.ENGINE_LOAD];

const gaugeLabels: Record<number, string> = {
  [PID.RPM]: "dashboard.rpm",
  [PID.SPEED]: "dashboard.speed",
  [PID.COOLANT_TEMP]: "dashboard.coolant",
  [PID.ENGINE_LOAD]: "dashboard.load",
};

const gaugeWarnings: Record<number, number | undefined> = {
  [PID.RPM]: 6500,
  [PID.SPEED]: undefined,
  [PID.COOLANT_TEMP]: 100,
  [PID.ENGINE_LOAD]: 80,
};

const gaugeDangers: Record<number, number | undefined> = {
  [PID.RPM]: 7500,
  [PID.SPEED]: undefined,
  [PID.COOLANT_TEMP]: 110,
  [PID.ENGINE_LOAD]: 95,
};

const chartLabels: Record<number, string> = {
  [PID.RPM]: "dashboard.rpm",
  [PID.SPEED]: "dashboard.speed",
  [PID.COOLANT_TEMP]: "dashboard.coolant",
  [PID.ENGINE_LOAD]: "dashboard.load",
};

const chartUnits: Record<number, string> = {
  [PID.RPM]: "RPM",
  [PID.SPEED]: "km/h",
  [PID.COOLANT_TEMP]: "°C",
  [PID.ENGINE_LOAD]: "%",
};

const chartColors: Record<number, string> = {
  [PID.RPM]: "var(--obd-chart-cyan)",
  [PID.SPEED]: "var(--obd-chart-cyan-light)",
  [PID.COOLANT_TEMP]: "var(--obd-chart-amber)",
  [PID.ENGINE_LOAD]: "var(--obd-chart-green)",
};

function getGaugeMaxValue(pidValue: PidValue): number {
  const u = pidValue.unit;
  if (u?.includes("tr/min") || u?.includes("rpm")) return 8000;
  if (u?.includes("km/h") || u?.includes("mph")) return 260;
  if (u?.includes("°C") || u?.includes("°F")) return 150;
  if (u === "%") return 100;
  if (u === "V") return 16;
  if (u?.includes("kPa")) return 800;
  if (u === "λ") return 2;
  return 100;
}

function getGaugeMinValue(pidValue: PidValue): number {
  const u = pidValue.unit;
  if (u?.includes("°C") || u?.includes("°F")) return -40;
  if (u === "λ") return 0;
  return 0;
}

export default function Dashboard({ pidData }: DashboardProps) {
  const { t } = useTranslation();
  const [selectedGauges, setSelectedGauges] = useState<number[]>(() => {
    try {
      const s = localStorage.getItem("bricarobd_dashboard_gauges");
      if (s) return JSON.parse(s);
    } catch {}
    return DEFAULT_GAUGES;
  });
  const [showConfig, setShowConfig] = useState(false);
  const { system: unitSystem } = useUnitSystem();

  const toggleGauge = useCallback((pidCode: number) => {
    setSelectedGauges((prev) => {
      const updated = prev.includes(pidCode)
        ? prev.filter((p) => p !== pidCode)
        : [...prev, pidCode].slice(0, 6);
      localStorage.setItem("bricarobd_dashboard_gauges", JSON.stringify(updated));
      return updated;
    });
  }, []);

  const availableGauges = useMemo(
    () => selectedGauges.filter((pid) => pidData.has(pid)),
    [selectedGauges, pidData]
  );

  const otherPids = useMemo(() => {
    const set = new Set(selectedGauges);
    return Array.from(pidData.keys()).filter((k) => !set.has(k));
  }, [selectedGauges, pidData]);

  const chartsData = useMemo(
    () =>
      selectedGauges
        .filter((pidCode) => pidData.get(pidCode)?.history && pidData.get(pidCode)!.history!.length > 0)
        .slice(0, 4),
    [selectedGauges, pidData]
  );

  if (pidData.size === 0) {
    return (
      <div className="p-6 animate-slide-in">
        <div className="flex items-center gap-3 mb-6">
          <div className="w-10 h-10 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
            <LayoutDashboard className="text-obd-accent" size={20} />
          </div>
          <div>
            <h2 className="text-lg font-semibold">{t("dashboard.title")}</h2>
            <p className="text-xs text-obd-text-muted">{t("dashboard.engineData")}</p>
          </div>
        </div>
        <div className="glass-card p-8 flex flex-col items-center justify-center gap-3">
          <Circle size={32} className="text-obd-text-muted" />
          <p className="text-obd-text-muted">{t("dashboard.noData")}</p>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="p-6 space-y-6 animate-slide-in">
        {/* Header */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
              <LayoutDashboard className="text-obd-accent" size={20} />
            </div>
            <div>
              <h2 className="text-lg font-semibold">{t("dashboard.title")}</h2>
              <p className="text-xs text-obd-text-muted">{t("dashboard.engineData")}</p>
            </div>
          </div>
          <button
            onClick={() => setShowConfig(!showConfig)}
            className="p-2 rounded-lg bg-obd-surface/50 hover:bg-obd-surface border border-obd-border/50 hover:border-obd-accent/50 transition-colors"
            title={t("dashboard.configGauges")}
          >
            <Settings size={20} className="text-obd-accent" />
          </button>
        </div>

      {/* Main Gauges */}
      {availableGauges.length > 0 && (
        <div className={`grid gap-4 ${availableGauges.length === 1 ? "grid-cols-1" : availableGauges.length === 2 ? "grid-cols-1 sm:grid-cols-2" : "grid-cols-2 md:grid-cols-4"}`}>
          {availableGauges.map((pidCode) => {
            const pidValue = pidData.get(pidCode)!;
            const converted = convertValue(pidValue.value ?? 0, pidValue.unit, unitSystem);
            const maxValue = getGaugeMaxValue(pidValue);
            const minValue = getGaugeMinValue(pidValue);

            return (
              <div key={pidCode} className="glass-card p-4 flex items-center justify-center">
                <CircularGauge
                  value={converted.value}
                  min={minValue}
                  max={maxValue}
                  label={t(gaugeLabels[pidCode] ?? "")}
                  unit={converted.unit}
                  size={170}
                  warningThreshold={gaugeWarnings[pidCode]}
                  dangerThreshold={gaugeDangers[pidCode]}
                />
              </div>
            );
          })}
        </div>
      )}

      {/* Dynamic Data Cards */}
      {otherPids.length > 0 && (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
          {otherPids.map((pidCode) => {
            const pidValue = pidData.get(pidCode)!;
            const converted = convertValue(pidValue.value ?? 0, pidValue.unit, unitSystem);
            return (
              <DataCard
                key={pidCode}
                icon={pidIconMap[pidCode] ?? <Circle size={16} />}
                label={pidValue.name}
                value={converted.value}
                unit={converted.unit}
                color={pidColorMap[pidCode] ?? "text-obd-text"}
                decimals={converted.unit === "°C" || converted.unit === "%" || converted.unit === "km/h" ? 0 : 1}
              />
            );
          })}
        </div>
      )}

      {/* Live Charts */}
      {chartsData.length > 0 && (
        <div className={`grid gap-4 ${chartsData.length === 1 ? "grid-cols-1" : chartsData.length === 2 ? "grid-cols-2" : "grid-cols-2"}`}>
          {chartsData.map((pidCode) => {
            const pidValue = pidData.get(pidCode)!;
            const convertedHistory = (pidValue.history ?? []).map((v) =>
              convertValue(v, pidValue.unit, unitSystem).value
            );

            return (
              <LiveChart
                key={pidCode}
                data={convertedHistory}
                label={t(chartLabels[pidCode] ?? "")}
                unit={chartUnits[pidCode] ?? ""}
                color={chartColors[pidCode] ?? "var(--obd-chart-cyan)"}
                height={100}
              />
            );
          })}
        </div>
      )}
      </div>

      {/* Configuration Modal */}
      {showConfig && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 animate-fade-in">
          <div className="glass-card max-w-md w-full mx-4 p-6 space-y-4 animate-scale-in">
            <h3 className="text-lg font-semibold">{t("dashboard.configGauges")}</h3>
            <p className="text-sm text-obd-text-muted">
              {t("dashboard.configDesc")}
            </p>
            <div className="space-y-2 max-h-80 overflow-y-auto">
              {Array.from(pidData.entries()).map(([pidCode, pidValue]) => (
                <label key={pidCode} className="flex items-center gap-3 p-3 rounded-lg hover:bg-obd-surface/50 cursor-pointer transition-colors">
                  <input
                    type="checkbox"
                    checked={selectedGauges.includes(pidCode)}
                    onChange={() => toggleGauge(pidCode)}
                    disabled={!selectedGauges.includes(pidCode) && selectedGauges.length >= 6}
                    className="w-4 h-4 rounded border-obd-border bg-obd-surface accent-obd-accent"
                  />
                  <span className="flex-1 text-sm">{pidValue.name}</span>
                  <span className="text-xs text-obd-text-muted">{pidValue.unit}</span>
                </label>
              ))}
            </div>
            <div className="flex gap-2 pt-4">
              <button
                onClick={() => setShowConfig(false)}
                className="flex-1 px-4 py-2 rounded-lg bg-obd-surface border border-obd-border hover:bg-obd-surface/50 transition-colors"
              >
                {t("common.close")}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

function DataCard({
  icon,
  label,
  value,
  unit,
  color,
  decimals = 0,
}: {
  icon: React.ReactNode;
  label: string;
  value: number;
  unit: string;
  color: string;
  decimals?: number;
}) {
  return (
    <div className="glass-card p-4">
      <div className="flex items-center gap-2 mb-2">
        <span className={color}>{icon}</span>
        <span className="text-xs text-obd-text-muted">{label}</span>
      </div>
      <div className="flex items-baseline gap-1">
        <span className="text-2xl font-semibold font-mono">
          {value.toFixed(decimals)}
        </span>
        <span className="text-xs text-obd-text-muted">{unit}</span>
      </div>
    </div>
  );
}
