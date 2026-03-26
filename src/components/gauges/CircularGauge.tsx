import { useMemo } from "react";
import { cn, clamp } from "@/lib/utils";

interface CircularGaugeProps {
  value: number;
  min: number;
  max: number;
  label: string;
  unit: string;
  size?: number;
  warningThreshold?: number;
  dangerThreshold?: number;
  decimals?: number;
  className?: string;
}

export default function CircularGauge({
  value,
  min,
  max,
  label,
  unit,
  size = 180,
  warningThreshold,
  dangerThreshold,
  decimals = 0,
  className,
}: CircularGaugeProps) {
  const normalizedValue = clamp(value, min, max);
  const percentage = ((normalizedValue - min) / (max - min)) * 100;

  const { arcPath, valuePath, color } = useMemo(() => {
    const startAngle = 135;
    const endAngle = 405;
    const totalArc = endAngle - startAngle;
    const valueAngle = startAngle + (totalArc * percentage) / 100;

    const cx = size / 2;
    const cy = size / 2;
    const radius = size / 2 - 20;

    const toRad = (deg: number) => (deg * Math.PI) / 180;

    const arcPoint = (angle: number) => ({
      x: cx + radius * Math.cos(toRad(angle)),
      y: cy + radius * Math.sin(toRad(angle)),
    });

    const start = arcPoint(startAngle);
    const end = arcPoint(endAngle);
    const valueEnd = arcPoint(valueAngle);

    const largeArc = endAngle - startAngle > 180 ? 1 : 0;
    const valueLargeArc = valueAngle - startAngle > 180 ? 1 : 0;

    const bgPath = `M ${start.x} ${start.y} A ${radius} ${radius} 0 ${largeArc} 1 ${end.x} ${end.y}`;
    const valPath = percentage > 0
      ? `M ${start.x} ${start.y} A ${radius} ${radius} 0 ${valueLargeArc} 1 ${valueEnd.x} ${valueEnd.y}`
      : "";

    let col = "var(--obd-chart-cyan)"; // cyan
    if (dangerThreshold && normalizedValue >= dangerThreshold) {
      col = "var(--obd-chart-red)";
    } else if (warningThreshold && normalizedValue >= warningThreshold) {
      col = "var(--obd-chart-amber)";
    }

    return { arcPath: bgPath, valuePath: valPath, color: col };
  }, [size, percentage, normalizedValue, warningThreshold, dangerThreshold]);

  const displayValue = decimals > 0 ? normalizedValue.toFixed(decimals) : Math.round(normalizedValue);
  const sanitizedLabel = label.replace(/\s+|[^a-z0-9-]/gi, "-").toLowerCase();

  return (
    <div className={cn("flex flex-col items-center", className)}>
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="drop-shadow-lg">
        {/* Background gradient */}
        <defs>
          <linearGradient id={`gauge-bg-${sanitizedLabel}`} x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="var(--obd-gauge-grad-start)" stopOpacity="0.3" />
            <stop offset="100%" stopColor="var(--obd-gauge-grad-end)" stopOpacity="0.5" />
          </linearGradient>
          <filter id={`glow-${sanitizedLabel}`}>
            <feGaussianBlur stdDeviation="3" result="coloredBlur" />
            <feMerge>
              <feMergeNode in="coloredBlur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* Outer ring background */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={size / 2 - 8}
          fill="none"
          stroke="var(--obd-gauge-grad-start)"
          strokeWidth="1"
          opacity="0.5"
        />

        {/* Background arc */}
        <path
          d={arcPath}
          fill="none"
          stroke="var(--obd-gauge-grad-start)"
          strokeWidth="8"
          strokeLinecap="round"
        />

        {/* Value arc */}
        {valuePath && (
          <path
            d={valuePath}
            fill="none"
            stroke={color}
            strokeWidth="8"
            strokeLinecap="round"
            filter={`url(#glow-${sanitizedLabel})`}
            style={{
              transition: "stroke 0.3s ease, d 0.3s ease",
            }}
          />
        )}

        {/* Center text */}
        <text
          x={size / 2}
          y={size / 2 - 8}
          textAnchor="middle"
          className="fill-obd-text font-mono"
          style={{ fontSize: size * 0.18, fontWeight: 600 }}
        >
          {displayValue}
        </text>
        <text
          x={size / 2}
          y={size / 2 + size * 0.08}
          textAnchor="middle"
          className="fill-obd-text-muted"
          style={{ fontSize: size * 0.075 }}
        >
          {unit}
        </text>
      </svg>
      <span className="text-xs font-medium text-obd-text-secondary -mt-2">{label}</span>
    </div>
  );
}
