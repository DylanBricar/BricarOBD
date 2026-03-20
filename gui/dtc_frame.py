"""DTC (Diagnostic Trouble Code) management frame."""

import customtkinter as ctk
import webbrowser
import threading
from tkinter import filedialog
from pathlib import Path
from gui.theme import COLORS, FONTS
from config import DTC_SEARCH_URL
from i18n import t, on_lang_change


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
        self.title_label.pack(anchor="w", padx=16, pady=(8, 2))

        ctk.CTkLabel(self, text=t("dtc_help"), font=FONTS["small"], text_color=COLORS["text_muted"]).pack(anchor="w", padx=16, pady=(0, 4))

        button_bar = ctk.CTkFrame(self, fg_color="transparent")
        button_bar.pack(anchor="w", padx=16, pady=(0, 4), fill="x")

        ctk.CTkButton(
            button_bar, text=t("dtc_read_all"), fg_color=COLORS["accent"],
            width=120, height=26, font=FONTS["small"],
            command=self.read_all_dtcs
        ).pack(side="left", padx=4)

        ctk.CTkButton(
            button_bar, text=t("dtc_read_pending"),
            width=120, height=26, font=FONTS["small"],
            command=self.read_pending
        ).pack(side="left", padx=4)

        ctk.CTkButton(
            button_bar, text=t("dtc_read_permanent"),
            width=120, height=26, font=FONTS["small"],
            command=self.read_permanent
        ).pack(side="left", padx=4)

        separator = ctk.CTkFrame(button_bar, fg_color=COLORS["border"], width=2)
        separator.pack(side="left", padx=8, fill="y")

        ctk.CTkButton(
            button_bar, text=t("dtc_clear_all"), fg_color=COLORS["danger"],
            width=120, height=26, font=FONTS["small"],
            command=self.clear_dtcs
        ).pack(side="left", padx=4)

        table_header_frame = ctk.CTkFrame(self, fg_color="transparent")
        table_header_frame.pack(anchor="w", padx=16, pady=8, fill="x")

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

        self.empty_state = ctk.CTkLabel(
            self.table_frame, text=t("dtc_no_codes"),
            font=FONTS["body"], text_color=COLORS["text_muted"]
        )
        self.empty_state.pack(anchor="center", pady=40)

        bottom_bar = ctk.CTkFrame(self, fg_color="transparent")
        bottom_bar.pack(anchor="w", padx=16, pady=16, fill="x")

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

        self.dtc_rows[dtc_record.code] = row_frame

    def search_web(self, dtc_code):
        """Open web browser to search for DTC code.

        Args:
            dtc_code: DTC code string (e.g., "P0420")
        """
        url = DTC_SEARCH_URL.format(code=dtc_code, vehicle="vehicle")
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

    def _on_lang_change(self, lang=None):
        """Update text on language change."""
        self.title_label.configure(text=t("dtc_title"))
        self.count_label.configure(text=t("dtc_found_zero"))
