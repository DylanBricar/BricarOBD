import { Radio } from "lucide-react";
import { cn } from "@/lib/utils";

interface DemoModeButtonProps {
  isConnected: boolean;
  onClick: () => void;
  t: (key: string) => string;
}

export default function DemoModeButton({
  isConnected,
  onClick,
  t,
}: DemoModeButtonProps) {
  return (
    <div className="pt-2 border-t border-obd-border/30">
      <button
        onClick={onClick}
        disabled={isConnected}
        className={cn(
          "w-full flex items-center gap-3 p-3 rounded-lg transition-all",
          "bg-obd-warning/5 border border-obd-warning/20 hover:bg-obd-warning/10",
          isConnected && "opacity-30 cursor-not-allowed"
        )}
      >
        <Radio size={18} className="text-obd-warning" />
        <div className="text-left">
          <p className="text-sm font-medium text-obd-warning">{t("connection.demo")}</p>
          <p className="text-[10px] text-obd-text-muted">{t("connection.demoDesc")}</p>
        </div>
      </button>
    </div>
  );
}
