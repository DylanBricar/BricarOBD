import { useTranslation } from "react-i18next";
import { LayoutDashboard, Circle, Thermometer, Gauge, Fuel, Battery, Wind, Zap } from "lucide-react";
import CircularGauge from "@/components/gauges/CircularGauge";
import LiveChart from "@/components/charts/LiveChart";
import type { PidValue } from "@/stores/vehicle";

interface DashboardProps {
  pidData: Map<number, PidValue>;
}

const pidIconMap: Record<number, React.ReactNode> = {
  0x04: <Gauge size={16} />,
  0x05: <Thermometer size={16} />,
  0x0c: <Zap size={16} />,
  0x0d: <Wind size={16} />,
  0x0f: <Thermometer size={16} />,
  0x10: <Wind size={16} />,
  0x2f: <Fuel size={16} />,
  0x42: <Battery size={16} />,
};

const pidColorMap: Record<number, string> = {
  0x04: "text-obd-warning",
  0x05: "text-obd-info",
  0x0c: "text-obd-accent",
  0x0d: "text-obd-accent",
  0x0f: "text-obd-info",
  0x10: "text-obd-warning",
  0x2f: "text-obd-accent",
  0x42: "text-obd-success",
};

const mainGauges = [0x0c, 0x0d, 0x05, 0x04];

const gaugeLabels: Record<number, string> = {
  0x0c: "dashboard.rpm",
  0x0d: "dashboard.speed",
  0x05: "dashboard.coolant",
  0x04: "dashboard.load",
};

const gaugeMaxValues: Record<number, number> = {
  0x0c: 8000,
  0x0d: 250,
  0x05: 215,
  0x04: 100,
};

const gaugeMinValues: Record<number, number> = {
  0x0c: 0,
  0x0d: 0,
  0x05: -40,
  0x04: 0,
};

const gaugeUnits: Record<number, string> = {
  0x0c: "RPM",
  0x0d: "km/h",
  0x05: "°C",
  0x04: "%",
};

const gaugeWarnings: Record<number, number | undefined> = {
  0x0c: 6500,
  0x0d: undefined,
  0x05: 100,
  0x04: 80,
};

const gaugeDangers: Record<number, number | undefined> = {
  0x0c: 7500,
  0x0d: undefined,
  0x05: 110,
  0x04: 95,
};

const chartLabels: Record<number, string> = {
  0x0c: "dashboard.rpm",
  0x0d: "dashboard.speed",
  0x05: "dashboard.coolant",
  0x04: "dashboard.load",
};

const chartUnits: Record<number, string> = {
  0x0c: "RPM",
  0x0d: "km/h",
  0x05: "°C",
  0x04: "%",
};

const chartColors: Record<number, string> = {
  0x0c: "#06B6D4",
  0x0d: "#22D3EE",
  0x05: "#F59E0B",
  0x04: "#10B981",
};

export default function Dashboard({ pidData }: DashboardProps) {
  const { t } = useTranslation();

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

  const availableGauges = mainGauges.filter((pidCode) => pidData.has(pidCode));
  const otherPids = Array.from(pidData.keys()).filter(
    (pidCode) => !mainGauges.includes(pidCode)
  );
  const chartsData = mainGauges
    .filter((pidCode) => pidData.get(pidCode)?.history && pidData.get(pidCode)!.history!.length > 0)
    .slice(0, 4);

  return (
    <div className="p-6 space-y-6 animate-slide-in">
      {/* Header */}
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
          <LayoutDashboard className="text-obd-accent" size={20} />
        </div>
        <div>
          <h2 className="text-lg font-semibold">{t("dashboard.title")}</h2>
          <p className="text-xs text-obd-text-muted">{t("dashboard.engineData")}</p>
        </div>
      </div>

      {/* Main Gauges */}
      {availableGauges.length > 0 && (
        <div className={`grid gap-4 ${availableGauges.length === 1 ? "grid-cols-1" : availableGauges.length === 2 ? "grid-cols-2" : "grid-cols-3"}`}>
          {availableGauges.map((pidCode) => {
            const pidValue = pidData.get(pidCode)!;

            return (
              <div key={pidCode} className="glass-card p-4 flex items-center justify-center">
                <CircularGauge
                  value={pidValue.value ?? 0}
                  min={gaugeMinValues[pidCode] ?? 0}
                  max={gaugeMaxValues[pidCode] ?? 100}
                  label={t(gaugeLabels[pidCode] ?? "")}
                  unit={gaugeUnits[pidCode] ?? ""}
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
            return (
              <DataCard
                key={pidCode}
                icon={pidIconMap[pidCode] ?? <Circle size={16} />}
                label={pidValue.name}
                value={pidValue.value ?? 0}
                unit={pidValue.unit}
                color={pidColorMap[pidCode] ?? "text-obd-text"}
                decimals={pidValue.unit === "°C" || pidValue.unit === "%" || pidValue.unit === "km/h" ? 0 : 1}
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

            return (
              <LiveChart
                key={pidCode}
                data={pidValue.history ?? []}
                label={t(chartLabels[pidCode] ?? "")}
                unit={chartUnits[pidCode] ?? ""}
                color={chartColors[pidCode] ?? "#06B6D4"}
                height={100}
              />
            );
          })}
        </div>
      )}
    </div>
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
