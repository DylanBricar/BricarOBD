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
            width=110, height=24, font=FONTS["small"],
            command=self.read_all_dtcs
        ).pack(side="left", padx=2)

        ctk.CTkButton(
            button_bar, text=t("dtc_read_pending"),
            width=110, height=24, font=FONTS["small"],
            command=self.read_pending
        ).pack(side="left", padx=2)

        ctk.CTkButton(
            button_bar, text=t("dtc_read_permanent"),
            width=110, height=24, font=FONTS["small"],
            command=self.read_permanent
        ).pack(side="left", padx=2)

        separator = ctk.CTkFrame(button_bar, fg_color=COLORS["border"], width=1, height=24)
        separator.pack(side="left", padx=10)

        ctk.CTkButton(
            button_bar, text=t("dtc_clear_all"), fg_color=COLORS["danger"],
            width=110, height=24, font=FONTS["small"],
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

    def read_all_dtcs(self):
        """Read all DTCs in a background thread."""
        def task():
            dtcs = self.app.dtc_manager.read_all_dtcs()
            self.after(0, self.populate_table, dtcs)

        thread = threading.Thread(target=task, daemon=True)
        thread.start()

    def read_pending(self):
        """Read pending DTCs in a background thread."""
        def task():
            dtcs = self.app.dtc_manager.read_pending_dtcs()
            self.after(0, self.populate_table, dtcs)

        thread = threading.Thread(target=task, daemon=True)
        thread.start()

    def read_permanent(self):
        """Read permanent DTCs in a background thread."""
        def task():
            dtcs = self.app.dtc_manager.read_permanent_dtcs()
            self.after(0, self.populate_table, dtcs)

        thread = threading.Thread(target=task, daemon=True)
        thread.start()

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

        desc_label = ctk.CTkLabel(
            row_frame, text=dtc_record.description[:40], font=FONTS["body"],
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

        url = DTC_SEARCH_URL.format(code=dtc_code, vehicle=f"{vehicle} {search_term}")
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

    def _on_lang_change(self, lang=None):
        """Rebuild UI on language change."""
        for widget in self.winfo_children():
            widget.destroy()
        self._setup_ui()
