"""Session history browser frame."""

import customtkinter as ctk
from datetime import datetime
from pathlib import Path
from tkinter import messagebox
from config import SESSIONS_DIR
from gui.theme import COLORS, FONTS, _bind_scroll_recursive
from i18n import t, on_lang_change


class HistoryFrame(ctk.CTkFrame):
    """Frame for browsing and managing saved diagnostic sessions."""

    def __init__(self, parent, app):
        """Initialize History frame.

        Args:
            parent: Parent widget
            app: Application instance with dtc_manager
        """
        super().__init__(parent, fg_color=COLORS["bg_primary"])
        self.app = app
        self.loaded_session = None
        self.loaded_filename = None
        self.loaded_dtcs = []
        self.title_label = None
        self.refresh_btn = None

        self._setup_ui()
        on_lang_change(self._on_lang_change)
        self.refresh_sessions()

    def _setup_ui(self):
        """Setup the frame UI layout."""
        # Title
        self.title_label = ctk.CTkLabel(
            self, text=t("hist_title"), font=FONTS["heading"],
            text_color=COLORS["text_primary"]
        )
        self.title_label.pack(pady=(0, 4), padx=16, anchor="w")

        # Help description
        ctk.CTkLabel(self, text=t("hist_help"), font=FONTS["small"], text_color=COLORS["text_muted"]).pack(anchor="w", padx=16, pady=(0, 12))

        # Control panel
        control_frame = ctk.CTkFrame(self, fg_color="transparent")
        control_frame.pack(pady=(0, 12), padx=16, fill="x")

        # Refresh button
        self.refresh_btn = ctk.CTkButton(
            control_frame, text=t("hist_refresh"), font=FONTS["body"],
            fg_color=COLORS["accent"], hover_color=COLORS["accent_light"],
            command=self.refresh_sessions
        )
        self.refresh_btn.pack(side="left", padx=4)

        # Main content
        content_frame = ctk.CTkFrame(self, fg_color="transparent")
        content_frame.pack(pady=8, padx=16, fill="both", expand=True)

        # Sessions list (left side)
        list_frame = ctk.CTkFrame(content_frame, fg_color="transparent")
        list_frame.pack(side="left", fill="both", expand=True)

        list_label = ctk.CTkLabel(
            list_frame, text=t("hist_sessions"), font=FONTS["body_bold"],
            text_color=COLORS["text_secondary"]
        )
        list_label.pack(pady=4, anchor="w")

        self.sessions_scroll = ctk.CTkScrollableFrame(
            list_frame, fg_color=COLORS["bg_secondary"], corner_radius=8
        )
        self.sessions_scroll.pack(fill="both", expand=True)
        self.after(500, lambda: _bind_scroll_recursive(self.sessions_scroll))

        # Details area (right side)
        details_frame = ctk.CTkFrame(content_frame, fg_color="transparent")
        details_frame.pack(side="right", fill="both", expand=True, padx=(12, 0))

        details_label = ctk.CTkLabel(
            details_frame, text=t("hist_details"), font=FONTS["body_bold"],
            text_color=COLORS["text_secondary"]
        )
        details_label.pack(pady=4, anchor="w")

        self.details_scroll = ctk.CTkScrollableFrame(
            details_frame, fg_color=COLORS["bg_secondary"], corner_radius=8
        )
        self.details_scroll.pack(fill="both", expand=True)
        self.after(500, lambda: _bind_scroll_recursive(self.details_scroll))

        # Export button
        self.export_button = ctk.CTkButton(
            self, text=t("hist_export"), font=FONTS["body"],
            fg_color=COLORS["accent"], hover_color=COLORS["accent_light"],
            command=self._export_session, state="disabled"
        )
        self.export_button.pack(pady=8, padx=16)

    def refresh_sessions(self):
        """Reload and display sessions from storage."""
        # Clear previous
        for widget in self.sessions_scroll.winfo_children():
            widget.destroy()

        try:
            sessions = self.app.dtc_manager.get_saved_sessions()
        except Exception:
            sessions = []

        if not sessions:
            empty_label = ctk.CTkLabel(
                self.sessions_scroll, text=t("hist_no_sessions"), font=FONTS["small"],
                text_color=COLORS["text_muted"]
            )
            empty_label.pack(pady=20, padx=12)
            return

        self.populate_session_list(sessions)

    def populate_session_list(self, sessions):
        """Create session cards.

        Args:
            sessions: List of session file paths
        """
        for session_path in sessions:
            self._create_session_card(session_path)

    def _create_session_card(self, session):
        """Create a single session card.

        Args:
            session: Session dict with keys: filename, date, dtc_count
        """
        session_path = SESSIONS_DIR / session["filename"]

        card_frame = ctk.CTkFrame(
            self.sessions_scroll, fg_color=COLORS["bg_card"], corner_radius=8
        )
        card_frame.pack(pady=6, padx=0, fill="x")

        # Header
        header_frame = ctk.CTkFrame(card_frame, fg_color="transparent")
        header_frame.pack(pady=8, padx=12, fill="x")

        filename_label = ctk.CTkLabel(
            header_frame, text=session["filename"], font=FONTS["body_bold"],
            text_color=COLORS["text_primary"]
        )
        filename_label.pack(side="left", anchor="w")

        # Info
        info_frame = ctk.CTkFrame(card_frame, fg_color="transparent")
        info_frame.pack(pady=4, padx=12, fill="x")

        try:
            date_str = datetime.fromtimestamp(session['date']).strftime('%Y-%m-%d %H:%M')
        except (ValueError, TypeError, OSError):
            date_str = str(session['date'])

        datetime_label = ctk.CTkLabel(
            info_frame, text=f"Date: {date_str}", font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        )
        datetime_label.pack(anchor="w")

        dtc_label = ctk.CTkLabel(
            info_frame, text=f"DTCs: {session['dtc_count']}", font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        )
        dtc_label.pack(anchor="w")

        # Buttons
        button_frame = ctk.CTkFrame(info_frame, fg_color="transparent")
        button_frame.pack(pady=8, fill="x")

        load_button = ctk.CTkButton(
            button_frame, text=t("hist_load"), font=FONTS["small"],
            fg_color=COLORS["accent"], hover_color=COLORS["accent_light"],
            height=28,
            command=lambda: self.load_session(str(session_path), session["filename"])
        )
        load_button.pack(side="left", padx=4)

        delete_button = ctk.CTkButton(
            button_frame, text=t("hist_delete"), font=FONTS["small"],
            fg_color=COLORS["danger"], hover_color=COLORS["danger_dark"],
            height=28,
            command=lambda: self.delete_session(str(session_path))
        )
        delete_button.pack(side="left", padx=4)

    def load_session(self, filepath, filename):
        """Load and display DTCs from a session.

        Args:
            filepath: Full path to session file
            filename: Session filename for tracking
        """
        try:
            filepath = Path(filepath) if not isinstance(filepath, Path) else filepath
            dtcs = self.app.dtc_manager.load_dtcs(filepath)
            self.loaded_session = dtcs
            self.loaded_filename = filename
            self.loaded_dtcs = dtcs
            self._display_session_dtcs(dtcs)
            self.export_button.configure(state="normal")
        except Exception as e:
            self._show_error(f"Failed to load session: {str(e)}")

    def _display_session_dtcs(self, dtcs):
        """Display DTCs in the details area.

        Args:
            dtcs: List of DTCRecord objects
        """
        for widget in self.details_scroll.winfo_children():
            widget.destroy()

        if not dtcs:
            empty_label = ctk.CTkLabel(
                self.details_scroll, text=t("hist_no_dtcs"), font=FONTS["small"],
                text_color=COLORS["text_muted"]
            )
            empty_label.pack(pady=20, padx=12)
            return

        for dtc in dtcs:
            dtc_frame = ctk.CTkFrame(self.details_scroll, fg_color=COLORS["bg_card_hover"], corner_radius=6)
            dtc_frame.pack(pady=4, padx=4, fill="x")

            code_label = ctk.CTkLabel(
                dtc_frame, text=dtc.code, font=FONTS["body_bold"],
                text_color=COLORS["text_primary"]
            )
            code_label.pack(pady=4, padx=8, anchor="w")

            desc_label = ctk.CTkLabel(
                dtc_frame, text=dtc.description, font=FONTS["small"],
                text_color=COLORS["text_secondary"], wraplength=300, justify="left"
            )
            desc_label.pack(pady=2, padx=8, anchor="w")

    def delete_session(self, filepath):
        """Delete a session file with confirmation.

        Args:
            filepath: Path to session file
        """
        # Confirm deletion
        confirm = messagebox.askyesno(t("dialog_warning"), t("hist_delete_confirm"))
        if not confirm:
            return

        try:
            Path(filepath).unlink()
            self.refresh_sessions()
            if self.loaded_filename and Path(filepath).name == self.loaded_filename:
                self.loaded_session = None
                self.loaded_filename = None
                self.loaded_dtcs = []
                self.export_button.configure(state="disabled")
                for widget in self.details_scroll.winfo_children():
                    widget.destroy()
        except Exception as e:
            self._show_error(f"Failed to delete session: {str(e)}")

    def _export_session(self):
        """Export the loaded session."""
        if not self.loaded_session:
            return
        from tkinter import filedialog
        filepath = filedialog.asksaveasfilename(
            defaultextension=".json",
            filetypes=[("JSON", "*.json")],
        )
        if filepath:
            from pathlib import Path
            self.app.dtc_manager.export_to_json(self.loaded_session, Path(filepath))

    def _show_error(self, message):
        """Show error message in details area.

        Args:
            message: Error message
        """
        for widget in self.details_scroll.winfo_children():
            widget.destroy()

        error_label = ctk.CTkLabel(
            self.details_scroll, text=f"{t('hist_error')}: {message}", font=FONTS["small"],
            text_color=COLORS["danger"]
        )
        error_label.pack(pady=20, padx=12)

    def _on_lang_change(self, lang=None):
        """Rebuild UI on language change."""
        for widget in self.winfo_children():
            widget.destroy()
        self._setup_ui()
        self.refresh_sessions()
