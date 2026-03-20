"""ECU Information and Scanner frame."""

import customtkinter as ctk
from threading import Thread
from gui.theme import COLORS, FONTS
from i18n import t, on_lang_change


class ECUInfoFrame(ctk.CTkFrame):
    """Frame for ECU scanning and information display."""

    def __init__(self, parent, app):
        """Initialize ECU Info frame.

        Args:
            parent: Parent widget
            app: Application instance with uds_client
        """
        super().__init__(parent, fg_color=COLORS["bg_primary"])
        self.app = app
        self.scanning = False
        self.discovered_ecus = []
        self.title_label = None
        self.scan_btn = None

        self._setup_ui()
        on_lang_change(self._on_lang_change)

    def _setup_ui(self):
        """Setup the frame UI layout."""
        # Title
        self.title_label = ctk.CTkLabel(
            self, text=t("ecu_title"), font=FONTS["heading"],
            text_color=COLORS["text_primary"]
        )
        self.title_label.pack(pady=12, padx=16, anchor="w")

        # Help description
        ctk.CTkLabel(self, text=t("ecu_help"), font=FONTS["small"], text_color=COLORS["text_muted"]).pack(anchor="w", padx=16, pady=(0, 8))

        # Control panel
        control_frame = ctk.CTkFrame(self, fg_color="transparent")
        control_frame.pack(pady=8, padx=16, fill="x")

        # Scan button
        self.scan_btn = ctk.CTkButton(
            control_frame, text=t("ecu_scan"), font=FONTS["body_bold"],
            fg_color=COLORS["accent"], hover_color=COLORS["accent_light"],
            command=self.scan_ecus
        )
        self.scan_btn.pack(side="left", padx=4)

        # Detected vehicle display
        detected = getattr(self.app, 'detected_make', '')
        vehicle_text = f"{t('ecu_profile')}: {detected}" if detected else t("ecu_profile")
        self.vehicle_label = ctk.CTkLabel(self, text=vehicle_text, font=FONTS["body"],
                                           text_color=COLORS["text_secondary"])
        self.vehicle_label.pack(anchor="w", padx=16, pady=4)

        # Status label
        self.status_label = ctk.CTkLabel(
            self, text=t("ecu_ready"), font=FONTS["small"],
            text_color=COLORS["text_muted"]
        )
        self.status_label.pack(pady=4, padx=16, anchor="w")

        # Results area
        self.results_frame = ctk.CTkScrollableFrame(
            self, fg_color=COLORS["bg_secondary"], corner_radius=8
        )
        self.results_frame.pack(pady=12, padx=16, fill="both", expand=True)

    def scan_ecus(self):
        """Scan for responding ECUs in background thread."""
        if self.scanning:
            return

        self.scanning = True
        self.scan_btn.configure(state="disabled")
        self.status_label.configure(text=t("ecu_scanning"))

        # Clear previous results
        for widget in self.results_frame.winfo_children():
            widget.destroy()

        def _scan():
            try:
                # Use detected make for manufacturer-specific ECU scanning
                make = getattr(self.app, 'detected_make', '') or ''
                if make and make != 'Unknown':
                    self.after(0, self._update_status, f"Scan {make}...")

                # Scan with manufacturer-specific addresses
                ecus = self.app.uds_client.scan_ecus(make=make) if self.app.uds_client else []

                # Also use pre-discovered ECUs from connection if available
                pre_discovered = getattr(self.app, 'discovered_ecus', [])
                if pre_discovered and not ecus:
                    ecus = pre_discovered

                self.discovered_ecus = ecus
                self.after(0, self.populate_results, ecus)
                self.after(0, self._update_status, t("ecu_found", count=len(ecus)))
            except Exception as e:
                self.after(0, self._update_status, f"{t('ecu_scan_failed')}: {str(e)}")
            finally:
                self.scanning = False
                self.after(0, lambda: self.scan_btn.configure(state="normal"))

        thread = Thread(target=_scan, daemon=True)
        thread.start()

    def populate_results(self, ecu_list):
        """Create ECU cards for discovered ECUs.

        Args:
            ecu_list: List of discovered ECUs with request_id and response_id
        """
        if not ecu_list:
            empty_label = ctk.CTkLabel(
                self.results_frame, text=t("ecu_no_found"), font=FONTS["small"],
                text_color=COLORS["text_muted"]
            )
            empty_label.pack(pady=20)
            return

        for ecu in ecu_list:
            self._create_ecu_card(ecu)

        # Auto-trigger info read for each discovered ECU
        for ecu in ecu_list:
            self.read_ecu_info(ecu.get("request_id"), ecu.get("response_id"))

    def _create_ecu_card(self, ecu):
        """Create a single ECU info card.

        Args:
            ecu: ECU info dict with request_id, response_id, name
        """
        card_frame = ctk.CTkFrame(
            self.results_frame, fg_color=COLORS["bg_card"], corner_radius=8
        )
        card_frame.pack(pady=8, padx=0, fill="x")

        # Header
        header_frame = ctk.CTkFrame(card_frame, fg_color="transparent")
        header_frame.pack(pady=8, padx=12, fill="x")

        name_label = ctk.CTkLabel(
            header_frame, text=ecu.get("name", "Unknown ECU"), font=FONTS["body_bold"],
            text_color=COLORS["text_primary"]
        )
        name_label.pack(side="left", anchor="w")

        ids_label = ctk.CTkLabel(
            header_frame, text=f"REQ: 0x{ecu.get('request_id', 0):03X} / RES: 0x{ecu.get('response_id', 0):03X}",
            font=FONTS["small"], text_color=COLORS["text_muted"]
        )
        ids_label.pack(side="left", padx=8, anchor="w")

        # Details area (expandable)
        details_frame = ctk.CTkFrame(card_frame, fg_color=COLORS["bg_card_hover"], corner_radius=6)
        details_frame.pack(pady=8, padx=12, fill="x")

        # Placeholder for details
        details_label = ctk.CTkLabel(
            details_frame, text=t("ecu_loading"), font=FONTS["small"],
            text_color=COLORS["text_muted"]
        )
        details_label.pack(pady=8, padx=8)

        # Store details frame for later updates
        ecu["details_frame"] = details_frame

    def read_ecu_info(self, request_id, response_id):
        """Read ECU information via UDS.

        Args:
            request_id: CAN request ID
            response_id: CAN response ID
        """
        def _read():
            try:
                if self.app.uds_client:
                    self.app.uds_client.set_target_ecu(request_id, response_id)
                    info = self.app.uds_client.read_ecu_info()
                    self.after(0, self._display_ecu_info, request_id, info)
            except Exception as e:
                self.after(0, self._update_status, f"{t('ecu_read_failed')}: {str(e)}")

        thread = Thread(target=_read, daemon=True)
        thread.start()

    def _display_ecu_info(self, request_id, info):
        """Display ECU information in the card details.

        Args:
            request_id: ECU request ID
            info: ECU info dict
        """
        # Find the card for this ECU
        for ecu in self.discovered_ecus:
            if ecu.get("request_id") == request_id:
                details_frame = ecu.get("details_frame")
                if details_frame:
                    for widget in details_frame.winfo_children():
                        widget.destroy()

                    # Create info display
                    info_text = f"VIN: {info.get('vin', 'N/A')}\n"
                    info_text += f"Serial: {info.get('serial', 'N/A')}\n"
                    info_text += f"HW Version: {info.get('hw_version', 'N/A')}\n"
                    info_text += f"SW Version: {info.get('sw_version', 'N/A')}\n"
                    info_text += f"Session: {info.get('Session', 'N/A')}\n"
                    info_text += f"DIDs: {info.get('DIDs', 'N/A')}"

                    info_label = ctk.CTkLabel(
                        details_frame, text=info_text, font=FONTS["small"],
                        text_color=COLORS["text_primary"], justify="left"
                    )
                    info_label.pack(pady=8, padx=8, anchor="w")
                break

    def on_profile_change(self, profile_name):
        """Update expected ECU list based on vehicle profile.

        Args:
            profile_name: Selected vehicle profile
        """
        # Profile definitions for future use
        profiles = {
            "Generic": [],
            "Peugeot 206": ["Engine", "Transmission", "Body Control"],
            "Peugeot 207": ["Engine", "Transmission", "ABS/ESP", "Airbag"],
            "Audi A3": ["Engine", "Transmission", "ABS/ESP", "Airbag", "Body Control"],
            "Audi A4": ["Engine", "Transmission", "ABS/ESP", "Airbag", "Body Control", "Steering"],
            "VW Golf": ["Engine", "Transmission", "ABS/ESP", "Body Control"],
        }
        # Store selected profile for later use
        self.selected_profile = profile_name
        self._update_status(f"Profile: {profile_name}")

    def _update_status(self, message):
        """Update status label.

        Args:
            message: Status message to display
        """
        self.status_label.configure(text=message)

    def _on_lang_change(self, lang=None):
        """Update text on language change."""
        self.title_label.configure(text=t("ecu_title"))
        self.scan_btn.configure(text=t("ecu_scan"))
        detected = getattr(self.app, 'detected_make', '')
        self.vehicle_label.configure(text=f"{t('ecu_profile')}: {detected}" if detected else t("ecu_profile"))
