import { Terminal } from "lucide-react";

interface ConsoleResponse {
  cmd: string;
  res: string;
  time: string;
  isError: boolean;
}

interface AdvancedConsoleProps {
  responses: ConsoleResponse[];
  t: (key: string) => string;
}

export default function AdvancedConsole({ responses, t }: AdvancedConsoleProps) {
  return (
    <div className="w-full md:w-96 glass-card p-5 flex flex-col overflow-hidden">
      <h3 className="text-sm font-semibold text-obd-text-secondary uppercase tracking-wider mb-3">
        {t("advanced.console")}
      </h3>
      <div className="flex-1 rounded-lg bg-obd-bg/80 border border-obd-border/30 p-3 overflow-y-auto font-mono text-xs space-y-2">
        {responses.length === 0 ? (
          <div className="flex items-center gap-2 text-obd-text-muted">
            <Terminal size={14} />
            <span>{t("advanced.awaitingCommands")}</span>
          </div>
        ) : (
          responses.map((r, i) => (
            <div key={i} className="space-y-0.5">
              <div className="flex gap-2">
                <span className="text-obd-text-muted">[{r.time}]</span>
                <span className="text-obd-accent">→</span>
                <span className="text-obd-text flex-1">{r.cmd}</span>
              </div>
              <div className="flex gap-2 pl-[4.5rem]">
                <span className={r.isError ? "text-obd-danger" : "text-obd-success"}>
                  {r.isError ? "✗" : "✓"}
                </span>
                <span className="text-obd-text">{r.res}</span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
