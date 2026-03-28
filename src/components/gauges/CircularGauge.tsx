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

  const { radius, circumference, offset, color } = useMemo(() => {
    const r = size / 2 - 20;
    const circ = 2 * Math.PI * r;
    const off = circ - (circ * percentage) / 100;

    let col = "var(--obd-chart-cyan)"; // cyan
    if (dangerThreshold && normalizedValue >= dangerThreshold) {
      col = "var(--obd-chart-red)";
    } else if (warningThreshold && normalizedValue >= warningThreshold) {
      col = "var(--obd-chart-amber)";
    }

    return { radius: r, circumference: circ, offset: off, color: col };
  }, [size, percentage, normalizedValue, warningThreshold, dangerThreshold]);

  const displayValue = decimals > 0 ? normalizedValue.toFixed(decimals) : Math.round(normalizedValue);
  const sanitizedLabel = label.replace(/\s+|[^a-z0-9-]/gi, "-").toLowerCase();

  return (
    <div className={cn("flex flex-col items-center", className)}>
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="drop-shadow-lg" role="img" aria-label={`${label}: ${displayValue} ${unit}`}>
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

        {/* Background circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--obd-gauge-grad-start)"
          strokeWidth="8"
          strokeLinecap="round"
        />

        {/* Value circle with strokeDasharray */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke={color}
          strokeWidth="8"
          strokeLinecap="round"
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          transform={`rotate(-90 ${size / 2} ${size / 2})`}
          filter={`url(#glow-${sanitizedLabel})`}
          style={{ transition: "stroke-dashoffset 0.3s ease, stroke 0.3s ease" }}
        />

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
