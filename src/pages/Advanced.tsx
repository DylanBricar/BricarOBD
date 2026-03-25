import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Wrench, AlertTriangle, Send, Terminal, ChevronDown, AlertCircle, Zap, Settings, RefreshCw } from "lucide-react";
import { cn } from "@/lib/utils";

interface AdvancedOperation {
  id: string;
  name_fr: string;
  description: string;
  risk_level: "low" | "medium" | "high" | "critical";
  needs_value?: boolean;
  unit?: string;
}

interface OperationCategory {
  id: string;
  name: string;
  icon: React.ReactNode;
  operations: AdvancedOperation[];
}

const riskColors = {
  low: "bg-obd-success/10 text-obd-success border-obd-success/20",
  medium: "bg-obd-warning/10 text-obd-warning border-obd-warning/20",
  high: "bg-obd-danger/10 text-obd-danger border-obd-danger/20",
  critical: "bg-obd-danger/15 text-obd-danger border-obd-danger/30",
};

export default function Advanced() {
  const { t } = useTranslation();
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set(["entretien"]));
  const [operationValues, setOperationValues] = useState<Record<string, string>>({});
  const [responses, setResponses] = useState<{ cmd: string; res: string; time: string; isError: boolean }[]>([]);
  const [executingOp, setExecutingOp] = useState<string | null>(null);

  const categories: OperationCategory[] = [
    {
      id: "entretien",
      name: t("advanced.cat.entretien"),
      icon: <Settings size={18} />,
      operations: [
        {
          id: "reset_service",
          name_fr: t("advanced.op.resetService"),
          description: t("advanced.op.resetServiceDesc"),
          risk_level: "medium",
          needs_value: false,
        },
        {
          id: "set_service_threshold",
          name_fr: t("advanced.op.setServiceThreshold"),
          description: t("advanced.op.setServiceThresholdDesc"),
          risk_level: "medium",
          needs_value: true,
          unit: "km",
        },
      ],
    },
    {
      id: "config_vehicle",
      name: t("advanced.cat.configVehicle"),
      icon: <Settings size={18} />,
      operations: [
        {
          id: "write_config",
          name_fr: t("advanced.op.writeConfig"),
          description: t("advanced.op.writeConfigDesc"),
          risk_level: "high",
          needs_value: true,
        },
      ],
    },
    {
      id: "regen_fap",
      name: t("advanced.cat.regenFap"),
      icon: <Zap size={18} />,
      operations: [
        {
          id: "force_regen",
          name_fr: t("advanced.op.forceRegen"),
          description: t("advanced.op.forceRegenDesc"),
          risk_level: "high",
        },
      ],
    },
    {
      id: "test_actuators",
      name: t("advanced.cat.testActuators"),
      icon: <Zap size={18} />,
      operations: [
        {
          id: "test_injectors",
          name_fr: t("advanced.op.testInjectors"),
          description: t("advanced.op.testInjectorDesc"),
          risk_level: "high",
        },
        {
          id: "test_relays",
          name_fr: t("advanced.op.testRelays"),
          description: t("advanced.op.testRelaysDesc"),
          risk_level: "medium",
        },
      ],
    },
  ];

  const toggleCategory = (categoryId: string) => {
    const newExpanded = new Set(expandedCategories);
    if (newExpanded.has(categoryId)) {
      newExpanded.delete(categoryId);
    } else {
      newExpanded.add(categoryId);
    }
    setExpandedCategories(newExpanded);
  };

  const executeOperation = async (op: AdvancedOperation) => {
    const now = new Date().toLocaleTimeString();
    const value = operationValues[op.id];
    const cmd = `${op.name_fr}${value ? ` [${value}]` : ""}`;
    setExecutingOp(op.id);
    try {
      const result = await invoke<string>("send_raw_command", { ecuAddress: "0x7E0", command: op.id });
      setResponses(prev => [{ cmd, res: result, time: now, isError: false }, ...prev]);
    } catch (e) {
      setResponses(prev => [{ cmd, res: `${t("common.error")}: ${e}`, time: now, isError: true }, ...prev]);
    } finally {
      setExecutingOp(null);
    }
  };

  return (
    <div className="p-6 space-y-6 animate-slide-in h-full flex flex-col">
      {/* Header + Warning */}
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-obd-danger/10 border border-obd-danger/20 flex items-center justify-center">
          <Wrench className="text-obd-danger" size={20} />
        </div>
        <div>
          <h2 className="text-lg font-semibold">{t("advanced.title")}</h2>
          <p className="text-xs text-obd-text-muted">{t("advanced.subtitle")}</p>
        </div>
      </div>

      {/* Warning banner */}
      <div className="flex items-start gap-3 p-4 rounded-xl bg-obd-danger/5 border border-obd-danger/20">
        <AlertTriangle className="text-obd-danger flex-shrink-0 mt-0.5" size={18} />
        <p className="text-xs text-obd-danger/80 leading-relaxed">
          {t("advanced.warning")}
        </p>
      </div>

      <div className="flex gap-6 flex-1 min-h-0">
        {/* Operations panel */}
        <div className="flex-1 overflow-y-auto space-y-3">
          {categories.map((category) => (
            <div key={category.id} className="glass-card overflow-hidden">
              <button
                onClick={() => toggleCategory(category.id)}
                className="w-full flex items-center gap-3 p-4 hover:bg-white/[0.02] transition-colors"
              >
                <div className="text-obd-accent">
                  {category.icon}
                </div>
                <div className="flex-1 text-left">
                  <h3 className="text-sm font-semibold text-obd-text">{category.name}</h3>
                  <p className="text-xs text-obd-text-muted">{category.operations.length} {t("advanced.operations")}</p>
                </div>
                <ChevronDown
                  size={16}
                  className={cn(
                    "text-obd-text-muted transition-transform",
                    expandedCategories.has(category.id) && "rotate-180"
                  )}
                />
              </button>


              {expandedCategories.has(category.id) && (
                <div className="border-t border-obd-border/20 space-y-2 p-4">
                  {category.operations.map((op) => (
                    <div key={op.id} className="space-y-2">
                      <div className="space-y-1.5">
                        <div className="flex items-start gap-2">
                          <div className="flex-1">
                            <p className="text-sm font-medium text-obd-text">{op.name_fr}</p>
                            <p className="text-xs text-obd-text-muted mt-0.5">{op.description}</p>
                          </div>
                          <div className={cn("px-2 py-1 rounded-md text-[10px] font-semibold border", riskColors[op.risk_level])}>
                            {op.risk_level === "low" && t("advanced.riskLow")}
                            {op.risk_level === "medium" && t("advanced.riskMedium")}
                            {op.risk_level === "high" && t("advanced.riskHigh")}
                            {op.risk_level === "critical" && t("advanced.riskCritical")}
                          </div>
                        </div>

                        {op.needs_value && (
                          <div className="flex gap-2">
                            <input
                              type="text"
                              value={operationValues[op.id] || ""}
                              onChange={(e) =>
                                setOperationValues((prev) => ({
                                  ...prev,
                                  [op.id]: e.target.value,
                                }))
                              }
                              placeholder={`${t("advanced.value")} (${op.unit || ""})`}
                              className="input-field text-xs flex-1"
                            />
                            <button
                              onClick={() => executeOperation(op)}
                              disabled={executingOp === op.id}
                              className={cn("btn-accent-solid px-3 flex items-center gap-1.5 text-xs", executingOp === op.id && "opacity-50 cursor-not-allowed")}
                            >
                              {executingOp === op.id ? <RefreshCw size={12} className="animate-spin" /> : <Send size={12} />}
                              {t("advanced.execute")}
                            </button>
                          </div>
                        )}

                        {!op.needs_value && (
                          <button
                            onClick={() => executeOperation(op)}
                            disabled={executingOp === op.id}
                            className={cn("w-full btn-accent-solid px-3 py-2 text-xs flex items-center justify-center gap-1.5", executingOp === op.id && "opacity-50 cursor-not-allowed")}
                          >
                            {executingOp === op.id ? <RefreshCw size={12} className="animate-spin" /> : <Send size={12} />}
                            {t("advanced.execute")}
                          </button>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>

        <div className="w-96 glass-card p-5 flex flex-col">
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
      </div>
    </div>
  );
}
