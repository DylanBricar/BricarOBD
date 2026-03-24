"""Connection setup panel for OBD diagnostic tool."""

import customtkinter as ctk
import threading
from datetime import datetime

from gui.theme import COLORS, FONTS
from obd_core.connection import ELM327Connection
from i18n import t, on_lang_change


class ConnectionFrame(ctk.CTkFrame):
    """Connection setup and status panel."""

    def __init__(self, parent, app):
        """Initialize connection frame.

        Args:
            parent: Parent widget (content_area)
            app: OBDApp reference
        """
        super().__init__(parent, fg_color=COLORS["bg_primary"])
        self.app = app
        self.pack(fill="both", expand=True)

        # Register language change callback
        on_lang_change(self._on_lang_change)

        self._setup_ui()

    def _create_card(self, parent, title=""):
        """Create a styled card frame with optional title.

        Args:
            parent: Parent widget
            title: Optional card title

        Returns:
            CTkFrame configured as a premium card
        """
        card = ctk.CTkFrame(
            parent,
            fg_color=COLORS["bg_card"],
            corner_radius=12,
            border_width=1,
            border_color=COLORS["card_border"],
        )
        if title:
            ctk.CTkLabel(
                card, text=title, font=FONTS["small_bold"],
                text_color=COLORS["text_secondary"],
            ).pack(anchor="w", padx=20, pady=(16, 8))
        return card

    def _setup_ui(self):
        """Build the connection UI."""
        self.title_label = ctk.CTkLabel(
            self, text=t("conn_title"), font=FONTS["heading"],
            text_color=COLORS["text_primary"]
        )
        self.title_label.pack(anchor="w", pady=(0, 12))

        self.config_card = self._create_card(self, t("conn_configuration"))
        self.config_card.pack(fill="x", pady=(0, 12))

        config_content = ctk.CTkFrame(self.config_card, fg_color="transparent")
        config_content.pack(fill="both", padx=20, pady=(0, 20))

        self.port_label = ctk.CTkLabel(
            config_content, text=t("conn_port"), font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        )
        self.port_label.pack(anchor="w", pady=(0, 6))

        port_row = ctk.CTkFrame(config_content, fg_color="transparent")
        port_row.pack(fill="x", pady=(0, 16))

        self.port_combobox = ctk.CTkComboBox(
            port_row, values=self._get_available_ports(),
            fg_color=COLORS["bg_input"], border_color=COLORS["input_border"],
            border_width=1,
            text_color=COLORS["text_primary"]
        )
        self.port_combobox.pack(side="left", fill="x", expand=True, padx=(0, 12))

        self.refresh_btn = ctk.CTkButton(
            port_row, text=t("conn_refresh"), width=100,
            command=self.refresh_ports,
            fg_color=COLORS["accent"], hover_color=COLORS["accent_hover"],
            corner_radius=8
        )
        self.refresh_btn.pack(side="left")

        self.baud_label = ctk.CTkLabel(
            config_content, text=t("conn_baud_rate"), font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        )
        self.baud_label.pack(anchor="w", pady=(0, 6))

        self.baud_combobox = ctk.CTkComboBox(
            config_content, values=["9600", "38400", "115200", "230400", "500000"],
            fg_color=COLORS["bg_input"], border_color=COLORS["input_border"],
            border_width=1,
            text_color=COLORS["text_primary"]
        )
        self.baud_combobox.set("38400")
        self.baud_combobox.pack(fill="x", pady=(0, 20))

        button_row = ctk.CTkFrame(config_content, fg_color="transparent")
        button_row.pack(fill="x")

        self.connect_btn = ctk.CTkButton(
            button_row, text=t("conn_connect"), width=140,
            command=self._on_connect_btn_clicked,
            fg_color=COLORS["success"], hover_color="#0DA574",
            text_color="#FFFFFF", text_color_disabled="#FFFFFF",
            corner_radius=8
        )
        self.connect_btn.pack(side="left", padx=(0, 12))

        self.disconnect_btn = ctk.CTkButton(
            button_row, text=t("conn_disconnect"), width=140,
            command=self.disconnect,
            fg_color=COLORS["text_muted"], hover_color=COLORS["text_muted"],
            text_color_disabled="#FFFFFF",
            corner_radius=8,
            state="disabled"
        )
        self.disconnect_btn.pack(side="left")

        self.status_card = self._create_card(self, t("conn_status"))
        self.status_card.pack(fill="x", pady=(0, 12))

        status_content = ctk.CTkFrame(self.status_card, fg_color="transparent")
        status_content.pack(fill="both", padx=20, pady=(0, 20))

        self.status_indicator = ctk.CTkLabel(
            status_content, text=f"● {t('status_disconnected')}", font=FONTS["body"],
            text_color=COLORS["danger"]
        )
        self.status_indicator.pack(anchor="w", pady=(0, 8))

        self.protocol_label = ctk.CTkLabel(
            status_content, text=f"{t('status_protocol')}: {t('conn_not_detected')}", font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        )
        self.protocol_label.pack(anchor="w", pady=(0, 4))

        self.elm_version_label = ctk.CTkLabel(
            status_content, text=f"{t('conn_elm_version')}: --", font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        )
        self.elm_version_label.pack(anchor="w", pady=(0, 4))

        self.port_info_label = ctk.CTkLabel(
            status_content, text=f"{t('status_port')}: --", font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        )
        self.port_info_label.pack(anchor="w")

        self.log_label = ctk.CTkLabel(
            self, text=t("conn_log"), font=FONTS["small_bold"],
            text_color=COLORS["text_secondary"],
        )
        self.log_card = self._create_card(self, t("conn_log"))
        self.log_card.pack(fill="both", expand=True, pady=(0, 0))

        log_content = ctk.CTkFrame(self.log_card, fg_color="transparent")
        log_content.pack(fill="both", expand=True, padx=20, pady=(0, 16))

        self.log_textbox = ctk.CTkTextbox(
            log_content, fg_color=COLORS["bg_input"], text_color=COLORS["text_primary"],
            border_color=COLORS["input_border"], border_width=1
        )
        self.log_textbox.pack(fill="both", expand=True, pady=(8, 0))

    def _get_available_ports(self):
        """Get list of available serial ports.

        Returns:
            List of port names
        """
        try:
            ports = ELM327Connection.available_ports()
            port_list = [f"{p['port']} ({p['description']})" for p in ports]
            # Always add demo mode option at the top
            port_list.insert(0, "DEMO (BricarOBD Demo Mode)")
            return port_list if port_list else ["DEMO (BricarOBD Demo Mode)"]
        except Exception:
            return ["DEMO (BricarOBD Demo Mode)"]

    def refresh_ports(self):
        """Rescan and update port combobox."""
        ports = self._get_available_ports()
        self.port_combobox.configure(values=ports)
        self.log_message(t("conn_ports_refreshed"))

    def _on_connect_btn_clicked(self):
        """Handle connect button click."""
        port_display = self.port_combobox.get()
        baud = int(self.baud_combobox.get())

        if not port_display or port_display == t("conn_no_port"):
            self.log_message(f"ERROR: {t('conn_invalid_port')}")
            return

        port = port_display.split(" (")[0]

        self.connect_btn.configure(state="disabled")
        self.port_combobox.configure(state="disabled")
        self.baud_combobox.configure(state="disabled")

        thread = threading.Thread(target=self._connect_thread, args=(port, baud), daemon=True)
        thread.start()

    def _connect_thread(self, port, baud):
        """Background thread for connection.

        Args:
            port: Serial port name
            baud: Baud rate
        """
        # Handle demo mode
        if port == "DEMO":
            from obd_core.demo_mode import DemoConnection
            from obd_core.obd_reader import OBDReader
            from obd_core.uds_client import UDSClient
            from obd_core.dtc_manager import DTCManager

            self.after(0, self.log_message, t("conn_connecting_to", port=port))
            try:
                demo_conn = DemoConnection()
                success = demo_conn.connect()
                # Replace the real connection with demo
                self.app.connection = demo_conn
                # Recreate readers with demo connection
                self.app.obd_reader = OBDReader(demo_conn)
                self.app.uds_client = UDSClient(demo_conn, self.app.safety)
                self.app.dtc_manager = DTCManager(self.app.obd_reader, self.app.uds_client, self.app.safety)
                self.after(0, self._on_connect_result, success)
            except Exception as e:
                self.after(0, self.log_message, t("conn_demo_error", error=str(e)))
                self.after(0, self._on_connect_result, False)
            return

        self.after(0, self.log_message, t("conn_connecting_to", port=f"{port} @ {baud}"))

        # Install a logging handler to forward protocol cycling info to the GUI log
        import logging
        class _GUILogHandler(logging.Handler):
            def __init__(self, frame):
                super().__init__()
                self.frame = frame
            def emit(self, record):
                msg = record.getMessage()
                if "Trying protocol" in msg or "ECU responded" in msg or "No ECU response" in msg:
                    self.frame.after(0, self.frame.log_message, f"   {msg}")

        gui_handler = _GUILogHandler(self)
        gui_handler.setLevel(logging.INFO)
        hybrid_logger = logging.getLogger("obd_core.hybrid_reader")
        hybrid_logger.addHandler(gui_handler)

        try:
            success = self.app.connection.connect(port, baud)
            self.after(0, self._on_connect_result, success)
        except Exception as e:
            self.after(0, self.log_message, t("conn_error", error=str(e)))
            self.after(0, self._on_connect_result, False)
        finally:
            hybrid_logger.removeHandler(gui_handler)

    def _on_connect_result(self, success):
        """Called after connect thread finishes.

        Args:
            success: Whether connection succeeded
        """
        if success:
            self.log_message(t("conn_success"))
            self.connect_btn.configure(state="disabled", fg_color="#374151", text_color="#6B7280")
            self.disconnect_btn.configure(state="normal", fg_color=COLORS["danger"], text_color="#FFFFFF")
            self.port_combobox.configure(state="disabled")
            self.baud_combobox.configure(state="disabled")
            self.update_connection_info()
            self.app.update_status(True, self.app.connection.protocol_name, self.app.connection.port)

            # Auto-detect vehicle via VIN + discover supported PIDs
            import threading
            def detect_vehicle():
                try:
                    from obd_core.vin_decoder import decode_vin, get_profile_key_for_make

                    # Step 1: VIN — skip Mode 09 queries as they crash some ELM327 clones.
                    # VIN will be available if python-obd already read it during handshake.
                    self.after(0, self.log_message, t("conn_step_vin"))
                    make = "Unknown"
                    year = ""
                    self.app.detected_vehicle = None
                    self.app.detected_make = ""
                    self.after(0, self.log_message, t("conn_step_vin_unavailable"))

                    # Step 2: Discover supported PIDs
                    self.after(0, self.log_message, t("conn_step_pids"))
                    supported = self.app.obd_reader.discover_supported_pids()
                    self.after(0, self.log_message, t("conn_step_pids_result", count=len(supported)))

                    # Step 3: Scan ECUs with manufacturer-specific addresses
                    # Skip UDS scan when python-obd is the active connection —
                    # sending AT SH commands corrupts python-obd's internal state
                    # and breaks all subsequent PID queries.
                    conn = self.app.connection
                    if hasattr(conn, '_obd_conn') and conn._obd_conn is not None:
                        self.after(0, self.log_message, t("conn_step_ecus", make=make))
                        self.app.discovered_ecus = []
                        self.after(0, self.log_message, "   ECU scan skipped (python-obd active — use Advanced tab)")
                    else:
                        self.after(0, self.log_message, t("conn_step_ecus", make=make))
                        ecus = self.app.uds_client.scan_ecus(make=make)
                        self.app.discovered_ecus = ecus
                        self.after(0, self.log_message, t("conn_step_ecus_result", count=len(ecus)))

                    # Final summary
                    self.after(0, self._update_vehicle_display, make, year)

                except Exception as e:
                    self.after(0, self.log_message, t("conn_autodetect_error", error=str(e)))

            threading.Thread(target=detect_vehicle, daemon=True).start()
        else:
            self.log_message(t("conn_failed"))
            self.connect_btn.configure(state="normal", fg_color=COLORS["success"], text_color="#FFFFFF")
            self.port_combobox.configure(state="normal")
            self.baud_combobox.configure(state="normal")

    def disconnect(self):
        """Disconnect from OBD device."""
        thread = threading.Thread(target=self._disconnect_thread, daemon=True)
        thread.start()

    def _disconnect_thread(self):
        """Background thread for disconnection."""
        self.after(0, self.log_message, t("conn_disconnecting"))
        try:
            self.app.connection.disconnect()
            self.after(0, self._on_disconnect_result, True)
        except Exception as e:
            self.after(0, self.log_message, f"Disconnect error: {str(e)}")
            self.after(0, self._on_disconnect_result, False)

    def _on_disconnect_result(self, success):
        """Called after disconnect thread finishes.

        Args:
            success: Whether disconnection succeeded
        """
        self.log_message(t("conn_disconnected"))
        self.connect_btn.configure(state="normal", fg_color=COLORS["success"], text_color="#FFFFFF")
        self.disconnect_btn.configure(state="disabled", fg_color="#374151", text_color="#6B7280")
        self.port_combobox.configure(state="normal")
        self.baud_combobox.configure(state="normal")
        self.status_indicator.configure(text=f"● {t('status_disconnected')}", text_color=COLORS["danger"])
        self.protocol_label.configure(text=f"{t('status_protocol')}: {t('conn_not_detected')}")
        self.elm_version_label.configure(text=f"{t('conn_elm_version')}: --")
        self.port_info_label.configure(text=f"{t('status_port')}: --")
        self.app.update_status(False)

    def update_connection_info(self):
        """Update the status card with current connection details."""
        try:
            port = self.app.connection.port
            protocol = self.app.connection.protocol_name or "Unknown"
            elm_version = self.app.connection.elm_version or "Unknown"

            self.status_indicator.configure(text=f"● {t('status_connected')}", text_color=COLORS["success"])
            self.protocol_label.configure(text=f"{t('status_protocol')}: {protocol}")
            self.elm_version_label.configure(text=f"{t('conn_elm_version')}: {elm_version}")
            self.port_info_label.configure(text=f"{t('status_port')}: {port}")
        except Exception as e:
            self.log_message(f"Error updating info: {str(e)}")

    def log_message(self, msg):
        """Append a message to the log textbox with timestamp.

        Args:
            msg: Message text
        """
        timestamp = datetime.now().strftime("%H:%M:%S")
        self.log_textbox.insert("end", f"[{timestamp}] {msg}\n")
        self.log_textbox.see("end")

    def _update_vehicle_display(self, make, year):
        """Update UI with detected vehicle info.

        Args:
            make: Vehicle manufacturer
            year: Model year
        """
        vehicle_text = f"{make}"
        if year:
            vehicle_text += f" ({year})"
        self.app.update_status(True, self.app.connection.protocol_name, self.app.connection.port)

    def _on_lang_change(self, lang=None):
        """Update all text when language changes.

        Args:
            lang: Language code (unused, kept for callback signature)
        """
        self.title_label.configure(text=t("conn_title"))
        self.config_card.winfo_children()[0].configure(text=t("conn_configuration"))
        self.port_label.configure(text=t("conn_port"))
        self.baud_label.configure(text=t("conn_baud_rate"))
        self.refresh_btn.configure(text=t("conn_refresh"))
        self.connect_btn.configure(text=t("conn_connect"))
        self.disconnect_btn.configure(text=t("conn_disconnect"))
        self.status_card.winfo_children()[0].configure(text=t("conn_status"))
        self.log_label.configure(text=t("conn_log"))
        # Update log card title
        if self.log_card.winfo_children():
            self.log_card.winfo_children()[0].configure(text=t("conn_log"))

        # Guard: don't wipe connection info if connected
        if self.app.connection and self.app.connection.is_connected():
            self.update_connection_info()
            self.status_indicator.configure(text=f"● {t('status_connected')}", text_color=COLORS["success"])
        else:
            self.status_indicator.configure(text=f"● {t('status_disconnected')}", text_color=COLORS["danger"])
            self.protocol_label.configure(text=f"{t('status_protocol')}: {t('conn_not_detected')}")
            self.elm_version_label.configure(text=f"{t('conn_elm_version')}: --")
            self.port_info_label.configure(text=f"{t('status_port')}: --")
