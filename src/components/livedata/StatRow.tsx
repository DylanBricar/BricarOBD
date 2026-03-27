interface StatRowProps {
  label: string;
  value: string;
  unit: string;
}

export function StatRow({ label, value, unit }: StatRowProps) {
  return (
    <div className="flex items-center justify-between py-1.5">
      <span className="text-xs text-obd-text-muted">{label}</span>
      <span className="text-xs font-mono text-obd-text">
        {value} <span className="text-obd-text-muted">{unit}</span>
      </span>
    </div>
  );
}
