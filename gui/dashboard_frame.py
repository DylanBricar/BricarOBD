"""Dashboard frame with live OBD data display."""

import csv
import customtkinter as ctk
import logging
import threading
from datetime import datetime

from gui.theme import COLORS, FONTS, GaugeWidget, GraphWidget, StatusCard, _bind_scroll_recursive
from config import DASHBOARD_REFRESH_MS, GRAPH_HISTORY_SAMPLES, CSV_DIR
from i18n import t, on_lang_change

logger = logging.getLogger(__name__)


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
        self._csv_recording = False
        self._csv_file = None
        self._csv_writer = None
        self._csv_lock = threading.Lock()
        self.pack(fill="both", expand=True)

        # Register language change callback
        on_lang_change(self._on_lang_change)

        self._setup_ui()

    def _setup_ui(self):
        """Setup scrollable container."""
        self.scroll_container = ctk.CTkScrollableFrame(self, fg_color="transparent")
        self.scroll_container.pack(fill="both", expand=True)
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
        self.title_label.pack(anchor="w", padx=16, pady=(0, 4))

        self.help_label = ctk.CTkLabel(c, text=t("dash_help"), font=FONTS["small"],
                     text_color=COLORS["text_muted"])
        self.help_label.pack(anchor="w", padx=16, pady=(0, 12))

        button_frame = ctk.CTkFrame(c, fg_color="transparent")
        button_frame.pack(fill="x", padx=16, pady=(0, 12))

        self.monitor_btn = ctk.CTkButton(
            button_frame, text=t("dash_start"), width=140,
            command=self._on_monitor_btn_clicked,
            fg_color=COLORS["success"]
        )
        self.monitor_btn.pack(side="left")

        self.csv_btn = ctk.CTkButton(
            button_frame, text=t("csv_record"), width=140,
            command=self._toggle_csv_recording,
            fg_color=COLORS["accent"], hover_color=COLORS["accent_hover"],
        )
        self.csv_btn.pack(side="left", padx=(12, 0))

        self.csv_status = ctk.CTkLabel(
            button_frame, text="", font=FONTS["small"],
            text_color=COLORS["text_muted"]
        )
        self.csv_status.pack(side="left", padx=(8, 0))

        gauge_row = ctk.CTkFrame(c, fg_color="transparent")
        gauge_row.pack(fill="x", padx=16, pady=(0, 8))

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

        # Live graphs
        graph_row = ctk.CTkFrame(c, fg_color="transparent")
        graph_row.pack(fill="x", padx=16, pady=(0, 8))

        self.rpm_graph = GraphWidget(
            graph_row, t("graph_rpm"), "RPM", 0, 8000,
            color=COLORS["success"], width=280, height=90,
            max_samples=GRAPH_HISTORY_SAMPLES,
            warning_threshold=5500, danger_threshold=7000
        )
        self.rpm_graph.pack(side="left", padx=(0, 6))

        self.speed_graph = GraphWidget(
            graph_row, t("graph_speed"), "km/h", 0, 250,
            color=COLORS["accent"], width=280, height=90,
            max_samples=GRAPH_HISTORY_SAMPLES,
            warning_threshold=130, danger_threshold=180
        )
        self.speed_graph.pack(side="left", padx=(0, 6))

        self.temp_graph = GraphWidget(
            graph_row, t("graph_temp"), "°C", -40, 130,
            color=COLORS["success"], width=280, height=90,
            max_samples=GRAPH_HISTORY_SAMPLES,
            warning_threshold=100, danger_threshold=115
        )
        self.temp_graph.pack(side="left", padx=(0, 6))

        self.load_graph = GraphWidget(
            graph_row, t("graph_load"), "%", 0, 100,
            color="#8B5CF6", width=280, height=90,
            max_samples=GRAPH_HISTORY_SAMPLES,
            warning_threshold=80, danger_threshold=95
        )
        self.load_graph.pack(side="left")

        # Second row of graphs
        graph_row2 = ctk.CTkFrame(c, fg_color="transparent")
        graph_row2.pack(fill="x", padx=16, pady=(0, 8))

        self.throttle_graph = GraphWidget(
            graph_row2, t("dash_throttle"), "%", 0, 100,
            color=COLORS["accent"], width=280, height=90,
            max_samples=GRAPH_HISTORY_SAMPLES,
        )
        self.throttle_graph.pack(side="left", padx=(0, 6))

        self.fuel_graph = GraphWidget(
            graph_row2, t("dash_fuel"), "%", 0, 100,
            color="#F59E0B", width=280, height=90,
            max_samples=GRAPH_HISTORY_SAMPLES,
        )
        self.fuel_graph.pack(side="left", padx=(0, 6))

        self.voltage_graph = GraphWidget(
            graph_row2, t("dash_voltage"), "V", 10, 16,
            color="#10B981", width=280, height=90,
            max_samples=GRAPH_HISTORY_SAMPLES,
            warning_threshold=11.5, danger_threshold=11.0
        )
        self.voltage_graph.pack(side="left", padx=(0, 6))

        self.intake_graph = GraphWidget(
            graph_row2, t("dash_intake"), "°C", -40, 80,
            color="#6366F1", width=280, height=90,
            max_samples=GRAPH_HISTORY_SAMPLES,
        )
        self.intake_graph.pack(side="left")

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

        # Vehicle info labels — hidden until data is available
        self._vehicle_info_frame = vehicle_info_frame
        self.vin_label = ctk.CTkLabel(
            vehicle_info_frame, text="", font=FONTS["mono_small"],
            text_color=COLORS["text_muted"]
        )
        self.protocol_label = ctk.CTkLabel(
            vehicle_info_frame, text="", font=FONTS["small"],
            text_color=COLORS["text_muted"]
        )
        self.mil_label = ctk.CTkLabel(
            vehicle_info_frame, text="", font=FONTS["small"],
            text_color=COLORS["text_muted"]
        )
        self.fuel_type_label = ctk.CTkLabel(vehicle_info_frame, text="",
            font=FONTS["small"], text_color=COLORS["text_muted"])
        self.obd_standard_label = ctk.CTkLabel(vehicle_info_frame, text="",
            font=FONTS["small"], text_color=COLORS["text_muted"])

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

        from utils.dev_console import log_user_action
        log_user_action("Dashboard", "Start monitoring")
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

        # Reset all gauges and graphs
        self.rpm_gauge.reset()
        self.speed_gauge.reset()
        self.temp_gauge.reset()
        self.load_gauge.reset()
        self.rpm_graph.reset()
        self.speed_graph.reset()
        self.temp_graph.reset()
        self.load_graph.reset()
        self.throttle_graph.reset()
        self.fuel_graph.reset()
        self.voltage_graph.reset()
        self.intake_graph.reset()

        # Stop CSV if recording
        if self._csv_recording:
            self._stop_csv()

        # Reset all cards
        self.throttle_card.set_value("--")
        self.intake_temp_card.set_value("--")
        self.maf_card.set_value("--")
        self.fuel_level_card.set_value("--")
        self.voltage_card.set_value("--")
        self.timing_card.set_value("--")

        # Hide vehicle info labels (they'll re-show when data arrives)
        for lbl in [self.vin_label, self.protocol_label, self.mil_label,
                     self.fuel_type_label, self.obd_standard_label]:
            lbl.pack_forget()

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
                self.after(0, lambda v=rpm_val: (self.rpm_gauge.set_value(v), self.rpm_graph.add_value(v)))

            speed_val, _ = self.app.obd_reader.read_pid(0x0D)
            if speed_val is not None:
                self.after(0, lambda v=speed_val: (self.speed_gauge.set_value(v), self.speed_graph.add_value(v)))

            coolant_val, _ = self.app.obd_reader.read_pid(0x05)
            if coolant_val is not None:
                self.after(0, lambda v=coolant_val: (self.temp_gauge.set_value(v), self.temp_graph.add_value(v)))

            load_val, _ = self.app.obd_reader.read_pid(0x04)
            if load_val is not None:
                self.after(0, lambda v=load_val: (self.load_gauge.set_value(v), self.load_graph.add_value(v)))

            throttle_val, _ = self.app.obd_reader.read_pid(0x11)
            if throttle_val is not None:
                self.after(0, lambda v=throttle_val: (self.throttle_card.set_value(f"{v:.1f}"), self.throttle_graph.add_value(v)))

            intake_val, _ = self.app.obd_reader.read_pid(0x0F)
            if intake_val is not None:
                self.after(0, lambda v=intake_val: (self.intake_temp_card.set_value(f"{v:.1f}"), self.intake_graph.add_value(v)))

            maf_val, _ = self.app.obd_reader.read_pid(0x10)
            if maf_val is not None:
                self.after(0, lambda v=maf_val: self.maf_card.set_value(f"{v:.2f}"))

            fuel_val, _ = self.app.obd_reader.read_pid(0x2F)
            if fuel_val is not None:
                self.after(0, lambda v=fuel_val: (self.fuel_level_card.set_value(f"{v:.1f}"), self.fuel_graph.add_value(v)))

            voltage_val, _ = self.app.obd_reader.read_pid(0x42)
            if voltage_val is not None:
                self.after(0, lambda v=voltage_val: (self.voltage_card.set_value(f"{v:.2f}"), self.voltage_graph.add_value(v)))

            timing_val, _ = self.app.obd_reader.read_pid(0x0E)
            if timing_val is not None:
                self.after(0, lambda v=timing_val: self.timing_card.set_value(f"{v:.1f}"))

            # Log first few reads for diagnostics
            if not hasattr(self, '_dash_log_count'):
                self._dash_log_count = 0
            if self._dash_log_count < 5:
                self._dash_log_count += 1
                logger.info(f"Dashboard read #{self._dash_log_count}: RPM={rpm_val} Speed={speed_val} Coolant={coolant_val} Load={load_val} Throttle={throttle_val}")

            # CSV recording (thread-safe)
            with self._csv_lock:
                if self._csv_recording and self._csv_writer:
                    row = [datetime.now().isoformat()]
                    for val in [rpm_val, speed_val, coolant_val, load_val,
                                throttle_val, intake_val, maf_val, fuel_val,
                                voltage_val, timing_val]:
                        row.append(f"{val:.2f}" if val is not None else "")
                    self._csv_writer.writerow(row)
                    self._csv_file.flush()

        except Exception as e:
            logger.debug(f"Dashboard update error: {e}")

    def _toggle_csv_recording(self):
        """Start or stop CSV recording."""
        if self._csv_recording:
            self._stop_csv()
        else:
            self._start_csv()

    def _start_csv(self):
        """Start recording to CSV file."""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filepath = CSV_DIR / f"recording_{timestamp}.csv"
        self._csv_file = open(filepath, "w", newline="", encoding="utf-8")
        self._csv_writer = csv.writer(self._csv_file)
        self._csv_writer.writerow([
            "timestamp", "rpm", "speed_kmh", "coolant_c", "load_pct",
            "throttle_pct", "intake_c", "maf_gs", "fuel_pct",
            "voltage_v", "timing_deg"
        ])
        self._csv_recording = True
        self._csv_filepath = filepath
        self.csv_btn.configure(text=t("csv_stop"), fg_color=COLORS["danger"])
        self.csv_status.configure(text=t("csv_recording"), text_color=COLORS["danger"])

    def _stop_csv(self):
        """Stop CSV recording and close file."""
        with self._csv_lock:
            self._csv_recording = False
            if self._csv_file:
                self._csv_file.flush()
                self._csv_file.close()
                self._csv_file = None
                self._csv_writer = None
        self.csv_btn.configure(text=t("csv_record"), fg_color=COLORS["accent"])
        filename = self._csv_filepath.name if hasattr(self, '_csv_filepath') else ""
        self.csv_status.configure(text=t("csv_saved", file=filename), text_color=COLORS["success"])

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
            except Exception as e:
                logger.debug(f"ELM voltage read error: {e}")

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

            # Monitor readiness + MIL
            status_val, _ = self.app.obd_reader.read_pid(0x01)
            if status_val is not None:
                mil_on = bool(int(status_val) & 0x80) if isinstance(status_val, (int, float)) else False
                dtc_count = int(status_val) & 0x7F if isinstance(status_val, (int, float)) else 0
                self.after(0, lambda m=mil_on, d=dtc_count: self.monitors_status_label.configure(
                    text=f"{t('dash_monitors')}: {'MIL ON' if m else 'MIL OFF'} | {d} DTC(s)"))

                # Update MIL indicator label
                if mil_on:
                    mil_text = f"{t('dash_mil')}: ⚠ ON — {dtc_count} DTC(s)"
                    mil_color = COLORS["danger"]
                else:
                    mil_text = f"{t('dash_mil')}: ✓ OFF"
                    mil_color = COLORS.get("success", "#2ecc71")
                def _show_mil(txt=mil_text, col=mil_color):
                    self.mil_label.configure(text=txt, text_color=col)
                    self.mil_label.pack(anchor="w", padx=12, pady=(1, 8))
                self.after(0, _show_mil)

            def update_ui():
                # Only show labels that have actual data
                if vin_val:
                    self.vin_label.configure(text=f"VIN: {vin_val}")
                    self.vin_label.pack(anchor="w", padx=12, pady=1)
                if protocol:
                    self.protocol_label.configure(text=f"{t('dash_obd_protocol')}: {protocol}")
                    self.protocol_label.pack(anchor="w", padx=12, pady=1)
                if fuel_type_val is not None:
                    fuel_name = FUEL_TYPES.get(int(fuel_type_val), "Unknown")
                    if fuel_name != "Unknown":
                        self.fuel_type_label.configure(text=f"{t('dash_fuel_type')}: {fuel_name}")
                        self.fuel_type_label.pack(anchor="w", padx=12, pady=1)
                if obd_std_val is not None:
                    obd_name = OBD_STANDARDS.get(int(obd_std_val), "Unknown")
                    if obd_name != "Unknown":
                        self.obd_standard_label.configure(text=f"{t('dash_obd_standard')}: {obd_name}")
                        self.obd_standard_label.pack(anchor="w", padx=12, pady=1)

            self.after(0, update_ui)
        except Exception as e:
            logger.debug(f"Vehicle info update error: {e}")

    def _on_lang_change(self, lang=None):
        """Rebuild UI with new language.

        Args:
            lang: Language code (unused, kept for callback signature)
        """
        if not self.winfo_exists():
            return
        was_monitoring = self.monitoring
        if was_monitoring:
            self.stop_monitoring()

        for widget in self.scroll_container.winfo_children():
            widget.destroy()
        self._build_content()

        if was_monitoring:
            self.start_monitoring()
