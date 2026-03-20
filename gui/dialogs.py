"""Reusable dialog classes for the OBD application."""

import customtkinter as ctk
from pathlib import Path
import tkinter.filedialog as filedialog
from gui.theme import COLORS, FONTS
from i18n import t


class SafetyConfirmDialog(ctk.CTkToplevel):
    """Simple centered confirmation dialog."""

    def __init__(self, parent, title, message):
        """Initialize confirmation dialog.

        Args:
            parent: Parent window
            title: Dialog title
            message: Confirmation message
        """
        super().__init__(parent)
        self.title(title)
        self.confirmed = False
        self.transient(parent)
        self.grab_set()

        # Size and center
        w, h = 420, 200
        self.geometry(f"{w}x{h}")
        self.resizable(False, False)
        self.configure(fg_color=COLORS["bg_primary"])

        # Center on screen
        self.update_idletasks()
        x = (self.winfo_screenwidth() - w) // 2
        y = (self.winfo_screenheight() - h) // 2
        self.geometry(f"{w}x{h}+{x}+{y}")

        # Warning icon + message
        ctk.CTkLabel(self, text="⚠", font=("Helvetica", 32),
                     text_color=COLORS["warning"]).pack(pady=(20, 8))

        ctk.CTkLabel(self, text=message, font=FONTS["body"],
                     text_color=COLORS["text_primary"],
                     wraplength=380).pack(padx=20, pady=(0, 16))

        # Buttons - centered
        btn_frame = ctk.CTkFrame(self, fg_color="transparent")
        btn_frame.pack(pady=(0, 20))

        ctk.CTkButton(btn_frame, text=t("dialog_cancel"), width=120,
                      fg_color=COLORS["bg_card"], hover_color=COLORS["bg_card_hover"],
                      command=self._cancel).pack(side="left", padx=8)

        ctk.CTkButton(btn_frame, text=t("dialog_confirm"), width=120,
                      fg_color=COLORS["danger"], hover_color=COLORS["danger_hover"],
                      command=self._confirm).pack(side="left", padx=8)

    def _cancel(self):
        """Cancel the dialog."""
        self.confirmed = False
        self.destroy()

    def _confirm(self):
        """Confirm the dialog."""
        self.confirmed = True
        self.destroy()

    def wait_for_result(self) -> bool:
        """Block until dialog is closed and return confirmation result."""
        self.wait_window()
        return self.confirmed


class ExportDialog(ctk.CTkToplevel):
    """Export format selection dialog."""

    def __init__(self, parent):
        """Initialize export dialog.

        Args:
            parent: Parent window
        """
        super().__init__(parent)
        self.title("Export Session")
        self.geometry("450x250")
        self.resizable(False, False)
        self.configure(fg_color=COLORS["bg_primary"])

        self.format = None
        self.filepath = None

        self.grab_set()
        self._setup_ui()

    def _setup_ui(self):
        """Setup dialog UI."""
        # Title
        title_label = ctk.CTkLabel(
            self, text=t("dialog_export_format"), font=FONTS["heading"],
            text_color=COLORS["text_primary"]
        )
        title_label.pack(pady=16, padx=16)

        # Format selection
        format_frame = ctk.CTkFrame(self, fg_color="transparent")
        format_frame.pack(pady=12, padx=16, fill="both", expand=True)

        self.format_var = ctk.StringVar(value="json")

        json_radio = ctk.CTkRadioButton(
            format_frame, text=t("dialog_json_format"), variable=self.format_var,
            value="json", font=FONTS["body"], text_color=COLORS["text_primary"]
        )
        json_radio.pack(anchor="w", pady=8)

        txt_radio = ctk.CTkRadioButton(
            format_frame, text=t("dialog_txt_format"), variable=self.format_var,
            value="txt", font=FONTS["body"], text_color=COLORS["text_primary"]
        )
        txt_radio.pack(anchor="w", pady=8)

        # Info label
        info_label = ctk.CTkLabel(
            format_frame, text=t("dialog_select_format"), font=FONTS["small"],
            text_color=COLORS["text_muted"]
        )
        info_label.pack(anchor="w", pady=8)

        # Buttons
        button_frame = ctk.CTkFrame(self, fg_color="transparent")
        button_frame.pack(pady=16, padx=16, fill="x")

        cancel_button = ctk.CTkButton(
            button_frame, text=t("dialog_cancel"), font=FONTS["body"],
            fg_color=COLORS["bg_card"], hover_color=COLORS["bg_card_hover"],
            command=self._cancel
        )
        cancel_button.pack(side="left", padx=4)

        export_button = ctk.CTkButton(
            button_frame, text=t("dialog_export"), font=FONTS["body"],
            fg_color=COLORS["accent"], hover_color=COLORS["accent_light"],
            command=self._export
        )
        export_button.pack(side="right", padx=4)

    def _export(self):
        """Select file path and export."""
        fmt = self.format_var.get()
        ext = ".json" if fmt == "json" else ".txt"

        filepath = filedialog.asksaveasfilename(
            defaultextension=ext,
            filetypes=[("JSON files", "*.json") if fmt == "json" else ("Text files", "*.txt")]
        )

        if filepath:
            self.format = fmt
            self.filepath = Path(filepath)
            self.destroy()

    def _cancel(self):
        """Cancel export."""
        self.format = None
        self.filepath = None
        self.destroy()

    def wait_for_result(self) -> tuple:
        """Block until dialog is closed and return format and filepath."""
        self.wait_window()
        return (self.format, self.filepath)


class ImportDialog:
    """Simple file picker for importing sessions."""

    @staticmethod
    def show(parent) -> Path:
        """Show import file dialog.

        Args:
            parent: Parent window

        Returns:
            Path to selected file or None
        """
        filepath = filedialog.askopenfilename(
            filetypes=[("JSON files", "*.json"), ("All files", "*.*")]
        )
        return Path(filepath) if filepath else None


class AboutDialog(ctk.CTkToplevel):
    """Application information and safety warnings dialog."""

    def __init__(self, parent, app_name="OBD Diagnostic Pro", version="1.0.0"):
        """Initialize about dialog.

        Args:
            parent: Parent window
            app_name: Application name
            version: Application version
        """
        super().__init__(parent)
        self.title(f"About {app_name}")
        self.geometry("500x500")
        self.resizable(False, False)
        self.configure(fg_color=COLORS["bg_primary"])

        self.app_name = app_name
        self.version = version

        self.grab_set()
        self._setup_ui()

    def _setup_ui(self):
        """Setup dialog UI."""
        # Scroll frame for content
        scroll_frame = ctk.CTkScrollableFrame(
            self, fg_color="transparent"
        )
        scroll_frame.pack(fill="both", expand=True, padx=16, pady=16)

        # App info
        title_label = ctk.CTkLabel(
            scroll_frame, text=self.app_name, font=FONTS["heading"],
            text_color=COLORS["text_primary"]
        )
        title_label.pack(pady=8)

        version_label = ctk.CTkLabel(
            scroll_frame, text=f"Version {self.version}", font=FONTS["body"],
            text_color=COLORS["text_secondary"]
        )
        version_label.pack(pady=4)

        # Description
        desc = "Professional OBD-II diagnostic and monitoring tool for vehicle diagnostics."
        desc_label = ctk.CTkLabel(
            scroll_frame, text=desc, font=FONTS["body"],
            text_color=COLORS["text_primary"], wraplength=450, justify="center"
        )
        desc_label.pack(pady=16)

        # Safety warnings section
        warning_title = ctk.CTkLabel(
            scroll_frame, text="Safety Warnings", font=FONTS["body_bold"],
            text_color=COLORS["danger"]
        )
        warning_title.pack(pady=12, anchor="w")

        warnings = [
            "Only use this tool on vehicles you own or have permission to service.",
            "Always consult the vehicle manual before performing diagnostics.",
            "Clearing DTCs may affect emissions and fuel economy.",
            "Do not perform unsafe operations while driving.",
            "Some operations require professional diagnostic equipment.",
        ]

        for warning in warnings:
            warning_label = ctk.CTkLabel(
                scroll_frame, text=f"• {warning}", font=FONTS["small"],
                text_color=COLORS["text_secondary"], wraplength=450, justify="left"
            )
            warning_label.pack(pady=4, anchor="w")

        # Close button
        close_button = ctk.CTkButton(
            self, text=t("dialog_close"), font=FONTS["body"],
            fg_color=COLORS["accent"], hover_color=COLORS["accent_light"],
            command=self.destroy
        )
        close_button.pack(pady=12, padx=16)
