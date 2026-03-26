import { useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Wrench, AlertTriangle, Send, Terminal, ChevronDown, Zap, Settings, RefreshCw, Lock } from "lucide-react";
import { cn } from "@/lib/utils";

interface AdvancedOperation {
  id: string;
  name: string;
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

/// Parse service ID from hex command (handles spaced and unspaced formats)
const parseServiceId = (command: string): number | null => {
  const trimmed = command.trim().toUpperCase();
  if (!trimmed) return null;

  // Spaced format: "2E F1 90"
  const parts = trimmed.split(/\s+/);
  if (parts[0].length === 2) {
    const val = parseInt(parts[0], 16);
    return isNaN(val) ? null : val;
  }

  // Unspaced format: "2EF190"
  if (trimmed.length >= 2) {
    const val = parseInt(trimmed.substring(0, 2), 16);
    return isNaN(val) ? null : val;
  }

  return null;
};

/// Check if command is blocked (matches ALWAYS_BLOCKED list from backend)
const isCommandBlocked = (command: string): boolean => {
  const trimmed = command.trim().toUpperCase();

  // Blocked AT commands
  const blockedAt = ["ATMA", "ATBD", "ATBI", "ATPP", "ATWS"];
  for (const at of blockedAt) {
    if (trimmed.startsWith(at)) {
      return true;
    }
  }

  // Blocked service IDs (always blocked even in advanced mode)
  const alwaysBlocked = [0x11, 0x27, 0x28, 0x34, 0x35, 0x36, 0x37, 0x3D];
  const serviceId = parseServiceId(command);
  if (serviceId !== null && alwaysBlocked.includes(serviceId)) {
    return true;
  }

  return false;
};

export default function Advanced() {
  const { t, i18n } = useTranslation();
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set(["entretien"]));
  const [operationValues, setOperationValues] = useState<Record<string, string>>({});
  const [responses, setResponses] = useState<{ cmd: string; res: string; time: string; isError: boolean }[]>([]);
  const [executingOp, setExecutingOp] = useState<string | null>(null);
  const [rawCommand, setRawCommand] = useState("");
  const [executingRaw, setExecutingRaw] = useState(false);
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [pendingOperation, setPendingOperation] = useState<{ type: "operation" | "raw"; op?: AdvancedOperation; cmd?: string } | null>(null);

  const isBlocked = useMemo(() => isCommandBlocked(rawCommand), [rawCommand]);

  const categories = useMemo<OperationCategory[]>(() => [
    {
      id: "entretien",
      name: t("advanced.cat.entretien"),
      icon: <Settings size={18} />,
      operations: [
        {
          id: "reset_service",
          name: t("advanced.op.resetService"),
          description: t("advanced.op.resetServiceDesc"),
          risk_level: "medium",
          needs_value: false,
        },
        {
          id: "set_service_threshold",
          name: t("advanced.op.setServiceThreshold"),
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
          name: t("advanced.op.writeConfig"),
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
          name: t("advanced.op.forceRegen"),
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
          name: t("advanced.op.testInjectors"),
          description: t("advanced.op.testInjectorDesc"),
          risk_level: "high",
        },
        {
          id: "test_relays",
          name: t("advanced.op.testRelays"),
          description: t("advanced.op.testRelaysDesc"),
          risk_level: "medium",
        },
      ],
    },
  ], [t]);

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
    const value = operationValues[op.id];
    setPendingOperation({ type: "operation", op, cmd: value });
    setShowConfirmDialog(true);
  };

  const confirmOperation = async () => {
    if (!pendingOperation || pendingOperation.type !== "operation" || !pendingOperation.op) return;
    const op = pendingOperation.op;
    const now = new Date().toLocaleTimeString(i18n.language === "fr" ? "fr-FR" : "en-US");
    const value = operationValues[op.id];
    const cmd = `${op.name}${value ? ` [${value}]` : ""}`;

    setShowConfirmDialog(false);
    setPendingOperation(null);
    setExecutingOp(op.id);

    try {
      let result: string;
      try {
        result = await invoke<string>("send_raw_command", { ecuAddress: "0x7E0", command: op.id });
      } catch (err) {
        if (String(err) === "CONFIRM_REQUIRED") {
          result = await invoke<string>("send_raw_command", { ecuAddress: "0x7E0", command: op.id, confirmed: true });
        } else {
          throw err;
        }
      }
      setResponses(prev => [{ cmd, res: result, time: now, isError: false }, ...prev].slice(0, 200));
    } catch (e) {
      setResponses(prev => [{ cmd, res: `${t("common.error")}: ${e}`, time: now, isError: true }, ...prev].slice(0, 200));
    } finally {
      setExecutingOp(null);
    }
  };

  const executeRawCommand = async () => {
    if (!rawCommand.trim()) return;
    if (isBlocked) return;

    const cmd = rawCommand.trim();
    setPendingOperation({ type: "raw", cmd });
    setShowConfirmDialog(true);
  };

  const confirmRawCommand = async () => {
    if (!pendingOperation || pendingOperation.type !== "raw" || !pendingOperation.cmd) return;
    const cmd = pendingOperation.cmd;
    const now = new Date().toLocaleTimeString(i18n.language === "fr" ? "fr-FR" : "en-US");

    setShowConfirmDialog(false);
    setPendingOperation(null);
    setExecutingRaw(true);

    try {
      let result: string;
      try {
        result = await invoke<string>("send_raw_command", { ecuAddress: "0x7E0", command: cmd });
      } catch (err) {
        if (String(err) === "CONFIRM_REQUIRED") {
          result = await invoke<string>("send_raw_command", { ecuAddress: "0x7E0", command: cmd, confirmed: true });
        } else {
          throw err;
        }
      }
      setResponses(prev => [{ cmd, res: result, time: now, isError: false }, ...prev].slice(0, 200));
      setRawCommand("");
    } catch (e) {
      setResponses(prev => [{ cmd, res: `${t("common.error")}: ${e}`, time: now, isError: true }, ...prev].slice(0, 200));
    } finally {
      setExecutingRaw(false);
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

      <div className="flex flex-col md:flex-row gap-4 md:gap-6 flex-1 min-h-0">
        {/* Operations and Raw Command panel */}
        <div className="flex-1 overflow-y-auto space-y-3">
          {/* Raw Command Section */}
          <div className="glass-card p-4 space-y-3">
            <h3 className="text-sm font-semibold text-obd-text">{t("advanced.command")}</h3>
            <div className="space-y-2">
              <input
                type="text"
                value={rawCommand}
                onChange={(e) => setRawCommand(e.target.value)}
                placeholder={t("advanced.command")}
                className={cn("input-field text-sm w-full", isBlocked && "border-obd-danger/50 bg-obd-danger/5")}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !isBlocked && rawCommand.trim()) {
                    executeRawCommand();
                  }
                }}
              />
              {isBlocked && (
                <div className="flex items-start gap-2 p-3 rounded-lg bg-obd-danger/10 border border-obd-danger/30">
                  <Lock size={14} className="text-obd-danger flex-shrink-0 mt-0.5" />
                  <p className="text-xs text-obd-danger">{t("advanced.blockedCommand")}</p>
                </div>
              )}
              <button
                onClick={executeRawCommand}
                disabled={!rawCommand.trim() || isBlocked || executingRaw}
                className={cn("w-full btn-accent-solid px-3 py-2 text-xs flex items-center justify-center gap-1.5", (isBlocked || !rawCommand.trim() || executingRaw) && "opacity-50 cursor-not-allowed")}
              >
                {executingRaw ? <RefreshCw size={12} className="animate-spin" /> : <Send size={12} />}
                {t("advanced.send")}
              </button>
            </div>
          </div>

          {/* Operations Categories */}
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
                            <p className="text-sm font-medium text-obd-text">{op.name}</p>
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
      </div>

      {/* Confirmation Modal */}
      {showConfirmDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm">
          <div className="glass-card p-6 max-w-sm mx-4 space-y-4">
            <div className="flex items-start gap-3">
              <div className="w-10 h-10 rounded-lg bg-obd-warning/10 border border-obd-warning/20 flex items-center justify-center flex-shrink-0">
                <AlertTriangle className="text-obd-warning" size={20} />
              </div>
              <div className="flex-1">
                <h3 className="font-semibold text-obd-text">{t("advanced.confirmDialog.title")}</h3>
                <p className="text-xs text-obd-text-muted mt-1">{t("advanced.confirmDialog.message")}</p>
                {pendingOperation && (
                  <p className="text-xs text-obd-accent mt-2 font-mono">
                    {pendingOperation.type === "operation" ? pendingOperation.op?.name : pendingOperation.cmd}
                  </p>
                )}
              </div>
            </div>
            <div className="flex gap-2 pt-2">
              <button
                onClick={() => {
                  setShowConfirmDialog(false);
                  setPendingOperation(null);
                }}
                className="flex-1 btn-secondary px-3 py-2 text-xs font-medium"
              >
                {t("advanced.confirmDialog.cancel")}
              </button>
              <button
                onClick={() => {
                  if (pendingOperation?.type === "operation") {
                    confirmOperation();
                  } else if (pendingOperation?.type === "raw") {
                    confirmRawCommand();
                  }
                }}
                className="flex-1 btn-danger px-3 py-2 text-xs font-medium"
              >
                {t("advanced.confirmDialog.confirm")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
