"""Real-time PID monitoring frame."""

import customtkinter as ctk
import threading
from gui.theme import COLORS, FONTS, _bind_scroll_recursive
from obd_core.pid_definitions import STANDARD_PIDS
from config import LIVE_DATA_REFRESH_MS
from i18n import t, on_lang_change


class LiveDataFrame(ctk.CTkFrame):
    """Frame for real-time PID data monitoring."""

    def __init__(self, parent, app):
        """Initialize live data frame.

        Args:
            parent: Parent widget
            app: Main application instance with pid_reader
        """
        super().__init__(parent, fg_color=COLORS["bg_primary"])
        self.app = app
        self.monitoring = False
        self.refresh_ms = LIVE_DATA_REFRESH_MS
        self.selected_pids = [0x0C, 0x0D, 0x05, 0x04, 0x11]
        self.min_max_data = {}
        self.pid_rows = {}
        self.pid_selection_visible = False
        self.update_count = 0
        self._update_running = False
        self.title_label = None
        self.update_id = None

        self._setup_ui()
        on_lang_change(self._on_lang_change)

    def _setup_ui(self):
        """Setup the live data frame UI."""
        self._build_content()

    def _setup_pid_selection(self):
        """Setup the PID selection panel."""
        header_frame = ctk.CTkFrame(self.pid_selection_frame, fg_color="transparent")
        header_frame.pack(anchor="w", padx=12, pady=(12, 4), fill="x")

        label = ctk.CTkLabel(
            header_frame, text=t("live_select_label"),
            font=FONTS["body_bold"], text_color=COLORS["text_primary"]
        )
        label.pack(side="left")

        ctk.CTkButton(
            header_frame, text=t("live_select_all"), width=100, height=24,
            font=FONTS["small"], command=self.select_all_pids
        ).pack(side="right", padx=4)

        ctk.CTkButton(
            header_frame, text=t("live_deselect_all"), width=100, height=24,
            font=FONTS["small"], command=self.deselect_all_pids
        ).pack(side="right", padx=4)

        pid_scroll_frame = ctk.CTkScrollableFrame(self.pid_selection_frame, fg_color="transparent")
        pid_scroll_frame.pack(anchor="w", padx=12, pady=8, fill="x", expand=True)
        self.after(500, lambda: _bind_scroll_recursive(pid_scroll_frame))

        self.pid_checkboxes = {}
        col = 0
        for pid_code, pid_def in STANDARD_PIDS.items():
            checkbox = ctk.CTkCheckBox(
                pid_scroll_frame,
                text=f"0x{pid_code:02X} {pid_def.name}",
                font=FONTS["small"],
                command=lambda p=pid_code: self._on_pid_checkbox_change(p)
            )
            checkbox.grid(row=col // 5, column=col % 5, padx=8, pady=4, sticky="w")
            self.pid_checkboxes[pid_code] = checkbox

            checkbox.select()
            if pid_code not in self.selected_pids:
                self.selected_pids.append(pid_code)

            col += 1

    def _on_pid_checkbox_change(self, pid_code):
        """Handle PID checkbox changes."""
        checkbox = self.pid_checkboxes[pid_code]
        if checkbox.get():
            if pid_code not in self.selected_pids:
                self.selected_pids.append(pid_code)
        else:
            if pid_code in self.selected_pids:
                self.selected_pids.remove(pid_code)

    def _on_refresh_rate_change(self, value):
        """Handle refresh rate combo changes."""
        rate_map = {"250ms": 250, "500ms": 500, "1000ms": 1000, "2000ms": 2000}
        self.refresh_ms = rate_map.get(value, LIVE_DATA_REFRESH_MS)
        self._update_status()

    def toggle_monitoring(self):
        """Toggle monitoring on/off."""
        if self.monitoring:
            self.stop_monitoring()
        else:
            self.start_monitoring()

    def start_monitoring(self):
        """Start periodic PID reads."""
        self.monitoring = True
        self.start_stop_btn.configure(text=t("live_stop"), fg_color=COLORS["danger"])
        self.min_max_data = {}
        self.update_count = 0

        for pid_code in self.selected_pids:
            self.min_max_data[pid_code] = {"min": float('inf'), "max": float('-inf'), "samples": 0}

        self._create_initial_rows()
        self._schedule_update()

    def stop_monitoring(self):
        """Stop periodic PID reads."""
        self.monitoring = False
        self.start_stop_btn.configure(text=t("live_start"), fg_color=COLORS["success"])
        if hasattr(self, 'update_id') and self.update_id:
            self.after_cancel(self.update_id)
            self.update_id = None

    def toggle_monitoring_state(self):
        """Toggle start/stop state."""
        self.toggle_monitoring()

    def _schedule_update(self):
        """Schedule the next PID update."""
        if self.monitoring:
            self.update_live_data()
            self.update_id = self.after(self.refresh_ms, self._schedule_update)

    def update_live_data(self):
        """Read selected PIDs and update table."""
        if not self.app.obd_reader:
            return
        if not self.app.connection or not self.app.connection.is_connected():
            return
        if not self.monitoring or not self.selected_pids:
            return
        if self._update_running:
            return

        def task():
            self._update_running = True
            try:
                updates = []
                for pid_code in self.selected_pids:
                    try:
                        value, unit = self.app.obd_reader.read_pid(pid_code)
                        if value is not None:
                            updates.append((pid_code, value, unit))
                    except Exception:
                        pass

                if updates:
                    self.after(0, self._apply_updates, updates)
            finally:
                self._update_running = False

        thread = threading.Thread(target=task, daemon=True)
        thread.start()

    def _create_initial_rows(self):
        """Create initial data rows for selected PIDs."""
        for widget in self.table_frame.winfo_children():
            widget.destroy()

        self.pid_rows = {}

        for idx, pid_code in enumerate(self.selected_pids):
            self._create_pid_row(self.table_frame, pid_code, idx)

    def _create_pid_row(self, parent, pid_code, row_idx):
        """Create a single PID data row.

        Args:
            parent: Parent frame
            pid_code: PID code (int)
            row_idx: Row index for alternating colors
        """
        pid_def = STANDARD_PIDS.get(pid_code)
        if not pid_def:
            return

        bg_color = COLORS["bg_card"] if row_idx % 2 == 0 else COLORS["bg_secondary"]
        row_frame = ctk.CTkFrame(parent, fg_color=bg_color, corner_radius=4)
        row_frame.pack(pady=2, fill="x")

        pid_label = ctk.CTkLabel(
            row_frame, text=f"0x{pid_code:02X}", font=FONTS["mono"],
            text_color=COLORS["highlight"], width=60, anchor="w"
        )
        pid_label.pack(side="left", padx=8, pady=8)

        name_label = ctk.CTkLabel(
            row_frame, text=pid_def.name, font=FONTS["body"],
            text_color=COLORS["text_primary"], width=150, anchor="w"
        )
        name_label.pack(side="left", padx=8)

        value_label = ctk.CTkLabel(
            row_frame, text="--", font=FONTS["body_bold"],
            text_color=COLORS["highlight"], width=100, anchor="w"
        )
        value_label.pack(side="left", padx=8)

        unit_label = ctk.CTkLabel(
            row_frame, text=pid_def.unit, font=FONTS["small"],
            text_color=COLORS["text_secondary"], width=60, anchor="w"
        )
        unit_label.pack(side="left", padx=8)

        min_label = ctk.CTkLabel(
            row_frame, text="--", font=FONTS["small"],
            text_color=COLORS["text_muted"], width=100, anchor="w"
        )
        min_label.pack(side="left", padx=8)

        max_label = ctk.CTkLabel(
            row_frame, text="--", font=FONTS["small"],
            text_color=COLORS["text_muted"], width=100, anchor="w"
        )
        max_label.pack(side="left", padx=8)

        progress_bar = ctk.CTkProgressBar(row_frame, width=150)
        progress_bar.set(0.0)
        progress_bar.pack(side="left", padx=8, pady=8)

        self.pid_rows[pid_code] = {
            "frame": row_frame,
            "value_label": value_label,
            "unit_label": unit_label,
            "min_label": min_label,
            "max_label": max_label,
            "progress_bar": progress_bar
        }

    def _update_pid_row(self, pid_code, value, unit):
        """Update an existing PID row with new data.

        Args:
            pid_code: PID code
            value: New value
            unit: Unit string
        """
        if pid_code not in self.pid_rows:
            return

        pid_def = STANDARD_PIDS.get(pid_code)
        if not pid_def:
            return

        row = self.pid_rows[pid_code]
        row["value_label"].configure(text=f"{value:.1f}")
        row["unit_label"].configure(text=unit)

        if pid_code in self.min_max_data:
            data = self.min_max_data[pid_code]
            if value < data["min"]:
                data["min"] = value
            if value > data["max"]:
                data["max"] = value
            data["samples"] += 1

            row["min_label"].configure(text=f"{data['min']:.1f}")
            row["max_label"].configure(text=f"{data['max']:.1f}")

            range_span = pid_def.max_val - pid_def.min_val
            if range_span > 0:
                progress = (value - pid_def.min_val) / range_span
                progress = max(0.0, min(1.0, progress))
                row["progress_bar"].set(progress)

        self.update_count += 1

    def _apply_updates(self, updates):
        """Apply all pending updates and refresh status once.

        Args:
            updates: List of (pid_code, value, unit) tuples
        """
        for pid_code, value, unit in updates:
            self._update_pid_row(pid_code, value, unit)
        self._update_status()

    def _update_status(self):
        """Update the status label."""
        status_text = (
            f"Monitoring {len(self.selected_pids)} PIDs | "
            f"Update rate: {self.refresh_ms}ms | "
            f"Samples: {self.update_count}"
        )
        self.status_label.configure(text=status_text)

    def toggle_pid_selection(self):
        """Show/hide PID selection panel."""
        if self.pid_selection_visible:
            self.pid_selection_frame.pack_forget()
            self.pid_selection_visible = False
        else:
            self.pid_selection_frame.pack(padx=16, pady=8, fill="x")
            self.pid_selection_visible = True

    def select_all_pids(self):
        """Select all available PIDs."""
        self.selected_pids = list(STANDARD_PIDS.keys())
        for checkbox in self.pid_checkboxes.values():
            checkbox.select()

    def deselect_all_pids(self):
        """Deselect all PIDs."""
        self.selected_pids = []
        for checkbox in self.pid_checkboxes.values():
            checkbox.deselect()

    def get_selected_pids(self):
        """Get list of currently selected PIDs.

        Returns:
            List of selected PID codes
        """
        return self.selected_pids.copy()

    def reset_min_max(self):
        """Reset tracked min/max values."""
        for pid_code in self.min_max_data:
            self.min_max_data[pid_code] = {"min": float('inf'), "max": float('-inf'), "samples": 0}
        self.update_count = 0
        self._update_status()

    def _build_content(self):
        """Extract UI content building (called by both __init__ and _on_lang_change)."""
        self.title_label = ctk.CTkLabel(
            self, text=t("live_title"), font=FONTS["heading"],
            text_color=COLORS["text_primary"]
        )
        self.title_label.pack(anchor="w", padx=16, pady=(16, 8))

        ctk.CTkLabel(self, text=t("live_help"), font=FONTS["small"], text_color=COLORS["text_muted"]).pack(anchor="w", padx=16, pady=(0, 8))

        control_bar = ctk.CTkFrame(self, fg_color="transparent")
        control_bar.pack(anchor="w", padx=16, pady=8, fill="x")

        self.start_stop_btn = ctk.CTkButton(
            control_bar, text=t("live_start"), fg_color=COLORS["success"],
            command=self.toggle_monitoring, width=150
        )
        self.start_stop_btn.pack(side="left", padx=4)

        ctk.CTkLabel(
            control_bar, text=t("live_refresh"), font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        ).pack(side="left", padx=(20, 4))

        self.refresh_combo = ctk.CTkComboBox(
            control_bar, values=["250ms", "500ms", "1000ms", "2000ms"],
            command=self._on_refresh_rate_change, width=100
        )
        self.refresh_combo.set("500ms")
        self.refresh_combo.pack(side="left", padx=4)

        ctk.CTkButton(
            control_bar, text=t("live_select_pids"), width=120,
            command=self.toggle_pid_selection
        ).pack(side="left", padx=4)

        self.pid_selection_frame = ctk.CTkFrame(self, fg_color=COLORS["bg_card"])
        self.pid_selection_frame.pack(padx=16, pady=8, fill="x")
        self.pid_selection_frame.pack_forget()

        self._setup_pid_selection()

        separator_line = ctk.CTkFrame(self, fg_color=COLORS["border"], height=1)
        separator_line.pack(anchor="w", fill="x", padx=16)

        header_frame = ctk.CTkFrame(self, fg_color="transparent")
        header_frame.pack(anchor="w", padx=16, pady=8, fill="x")

        header_texts = [t("live_pid"), t("live_name"), t("live_value"), t("live_unit"), t("live_min"), t("live_max"), t("live_trend")]
        header_widths = [60, 150, 100, 60, 100, 100, 150]

        for text, width in zip(header_texts, header_widths):
            label = ctk.CTkLabel(
                header_frame, text=text, font=FONTS["body_bold"],
                text_color=COLORS["text_secondary"], width=width, anchor="w"
            )
            label.pack(side="left", padx=4)

        self.table_frame = ctk.CTkScrollableFrame(self, fg_color="transparent")
        self.table_frame.pack(fill="both", expand=True, padx=16, pady=8)
        self.after(500, lambda: _bind_scroll_recursive(self.table_frame))

        self.status_label = ctk.CTkLabel(
            self, text="Monitoring 0 PIDs | Update rate: 500ms | Samples: 0",
            font=FONTS["small"], text_color=COLORS["text_muted"]
        )
        self.status_label.pack(anchor="w", padx=16, pady=8)

    def _on_lang_change(self, lang=None):
        """Update text on language change."""
        was_monitoring = self.monitoring
        if was_monitoring:
            self.stop_monitoring()
        for widget in self.winfo_children():
            widget.destroy()
        self._build_content()
        if was_monitoring:
            self.start_monitoring()
