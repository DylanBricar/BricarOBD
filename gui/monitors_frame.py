"""Monitors frame — Mode 06 test results + Mode 09 vehicle info."""

import customtkinter as ctk
import threading
from gui.theme import COLORS, FONTS
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
        self.title_label.pack(anchor="w", padx=16, pady=(12, 2))

        ctk.CTkLabel(self, text=t("monitors_help"), font=FONTS["small"],
                     text_color=COLORS["text_muted"]).pack(anchor="w", padx=16, pady=(0, 8))

        # Buttons
        btn_frame = ctk.CTkFrame(self, fg_color="transparent")
        btn_frame.pack(anchor="w", padx=16, pady=(0, 8), fill="x")

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

        # Empty state
        self.empty_label = ctk.CTkLabel(
            self.results_frame, text=t("monitors_empty"),
            font=FONTS["body"], text_color=COLORS["text_muted"]
        )
        self.empty_label.pack(pady=40)

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
                    # Query all Mode 06 commands
                    for cmd in obd.commands[6]:
                        if cmd and not cmd.name.startswith('MIDS'):
                            try:
                                response = obd_conn.query(cmd)
                                if not response.is_null():
                                    results.append({
                                        "name": cmd.name.replace("MONITOR_", "").replace("_", " ").title(),
                                        "desc": cmd.desc,
                                        "value": str(response.value),
                                        "passed": True  # If we got a response, test exists
                                    })
                            except Exception:
                                pass
                else:
                    # Fallback: read monitor status via Mode 01 PID 01
                    if self.app.obd_reader:
                        val, _ = self.app.obd_reader.read_pid(0x01)
                        if val is not None:
                            results.append({
                                "name": "Monitor Status",
                                "desc": "Status since DTCs cleared",
                                "value": str(int(val)),
                                "passed": True
                            })
            except Exception as e:
                self.after(0, lambda: self.status_label.configure(
                    text=f"Error: {str(e)[:50]}"))

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
        self.title_label.configure(text=t("monitors_title"))
        self.read_btn.configure(text=t("monitors_read"))
        self.read_info_btn.configure(text=t("monitors_vehicle_info"))
