"""DTC (Diagnostic Trouble Code) management frame."""

import customtkinter as ctk
import webbrowser
import threading
from tkinter import filedialog
from pathlib import Path
from gui.theme import COLORS, FONTS, _bind_scroll_recursive
from config import DTC_SEARCH_URL
from i18n import t, get_lang, on_lang_change


class DTCFrame(ctk.CTkFrame):
    """Frame for managing and viewing Diagnostic Trouble Codes."""

    def __init__(self, parent, app):
        """Initialize DTC frame.

        Args:
            parent: Parent widget
            app: Main application instance with dtc_manager
        """
        super().__init__(parent, fg_color=COLORS["bg_primary"])
        self.app = app
        self.dtc_manager = app.dtc_manager
        self.dtc_records = []
        self.dtc_rows = {}
        self.title_label = None

        self._setup_ui()
        on_lang_change(self._on_lang_change)

    def _setup_ui(self):
        """Setup the DTC frame UI."""
        self.title_label = ctk.CTkLabel(
            self, text=t("dtc_title"), font=FONTS["h3"],
            text_color=COLORS["text_primary"]
        )
        self.title_label.pack(anchor="w", padx=16, pady=(0, 4))

        ctk.CTkLabel(self, text=t("dtc_help"), font=FONTS["small"], text_color=COLORS["text_muted"]).pack(anchor="w", padx=16, pady=(0, 12))

        button_bar = ctk.CTkFrame(self, fg_color="transparent")
        button_bar.pack(anchor="w", padx=16, pady=(0, 12), fill="x")

        ctk.CTkButton(
            button_bar, text=t("dtc_read_all"), fg_color=COLORS["accent"],
            width=160, height=32, font=FONTS["body_bold"],
            command=self.read_all_dtcs
        ).pack(side="left", padx=4)

        separator = ctk.CTkFrame(button_bar, fg_color=COLORS["border"], width=1, height=28)
        separator.pack(side="left", padx=10)

        ctk.CTkButton(
            button_bar, text=t("dtc_clear_all"), fg_color=COLORS["danger"],
            width=140, height=32, font=FONTS["body_bold"],
            command=self.clear_dtcs
        ).pack(side="left", padx=2)

        table_header_frame = ctk.CTkFrame(self, fg_color="transparent")
        table_header_frame.pack(anchor="w", padx=16, pady=(4, 2), fill="x")

        header_texts = [t("dtc_code"), t("dtc_desc"), t("dtc_status"), t("dtc_source"), t("dtc_actions")]
        header_widths = [80, 250, 100, 80, 150]

        for text, width in zip(header_texts, header_widths):
            label = ctk.CTkLabel(
                table_header_frame, text=text, font=FONTS["body_bold"],
                text_color=COLORS["text_secondary"], width=width, anchor="w"
            )
            label.pack(side="left", padx=4)

        separator_line = ctk.CTkFrame(self, fg_color=COLORS["border"], height=1)
        separator_line.pack(anchor="w", fill="x", padx=16)

        self.table_frame = ctk.CTkScrollableFrame(self, fg_color="transparent")
        self.table_frame.pack(fill="both", expand=True, padx=16, pady=(4, 4))
        self.after(500, lambda: _bind_scroll_recursive(self.table_frame))

        self.empty_state = ctk.CTkLabel(
            self.table_frame, text=t("dtc_no_codes"),
            font=FONTS["body"], text_color=COLORS["text_muted"]
        )
        self.empty_state.pack(anchor="center", pady=40)

        bottom_bar = ctk.CTkFrame(self, fg_color="transparent")
        bottom_bar.pack(anchor="w", padx=16, pady=(4, 4), fill="x")

        self.count_label = ctk.CTkLabel(
            bottom_bar, text=t("dtc_found_zero"), font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        )
        self.count_label.pack(side="left", padx=4)

        ctk.CTkButton(
            bottom_bar, text=t("dtc_save_all"), width=100,
            command=self.save_all
        ).pack(side="left", padx=4)

        ctk.CTkButton(
            bottom_bar, text=t("dtc_export_json"), width=100,
            command=self.export_json
        ).pack(side="left", padx=4)

        ctk.CTkButton(
            bottom_bar, text=t("dtc_import"), width=100,
            command=self.import_session
        ).pack(side="left", padx=4)

    def _check_connected(self) -> bool:
        """Check connection before DTC operations."""
        if not self.app.connection or not self.app.connection.is_connected():
            return False
        return True

    def read_all_dtcs(self):
        """Read all DTCs in a background thread."""
        if not self._check_connected():
            return
        from utils.dev_console import log_user_action
        log_user_action("DTC", "Read All DTCs")
        def task():
            try:
                make = getattr(self.app, 'detected_make', '') or ''

                # Step 1: Read all standard DTCs (Mode 03 + 07 + 0A + UDS)
                dtcs = self.app.dtc_manager.read_all_dtcs(make=make)
                log_user_action("DTC", f"Standard DTCs: {len(dtcs)}")

                # Step 2: Also read ECU fault history (S2000 trames pannes)
                ecu_faults = self._read_ecu_faults_sync()
                if ecu_faults:
                    log_user_action("DTC", f"ECU faults: {len(ecu_faults)}")
                    dtcs = list(dtcs) + ecu_faults

                log_user_action("DTC", f"Total: {len(dtcs)} DTCs: {[d.code for d in dtcs]}")
                self.after(0, self.populate_table, dtcs)
            except Exception as e:
                self.after(0, self._show_error, str(e))

        threading.Thread(target=task, daemon=True).start()

    def read_pending(self):
        """Read pending DTCs in a background thread."""
        if not self._check_connected():
            return
        from utils.dev_console import log_user_action
        log_user_action("DTC", "Read Pending DTCs")
        def task():
            try:
                dtcs = self.app.dtc_manager.read_pending_dtcs()
                log_user_action("DTC", f"Found {len(dtcs)} pending DTCs")
                self.after(0, self.populate_table, dtcs)
            except Exception as e:
                self.after(0, self._show_error, str(e))

        threading.Thread(target=task, daemon=True).start()

    def read_permanent(self):
        """Read permanent DTCs in a background thread."""
        if not self._check_connected():
            return
        from utils.dev_console import log_user_action
        log_user_action("DTC", "Read Permanent DTCs")
        def task():
            try:
                dtcs = self.app.dtc_manager.read_permanent_dtcs()
                log_user_action("DTC", f"Found {len(dtcs)} permanent DTCs")
                self.after(0, self.populate_table, dtcs)
            except Exception as e:
                self.after(0, self._show_error, str(e))

        threading.Thread(target=task, daemon=True).start()

    def _read_ecu_faults_sync(self):
        """Read ECU fault history synchronously (called from read_all_dtcs thread)."""
        try:
            from obd_core.dtc_manager import DTCRecord
            from obd_core.ecu_identifier import KNOWN_ADDRESSES, MAKE_ADDRESSES

            make = getattr(self.app, 'detected_make', '') or ''
            ecu_database = getattr(self.app, 'ecu_database', None)
            if not ecu_database or not ecu_database.is_loaded:
                return []
            if not self.app.connection or not self.app.connection.is_connected():
                return []

            matched_files = None
            for frame in self.app.frames.values():
                if hasattr(frame, '_matched_ecu_files'):
                    matched_files = frame._matched_ecu_files
                    break

            allowed_addrs = MAKE_ADDRESSES.get(make.lower(), ["7A"]) if make else ["7A"]

            def _read_faults(conn):
                    conn.send_command("ATE0")
                    results = []

                    for addr in allowed_addrs:
                        addr_info = KNOWN_ADDRESSES.get(addr, {})
                        can_tx = addr_info.get("can_tx", 0x7E0)
                        can_rx = addr_info.get("can_rx", 0x7E8)

                        # Load ECU definition
                        if matched_files and addr in matched_files:
                            ecu_def = ecu_database.load_ecu_definition(matched_files[addr])
                        else:
                            ecus = ecu_database.find_ecus_by_address(addr)
                            ecu_def = ecu_database.load_ecu_definition(ecus[0].get("filename", "")) if ecus else None

                        if not ecu_def:
                            continue

                        conn.send_command(f"AT SH {can_tx:03X}")
                        conn.send_command(f"AT CRA {can_rx:03X}")

                        # Find fault-reading requests (Trame pannes)
                        for req in ecu_def.requests:
                            cmd = req.sentbytes.upper()
                            name_lower = req.name.lower()
                            # Only read fault trames (21Ax = pannes, 1200 = contexte panne)
                            if not (cmd.startswith("21A") and ("panne" in name_lower or "diag" in name_lower)):
                                continue

                            response = conn.send_command(req.sentbytes, timeout=3)
                            if not response or "NO DATA" in response:
                                continue

                            # Parse response to bytes
                            clean = response.replace(" ", "").replace("\r", "").replace("\n", "").replace(">", "")
                            hex_only = "".join(c for c in clean if c in "0123456789ABCDEFabcdef")
                            if not hex_only:
                                continue
                            try:
                                resp_bytes = bytes.fromhex(hex_only)
                            except ValueError:
                                continue

                            # Decode fault parameters
                            decoded = ecu_def.decode_response(req, resp_bytes)
                            for param_name, value in decoded.items():
                                if not isinstance(value, bool):
                                    continue
                                if not value:
                                    continue  # Only show active faults
                                # This is an active fault flag
                                is_memorized = "mémorisée" in param_name.lower() or "memorisee" in param_name.lower()
                                is_present = "présente" in param_name.lower() or "presente" in param_name.lower()

                                status = "Mémorisée" if is_memorized else ("Présente" if is_present else "Active")
                                results.append(DTCRecord(
                                    code=f"ECU:{addr}",
                                    description=param_name,
                                    status=status,
                                    status_byte=1,
                                    source=ecu_def.ecuname[:20],
                                ))

                    conn.send_command("AT D")
                    conn.send_command("AT CRA")
                    conn.send_command("AT H0")
                    return results

            return self.app.connection.use_custom_connection(_read_faults) or []
        except Exception:
            return []

    def _show_error(self, msg):
        """Show error in count label."""
        self.count_label.configure(text=f"Error: {msg[:60]}", text_color=COLORS.get("danger", "#e74c3c"))

    def clear_dtcs(self):
        """Clear all DTCs with confirmation."""
        from gui.dialogs import SafetyConfirmDialog

        dialog = SafetyConfirmDialog(
            self,
            t("dtc_clear_all"),
            t("dtc_confirm_clear")
        )
        if dialog.wait_for_result():
            def task():
                success, msg = self.app.dtc_manager.clear_all_dtcs(confirmed=True)
                if hasattr(self, 'log_message'):
                    self.after(0, self.log_message, msg)
                if success:
                    self.after(0, self.read_all_dtcs)
            threading.Thread(target=task, daemon=True).start()

    def populate_table(self, dtc_records):
        """Populate table with DTC records.

        Args:
            dtc_records: List of DTCRecord objects
        """
        for widget in self.table_frame.winfo_children():
            widget.destroy()

        self.dtc_records = dtc_records
        self.dtc_rows = {}
        self.count_label.configure(text=t("dtc_found", count=len(dtc_records)))

        if not dtc_records:
            self.empty_state = ctk.CTkLabel(
                self.table_frame, text=t("dtc_no_codes"),
                font=FONTS["body"], text_color=COLORS["text_muted"]
            )
            self.empty_state.pack(anchor="center", pady=40)
            return

        for idx, dtc_record in enumerate(dtc_records):
            self._create_dtc_row(self.table_frame, dtc_record, idx)

    def _create_dtc_row(self, parent, dtc_record, row_idx):
        """Create a single DTC row in the table.

        Args:
            parent: Parent frame
            dtc_record: DTCRecord object
            row_idx: Row index for alternating colors
        """
        bg_color = COLORS["bg_card"] if row_idx % 2 == 0 else COLORS["bg_secondary"]
        row_frame = ctk.CTkFrame(parent, fg_color=bg_color, corner_radius=4)
        row_frame.pack(pady=2, fill="x")

        code_label = ctk.CTkLabel(
            row_frame, text=dtc_record.code, font=FONTS["mono"],
            text_color=COLORS["highlight"], width=80, anchor="w"
        )
        code_label.pack(side="left", padx=8, pady=8)

        # Use FR description if available and language is French
        desc_text = dtc_record.description
        try:
            if get_lang() == 'fr':
                from data.dtc_descriptions import get_dtc_description_fr
                fr = get_dtc_description_fr(dtc_record.code)
                if fr:
                    desc_text = fr
        except Exception:
            pass
        desc_label = ctk.CTkLabel(
            row_frame, text=desc_text[:50], font=FONTS["body"],
            text_color=COLORS["text_primary"], width=250, anchor="w"
        )
        desc_label.pack(side="left", padx=8)

        status_color = COLORS["danger"]
        if dtc_record.status == "Pending":
            status_color = COLORS["warning"]
        elif dtc_record.status == "Confirmed":
            status_color = COLORS["warning"]

        status_label = ctk.CTkLabel(
            row_frame, text=dtc_record.status, font=FONTS["small"],
            text_color=status_color, width=100, anchor="w"
        )
        status_label.pack(side="left", padx=8)

        source_label = ctk.CTkLabel(
            row_frame, text=dtc_record.source, font=FONTS["small"],
            text_color=COLORS["text_secondary"], width=80, anchor="w"
        )
        source_label.pack(side="left", padx=8)

        action_frame = ctk.CTkFrame(row_frame, fg_color="transparent", width=150)
        action_frame.pack(side="left", padx=4)

        ctk.CTkButton(
            action_frame, text=t("dtc_search"), width=60, font=FONTS["small"],
            command=lambda: self.search_web(dtc_record.code)
        ).pack(side="left", padx=2)

        ctk.CTkButton(
            action_frame, text=t("dtc_save"), width=60, font=FONTS["small"],
            command=lambda: self.save_single(dtc_record)
        ).pack(side="left", padx=2)

        ctk.CTkButton(
            action_frame, text="Freeze", width=60, height=24,
            font=FONTS["small"], fg_color=COLORS["accent"],
            command=lambda c=dtc_record.code: self.show_freeze_frame(c)
        ).pack(side="left", padx=2)

        ctk.CTkButton(
            action_frame, text="?", width=28, height=24,
            font=FONTS["small_bold"], fg_color=COLORS["warning"],
            hover_color="#D97706",
            command=lambda c=dtc_record.code: self._show_help_panel(c)
        ).pack(side="left", padx=2)

        ctk.CTkButton(
            action_frame, text=t("dtc_details") if hasattr(t, '__call__') else "Details", width=60, font=FONTS["small"],
            command=lambda d=dtc_record: self._show_dtc_details(d)
        ).pack(side="left", padx=2)

        self.dtc_rows[dtc_record.code] = row_frame

    def search_web(self, dtc_code):
        """Open web browser to search for DTC code with detected vehicle info.

        Args:
            dtc_code: DTC code string (e.g., "P0420")
        """
        # Build vehicle string from detected vehicle
        make = getattr(self.app, 'detected_make', '') or ''
        vehicle_info = getattr(self.app, 'detected_vehicle', None)
        model = vehicle_info.get('model', '') if vehicle_info else ''
        vehicle = f"{make} {model}".strip() or "vehicle"

        # Use localized search term
        search_term = "diagnostic véhicule" if get_lang() == "fr" else "vehicle diagnostic"

        from urllib.parse import quote
        url = DTC_SEARCH_URL.format(code=quote(dtc_code), vehicle=quote(f"{vehicle} {search_term}"))
        webbrowser.open(url)

    def save_single(self, dtc_record):
        """Save a single DTC record.

        Args:
            dtc_record: DTCRecord to save
        """
        self.app.dtc_manager.save_single_dtc(dtc_record)

    def save_all(self):
        """Save all displayed DTCs."""
        self.app.dtc_manager.save_dtcs(self.dtc_records)

    def export_txt(self):
        """Export DTCs to text file."""
        filepath = filedialog.asksaveasfilename(
            defaultextension=".txt",
            filetypes=[("Text files", "*.txt"), ("All files", "*.*")]
        )
        if filepath:
            self.app.dtc_manager.export_to_text(self.dtc_records, Path(filepath))

    def export_json(self):
        """Export DTCs to JSON file."""
        filepath = filedialog.asksaveasfilename(
            defaultextension=".json",
            filetypes=[("JSON files", "*.json"), ("All files", "*.*")]
        )
        if filepath:
            self.app.dtc_manager.export_to_json(self.dtc_records, Path(filepath))

    def import_session(self):
        """Import DTCs from file."""
        filepath = filedialog.askopenfilename(
            filetypes=[("JSON files", "*.json"), ("All files", "*.*")],
            title=t("dtc_import")
        )
        if filepath:
            dtcs = self.app.dtc_manager.load_dtcs(Path(filepath))
            self.populate_table(dtcs)

    def show_freeze_frame(self, dtc_code):
        """Read and display freeze frame data for a DTC."""
        def task():
            freeze_data = {}
            pids_to_read = [0x04, 0x05, 0x0C, 0x0D, 0x0F, 0x11, 0x42, 0x0B, 0x0E]
            pid_names = {
                0x04: t("dash_load"), 0x05: t("dash_coolant"), 0x0C: t("dash_rpm"),
                0x0D: t("dash_speed"), 0x0F: t("dash_intake"), 0x11: t("dash_throttle"),
                0x42: t("dash_voltage"), 0x0B: "MAP (kPa)", 0x0E: t("dash_timing"),
            }
            if self.app.obd_reader:
                for pid in pids_to_read:
                    val, unit = self.app.obd_reader.read_freeze_frame(pid)
                    if val is not None:
                        name = pid_names.get(pid, f"PID 0x{pid:02X}")
                        freeze_data[name] = f"{val:.1f} {unit}"

            self.after(0, self._display_freeze_frame, dtc_code, freeze_data)

        threading.Thread(target=task, daemon=True).start()

    def _display_freeze_frame(self, dtc_code, freeze_data):
        """Display freeze frame data in a popup."""
        popup = ctk.CTkToplevel(self)
        popup.title(f"Freeze Frame — {dtc_code}")
        popup.configure(fg_color=COLORS["bg_primary"])

        w, h = 450, 350
        x = (popup.winfo_screenwidth() - w) // 2
        y = (popup.winfo_screenheight() - h) // 2
        popup.geometry(f"{w}x{h}+{x}+{y}")
        popup.resizable(False, False)
        popup.transient(self)

        ctk.CTkLabel(popup, text=f"Freeze Frame — {dtc_code}", font=FONTS["h3"],
                     text_color=COLORS["text_primary"]).pack(pady=(16, 4))
        ctk.CTkLabel(popup, text=t("freeze_help"), font=FONTS["small"],
                     text_color=COLORS["text_muted"]).pack(pady=(0, 12))

        scroll = ctk.CTkScrollableFrame(popup, fg_color="transparent")
        scroll.pack(fill="both", expand=True, padx=16, pady=(0, 8))

        if not freeze_data:
            ctk.CTkLabel(scroll, text=t("freeze_no_data"), font=FONTS["body"],
                         text_color=COLORS["text_muted"]).pack(pady=20)
        else:
            for idx, (name, value) in enumerate(freeze_data.items()):
                bg = COLORS["bg_card"] if idx % 2 == 0 else COLORS["bg_secondary"]
                row = ctk.CTkFrame(scroll, fg_color=bg, corner_radius=4)
                row.pack(fill="x", pady=1)
                ctk.CTkLabel(row, text=name, font=FONTS["body"],
                            text_color=COLORS["text_secondary"], width=180, anchor="w"
                            ).pack(side="left", padx=12, pady=6)
                ctk.CTkLabel(row, text=value, font=FONTS["mono"],
                            text_color=COLORS["success"], anchor="w"
                            ).pack(side="left", padx=8, pady=6)

        ctk.CTkButton(popup, text=t("dialog_close"), width=100,
                      command=popup.destroy).pack(pady=(4, 12))

    def _show_help_panel(self, dtc_code):
        """Show repair help popup for a DTC code."""
        from data.dtc_repair_tips import get_repair_tips, get_forum_url
        from data.dtc_descriptions import get_dtc_description

        lang = get_lang()
        tips = get_repair_tips(dtc_code)
        make = getattr(self.app, 'detected_make', '') or ''
        vehicle_info = getattr(self.app, 'detected_vehicle', None)
        model = vehicle_info.get('model', '') if vehicle_info else ''
        vehicle = f"{make} {model}".strip() or "vehicle"
        desc = get_dtc_description(dtc_code)

        popup = ctk.CTkToplevel(self)
        popup.title(f"{t('dtc_help_title')} — {dtc_code}")
        popup.configure(fg_color=COLORS["bg_primary"])
        w, h = 550, 520
        x = (popup.winfo_screenwidth() - w) // 2
        y = (popup.winfo_screenheight() - h) // 2
        popup.geometry(f"{w}x{h}+{x}+{y}")
        popup.resizable(False, False)
        popup.transient(self)

        # Header
        ctk.CTkLabel(popup, text=f"{dtc_code} — {desc[:60]}", font=FONTS["h3"],
                     text_color=COLORS["text_primary"]).pack(pady=(16, 4), padx=16)
        if make:
            ctk.CTkLabel(popup, text=vehicle, font=FONTS["small"],
                         text_color=COLORS["text_muted"]).pack(pady=(0, 8))

        scroll = ctk.CTkScrollableFrame(popup, fg_color="transparent")
        scroll.pack(fill="both", expand=True, padx=16, pady=(0, 8))

        if tips:
            # Causes
            ctk.CTkLabel(scroll, text=t("dtc_help_causes"), font=FONTS["body_bold"],
                         text_color=COLORS["warning"]).pack(anchor="w", pady=(8, 4))
            causes = tips["causes"].get(lang, tips["causes"].get("en", []))
            for cause in causes:
                ctk.CTkLabel(scroll, text=f"  • {cause}", font=FONTS["body"],
                             text_color=COLORS["text_primary"]).pack(anchor="w", pady=1)

            # Quick check
            ctk.CTkLabel(scroll, text=t("dtc_help_check"), font=FONTS["body_bold"],
                         text_color=COLORS["success"]).pack(anchor="w", pady=(12, 4))
            check_text = tips["quick_check"].get(lang, tips["quick_check"].get("en", ""))
            ctk.CTkLabel(scroll, text=check_text, font=FONTS["body"],
                         text_color=COLORS["text_primary"], wraplength=480,
                         justify="left").pack(anchor="w", pady=(0, 4))

            # Difficulty
            diff = tips.get("difficulty", 2)
            diff_labels = {1: t("dtc_help_diy_easy"), 2: t("dtc_help_diy_medium"), 3: t("dtc_help_mechanic")}
            diff_colors = {1: COLORS["success"], 2: COLORS["warning"], 3: COLORS["danger"]}
            ctk.CTkLabel(scroll, text=f"{t('dtc_help_difficulty')}: {diff_labels.get(diff, '?')}",
                         font=FONTS["body_bold"], text_color=diff_colors.get(diff, COLORS["text_muted"])
                         ).pack(anchor="w", pady=(8, 4))
        else:
            ctk.CTkLabel(scroll, text=t("dtc_help_no_tips"), font=FONTS["body"],
                         text_color=COLORS["text_muted"]).pack(anchor="w", pady=(8, 4))

        # Links section
        ctk.CTkFrame(scroll, fg_color=COLORS["border"], height=1).pack(fill="x", pady=(12, 8))
        ctk.CTkLabel(scroll, text=t("dtc_help_links"), font=FONTS["body_bold"],
                     text_color=COLORS["accent"]).pack(anchor="w", pady=(0, 8))

        repair_word = "réparation" if lang == "fr" else "repair"

        links = [
            (t("dtc_help_google"), f"https://www.google.com/search?q={dtc_code}+{vehicle}+{repair_word}"),
            (t("dtc_help_youtube"), f"https://www.youtube.com/results?search_query={dtc_code}+{vehicle}+{repair_word}"),
            (t("dtc_help_obd_codes"), f"https://www.obd-codes.com/{dtc_code.lower()}"),
        ]

        forum_url = get_forum_url(make)
        if forum_url:
            links.append((t("dtc_help_forum", make=make), f"{forum_url}search?q={dtc_code}"))

        for label, url in links:
            btn = ctk.CTkButton(
                scroll, text=f"  {label}", font=FONTS["body"],
                fg_color=COLORS["bg_card"], hover_color=COLORS["bg_card_hover"],
                text_color=COLORS["accent"], anchor="w", height=32,
                command=lambda u=url: webbrowser.open(u),
            )
            btn.pack(fill="x", pady=2)

        ctk.CTkButton(popup, text=t("dialog_close"), width=100,
                      command=popup.destroy).pack(pady=(4, 12))

    def _show_dtc_details(self, dtc_record):
        """Show detailed DTC info including freeze frame data."""
        popup = ctk.CTkToplevel(self)
        popup.title(f"DTC {dtc_record.code}")
        popup.geometry("500x400")
        popup.transient(self.winfo_toplevel())

        # Header
        ctk.CTkLabel(popup, text=f"DTC {dtc_record.code}", font=FONTS["h3"],
                     text_color=COLORS["highlight"]).pack(padx=16, pady=(16, 4))

        # Description (EN + FR)
        ctk.CTkLabel(popup, text=dtc_record.description, font=FONTS["body"],
                     text_color=COLORS["text_primary"], wraplength=460).pack(padx=16, pady=4)

        # French description
        try:
            from data.dtc_descriptions import get_dtc_description_fr
            fr_desc = get_dtc_description_fr(dtc_record.code)
            if fr_desc:
                ctk.CTkLabel(popup, text=f"FR: {fr_desc}", font=FONTS["body"],
                             text_color=COLORS["text_secondary"], wraplength=460).pack(padx=16, pady=4)
        except ImportError:
            pass

        # Status info
        info_frame = ctk.CTkFrame(popup, fg_color=COLORS["bg_card"], corner_radius=8)
        info_frame.pack(fill="x", padx=16, pady=8)

        ctk.CTkLabel(info_frame, text=f"Status: {dtc_record.status}", font=FONTS["small"],
                     text_color=COLORS["text_secondary"]).pack(anchor="w", padx=12, pady=4)
        ctk.CTkLabel(info_frame, text=f"Source: {dtc_record.source}", font=FONTS["small"],
                     text_color=COLORS["text_secondary"]).pack(anchor="w", padx=12, pady=4)
        if dtc_record.status_byte:
            ctk.CTkLabel(info_frame, text=f"Status byte: 0x{dtc_record.status_byte:02X}", font=FONTS["mono_small"],
                         text_color=COLORS["text_muted"]).pack(anchor="w", padx=12, pady=4)

        # Freeze frame data (read from vehicle if connected)
        ff_frame = ctk.CTkFrame(popup, fg_color=COLORS["bg_card"], corner_radius=8)
        ff_frame.pack(fill="x", padx=16, pady=8)
        ctk.CTkLabel(ff_frame, text=t("dtc_freeze_frame"), font=FONTS["small_bold"],
                     text_color=COLORS["text_secondary"]).pack(anchor="w", padx=12, pady=(8, 4))

        if self.app.obd_reader and self._check_connected():
            # Read freeze frame PIDs
            freeze_pids = [
                (0x0C, "RPM", "tr/min"),
                (0x0D, "Speed", "km/h"),
                (0x05, "Coolant", "°C"),
                (0x04, "Load", "%"),
                (0x11, "Throttle", "%"),
                (0x0F, "Intake Temp", "°C"),
            ]
            def _read_freeze():
                results = []
                for pid, name, unit in freeze_pids:
                    val, _ = self.app.obd_reader.read_freeze_frame(pid) if hasattr(self.app.obd_reader, 'read_freeze_frame') else (None, "")
                    if val is not None:
                        results.append(f"  {name}: {val:.1f} {unit}")
                if results:
                    self.after(0, lambda r=results: self._show_freeze_data(ff_frame, r))
                else:
                    self.after(0, lambda: ctk.CTkLabel(ff_frame, text="  No freeze frame data available",
                                                       font=FONTS["small"], text_color=COLORS["text_muted"]).pack(anchor="w", padx=12, pady=4))
            threading.Thread(target=_read_freeze, daemon=True).start()
        else:
            ctk.CTkLabel(ff_frame, text="  Connect to vehicle to read freeze frame",
                         font=FONTS["small"], text_color=COLORS["text_muted"]).pack(anchor="w", padx=12, pady=4)

        # Close button
        ctk.CTkButton(popup, text="Close", width=100, command=popup.destroy).pack(pady=12)

    def _show_freeze_data(self, parent, results):
        """Display freeze frame results."""
        for line in results:
            ctk.CTkLabel(parent, text=line, font=FONTS["mono_small"],
                         text_color=COLORS["text_primary"]).pack(anchor="w", padx=12, pady=2)

    def _on_lang_change(self, lang=None):
        """Rebuild UI on language change."""
        if not self.winfo_exists():
            return
        for widget in self.winfo_children():
            widget.destroy()
        self._setup_ui()
