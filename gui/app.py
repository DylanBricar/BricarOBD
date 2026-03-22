"""Main application window for OBD Diagnostic Pro."""

import customtkinter as ctk
from PIL import Image
import os
from config import APP_NAME, APP_VERSION, APP_WINDOW_SIZE, APP_MIN_SIZE
from gui.theme import COLORS, FONTS, apply_theme
from i18n import t, on_lang_change, set_lang, get_lang


class OBDApp(ctk.CTk):
    """Main application window with sidebar navigation and frame switching."""

    def __init__(self):
        super().__init__()

        self.title(f"{APP_NAME} v{APP_VERSION}")
        self.geometry(APP_WINDOW_SIZE)
        self.minsize(*APP_MIN_SIZE)

        apply_theme(self)
        self.configure(fg_color=COLORS["bg_primary"])

        self.connection = None
        self.obd_reader = None
        self.uds_client = None
        self.dtc_manager = None
        self.safety = None
        self.current_frame_name = None
        self.frames = {}
        self.nav_buttons = {}

        self._create_status_bar()
        self._create_sidebar()
        self._create_content_area()

    def _create_sidebar(self):
        self.sidebar = ctk.CTkFrame(
            self, fg_color=COLORS["bg_secondary"], width=200, corner_radius=0,
            border_width=1, border_color=COLORS["sidebar_border"]
        )
        self.sidebar.pack(side="left", fill="y")
        self.sidebar.pack_propagate(False)

        # Bug 3: Load logo from assets/logo.png
        logo_path = os.path.join(os.path.dirname(os.path.dirname(__file__)), "assets", "logo.png")
        if os.path.exists(logo_path):
            try:
                logo_img = ctk.CTkImage(light_image=Image.open(logo_path), dark_image=Image.open(logo_path), size=(160, 44))
                logo_label = ctk.CTkLabel(self.sidebar, image=logo_img, text="")
                logo_label.pack(pady=(20, 5))
            except Exception:
                # Fallback to text logo if image loading fails
                app_header = ctk.CTkLabel(
                    self.sidebar, text=APP_NAME, font=FONTS["h2"],
                    text_color=COLORS["text_primary"]
                )
                app_header.pack(pady=(20, 5), padx=12)
        else:
            # Fallback text logo
            app_header = ctk.CTkLabel(
                self.sidebar, text=APP_NAME, font=FONTS["h2"],
                text_color=COLORS["text_primary"]
            )
            app_header.pack(pady=(20, 5), padx=12)

        separator = ctk.CTkFrame(self.sidebar, fg_color=COLORS["border"], height=1)
        separator.pack(fill="x", padx=16, pady=(0, 12))

        self.nav_frame = ctk.CTkFrame(self.sidebar, fg_color="transparent")
        self.nav_frame.pack(fill="both", expand=True, padx=8, pady=4)

        self.nav_items = [
            ("nav_connection", "Connection"),
            ("nav_dashboard", "Dashboard"),
            ("nav_live_data", "Live Data"),
            ("nav_dtc", "DTC Codes"),
            ("nav_ecu", "ECU Info"),
            ("nav_monitors", "Monitors"),
            ("nav_history", "History"),
        ]

        self.nav_buttons_list = []
        for i18n_key, display_name in self.nav_items:
            btn = ctk.CTkButton(
                self.nav_frame,
                text=t(i18n_key),
                font=FONTS["body"],
                fg_color="transparent",
                hover_color=COLORS["bg_card_hover"],
                text_color=COLORS["text_secondary"],
                height=42,
                anchor="w",
                command=lambda name=display_name: self.show_frame(name),
            )
            btn.pack(fill="x", pady=6)
            self.nav_buttons[display_name] = btn
            self.nav_buttons_list.append((btn, i18n_key))

        # Separator before Advanced
        separator = ctk.CTkFrame(self.nav_frame, fg_color=COLORS["border"], height=1)
        separator.pack(fill="x", padx=4, pady=(12, 6))

        # Advanced button — RED, at the bottom of nav items
        self.advanced_btn = ctk.CTkButton(
            self.nav_frame,
            text=t("nav_advanced"),
            font=FONTS["body_bold"],
            fg_color=COLORS["danger_dark"],
            hover_color=COLORS["danger"],
            text_color=COLORS["danger_light"],
            height=42,
            anchor="w",
            command=lambda: self.show_frame("Advanced"),
        )
        self.advanced_btn.pack(fill="x", pady=6)
        self.nav_buttons["Advanced"] = self.advanced_btn
        self.nav_buttons_list.append((self.advanced_btn, "nav_advanced"))

        separator = ctk.CTkFrame(self.sidebar, fg_color=COLORS["border"], height=1)
        separator.pack(fill="x", padx=16, pady=(8, 12))

        # Bug 4: Move language selector to bottom with proper footer layout
        footer = ctk.CTkFrame(self.sidebar, fg_color="transparent")
        footer.pack(fill="x", padx=12, pady=12, side="bottom")

        # Dev console button
        self.dev_console_btn = ctk.CTkButton(
            footer, text="Dev Console", width=100, height=24,
            font=FONTS["small"], fg_color=COLORS["bg_card"],
            hover_color=COLORS["bg_card_hover"],
            text_color=COLORS["text_muted"],
            command=self._toggle_dev_console,
        )
        self.dev_console_btn.pack(pady=(0, 8))
        self._dev_console_window = None

        # Version label
        ctk.CTkLabel(
            footer, text=f"v{APP_VERSION}", font=FONTS["small"],
            text_color=COLORS["text_muted"],
        ).pack(pady=(0, 8))

        # Language selector frame (centered)
        lang_frame = ctk.CTkFrame(footer, fg_color="transparent")
        lang_frame.pack(pady=(8, 0))

        self.lang_fr_btn = ctk.CTkButton(
            lang_frame, text="FR", width=36, height=26,
            fg_color=COLORS["cyan"] if get_lang() == "fr" else COLORS["bg_card"],
            text_color=COLORS["text_primary"],
            command=lambda: self._change_language("fr"),
            font=FONTS["small"]
        )
        self.lang_fr_btn.pack(side="left", padx=2)

        self.lang_en_btn = ctk.CTkButton(
            lang_frame, text="EN", width=36, height=26,
            fg_color=COLORS["cyan"] if get_lang() == "en" else COLORS["bg_card"],
            text_color=COLORS["text_primary"],
            command=lambda: self._change_language("en"),
            font=FONTS["small"]
        )
        self.lang_en_btn.pack(side="left", padx=2)

        on_lang_change(self._on_language_changed)

    def _create_content_area(self):
        self.content_area = ctk.CTkFrame(
            self, fg_color=COLORS["bg_primary"], corner_radius=0
        )
        self.content_area.pack(side="right", fill="both", expand=True, padx=24, pady=20)

    def _create_status_bar(self):
        self.status_bar = ctk.CTkFrame(
            self, fg_color=COLORS["bg_secondary"], height=36, corner_radius=0,
            border_width=1, border_color=COLORS["border"]
        )
        self.status_bar.pack(side="bottom", fill="x")
        self.status_bar.pack_propagate(False)

        left = ctk.CTkFrame(self.status_bar, fg_color="transparent")
        left.pack(side="left", padx=12, pady=6)
        self.connection_indicator = ctk.CTkLabel(
            left, text="● Disconnected", font=FONTS["small"],
            text_color=COLORS["danger"],
        )
        self.connection_indicator.pack(side="left")

        center = ctk.CTkFrame(self.status_bar, fg_color="transparent")
        center.pack(side="left", expand=True, padx=12)
        self.protocol_label = ctk.CTkLabel(
            center, text="Protocol: —", font=FONTS["small"],
            text_color=COLORS["text_muted"],
        )
        self.protocol_label.pack()

        right = ctk.CTkFrame(self.status_bar, fg_color="transparent")
        right.pack(side="right", padx=12, pady=6)
        self.port_label = ctk.CTkLabel(
            right, text="Port: — | ELM: —", font=FONTS["small"],
            text_color=COLORS["text_muted"],
        )
        self.port_label.pack(side="right")

    def register_frame(self, name, frame):
        """Register a frame for navigation.

        Args:
            name: Frame name matching sidebar button text
            frame: CTkFrame instance
        """
        self.frames[name] = frame
        frame.pack_forget()

    def show_frame(self, frame_name):
        """Switch to the specified frame.

        Args:
            frame_name: Name of frame to show
        """
        # Don't reload if already showing
        if self.current_frame_name == frame_name:
            return

        from utils.dev_console import log_user_action
        log_user_action("Navigate", frame_name)

        if self.current_frame_name and self.current_frame_name in self.frames:
            self.frames[self.current_frame_name].pack_forget()

        if frame_name in self.frames:
            self.frames[frame_name].pack(in_=self.content_area, fill="both", expand=True)
            self.current_frame_name = frame_name

        # Notify frame if it has an on_frame_shown method
        if frame_name in self.frames and hasattr(self.frames[frame_name], "on_frame_shown"):
            try:
                self.frames[frame_name].on_frame_shown()
            except Exception:
                pass

        # Bug 2: Disable hover on active button (special handling for Advanced red button)
        for btn_name, btn in self.nav_buttons.items():
            if btn_name == frame_name:
                if btn_name == "Advanced":
                    btn.configure(fg_color=COLORS["danger"], hover_color=COLORS["danger"], text_color=COLORS["text_primary"])
                else:
                    btn.configure(fg_color=COLORS["cyan"], hover_color=COLORS["cyan"], text_color=COLORS["text_primary"])
            else:
                if btn_name == "Advanced":
                    btn.configure(fg_color=COLORS["danger_dark"], hover_color=COLORS["danger"], text_color=COLORS["danger_light"])
                else:
                    btn.configure(fg_color="transparent", hover_color=COLORS["bg_card_hover"], text_color=COLORS["text_secondary"])

    def update_status(self, connected, protocol="", port=""):
        """Update status bar.

        Args:
            connected: Connection state
            protocol: Protocol name
            port: Port name
        """
        if connected:
            self.connection_indicator.configure(
                text=f"● {t('status_connected')}", text_color=COLORS["success"]
            )
        else:
            self.connection_indicator.configure(
                text=f"● {t('status_disconnected')}", text_color=COLORS["danger"]
            )

        self.protocol_label.configure(
            text=f"{t('status_protocol')}: {protocol}" if protocol else f"{t('status_protocol')}: —"
        )

        if port:
            elm = getattr(self.connection, "elm_version", "—") if self.connection else "—"
            self.port_label.configure(text=f"{t('status_port')}: {port} | ELM: {elm}")
        else:
            self.port_label.configure(text=f"{t('status_port')}: — | ELM: —")

    def set_connection(self, connection):
        self.connection = connection

    def get_connection(self):
        return self.connection

    def _change_language(self, lang):
        """Change application language.

        Args:
            lang: Language code ("en" or "fr")
        """
        set_lang(lang)
        self._update_language_buttons()

    def _on_language_changed(self, lang=None):
        """Callback when language changes via i18n."""
        self._update_language_buttons()
        self._refresh_nav_labels()
        # Bug 5: Refresh status bar when language changes
        self._refresh_status_bar()

    def _update_language_buttons(self):
        """Update language selector button states."""
        lang = get_lang()
        if lang == "fr":
            self.lang_fr_btn.configure(fg_color=COLORS["cyan"])
            self.lang_en_btn.configure(fg_color=COLORS["bg_card"])
        else:
            self.lang_en_btn.configure(fg_color=COLORS["cyan"])
            self.lang_fr_btn.configure(fg_color=COLORS["bg_card"])

    def _refresh_nav_labels(self):
        """Refresh navigation button labels with new language."""
        for btn, i18n_key in self.nav_buttons_list:
            btn.configure(text=t(i18n_key))

    def _toggle_dev_console(self):
        """Open or focus the developer console window."""
        from utils.dev_console import DevConsoleWindow
        if self._dev_console_window and self._dev_console_window.winfo_exists():
            self._dev_console_window.lift()
            self._dev_console_window.focus_force()
            return
        self._dev_console_window = DevConsoleWindow(self)

    def _refresh_status_bar(self):
        """Refresh status bar text with current language."""
        connected = self.connection and hasattr(self.connection, 'is_connected') and self.connection.is_connected()
        protocol = getattr(self.connection, 'protocol_name', '') if self.connection else ''
        port = getattr(self.connection, 'port', '') if self.connection else ''
        self.update_status(connected, protocol, port)
