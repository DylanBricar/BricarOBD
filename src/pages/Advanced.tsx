import { useState, useMemo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Wrench, AlertTriangle, Send, ChevronDown, Zap, Settings, RefreshCw, Lock } from "lucide-react";
import { cn } from "@/lib/utils";
import type { AdvancedOperation, OperationCategory } from "@/components/advanced/types";
import AdvancedConsole from "@/components/advanced/AdvancedConsole";
import ConfirmDialog from "@/components/advanced/ConfirmDialog";
import { riskColors } from "@/components/advanced/constants";
import { isCommandBlocked } from "@/components/advanced/utils";

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
  const [ecuAddress, setEcuAddress] = useState("0x7E0");
  const [customEcuAddress, setCustomEcuAddress] = useState("");

  const isBlocked = useMemo(() => isCommandBlocked(rawCommand), [rawCommand]);

  const categories = useMemo<OperationCategory[]>(() => [
    {
      id: "entretien",
      name: t("advanced.cat.entretien"),
      Icon: Settings,
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
      Icon: Settings,
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
      Icon: Zap,
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
      Icon: Zap,
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

  const toggleCategory = useCallback((categoryId: string) => {
    setExpandedCategories(prev => {
      const newExpanded = new Set(prev);
      if (newExpanded.has(categoryId)) {
        newExpanded.delete(categoryId);
      } else {
        newExpanded.add(categoryId);
      }
      return newExpanded;
    });
  }, []);

  const executeOperation = async (op: AdvancedOperation) => {
    const value = operationValues[op.id];
    setPendingOperation({ type: "operation", op, cmd: value });
    setShowConfirmDialog(true);
  };

  const confirmOperation = useCallback(async () => {
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
      const currentAddress = ecuAddress === "custom" ? customEcuAddress : ecuAddress;
      try {
        result = await invoke<string>("send_raw_command", { ecuAddress: currentAddress, command: op.id });
      } catch (err) {
        if (String(err) === "CONFIRM_REQUIRED") {
          result = await invoke<string>("send_raw_command", { ecuAddress: currentAddress, command: op.id, confirmed: true });
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
  }, [pendingOperation, operationValues, ecuAddress, customEcuAddress, i18n.language, t]);

  const executeRawCommand = async () => {
    if (!rawCommand.trim()) return;
    if (isBlocked) return;

    const cmd = rawCommand.trim();
    setPendingOperation({ type: "raw", cmd });
    setShowConfirmDialog(true);
  };

  const confirmRawCommand = useCallback(async () => {
    if (!pendingOperation || pendingOperation.type !== "raw" || !pendingOperation.cmd) return;
    const cmd = pendingOperation.cmd;
    const now = new Date().toLocaleTimeString(i18n.language === "fr" ? "fr-FR" : "en-US");

    setShowConfirmDialog(false);
    setPendingOperation(null);
    setExecutingRaw(true);

    try {
      let result: string;
      const currentAddress = ecuAddress === "custom" ? customEcuAddress : ecuAddress;
      try {
        result = await invoke<string>("send_raw_command", { ecuAddress: currentAddress, command: cmd });
      } catch (err) {
        if (String(err) === "CONFIRM_REQUIRED") {
          result = await invoke<string>("send_raw_command", { ecuAddress: currentAddress, command: cmd, confirmed: true });
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
  }, [pendingOperation, ecuAddress, customEcuAddress, i18n.language, t]);

  const handleCancelDialog = () => {
    setShowConfirmDialog(false);
    setPendingOperation(null);
  };

  const handleConfirmDialog = () => {
    if (pendingOperation?.type === "operation") {
      confirmOperation();
    } else if (pendingOperation?.type === "raw") {
      confirmRawCommand();
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

      {/* ECU Address Selector */}
      <div className="glass-card p-3 flex items-center gap-3">
        <label className="text-xs text-obd-text-muted">{t("advanced.ecuAddress")}</label>
        <select value={ecuAddress} onChange={e => setEcuAddress(e.target.value)} className="bg-obd-surface border border-obd-border rounded px-2 py-1 text-sm">
          <option value="0x7E0">0x7E0 ({t("advanced.ecuEngine")})</option>
          <option value="0x7E1">0x7E1 ({t("advanced.ecuTransmission")})</option>
          <option value="0x7E2">0x7E2 ({t("advanced.ecuAbs")})</option>
          <option value="0x7E3">0x7E3 ({t("advanced.ecuAirbag")})</option>
          <option value="0x7E4">0x7E4 ({t("advanced.ecuHvac")})</option>
          <option value="0x75D">0x75D ({t("advanced.ecuBsi")})</option>
          <option value="custom">{t("advanced.ecuCustom")}</option>
        </select>
        {ecuAddress === "custom" && <input value={customEcuAddress} onChange={e => setCustomEcuAddress(e.target.value)} placeholder="0x7XX" className="bg-obd-surface border border-obd-border rounded px-2 py-1 text-sm w-20" />}
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
                  <category.Icon size={18} />
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

        <AdvancedConsole responses={responses} t={t} />
      </div>

      <ConfirmDialog
        show={showConfirmDialog}
        pendingOperation={pendingOperation}
        onCancel={handleCancelDialog}
        onConfirm={handleConfirmDialog}
        t={t}
      />
    </div>
  );
}
