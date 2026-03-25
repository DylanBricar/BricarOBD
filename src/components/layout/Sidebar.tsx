import { useTranslation } from "react-i18next";
import {
  Plug,
  LayoutDashboard,
  Activity,
  AlertTriangle,
  Cpu,
  MonitorCheck,
  Clock,
  Wrench,
  Globe,
  Terminal,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { ConnectionStatus } from "@/stores/connection";

interface SidebarProps {
  activePage: string;
  onNavigate: (page: string) => void;
  connectionStatus: ConnectionStatus;
  onToggleDevConsole?: () => void;
}

const navItems = [
  { id: "connection", icon: Plug, labelKey: "nav.connection" },
  { id: "dashboard", icon: LayoutDashboard, labelKey: "nav.dashboard" },
  { id: "liveData", icon: Activity, labelKey: "nav.liveData" },
  { id: "dtc", icon: AlertTriangle, labelKey: "nav.dtc" },
  { id: "ecuInfo", icon: Cpu, labelKey: "nav.ecuInfo" },
  { id: "monitors", icon: MonitorCheck, labelKey: "nav.monitors" },
  { id: "history", icon: Clock, labelKey: "nav.history" },
];

const advancedItems = [
  { id: "advanced", icon: Wrench, labelKey: "nav.advanced", danger: true },
];

export default function Sidebar({ activePage, onNavigate, connectionStatus, onToggleDevConsole }: SidebarProps) {
  const { t, i18n } = useTranslation();
  const isConnected = connectionStatus === "connected" || connectionStatus === "demo";

  const toggleLanguage = () => {
    i18n.changeLanguage(i18n.language === "fr" ? "en" : "fr");
  };

  return (
    <aside className="glass-sidebar w-[220px] flex flex-col h-full select-none">
      {/* Logo */}
      <div className="px-2 py-3 border-b border-obd-border/30 flex justify-center">
        <img src="/logo.png" alt="BricarOBD" className="w-40 h-auto object-contain" />
      </div>

      {/* Main Nav */}
      <nav className="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
        <p className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-widest text-obd-text-muted">
          {t("nav.principal")}
        </p>
        {navItems.map((item) => {
          const isActive = activePage === item.id;
          const Icon = item.icon;
          const isDisabled = item.id !== "connection" && !isConnected;

          return (
            <button
              key={item.id}
              onClick={() => !isDisabled && onNavigate(item.id)}
              disabled={isDisabled}
              className={cn(
                isActive ? "nav-item-active" : "nav-item",
                isDisabled && "opacity-30 cursor-not-allowed hover:bg-transparent"
              )}
            >
              <Icon size={18} strokeWidth={1.8} />
              <span>{t(item.labelKey)}</span>
            </button>
          );
        })}

        <div className="pt-3">
          <p className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-widest text-obd-text-muted">
            {t("nav.expert")}
          </p>
          {advancedItems.map((item) => {
            const isActive = activePage === item.id;
            const Icon = item.icon;
            const isDisabled = item.id === "advanced" && !isConnected;

            return (
              <button
                key={item.id}
                onClick={() => !isDisabled && onNavigate(item.id)}
                disabled={isDisabled}
                className={cn(
                  item.danger
                    ? isActive
                      ? "nav-item-danger-active"
                      : "nav-item-danger"
                    : isActive
                      ? "nav-item-active"
                      : "nav-item",
                  isDisabled && "opacity-30 cursor-not-allowed hover:bg-transparent"
                )}
              >
                <Icon size={18} strokeWidth={1.8} />
                <span>{t(item.labelKey)}</span>
              </button>
            );
          })}
        </div>
      </nav>

      {/* Footer */}
      <div className="px-3 pb-4 space-y-2 border-t border-obd-border/30 pt-3">
        <button
          onClick={onToggleDevConsole}
          className="nav-item w-full justify-center text-[10px] opacity-60 hover:opacity-100"
        >
          <Terminal size={14} strokeWidth={1.8} />
          <span className="text-[10px]">Dev Console</span>
        </button>
        <button
          onClick={toggleLanguage}
          className="nav-item w-full justify-center"
        >
          <Globe size={16} strokeWidth={1.8} />
          <span className="text-xs">{i18n.language === "fr" ? "FR" : "EN"}</span>
        </button>
      </div>
    </aside>
  );
}
