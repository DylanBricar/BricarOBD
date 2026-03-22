"""Monitors frame — Mode 06 test results + Mode 09 vehicle info."""

import customtkinter as ctk
import threading
from gui.theme import COLORS, FONTS, _bind_scroll_recursive
from i18n import t, on_lang_change


class MonitorsFrame(ctk.CTkFrame):
    """Displays OBD-II Mode 06 monitor test results and Mode 09 vehicle info."""

    def __init__(self, parent, app):
        super().__init__(parent, fg_color=COLORS["bg_primary"])
        self.app = app
        self.pack(fill="both", expand=True)
        on_lang_change(self._on_lang_change)
        self._setup_ui()

    def _setup_ui(self):
        # Title
        self.title_label = ctk.CTkLabel(
            self, text=t("monitors_title"), font=FONTS["h3"],
            text_color=COLORS["text_primary"]
        )
        self.title_label.pack(anchor="w", padx=16, pady=(0, 4))

        ctk.CTkLabel(self, text=t("monitors_help"), font=FONTS["small"],
                     text_color=COLORS["text_muted"]).pack(anchor="w", padx=16, pady=(0, 12))

        # Buttons
        btn_frame = ctk.CTkFrame(self, fg_color="transparent")
        btn_frame.pack(anchor="w", padx=16, pady=(0, 12), fill="x")

        self.read_btn = ctk.CTkButton(
            btn_frame, text=t("monitors_read"), width=160, height=32,
            fg_color=COLORS["accent"], font=FONTS["small"],
            command=self.read_monitors
        )
        self.read_btn.pack(side="left", padx=4)

        self.read_info_btn = ctk.CTkButton(
            btn_frame, text=t("monitors_vehicle_info"), width=160, height=32,
            font=FONTS["small"],
            command=self.read_vehicle_info
        )
        self.read_info_btn.pack(side="left", padx=4)

        self.status_label = ctk.CTkLabel(
            btn_frame, text="", font=FONTS["small"],
            text_color=COLORS["text_muted"]
        )
        self.status_label.pack(side="left", padx=12)

        # Separator
        ctk.CTkFrame(self, fg_color=COLORS["border"], height=1).pack(fill="x", padx=16)

        # Scrollable results area
        self.results_frame = ctk.CTkScrollableFrame(self, fg_color="transparent")
        self.results_frame.pack(fill="both", expand=True, padx=16, pady=8)
        self.after(500, lambda: _bind_scroll_recursive(self.results_frame))

        # Empty state
        self.empty_label = ctk.CTkLabel(
            self.results_frame, text=t("monitors_empty"),
            font=FONTS["body"], text_color=COLORS["text_muted"]
        )
        self.empty_label.pack(pady=40)

    # Mode 06 monitor test descriptions (ISO 15031-6 and SAE J1979)
    MONITOR_TESTS = {
        # Oxygen sensors
        0x01: ("O2 Sensor B1S1", "Oxygen sensor monitoring — Bank 1, Sensor 1"),
        0x02: ("O2 Sensor B1S2", "Oxygen sensor monitoring — Bank 1, Sensor 2"),
        0x03: ("O2 Sensor B2S1", "Oxygen sensor monitoring — Bank 2, Sensor 1"),
        0x04: ("O2 Sensor B2S2", "Oxygen sensor monitoring — Bank 2, Sensor 2"),
        0x05: ("O2 Sensor B1S3", "Oxygen sensor monitoring — Bank 1, Sensor 3"),
        0x06: ("O2 Sensor B1S4", "Oxygen sensor monitoring — Bank 1, Sensor 4"),
        0x07: ("O2 Sensor B2S3", "Oxygen sensor monitoring — Bank 2, Sensor 3"),
        0x08: ("O2 Sensor B2S4", "Oxygen sensor monitoring — Bank 2, Sensor 4"),
        # Catalyst
        0x09: ("Catalyst B1", "Catalyst efficiency — Bank 1"),
        0x0A: ("Catalyst B2", "Catalyst efficiency — Bank 2"),
        # Heated catalyst
        0x0B: ("Heated Cat B1", "Heated catalyst monitoring — Bank 1"),
        0x0C: ("Heated Cat B2", "Heated catalyst monitoring — Bank 2"),
        # EVAP system
        0x0D: ("EVAP System", "Evaporative emission system leak check"),
        0x0E: ("EVAP Purge", "Evaporative emission purge flow monitoring"),
        # Secondary air
        0x0F: ("Secondary Air B1", "Secondary air injection system — Bank 1"),
        0x10: ("Secondary Air B2", "Secondary air injection system — Bank 2"),
        # A/C system
        0x11: ("A/C System", "Air conditioning refrigerant monitoring"),
        # Oxygen sensor heater
        0x12: ("O2 Heater B1S1", "Oxygen sensor heater — Bank 1, Sensor 1"),
        0x13: ("O2 Heater B1S2", "Oxygen sensor heater — Bank 1, Sensor 2"),
        0x14: ("O2 Heater B2S1", "Oxygen sensor heater — Bank 2, Sensor 1"),
        0x15: ("O2 Heater B2S2", "Oxygen sensor heater — Bank 2, Sensor 2"),
        # EGR system
        0x1F: ("EGR System", "Exhaust Gas Recirculation (EGR) flow monitoring"),
        0x20: ("EGR VVT", "EGR / Variable Valve Timing monitoring"),
        # Misfire
        0x21: ("Misfire Cyl 1", "Misfire monitoring — Cylinder 1"),
        0x22: ("Misfire Cyl 2", "Misfire monitoring — Cylinder 2"),
        0x23: ("Misfire Cyl 3", "Misfire monitoring — Cylinder 3"),
        0x24: ("Misfire Cyl 4", "Misfire monitoring — Cylinder 4"),
        0x25: ("Misfire Cyl 5", "Misfire monitoring — Cylinder 5"),
        0x26: ("Misfire Cyl 6", "Misfire monitoring — Cylinder 6"),
        0x27: ("Misfire General", "General misfire monitoring"),
        # Fuel system
        0x31: ("Fuel System B1", "Fuel system monitoring — Bank 1"),
        0x32: ("Fuel System B2", "Fuel system monitoring — Bank 2"),
        0x33: ("Fuel Trim", "Fuel trim monitoring"),
        # DPF / GPF (diesel/petrol particulate filter)
        0x39: ("PM Filter B1", "Particulate filter monitoring — Bank 1"),
        0x3A: ("PM Filter B2", "Particulate filter monitoring — Bank 2"),
        # Boost pressure
        0x3B: ("Boost Pressure", "Turbocharger / supercharger boost pressure monitoring"),
        # NOx
        0x3C: ("NOx Sensor B1", "NOx sensor monitoring — Bank 1"),
        0x3D: ("NOx Sensor B2", "NOx sensor monitoring — Bank 2"),
        0x3E: ("NOx Catalyst", "NOx catalyst / SCR monitoring"),
        # Exhaust gas sensor
        0x3F: ("Exhaust Sensor B1", "Exhaust gas sensor — Bank 1"),
        0x40: ("Exhaust Sensor B2", "Exhaust gas sensor — Bank 2"),
    }

    def read_monitors(self):
        """Read Mode 06 monitor tests in background thread."""
        self.read_btn.configure(state="disabled")
        self.status_label.configure(text=t("monitors_reading"))

        def task():
            results = []
            try:
                # Try python-obd first
                import obd
                conn = getattr(self.app, 'connection', None)
                obd_conn = getattr(conn, '_obd_conn', None) if conn else None

                if obd_conn:
                    for cmd in obd.commands[6]:
                        if cmd and not cmd.name.startswith('MIDS'):
                            try:
                                response = obd_conn.query(cmd)
                                if not response.is_null():
                                    results.append({
                                        "name": cmd.name.replace("MONITOR_", "").replace("_", " ").title(),
                                        "desc": cmd.desc,
                                        "value": str(response.value),
                                        "passed": True
                                    })
                            except Exception:
                                pass
            except ImportError:
                pass

            # Fallback: read Mode 06 via raw OBD commands
            if not results and self.app.obd_reader:
                for mid, (name, desc) in self.MONITOR_TESTS.items():
                    try:
                        raw = self.app.connection.send_obd("06", f"{mid:02X}")
                        if raw and "46" in raw and "NO DATA" not in raw:
                            parts = raw.strip().split()
                            if len(parts) >= 7:
                                test_val = int(parts[3] + parts[4], 16)
                                min_val = int(parts[5], 16)
                                max_val = int(parts[6], 16)
                                passed = min_val <= test_val <= max_val if max_val > 0 else True
                                results.append({
                                    "name": name,
                                    "desc": desc,
                                    "value": f"{'PASS' if passed else 'FAIL'}  (val={test_val}, min={min_val}, max={max_val})",
                                    "passed": passed,
                                })
                    except Exception:
                        pass

                # Also read monitor readiness from PID 0x01
                try:
                    val, _ = self.app.obd_reader.read_pid(0x01)
                    if val is not None:
                        status = int(val)
                        mil_on = bool(status & 0x80)
                        dtc_count = status & 0x7F
                        results.insert(0, {
                            "name": "MIL Status",
                            "desc": "Malfunction Indicator Lamp",
                            "value": f"{'ON' if mil_on else 'OFF'} — {dtc_count} DTC(s)",
                            "passed": not mil_on,
                        })
                except Exception:
                    pass

            self.after(0, self._display_results, results, "monitors")
            self.after(0, lambda: self.read_btn.configure(state="normal"))

        threading.Thread(target=task, daemon=True).start()

    def read_vehicle_info(self):
        """Read Mode 09 vehicle info + calibration data."""
        self.read_info_btn.configure(state="disabled")
        self.status_label.configure(text=t("monitors_reading"))

        def task():
            results = []
            try:
                # VIN
                if self.app.obd_reader:
                    info = self.app.obd_reader.get_vehicle_info()
                    for key, value in info.items():
                        if value:
                            results.append({
                                "name": key.upper().replace("_", " "),
                                "desc": f"Mode 09: {key}",
                                "value": value,
                                "passed": True
                            })

                # Try python-obd for more Mode 09 data
                import obd
                conn = getattr(self.app, 'connection', None)
                obd_conn = getattr(conn, '_obd_conn', None) if conn else None

                if obd_conn:
                    for cmd in obd.commands[9]:
                        if cmd and cmd.name not in ('PIDS_9A', 'VIN_MESSAGE_COUNT',
                                                     'CALIBRATION_ID_MESSAGE_COUNT',
                                                     'CVN_MESSAGE_COUNT'):
                            try:
                                response = obd_conn.query(cmd)
                                if not response.is_null():
                                    # Don't duplicate VIN
                                    if cmd.name == 'VIN' and any(r['name'] == 'VIN' for r in results):
                                        continue
                                    results.append({
                                        "name": cmd.name.replace("_", " ").title(),
                                        "desc": cmd.desc,
                                        "value": str(response.value),
                                        "passed": True
                                    })
                            except Exception:
                                pass

                # Add detected vehicle info
                detected = getattr(self.app, 'detected_vehicle', None)
                if detected:
                    results.append({"name": t("monitors_make"), "desc": "VIN decode",
                                   "value": detected.get("make", "Unknown"), "passed": True})
                    results.append({"name": t("monitors_country"), "desc": "VIN decode",
                                   "value": detected.get("country", "Unknown"), "passed": True})
                    if detected.get("model_year"):
                        results.append({"name": t("monitors_year"), "desc": "VIN decode",
                                       "value": detected["model_year"], "passed": True})

                # Add OBD protocol info
                if self.app.connection:
                    results.append({"name": t("monitors_protocol"), "desc": "ELM327",
                                   "value": self.app.connection.protocol_name or "N/A", "passed": True})
                    results.append({"name": t("monitors_elm"), "desc": "ELM327",
                                   "value": self.app.connection.elm_version or "N/A", "passed": True})

                # Add supported PID count
                if self.app.obd_reader and hasattr(self.app.obd_reader, '_supported_pids'):
                    count = len(self.app.obd_reader._supported_pids)
                    results.append({"name": t("monitors_supported_pids"), "desc": "Mode 01",
                                   "value": str(count), "passed": True})

            except Exception as e:
                self.after(0, lambda: self.status_label.configure(
                    text=f"Error: {str(e)[:50]}"))

            self.after(0, self._display_results, results, "info")
            self.after(0, lambda: self.read_info_btn.configure(state="normal"))

        threading.Thread(target=task, daemon=True).start()

    def _display_results(self, results, result_type):
        """Display results in the scrollable frame."""
        # Clear previous
        for widget in self.results_frame.winfo_children():
            widget.destroy()

        if not results:
            ctk.CTkLabel(self.results_frame, text=t("monitors_no_results"),
                        font=FONTS["body"], text_color=COLORS["text_muted"]).pack(pady=40)
            self.status_label.configure(text=t("monitors_no_results"))
            return

        self.status_label.configure(text=f"{len(results)} {t('monitors_results_found')}")

        # Section header
        section_title = t("monitors_test_results") if result_type == "monitors" else t("monitors_vehicle_data")
        ctk.CTkLabel(self.results_frame, text=section_title,
                    font=FONTS["body_bold"], text_color=COLORS["text_secondary"]).pack(anchor="w", pady=(4, 8))

        # Results table
        for idx, result in enumerate(results):
            bg = COLORS["bg_card"] if idx % 2 == 0 else COLORS["bg_secondary"]
            row = ctk.CTkFrame(self.results_frame, fg_color=bg, corner_radius=4)
            row.pack(fill="x", pady=1)

            # Name
            ctk.CTkLabel(row, text=result["name"], font=FONTS["body_bold"],
                        text_color=COLORS["text_primary"], width=200, anchor="w"
                        ).pack(side="left", padx=12, pady=8)

            # Value
            value_color = COLORS["success"] if result.get("passed") else COLORS["danger"]
            ctk.CTkLabel(row, text=result["value"][:60], font=FONTS["mono"],
                        text_color=value_color, width=300, anchor="w"
                        ).pack(side="left", padx=8, pady=8)

            # Description
            ctk.CTkLabel(row, text=result["desc"][:40], font=FONTS["small"],
                        text_color=COLORS["text_muted"], anchor="w"
                        ).pack(side="left", padx=8, pady=8)

    def _on_lang_change(self, lang=None):
        """Rebuild UI on language change."""
        for widget in self.winfo_children():
            widget.destroy()
        self._setup_ui()
