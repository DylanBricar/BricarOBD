"""Real-time PID monitoring frame."""

import csv
import customtkinter as ctk
import threading
from datetime import datetime

from gui.theme import COLORS, FONTS, _bind_scroll_recursive
from obd_core.pid_definitions import STANDARD_PIDS, PID_NAMES_FR
from config import LIVE_DATA_REFRESH_MS, CSV_DIR
from i18n import t, on_lang_change, get_lang


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
        self._csv_recording = False
        self._csv_file = None
        self._csv_writer = None
        self._csv_lock = threading.Lock()
        self._mfr_param_defs = {}

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

        # Get supported PIDs from the vehicle (if connected)
        supported = []
        if self.app.obd_reader and self.app.obd_reader._supported_pids:
            supported = self.app.obd_reader._supported_pids

        self.pid_checkboxes = {}
        col = 0
        for pid_code, pid_def in STANDARD_PIDS.items():
            display_name = PID_NAMES_FR.get(pid_code, pid_def.name) if get_lang() == 'fr' else pid_def.name

            # Mark unsupported PIDs visually
            is_supported = pid_code in supported if supported else True
            text_color = COLORS["text_primary"] if is_supported else COLORS["text_muted"]

            checkbox = ctk.CTkCheckBox(
                pid_scroll_frame,
                text=f"0x{pid_code:02X} {display_name}",
                font=FONTS["small"],
                text_color=text_color,
                command=lambda p=pid_code: self._on_pid_checkbox_change(p)
            )
            checkbox.grid(row=col // 5, column=col % 5, padx=8, pady=4, sticky="w")
            self.pid_checkboxes[pid_code] = checkbox

            # Only pre-select supported PIDs (or common ones if no vehicle detected)
            default_pids = supported if supported else [0x0C, 0x0D, 0x05, 0x04, 0x11, 0x0F, 0x0E]
            if pid_code in default_pids:
                checkbox.select()
                if pid_code not in self.selected_pids:
                    self.selected_pids.append(pid_code)

            col += 1

        # ── Manufacturer parameters (if vehicle detected) ──
        make = getattr(self.app, 'detected_make', None) or ""
        if make:
            # Separator
            ctk.CTkLabel(
                pid_scroll_frame, text=f"── {make} ──",
                font=FONTS["small_bold"], text_color=COLORS["accent"]
            ).grid(row=(col // 5) + 1, column=0, columnspan=5, padx=8, pady=(12, 4), sticky="w")
            col = ((col // 5) + 2) * 5  # New row

            mfr_params = self._load_manufacturer_params(make)
            self._mfr_param_defs = {}  # Store for monitoring: key → SmartParam-like dict

            for mp in mfr_params:
                key = mp["key"]
                display = mp["name"]
                self._mfr_param_defs[key] = mp

                checkbox = ctk.CTkCheckBox(
                    pid_scroll_frame,
                    text=f"[{mp['source'].upper()}] {display}",
                    font=FONTS["small"],
                    text_color=COLORS["text_primary"],
                    command=lambda k=key: self._on_mfr_checkbox_change(k)
                )
                checkbox.grid(row=col // 5, column=col % 5, padx=8, pady=4, sticky="w")
                self.pid_checkboxes[key] = checkbox  # Reuse same dict, keys won't collide (strings vs ints)
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

    def _on_mfr_checkbox_change(self, key):
        """Handle manufacturer param checkbox change."""
        checkbox = self.pid_checkboxes.get(key)
        if not checkbox:
            return
        if checkbox.get():
            if key not in self.selected_pids:
                self.selected_pids.append(key)
        else:
            if key in self.selected_pids:
                self.selected_pids.remove(key)

    def _load_manufacturer_params(self, make):
        """Load all manufacturer params from all databases."""
        params = []
        seen = set()

        try:
            # Source: Unified DB read ops
            from obd_core.unified_db import get_unified_db, make_to_manufacturer
            udb = get_unified_db()
            if udb._loaded:
                mfr = make_to_manufacturer(make)
                for op in udb.get_operations_for_manufacturer(mfr, op_type="read", limit=100):
                    sb = op.get("sentbytes", "")
                    if sb and sb not in seen:
                        seen.add(sb)
                        ecu_tx = int(op.get("ecu_tx", "0"), 16) if op.get("ecu_tx") else 0
                        ecu_rx = int(op.get("ecu_rx", "0"), 16) if op.get("ecu_rx") else 0
                        params.append({
                            "key": f"mfr_{sb}", "name": op.get("name_fr", op.get("name", sb)),
                            "source": "uds", "sentbytes": sb,
                            "ecu_tx": ecu_tx, "ecu_rx": ecu_rx, "unit": op.get("unit", ""),
                        })
        except Exception:
            pass

        try:
            # Source: Extended DIDs
            from obd_core.ecu_database import get_extended_dids, get_ecus_for_make
            dids = get_extended_dids(make)
            default_tx, default_rx = 0x7E0, 0x7E8
            for ecu in get_ecus_for_make(make):
                if "engine" in ecu.name.lower() or "inject" in ecu.name.lower():
                    default_tx, default_rx = ecu.request_id, ecu.response_id
                    break
            for did, desc in dids.items():
                sb = f"22{did:04X}"
                if sb not in seen:
                    seen.add(sb)
                    params.append({
                        "key": f"did_{did:04X}", "name": desc, "source": "uds",
                        "sentbytes": sb, "ecu_tx": default_tx, "ecu_rx": default_rx, "unit": "",
                    })
        except Exception:
            pass

        try:
            # Source: EV params
            ev_parser = getattr(self.app, 'ev_parser', None)
            if ev_parser and ev_parser.is_loaded:
                for ep in ev_parser.get_params_for_vehicle(make)[:50]:
                    sb = ep.command
                    if sb and sb not in seen:
                        seen.add(sb)
                        try:
                            ecu_tx = int(ep.ecu_header, 16) if ep.ecu_header else 0x7E4
                        except ValueError:
                            ecu_tx = 0x7E4
                        params.append({
                            "key": f"ev_{sb}_{ep.short_name}", "name": ep.name,
                            "source": "ev", "sentbytes": sb,
                            "ecu_tx": ecu_tx, "ecu_rx": ecu_tx + 8, "unit": ep.unit,
                        })
        except Exception:
            pass

        try:
            # Source: ECU DDT2000 reads (only matched ECU files)
            ecu_database = getattr(self.app, 'ecu_database', None)
            if ecu_database and ecu_database.is_loaded:
                from obd_core.ecu_identifier import KNOWN_ADDRESSES, MAKE_ADDRESSES
                allowed_addrs = MAKE_ADDRESSES.get(make.lower(), ["7A"])

                # Use matched ECU files if available (from auto-identification)
                matched_files = None
                for frame in self.app.frames.values():
                    if hasattr(frame, '_matched_ecu_files'):
                        matched_files = frame._matched_ecu_files
                        break

                for addr in allowed_addrs:
                    addr_info = KNOWN_ADDRESSES.get(addr, {})
                    can_tx = addr_info.get("can_tx", 0x7E0)
                    can_rx = addr_info.get("can_rx", 0x7E8)

                    # Only load matched file, or first 1 if no match
                    if matched_files and addr in matched_files:
                        files_to_load = [matched_files[addr]]
                    else:
                        ecus = ecu_database.find_ecus_by_address(addr)
                        files_to_load = [e.get("filename", "") for e in ecus[:1]]

                    for filename in files_to_load:
                        ecu_def = ecu_database.load_ecu_definition(filename)
                        if not ecu_def:
                            continue
                        for req in ecu_def.get_read_requests():
                            sb = req.sentbytes
                            if sb in seen:
                                continue
                            seen.add(sb)
                            for mapping in req.params:
                                param_def = ecu_def.parameters.get(mapping.param_name)
                                if not param_def:
                                    continue
                                pkey = f"ecu_{sb}_{mapping.param_name}"
                                params.append({
                                    "key": pkey, "name": mapping.param_name,
                                    "source": "ecu", "sentbytes": sb,
                                    "ecu_tx": can_tx, "ecu_rx": can_rx,
                                    "unit": param_def.unit,
                                })
        except Exception:
            pass

        try:
            # Source: Advanced ops reads
            from obd_core.advanced_operations import _ALL_OPS
            from obd_core.unified_db import make_to_manufacturer as m2m
            mfr2 = m2m(make)
            for op in _ALL_OPS.get(mfr2, []):
                if op.get("command_type") != "read":
                    continue
                sb = f"{op.get('service', 0x22):02X}{op.get('did_or_rid', 0):04X}"
                if sb not in seen:
                    seen.add(sb)
                    params.append({
                        "key": f"adv_{op.get('id', '')}",
                        "name": op["name"].get("fr", op["name"].get("en", "")),
                        "source": "uds", "sentbytes": sb,
                        "ecu_tx": op.get("ecu_tx", 0x7E0),
                        "ecu_rx": op.get("ecu_rx", 0x7E8),
                        "unit": "",
                    })
        except Exception:
            pass

        return params

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

                # Separate standard PIDs from manufacturer params
                std_pids = [p for p in self.selected_pids if isinstance(p, int)]
                mfr_keys = [p for p in self.selected_pids if isinstance(p, str)]

                if not mfr_keys:
                    # Fast path: OBD only, no connection switch needed
                    for pid_code in std_pids:
                        try:
                            value, unit = self.app.obd_reader.read_pid(pid_code)
                            if value is not None:
                                updates.append((pid_code, value, unit))
                        except Exception:
                            pass
                else:
                    # Mixed path: read EVERYTHING in one custom connection session
                    # (avoids multiple disconnect/reconnect cycles)
                    if hasattr(self, '_mfr_param_defs') and self.app.connection:
                        all_updates = self._read_all_in_one_session(std_pids, mfr_keys)
                        updates.extend(all_updates)

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
            pid_code: PID code (int) or manufacturer param key (str)
            row_idx: Row index for alternating colors
        """
        pid_def = STANDARD_PIDS.get(pid_code) if isinstance(pid_code, int) else None
        if not pid_def:
            # Manufacturer param
            mp = getattr(self, '_mfr_param_defs', {}).get(pid_code)
            if not mp:
                return
            # Create a fake pid_def-like object
            class _FakePID:
                def __init__(self, name, unit):
                    self.name = name
                    self.unit = unit
                    self.min_val = 0
                    self.max_val = 100
            pid_def = _FakePID(mp["name"], mp.get("unit", ""))

        bg_color = COLORS["bg_card"] if row_idx % 2 == 0 else COLORS["bg_secondary"]
        row_frame = ctk.CTkFrame(parent, fg_color=bg_color, corner_radius=4)
        row_frame.pack(pady=2, fill="x")

        if isinstance(pid_code, int):
            pid_label = ctk.CTkLabel(
                row_frame, text=f"0x{pid_code:02X}", font=FONTS["mono"],
                text_color=COLORS["highlight"], width=60, anchor="w"
            )
        else:
            pid_label = ctk.CTkLabel(
                row_frame, text=pid_code[:8], font=FONTS["mono"],
                text_color=COLORS["highlight"], width=60, anchor="w"
            )
        pid_label.pack(side="left", padx=6, pady=8)

        if isinstance(pid_code, int):
            fr_name = PID_NAMES_FR.get(pid_code, pid_def.name) if get_lang() == 'fr' else pid_def.name
        else:
            fr_name = pid_def.name
        name_label = ctk.CTkLabel(
            row_frame, text=fr_name, font=FONTS["body"],
            text_color=COLORS["text_primary"], width=170, anchor="w"
        )
        name_label.pack(side="left", padx=6)

        value_label = ctk.CTkLabel(
            row_frame, text="--", font=FONTS["body_bold"],
            text_color=COLORS["highlight"], width=80, anchor="e"
        )
        value_label.pack(side="left", padx=6)

        unit_label = ctk.CTkLabel(
            row_frame, text=pid_def.unit, font=FONTS["small"],
            text_color=COLORS["text_secondary"], width=50, anchor="w"
        )
        unit_label.pack(side="left", padx=6)

        min_label = ctk.CTkLabel(
            row_frame, text="--", font=FONTS["small"],
            text_color=COLORS["text_muted"], width=80, anchor="e"
        )
        min_label.pack(side="left", padx=6)

        max_label = ctk.CTkLabel(
            row_frame, text="--", font=FONTS["small"],
            text_color=COLORS["text_muted"], width=80, anchor="e"
        )
        max_label.pack(side="left", padx=6)

        trend_label = ctk.CTkLabel(
            row_frame, text="—", font=("", 16),
            text_color=COLORS["text_muted"], width=60, anchor="center"
        )
        trend_label.pack(side="left", padx=6)

        progress_bar = ctk.CTkProgressBar(row_frame, width=150)
        progress_bar.set(0.0)
        progress_bar.pack(side="left", padx=6, pady=8)

        self.pid_rows[pid_code] = {
            "frame": row_frame,
            "value_label": value_label,
            "unit_label": unit_label,
            "min_label": min_label,
            "max_label": max_label,
            "trend_label": trend_label,
            "progress_bar": progress_bar,
            "prev_value": None,
        }

    def _update_pid_row(self, pid_code, value, unit):
        """Update an existing PID row with new data.

        Args:
            pid_code: PID code (int) or manufacturer param key (str)
            value: New value
            unit: Unit string
        """
        if pid_code not in self.pid_rows:
            return

        pid_def = STANDARD_PIDS.get(pid_code) if isinstance(pid_code, int) else None
        if not pid_def:
            # Manufacturer param
            mp = getattr(self, '_mfr_param_defs', {}).get(pid_code)
            if not mp:
                return
            # Create a fake pid_def-like object
            class _FakePID:
                def __init__(self, name, unit):
                    self.name = name
                    self.unit = unit
                    self.min_val = 0
                    self.max_val = 100
            pid_def = _FakePID(mp["name"], mp.get("unit", ""))

        row = self.pid_rows[pid_code]

        # Handle both numeric and string values
        try:
            if isinstance(value, str):
                row["value_label"].configure(text=value[:40])
            else:
                row["value_label"].configure(text=f"{value:.1f}")
        except (ValueError, TypeError):
            row["value_label"].configure(text=str(value)[:40])

        row["unit_label"].configure(text=unit)

        prev = row.get("prev_value")
        if prev is not None and isinstance(prev, (int, float)) and isinstance(value, (int, float)):
            diff = value - prev
            threshold = (pid_def.max_val - pid_def.min_val) * 0.005
            if diff > threshold:
                row["trend_label"].configure(text="▲", text_color=COLORS.get("success", "#2ecc71"))
            elif diff < -threshold:
                row["trend_label"].configure(text="▼", text_color=COLORS.get("danger", "#e74c3c"))
            else:
                row["trend_label"].configure(text="—", text_color=COLORS["text_muted"])
        row["prev_value"] = value

        if pid_code in self.min_max_data:
            data = self.min_max_data[pid_code]
            try:
                if isinstance(value, (int, float)):
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
            except (ValueError, TypeError):
                pass

        self.update_count += 1

    def _read_all_in_one_session(self, std_pids, mfr_keys):
        """Read OBD PIDs + manufacturer params in ONE custom connection session.

        This avoids the 5-10s overhead of switching connections per poll cycle.
        Instead of: python-obd → disconnect → custom → read mfr → disconnect → reconnect python-obd
        We do: disconnect python-obd ONCE → read OBD raw + read mfr → reconnect ONCE
        """
        if not self.app.connection or not self.app.connection.is_connected():
            return []

        mfr_params = [self._mfr_param_defs.get(k) for k in mfr_keys if k in self._mfr_param_defs]

        def _read_all(conn):
            conn.send_command("ATE0")
            results = []

            # Phase 1: Read standard PIDs via raw OBD commands (Mode 01)
            # No need for python-obd — we're already on the raw connection
            conn.send_command("AT SH 7DF")  # OBD broadcast
            conn.send_command("AT CRA")     # Accept all responses
            from obd_core.pid_definitions import STANDARD_PIDS
            for pid_code in std_pids:
                pid_def = STANDARD_PIDS.get(pid_code)
                if not pid_def:
                    continue
                resp = conn.send_command(f"01{pid_code:02X}", timeout=2)
                if resp and "NO DATA" not in resp:
                    # Parse multi-line response, find "41 XX" pattern
                    try:
                        # Use obd_reader's parser for robust handling
                        parsed = self.app.obd_reader._parse_obd_response(resp, "41")
                        if parsed:
                            data_line = parsed[0]
                            data_hex = data_line[4:]  # Skip "41XX"
                            if data_hex:
                                data_bytes = bytes.fromhex(data_hex[:pid_def.num_bytes * 2])
                                from obd_core.pid_definitions import decode_pid
                                result = decode_pid(pid_code, data_bytes)
                                if result:
                                    value, unit = result
                                    results.append((pid_code, value, unit))
                    except (ValueError, IndexError):
                        pass

            # Phase 2: Read manufacturer params
            current_tx = None
            for mp in mfr_params:
                tx = mp.get("ecu_tx", 0x7E0)
                rx = mp.get("ecu_rx", 0x7E8)
                if tx and tx != current_tx:
                    conn.send_command(f"AT SH {tx:03X}")
                    conn.send_command(f"AT CRA {rx:03X}")
                    current_tx = tx
                resp = conn.send_command(mp["sentbytes"], timeout=3)
                if resp and "NO DATA" not in resp:
                    clean = resp.strip()[:40]
                    results.append((mp["key"], clean, mp.get("unit", "")))

            # Restore
            conn.send_command("AT D")
            conn.send_command("AT CRA")
            conn.send_command("AT H0")
            return results

        try:
            return self.app.connection.use_custom_connection(_read_all) or []
        except Exception:
            return []

    def _read_mfr_params(self, keys):
        """Read manufacturer params via custom connection."""
        if not self.app.connection or not self.app.connection.is_connected():
            return []

        params_to_read = []
        for key in keys:
            mp = self._mfr_param_defs.get(key)
            if mp:
                params_to_read.append(mp)

        if not params_to_read:
            return []

        def _read(conn):
            conn.send_command("ATE0")
            results = []
            current_tx = None
            for mp in params_to_read:
                tx = mp.get("ecu_tx", 0x7E0)
                rx = mp.get("ecu_rx", 0x7E8)
                if tx and tx != current_tx:
                    conn.send_command(f"AT SH {tx:03X}")
                    conn.send_command(f"AT CRA {rx:03X}")
                    current_tx = tx
                resp = conn.send_command(mp["sentbytes"], timeout=3)
                if resp and "NO DATA" not in resp:
                    # Store raw response as the "value"
                    clean = resp.strip()[:40]
                    results.append((mp["key"], clean, mp.get("unit", "")))
            conn.send_command("AT D")
            conn.send_command("AT CRA")
            conn.send_command("AT H0")
            return results

        try:
            return self.app.connection.use_custom_connection(_read) or []
        except Exception:
            return []

    def _apply_updates(self, updates):
        """Apply all pending updates and refresh status once.

        Args:
            updates: List of (pid_code, value, unit) tuples
        """
        for pid_code, value, unit in updates:
            self._update_pid_row(pid_code, value, unit)
        self._write_csv_row(updates)
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
        self.title_label.pack(anchor="w", padx=16, pady=(0, 4))

        ctk.CTkLabel(self, text=t("live_help"), font=FONTS["small"], text_color=COLORS["text_muted"]).pack(anchor="w", padx=16, pady=(0, 12))

        control_bar = ctk.CTkFrame(self, fg_color="transparent")
        control_bar.pack(anchor="w", padx=16, pady=(0, 12), fill="x")

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

        self.csv_btn = ctk.CTkButton(
            control_bar, text=t("csv_record"), width=120,
            fg_color=COLORS["accent"], hover_color=COLORS["accent_hover"],
            command=self._toggle_csv,
        )
        self.csv_btn.pack(side="left", padx=4)

        self.csv_status_label = ctk.CTkLabel(
            control_bar, text="", font=FONTS["small"],
            text_color=COLORS["text_muted"],
        )
        self.csv_status_label.pack(side="left", padx=4)

        self.pid_selection_frame = ctk.CTkFrame(self, fg_color=COLORS["bg_card"])
        self.pid_selection_frame.pack(padx=16, pady=8, fill="x")
        self.pid_selection_frame.pack_forget()

        self._setup_pid_selection()

        separator_line = ctk.CTkFrame(self, fg_color=COLORS["border"], height=1)
        separator_line.pack(anchor="w", fill="x", padx=16)

        header_frame = ctk.CTkFrame(self, fg_color="transparent")
        header_frame.pack(anchor="w", padx=16, pady=8, fill="x")

        header_defs = [
            (t("live_pid"),   60,  "w"),
            (t("live_name"),  170, "w"),
            (t("live_value"), 80,  "e"),
            (t("live_unit"),  50,  "w"),
            (t("live_min"),   80,  "e"),
            (t("live_max"),   80,  "e"),
            (t("live_trend"), 60,  "center"),
            (t("live_level"), 150, "w"),
        ]

        for text, width, anchor in header_defs:
            label = ctk.CTkLabel(
                header_frame, text=text, font=FONTS["body_bold"],
                text_color=COLORS["text_secondary"], width=width, anchor=anchor
            )
            label.pack(side="left", padx=6)

        self.table_frame = ctk.CTkScrollableFrame(self, fg_color="transparent")
        self.table_frame.pack(fill="both", expand=True, padx=16, pady=8)
        self.after(500, lambda: _bind_scroll_recursive(self.table_frame))

        self.status_label = ctk.CTkLabel(
            self, text="Monitoring 0 PIDs | Update rate: 500ms | Samples: 0",
            font=FONTS["small"], text_color=COLORS["text_muted"]
        )
        self.status_label.pack(anchor="w", padx=16, pady=8)

    def _toggle_csv(self):
        """Toggle CSV recording."""
        if self._csv_recording:
            self._stop_csv()
        else:
            self._start_csv()

    def _start_csv(self):
        """Start CSV recording."""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filepath = CSV_DIR / f"live_data_{timestamp}.csv"
        try:
            CSV_DIR.mkdir(parents=True, exist_ok=True)
        except OSError:
            pass
        self._csv_file = open(filepath, "w", newline="", encoding="utf-8")
        self._csv_writer = csv.writer(self._csv_file)
        header = ["timestamp"]
        for pid_code in self.selected_pids:
            if isinstance(pid_code, int):
                pid_def = STANDARD_PIDS.get(pid_code)
                name = pid_def.name if pid_def else f"PID_{pid_code:02X}"
            else:
                mp = self._mfr_param_defs.get(pid_code)
                name = mp["name"] if mp else pid_code
            header.append(name)
        self._csv_writer.writerow(header)
        self._csv_recording = True
        self._csv_filepath = filepath
        self.csv_btn.configure(text=t("csv_stop"), fg_color=COLORS["danger"])
        self.csv_status_label.configure(text=t("csv_recording"), text_color=COLORS["danger"])

    def _stop_csv(self):
        """Stop CSV recording."""
        with self._csv_lock:
            self._csv_recording = False
            if self._csv_file:
                self._csv_file.flush()
                self._csv_file.close()
                self._csv_file = None
                self._csv_writer = None
        self.csv_btn.configure(text=t("csv_record"), fg_color=COLORS["accent"])
        filename = self._csv_filepath.name if hasattr(self, '_csv_filepath') else ""
        self.csv_status_label.configure(text=t("csv_saved", file=filename), text_color=COLORS["success"])

    def _write_csv_row(self, updates):
        """Write a row of PID values to CSV if recording."""
        with self._csv_lock:
            if not self._csv_recording or not self._csv_writer:
                return
            row = [datetime.now().isoformat()]
            val_map = {pid: val for pid, val, _ in updates}
            for pid_code in self.selected_pids:
                val = val_map.get(pid_code)
                row.append(f"{val:.2f}" if isinstance(val, (int, float)) else str(val) if val is not None else "")
            self._csv_writer.writerow(row)
            self._csv_file.flush()

    def _on_lang_change(self, lang=None):
        """Update text on language change."""
        if not self.winfo_exists():
            return
        was_monitoring = self.monitoring
        if was_monitoring:
            self.stop_monitoring()
        if self._csv_recording:
            self._stop_csv()
        for widget in self.winfo_children():
            widget.destroy()
        self._build_content()
        if was_monitoring:
            self.start_monitoring()
