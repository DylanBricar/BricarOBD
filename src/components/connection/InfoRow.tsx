import { cn } from "@/lib/utils";

interface InfoRowProps {
  icon: React.ReactNode;
  label: string;
  value: string;
  mono?: boolean;
}

export default function InfoRow({
  icon,
  label,
  value,
  mono,
}: InfoRowProps) {
  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-white/[0.02]">
      <span className="text-obd-text-muted">{icon}</span>
      <span className="text-xs text-obd-text-muted flex-1">{label}</span>
      <span className={cn("text-xs text-obd-text", mono && "font-mono")}>{value}</span>
    </div>
  );
}
