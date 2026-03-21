"""Dashboard frame with live OBD data display."""

import customtkinter as ctk
import threading

from gui.theme import COLORS, FONTS, GaugeWidget, StatusCard, _bind_scroll_recursive
from config import DASHBOARD_REFRESH_MS
from i18n import t, on_lang_change


class DashboardFrame(ctk.CTkFrame):
    """Main dashboard with live data display."""

    def __init__(self, parent, app):
        """Initialize dashboard frame.

        Args:
            parent: Parent widget (content_area)
            app: OBDApp reference
        """
        super().__init__(parent, fg_color=COLORS["bg_primary"])
        self.app = app
        self.monitoring = False
        self.update_id = None
        self._update_thread = None
        self.pack(fill="both", expand=True, padx=20, pady=20)

        # Register language change callback
        on_lang_change(self._on_lang_change)

        self._setup_ui()

    def _setup_ui(self):
        """Setup scrollable container."""
        self.scroll_container = ctk.CTkScrollableFrame(self, fg_color="transparent")
        self.scroll_container.pack(fill="both", expand=True, padx=16, pady=16)
        self._build_content()
        # Fix macOS trackpad scroll: propagate scroll events from all children
        self.after(500, lambda: _bind_scroll_recursive(self.scroll_container))

    def _build_content(self):
        """Build all dashboard content (called on init and language change)."""
        c = self.scroll_container

        self.title_label = ctk.CTkLabel(
            c, text=t("dash_title"), font=FONTS["heading"],
            text_color=COLORS["text_primary"]
        )
        self.title_label.pack(anchor="w", pady=(0, 20))

        self.help_label = ctk.CTkLabel(c, text=t("dash_help"), font=FONTS["small"],
                     text_color=COLORS["text_muted"])
        self.help_label.pack(anchor="w", pady=(0, 12))

        button_frame = ctk.CTkFrame(c, fg_color="transparent")
        button_frame.pack(fill="x", pady=(0, 12))

        self.monitor_btn = ctk.CTkButton(
            button_frame, text=t("dash_start"), width=140,
            command=self._on_monitor_btn_clicked,
            fg_color=COLORS["success"]
        )
        self.monitor_btn.pack(side="left")

        gauge_row = ctk.CTkFrame(c, fg_color="transparent")
        gauge_row.pack(fill="x", pady=(0, 12))

        self.rpm_gauge = GaugeWidget(
            gauge_row, t("dash_rpm"), "RPM", 0, 8000, warning_threshold=5500,
            danger_threshold=7000, size=140
        )
        self.rpm_gauge.pack(side="left", padx=(0, 10))

        self.speed_gauge = GaugeWidget(
            gauge_row, t("dash_speed"), "km/h", 0, 250, warning_threshold=130,
            danger_threshold=180, size=140
        )
        self.speed_gauge.pack(side="left", padx=(0, 10))

        self.temp_gauge = GaugeWidget(
            gauge_row, t("dash_coolant"), "°C", -40, 130, warning_threshold=100,
            danger_threshold=115, size=140
        )
        self.temp_gauge.pack(side="left", padx=(0, 10))

        self.load_gauge = GaugeWidget(
            gauge_row, t("dash_load"), "%", 0, 100, warning_threshold=80,
            danger_threshold=95, size=140
        )
        self.load_gauge.pack(side="left")

        self.status_title_label = ctk.CTkLabel(
            c, text=t("dash_parameters"), font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        )
        self.status_title_label.pack(anchor="w", pady=(10, 10))

        status_grid = ctk.CTkFrame(c, fg_color="transparent")
        status_grid.pack(fill="x", pady=(0, 8))

        self.throttle_card = StatusCard(status_grid, t("dash_throttle"), "%")
        self.throttle_card.grid(row=0, column=0, padx=4, pady=3, sticky="ew")

        self.intake_temp_card = StatusCard(status_grid, t("dash_intake"), "°C")
        self.intake_temp_card.grid(row=0, column=1, padx=4, pady=3, sticky="ew")

        self.maf_card = StatusCard(status_grid, t("dash_maf"), "g/s")
        self.maf_card.grid(row=0, column=2, padx=4, pady=3, sticky="ew")

        self.fuel_level_card = StatusCard(status_grid, t("dash_fuel"), "%")
        self.fuel_level_card.grid(row=1, column=0, padx=4, pady=3, sticky="ew")

        self.voltage_card = StatusCard(status_grid, t("dash_voltage"), "V")
        self.voltage_card.grid(row=1, column=1, padx=4, pady=3, sticky="ew")

        self.timing_card = StatusCard(status_grid, t("dash_timing"), "°")
        self.timing_card.grid(row=1, column=2, padx=4, pady=3, sticky="ew")

        for i in range(3):
            status_grid.grid_columnconfigure(i, weight=1)
        for i in range(2):
            status_grid.grid_rowconfigure(i, weight=0)

        vehicle_info_frame = ctk.CTkFrame(c, fg_color=COLORS["bg_card"], corner_radius=10, border_width=1, border_color=COLORS["card_border"])
        vehicle_info_frame.pack(fill="x", pady=(0, 8))

        self.vehicle_info_title = ctk.CTkLabel(
            vehicle_info_frame, text=t("dash_vehicle_info"), font=FONTS["small_bold"],
            text_color=COLORS["text_secondary"]
        )
        self.vehicle_info_title.pack(anchor="w", padx=12, pady=(8, 4))

        self.vin_label = ctk.CTkLabel(
            vehicle_info_frame, text="VIN: --", font=FONTS["mono_small"],
            text_color=COLORS["text_muted"]
        )
        self.vin_label.pack(anchor="w", padx=12, pady=1)

        self.protocol_label = ctk.CTkLabel(
            vehicle_info_frame, text=f"{t('dash_obd_protocol')}: --", font=FONTS["small"],
            text_color=COLORS["text_muted"]
        )
        self.protocol_label.pack(anchor="w", padx=12, pady=1)

        self.mil_label = ctk.CTkLabel(
            vehicle_info_frame, text=f"{t('dash_mil')}: --", font=FONTS["small"],
            text_color=COLORS["text_muted"]
        )
        self.mil_label.pack(anchor="w", padx=12, pady=(1, 8))

        # Additional vehicle info
        self.fuel_type_label = ctk.CTkLabel(vehicle_info_frame, text=f"{t('dash_fuel_type')}: --",
            font=FONTS["small"], text_color=COLORS["text_muted"])
        self.fuel_type_label.pack(anchor="w", padx=12, pady=1)

        self.obd_standard_label = ctk.CTkLabel(vehicle_info_frame, text=f"{t('dash_obd_standard')}: --",
            font=FONTS["small"], text_color=COLORS["text_muted"])
        self.obd_standard_label.pack(anchor="w", padx=12, pady=(1, 8))

        # System info card
        sys_card = ctk.CTkFrame(c, fg_color=COLORS["bg_card"], corner_radius=10,
                                border_width=1, border_color=COLORS["card_border"])
        sys_card.pack(fill="x", pady=(0, 8))

        ctk.CTkLabel(sys_card, text=t("dash_system"), font=FONTS["small_bold"],
                     text_color=COLORS["text_secondary"]).pack(anchor="w", padx=12, pady=(8, 4))

        self.elm_voltage_label = ctk.CTkLabel(sys_card, text=f"{t('dash_elm_voltage')}: --",
            font=FONTS["small"], text_color=COLORS["text_muted"])
        self.elm_voltage_label.pack(anchor="w", padx=12, pady=1)

        self.fuel_status_label = ctk.CTkLabel(sys_card, text=f"{t('dash_fuel_status')}: --",
            font=FONTS["small"], text_color=COLORS["text_muted"])
        self.fuel_status_label.pack(anchor="w", padx=12, pady=1)

        self.monitors_status_label = ctk.CTkLabel(sys_card, text=f"{t('dash_monitors')}: --",
            font=FONTS["small"], text_color=COLORS["text_muted"])
        self.monitors_status_label.pack(anchor="w", padx=12, pady=(1, 8))

    def _on_monitor_btn_clicked(self):
        """Handle monitor button click."""
        if self.monitoring:
            self.stop_monitoring()
        else:
            self.start_monitoring()

    def start_monitoring(self):
        """Start periodic PID reads."""
        if not self.app.connection or not self.app.connection.is_connected():
            return

        self.monitoring = True
        self.monitor_btn.configure(text=t("dash_stop"), fg_color=COLORS["danger"])
        self.update_vehicle_info()
        self._schedule_update()

    def stop_monitoring(self):
        """Cancel scheduled updates."""
        self.monitoring = False
        self.monitor_btn.configure(text=t("dash_start"), fg_color=COLORS["success"])
        if self.update_id:
            self.after_cancel(self.update_id)
            self.update_id = None

        # Reset all gauges
        self.rpm_gauge.set_value(0)
        self.speed_gauge.set_value(0)
        self.temp_gauge.set_value(0)
        self.load_gauge.set_value(0)

        # Reset all cards
        self.throttle_card.set_value("--")
        self.intake_temp_card.set_value("--")
        self.maf_card.set_value("--")
        self.fuel_level_card.set_value("--")
        self.voltage_card.set_value("--")
        self.timing_card.set_value("--")

        # Reset vehicle info
        self.vin_label.configure(text="VIN: --")
        self.protocol_label.configure(text=f"{t('dash_obd_protocol')}: --")
        self.mil_label.configure(text=f"{t('dash_mil')}: --", text_color=COLORS["text_muted"])

    def _schedule_update(self):
        """Schedule next update via self.after()."""
        if not self.monitoring:
            return

        # Don't spawn new thread if previous is still running
        if self._update_thread is None or not self._update_thread.is_alive():
            self._update_thread = threading.Thread(target=self.update_dashboard, daemon=True)
            self._update_thread.start()

        self.update_id = self.after(DASHBOARD_REFRESH_MS, self._schedule_update)

    def update_dashboard(self):
        """Read PIDs from obd_reader and update gauges and cards."""
        if not self.app.connection or not self.app.connection.is_connected() or not self.app.obd_reader:
            return

        try:
            rpm_val, _ = self.app.obd_reader.read_pid(0x0C)
            if rpm_val is not None:
                self.after(0, lambda v=rpm_val: self.rpm_gauge.set_value(v))

            speed_val, _ = self.app.obd_reader.read_pid(0x0D)
            if speed_val is not None:
                self.after(0, lambda v=speed_val: self.speed_gauge.set_value(v))

            coolant_val, _ = self.app.obd_reader.read_pid(0x05)
            if coolant_val is not None:
                self.after(0, lambda v=coolant_val: self.temp_gauge.set_value(v))

            load_val, _ = self.app.obd_reader.read_pid(0x04)
            if load_val is not None:
                self.after(0, lambda v=load_val: self.load_gauge.set_value(v))

            throttle_val, _ = self.app.obd_reader.read_pid(0x11)
            if throttle_val is not None:
                self.after(0, lambda v=throttle_val: self.throttle_card.set_value(f"{v:.1f}"))

            intake_val, _ = self.app.obd_reader.read_pid(0x0F)
            if intake_val is not None:
                self.after(0, lambda v=intake_val: self.intake_temp_card.set_value(f"{v:.1f}"))

            maf_val, _ = self.app.obd_reader.read_pid(0x10)
            if maf_val is not None:
                self.after(0, lambda v=maf_val: self.maf_card.set_value(f"{v:.2f}"))

            fuel_val, _ = self.app.obd_reader.read_pid(0x2F)
            if fuel_val is not None:
                self.after(0, lambda v=fuel_val: self.fuel_level_card.set_value(f"{v:.1f}"))

            voltage_val, _ = self.app.obd_reader.read_pid(0x42)
            if voltage_val is not None:
                self.after(0, lambda v=voltage_val: self.voltage_card.set_value(f"{v:.2f}"))

            timing_val, _ = self.app.obd_reader.read_pid(0x0E)
            if timing_val is not None:
                self.after(0, lambda v=timing_val: self.timing_card.set_value(f"{v:.1f}"))

        except Exception:
            pass

    def update_vehicle_info(self):
        """Read VIN and vehicle info once."""
        if not self.app.connection or not self.app.connection.is_connected() or not self.app.obd_reader:
            return

        thread = threading.Thread(target=self._update_vehicle_info_thread, daemon=True)
        thread.start()

    def _update_vehicle_info_thread(self):
        """Background thread for vehicle info retrieval."""
        try:
            vehicle_info = self.app.obd_reader.get_vehicle_info()
            vin_val = vehicle_info.get("vin", "")
            protocol = self.app.connection.protocol_name or "Unknown"

            # Read additional info
            fuel_type_val, _ = self.app.obd_reader.read_pid(0x51)
            obd_std_val, _ = self.app.obd_reader.read_pid(0x1C)

            # OBD standard names
            OBD_STANDARDS = {
                1: "OBD-II (CARB)", 2: "OBD (EPA)", 3: "OBD + OBD-II",
                6: "EOBD", 7: "EOBD + OBD-II", 9: "EOBD, OBD, OBD-II",
                13: "JOBD, EOBD, OBD-II", 17: "EMD", 18: "EMD+",
            }

            # Fuel type names
            FUEL_TYPES = {
                1: "Gasoline", 2: "Methanol", 3: "Ethanol", 4: "Diesel",
                5: "LPG", 6: "CNG", 8: "Electric",
            }

            # ELM voltage (AT command, not OBD)
            try:
                elm_v = self.app.connection.send_command("ATRV", timeout=2)
                if elm_v:
                    self.after(0, lambda v=elm_v: self.elm_voltage_label.configure(
                        text=f"{t('dash_elm_voltage')}: {v.strip()}"))
            except: pass

            # Fuel status
            FUEL_STATUS_NAMES = {
                1: "Open loop (cold)", 2: "Closed loop (O2)", 4: "Open loop (load)",
                8: "Open loop (fault)", 16: "Closed loop (fault)"
            }
            fuel_status_val, _ = self.app.obd_reader.read_pid(0x03)
            if fuel_status_val is not None:
                fs_name = FUEL_STATUS_NAMES.get(int(fuel_status_val), str(int(fuel_status_val)))
                self.after(0, lambda v=fs_name: self.fuel_status_label.configure(
                    text=f"{t('dash_fuel_status')}: {v}"))

            # Monitor readiness
            status_val, _ = self.app.obd_reader.read_pid(0x01)
            if status_val is not None:
                mil_on = bool(int(status_val) & 0x80) if isinstance(status_val, (int, float)) else False
                dtc_count = int(status_val) & 0x7F if isinstance(status_val, (int, float)) else 0
                self.after(0, lambda m=mil_on, d=dtc_count: self.monitors_status_label.configure(
                    text=f"{t('dash_monitors')}: {'MIL ON' if m else 'MIL OFF'} | {d} DTC(s)"))

            def update_ui():
                if vin_val:
                    self.vin_label.configure(text=f"VIN: {vin_val}")
                self.protocol_label.configure(text=f"{t('dash_obd_protocol')}: {protocol}")

                # Update fuel type
                if fuel_type_val is not None:
                    fuel_name = FUEL_TYPES.get(int(fuel_type_val), "Unknown")
                    self.fuel_type_label.configure(text=f"{t('dash_fuel_type')}: {fuel_name}")

                # Update OBD standard
                if obd_std_val is not None:
                    obd_name = OBD_STANDARDS.get(int(obd_std_val), "Unknown")
                    self.obd_standard_label.configure(text=f"{t('dash_obd_standard')}: {obd_name}")

            self.after(0, update_ui)
        except Exception:
            pass

    def _on_lang_change(self, lang=None):
        """Rebuild UI with new language.

        Args:
            lang: Language code (unused, kept for callback signature)
        """
        was_monitoring = self.monitoring
        if was_monitoring:
            self.stop_monitoring()

        for widget in self.scroll_container.winfo_children():
            widget.destroy()
        self._build_content()

        if was_monitoring:
            self.start_monitoring()
