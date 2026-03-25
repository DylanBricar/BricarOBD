import { useMemo } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
} from "recharts";
import { cn } from "@/lib/utils";

interface LiveChartProps {
  data: number[];
  label: string;
  unit: string;
  color?: string;
  height?: number;
  className?: string;
  showAxis?: boolean;
  minDomain?: number;
  maxDomain?: number;
}

export default function LiveChart({
  data,
  label,
  unit,
  color = "var(--obd-chart-cyan)",
  height = 120,
  className,
  showAxis = false,
  minDomain,
  maxDomain,
}: LiveChartProps) {
  const chartData = useMemo(
    () => data.map((value, i) => ({ index: i, value })),
    [data]
  );

  const currentValue = data.length > 0 ? data[data.length - 1] : 0;
  const gradientId = `gradient-${label.replace(/\s/g, "-")}`;

  return (
    <div className={cn("glass-card p-3", className)}>
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs font-medium text-obd-text-secondary">{label}</span>
        <span className="text-xs font-mono text-obd-text">
          {currentValue.toFixed(1)} <span className="text-obd-text-muted">{unit}</span>
        </span>
      </div>
      <ResponsiveContainer width="100%" height={height}>
        <AreaChart data={chartData} margin={{ top: 2, right: 2, left: 2, bottom: 2 }}>
          <defs>
            <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor={color} stopOpacity={0.3} />
              <stop offset="95%" stopColor={color} stopOpacity={0} />
            </linearGradient>
          </defs>
          {showAxis && (
            <>
              <XAxis dataKey="index" hide />
              <YAxis
                domain={[minDomain ?? "auto", maxDomain ?? "auto"]}
                hide
              />
            </>
          )}
          <Tooltip
            contentStyle={{
              backgroundColor: "var(--obd-chart-tooltip-bg)",
              border: "1px solid var(--obd-chart-tooltip-border)",
              borderRadius: "8px",
              fontSize: "11px",
              color: "var(--obd-chart-text)",
            }}
            formatter={(val: number) => [`${val.toFixed(1)} ${unit}`, label]}
            labelFormatter={() => ""}
          />
          <Area
            type="monotone"
            dataKey="value"
            stroke={color}
            strokeWidth={1.5}
            fill={`url(#${gradientId})`}
            isAnimationActive={false}
            dot={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
