import { useState, useEffect, useCallback, memo } from "react";
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
  Menu,
  X,
  Sun,
  Moon,
  Monitor,
  Gauge,
} from "lucide-react";
import type { LucideProps } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ConnectionStatus } from "@/stores/connection";
import { useThemeStore, type ThemeMode } from "@/stores/theme";
import { useUnitSystem } from "@/lib/units";

interface SidebarProps {
  activePage: string;
  onNavigate: (page: string) => void;
  connectionStatus: ConnectionStatus;
  canNavigate?: boolean;
  onToggleDevConsole?: () => void;
  dtcCount?: number;
}

interface NavItem {
  id: string;
  icon: React.ForwardRefExoticComponent<Omit<LucideProps, "ref"> & React.RefAttributes<SVGSVGElement>>;
  labelKey: string;
  danger?: boolean;
}

const navItems: NavItem[] = [
  { id: "connection", icon: Plug, labelKey: "nav.connection" },
  { id: "dashboard", icon: LayoutDashboard, labelKey: "nav.dashboard" },
  { id: "liveData", icon: Activity, labelKey: "nav.liveData" },
  { id: "dtc", icon: AlertTriangle, labelKey: "nav.dtc" },
  { id: "ecuInfo", icon: Cpu, labelKey: "nav.ecuInfo" },
  { id: "monitors", icon: MonitorCheck, labelKey: "nav.monitors" },
  { id: "history", icon: Clock, labelKey: "nav.history" },
];

const advancedItems: NavItem[] = [
  { id: "advanced", icon: Wrench, labelKey: "nav.advanced", danger: true },
];

const NavItemButton = memo(function NavItemButton({
  item,
  isActive,
  isDisabled,
  onClick,
  badge,
}: {
  item: NavItem;
  isActive: boolean;
  isDisabled: boolean;
  onClick: () => void;
  badge?: number;
}) {
  const { t } = useTranslation();
  const Icon = item.icon;

  return (
    <button
      key={item.id}
      onClick={onClick}
      disabled={isDisabled}
      className={cn(
        "relative",
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
      {badge && badge > 0 && (
        <span className="absolute -top-1 -right-1 bg-red-500 text-white text-[10px] font-bold rounded-full w-4 h-4 flex items-center justify-center">
          {badge}
        </span>
      )}
    </button>
  );
});

const NavItemsSection = memo(function NavItemsSection({
  items,
  sectionLabelKey,
  activePage,
  isConnected,
  onNavigate,
  dtcCount,
}: {
  items: NavItem[];
  sectionLabelKey: string;
  activePage: string;
  isConnected: boolean;
  onNavigate: (page: string) => void;
  dtcCount?: number;
}) {
  const { t } = useTranslation();

  return (
    <div>
      <p className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-widest text-obd-text-muted">
        {t(sectionLabelKey)}
      </p>
      {items.map((item) => {
        const isActive = activePage === item.id;
        const isDisabled =
          item.id === "connection" || item.id === "history" ? false : !isConnected;

        return (
          <NavItemButton
            key={item.id}
            item={item}
            isActive={isActive}
            isDisabled={isDisabled}
            onClick={() => !isDisabled && onNavigate(item.id)}
            badge={item.id === "dtc" ? dtcCount : undefined}
          />
        );
      })}
    </div>
  );
});

export default function Sidebar({ activePage, onNavigate, connectionStatus, canNavigate, onToggleDevConsole, dtcCount }: SidebarProps) {
  const { t, i18n } = useTranslation();
  const isConnected = canNavigate ?? (connectionStatus === "connected" || connectionStatus === "demo");
  const [mobileOpen, setMobileOpen] = useState(false);
  const [isMobile, setIsMobile] = useState(window.innerWidth < 768);
  const { system: unitSystem, setUnitSystem } = useUnitSystem();

  useEffect(() => {
    const handleResize = () => setIsMobile(window.innerWidth < 768);
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  const toggleLanguage = () => {
    i18n.changeLanguage(i18n.language === "fr" ? "en" : "fr");
  };

  const { mode: themeMode, setThemeMode } = useThemeStore();

  const cycleTheme = () => {
    const order: ThemeMode[] = ["system", "dark", "light"];
    const next = order[(order.indexOf(themeMode) + 1) % order.length];
    setThemeMode(next);
  };

  const ThemeIcon = themeMode === "system" ? Monitor : themeMode === "dark" ? Moon : Sun;

  const handleNavigate = useCallback((page: string) => {
    onNavigate(page);
    if (isMobile) setMobileOpen(false);
  }, [onNavigate, isMobile]);

  const toggleUnitSystem = () => {
    setUnitSystem(unitSystem === "metric" ? "imperial" : "metric");
  };

  const sidebarContent = (
    <>
      {/* Logo */}
      <div className="px-2 py-3 border-b border-obd-border/30 flex justify-center">
        <svg width="160" height="40" viewBox="0 0 160 40" className="text-obd-text" style={{ color: "currentColor" }}>
          <text x="8" y="28" fontSize="24" fontWeight="bold" fill="currentColor">BricarOBD</text>
        </svg>
      </div>

      {/* Main Nav */}
      <nav className="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
        <NavItemsSection
          items={navItems}
          sectionLabelKey="nav.principal"
          activePage={activePage}
          isConnected={isConnected}
          onNavigate={handleNavigate}
          dtcCount={dtcCount}
        />

        <div className="pt-3">
          <NavItemsSection
            items={advancedItems}
            sectionLabelKey="nav.expert"
            activePage={activePage}
            isConnected={isConnected}
            onNavigate={handleNavigate}
          />
        </div>
      </nav>

      {/* Footer */}
      <div className="px-3 pb-4 space-y-2 border-t border-obd-border/30 pt-3">
        <button
          onClick={onToggleDevConsole}
          className="nav-item w-full justify-center text-[10px] opacity-60 hover:opacity-100"
        >
          <Terminal size={14} strokeWidth={1.8} />
          <span className="text-[10px]">{t("nav.devConsole")}</span>
        </button>
        <button
          onClick={toggleUnitSystem}
          className="nav-item w-full justify-center"
          title={t("nav.units")}
        >
          <Gauge size={16} strokeWidth={1.8} />
          <span className="text-xs">{unitSystem === "metric" ? "km/h" : "mph"}</span>
        </button>
        <button
          onClick={cycleTheme}
          className="nav-item w-full justify-center"
          title={t("nav.theme")}
        >
          <ThemeIcon size={16} strokeWidth={1.8} />
          <span className="text-xs">{t(`nav.theme.${themeMode}`)}</span>
        </button>
        <button
          onClick={toggleLanguage}
          className="nav-item w-full justify-center"
        >
          <Globe size={16} strokeWidth={1.8} />
          <span className="text-xs">{i18n.language === "fr" ? "FR" : "EN"}</span>
        </button>
      </div>
    </>
  );

  if (isMobile) {
    return (
      <>
        <button
          onClick={() => setMobileOpen(!mobileOpen)}
          className="fixed top-4 left-4 z-40 p-2 rounded-lg bg-obd-border/20 hover:bg-obd-border/40 transition-colors md:hidden"
          aria-label={t("nav.toggleMenu")}
          aria-expanded={mobileOpen}
        >
          {mobileOpen ? <X size={20} /> : <Menu size={20} />}
        </button>

        {mobileOpen && (
          <div
            className="fixed inset-0 bg-black/40 z-30 md:hidden"
            onClick={() => setMobileOpen(false)}
          />
        )}

        <aside className={cn(
          "glass-sidebar w-[220px] flex flex-col h-full select-none fixed left-0 top-0 z-30 md:static transition-transform duration-300",
          mobileOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0"
        )}>
          {sidebarContent}
        </aside>
      </>
    );
  }

  return (
    <aside className="glass-sidebar w-[220px] flex flex-col h-full select-none">
      {sidebarContent}
    </aside>
  );
}
