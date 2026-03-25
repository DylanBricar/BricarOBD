"""Smart Parameters tab — unified monitoring of OBD PIDs, manufacturer UDS operations, and ECU definitions."""

import customtkinter as ctk
import threading
from collections import deque
from dataclasses import dataclass
from typing import Dict, List, Optional, Set

from gui.theme import COLORS, FONTS, _bind_scroll_recursive
from obd_core.database_reader import ECUDatabase, ECUDefinition, ECURequest
from obd_core.ecu_identifier import KNOWN_ADDRESSES
from obd_core.pid_definitions import STANDARD_PIDS, PID_NAMES_FR
from i18n import t, on_lang_change, get_lang


@dataclass
class SmartParam:
    """Unified parameter representation combining all sources."""
    name: str           # Display name
    key: str            # Unique key (e.g., "obd_0x0C", "psa_222282", "ecu_21A4_temps_injection")
    source: str         # "obd" | "uds" | "ecu"
    category: str       # "engine" | "fuel" | "sensors" | "electrical" | "vehicle" | "config"
    unit: str
    min_val: float = 0
    max_val: float = 100
    # For OBD PIDs
    pid: int = 0
    # For UDS/ECU operations
    sentbytes: str = ""
    ecu_tx: int = 0
    ecu_rx: int = 0
    ecu_request: Optional[ECURequest] = None
    ecu_def: Optional[ECUDefinition] = None
    param_name: str = ""   # name in ECU definition
    comment: str = ""


class MiniGraph(ctk.CTkCanvas):
    """Minimal inline graph widget (150x35px) for parameter value history."""

    def __init__(self, parent, width=150, height=35, color=None, max_samples=30):
        super().__init__(parent, width=width, height=height,
                        bg=COLORS["bg_card"], highlightthickness=0)
        self.w = width
        self.h = height
        self.color = color or COLORS["success"]
        self.max_samples = max_samples
        self.data = deque(maxlen=max_samples)

    def add_value(self, value):
        """Append a value and redraw."""
        self.data.append(value)
        self.delete("all")
        self._draw()

    def reset(self):
        """Clear all data."""
        self.data.clear()
        self.delete("all")
        self._draw()

    def _draw(self):
        """Draw a minimal line graph."""
        pad_l, pad_r, pad_t, pad_b = 4, 4, 2, 2
        gw = self.w - pad_l - pad_r
        gh = self.h - pad_t - pad_b

        # Background
        self.create_rectangle(0, 0, self.w, self.h, fill=COLORS["bg_card"], outline=COLORS["border"])

        if len(self.data) < 2:
            return

        # Compute min/max for scaling
        min_val = min(self.data)
        max_val = max(self.data)
        val_range = max(max_val - min_val, 1)

        # Draw line
        points = []
        for i, val in enumerate(self.data):
            x = pad_l + int(gw * i / (len(self.data) - 1)) if len(self.data) > 1 else pad_l
            normalized = (val - min_val) / val_range if val_range > 0 else 0.5
            y = pad_t + gh - int(gh * normalized)
            points.append((x, y))

        # Draw polyline
        if len(points) >= 2:
            for i in range(len(points) - 1):
                self.create_line(points[i][0], points[i][1], points[i+1][0], points[i+1][1],
                               fill=self.color, width=1)


class DatabaseFrame(ctk.CTkFrame):
    """Unified Smart Parameters tab for OBD PIDs, manufacturer operations, and ECU definitions."""

    def __init__(self, parent, app):
        """Initialize Smart Parameters frame."""
        super().__init__(parent, fg_color=COLORS["bg_primary"])
        self.app = app
        self.ecu_database = getattr(app, 'ecu_database', None)

        # Parameter management
        self.all_params: List[SmartParam] = []
        self.selected_keys: Set[str] = set()
        self.monitored_graphs: Dict[str, MiniGraph] = {}
        self.param_rows: Dict[str, ctk.CTkFrame] = {}

        # Monitoring state
        self.monitoring = False
        self.refresh_ms = 500
        self.update_count = 0
        self._update_running = False
        self.update_id = None

        # UI components
        self.title_label = None
        self.category_filter = "All"
        self.status_label = None
        self.monitor_status_label = None

        self._setup_ui()
        on_lang_change(self._on_lang_change)

    def _setup_ui(self):
        """Setup the UI."""
        self._build_content()

    def _build_content(self):
        """Build the frame content (called by __init__ and _on_lang_change)."""
        # Title
        self.title_label = ctk.CTkLabel(
            self, text="Smart Parameters", font=FONTS["heading"],
            text_color=COLORS["text_primary"]
        )
        self.title_label.pack(anchor="w", padx=16, pady=(0, 4))

        ctk.CTkLabel(
            self, text=t("db_help"), font=FONTS["small"],
            text_color=COLORS["text_muted"]
        ).pack(anchor="w", padx=16, pady=(0, 12))

        # Top bar: category filter + monitor controls
        top_bar = ctk.CTkFrame(self, fg_color="transparent")
        top_bar.pack(anchor="w", padx=16, pady=(0, 12), fill="x")

        # Category filter buttons
        cat_frame = ctk.CTkFrame(top_bar, fg_color="transparent")
        cat_frame.pack(side="left")

        for cat in ["All", "Engine", "Fuel", "Sensors", "Electrical", "Vehicle", "Config"]:
            btn = ctk.CTkButton(
                cat_frame, text=cat, width=70, height=32, font=FONTS["small"],
                command=lambda c=cat: self._on_category_filter(c)
            )
            btn.pack(side="left", padx=2)

        # Monitoring controls
        ctrl_frame = ctk.CTkFrame(top_bar, fg_color="transparent")
        ctrl_frame.pack(side="right")

        self.start_stop_btn = ctk.CTkButton(
            ctrl_frame, text="Start Monitor", fg_color=COLORS["success"],
            command=self._toggle_monitoring, width=140
        )
        self.start_stop_btn.pack(side="left", padx=4)

        ctk.CTkLabel(
            ctrl_frame, text="Refresh:", font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        ).pack(side="left", padx=(20, 4))

        self.refresh_combo = ctk.CTkComboBox(
            ctrl_frame, values=["250ms", "500ms", "1000ms", "2000ms"],
            command=self._on_refresh_rate_change, width=100
        )
        self.refresh_combo.set("500ms")
        self.refresh_combo.pack(side="left", padx=4)

        # Status line
        self.status_label = ctk.CTkLabel(
            self, text="", font=FONTS["small"],
            text_color=COLORS["text_secondary"]
        )
        self.status_label.pack(anchor="w", padx=16, pady=(0, 8))

        # Separator
        sep = ctk.CTkFrame(self, fg_color=COLORS["border"], height=1)
        sep.pack(anchor="w", fill="x", padx=16, pady=8)

        # Scrollable parameter list
        self.params_scroll_frame = ctk.CTkScrollableFrame(self, fg_color="transparent")
        self.params_scroll_frame.pack(fill="both", expand=True, padx=16, pady=8)
        self.after(500, lambda: _bind_scroll_recursive(self.params_scroll_frame))

        # Placeholder
        self.no_params_label = ctk.CTkLabel(
            self.params_scroll_frame, text="Connect and detect vehicle first, or scroll to see standard PIDs",
            font=FONTS["small"], text_color=COLORS["text_muted"]
        )
        self.no_params_label.pack(pady=32)

        # Bottom status
        self.monitor_status_label = ctk.CTkLabel(
            self, text="", font=FONTS["small"],
            text_color=COLORS["text_muted"]
        )
        self.monitor_status_label.pack(anchor="w", padx=16, pady=8)

    def on_frame_shown(self):
        """Called when user navigates to this tab. Auto-identifies ECUs if connected."""
        make = getattr(self.app, 'detected_make', None) or ""

        # Auto-identify ECUs on first visit if connected (runs in background)
        if (make and not getattr(self, '_ecus_identified', False)
                and self.app.connection and self.app.connection.is_connected()
                and self.ecu_database and self.ecu_database.is_loaded):
            self._ecus_identified = True
            self.status_label.configure(text="Identifying ECUs...", text_color=COLORS["text_secondary"])
            import threading
            def _auto_identify():
                try:
                    from obd_core.ecu_identifier import ECUIdentifier
                    identifier = ECUIdentifier(self.ecu_database)
                    def _scan(conn):
                        conn.send_command("ATE0")
                        return identifier.identify_ecus(conn, make=make)
                    matched_ecus = self.app.connection.use_custom_connection(_scan)
                    if matched_ecus:
                        # Store matched ECU definitions for precise param loading
                        self._matched_ecu_files = {}
                        for ecu in matched_ecus:
                            if ecu.get("matched") and ecu.get("filename"):
                                self._matched_ecu_files[ecu["address"]] = ecu["filename"]
                        self.after(0, self._on_ecus_identified, matched_ecus)
                    else:
                        self.after(0, self._load_and_show)
                except Exception as e:
                    logger.error(f"ECU identification failed: {e}")
                    self.after(0, self._load_and_show)
            threading.Thread(target=_auto_identify, daemon=True).start()
        else:
            self._load_and_show()

    def _on_ecus_identified(self, matched_ecus):
        """Handle ECU identification results."""
        matched = [e for e in matched_ecus if e.get("matched")]
        total = len(matched_ecus)
        self.status_label.configure(
            text=f"Identified {len(matched)}/{total} ECUs",
            text_color=COLORS["success"] if matched else COLORS["text_secondary"]
        )
        for ecu in matched:
            logger.info(f"ECU [{ecu['address']}] → {ecu['ecuname']} ({ecu['filename']})")
        self._load_and_show()

    def _load_and_show(self):
        """Load params and rebuild UI."""
        self._load_all_params()
        self._rebuild_param_list()

    def _load_all_params(self):
        """Populate self.all_params from ALL data sources, filtered by detected vehicle."""
        self.all_params = []
        self.selected_keys.clear()  # Fix #8: reset selection on reload

        make = getattr(self.app, 'detected_make', None) or ""
        vehicle = getattr(self.app, 'detected_vehicle', None) or {}
        model_hint = vehicle.get("model_year", "")

        # ── Source 1: Standard OBD-II PIDs (always available) ──
        for pid_code, pid_def in STANDARD_PIDS.items():
            display_name = PID_NAMES_FR.get(pid_code, pid_def.name) if get_lang() == 'fr' else pid_def.name
            self.all_params.append(SmartParam(
                name=display_name, key=f"obd_{pid_code:02X}", source="obd",
                category=self._categorize(pid_def.name, pid_def.unit),
                unit=pid_def.unit, min_val=pid_def.min_val, max_val=pid_def.max_val,
                pid=pid_code, comment=pid_def.description
            ))

        if not make:
            self.status_label.configure(
                text=f"{len(self.all_params)} standard PIDs (connect vehicle for more)",
                text_color=COLORS["text_secondary"]
            )
            return

        # ── Source 2: Unified DB operations (filtered by manufacturer) ──
        try:
            from obd_core.unified_db import get_unified_db, make_to_manufacturer
            udb = get_unified_db()
            if udb.is_available():
                if not udb._loaded:
                    udb.load()
                if udb._loaded:
                    manufacturer = make_to_manufacturer(make)
                    mfr_ops = udb.get_operations_for_manufacturer(manufacturer, op_type="read", limit=200)
                    for op in mfr_ops:
                        ecu_tx = int(op.get("ecu_tx", "0"), 16) if op.get("ecu_tx") else 0
                        ecu_rx = int(op.get("ecu_rx", "0"), 16) if op.get("ecu_rx") else 0
                        self.all_params.append(SmartParam(
                            name=op.get("name_fr", op.get("name", "")),
                            key=f"uds_{op.get('sentbytes', '')}_{op.get('did', '')}",
                            source="uds", category=self._categorize(op.get("name", ""), ""),
                            unit=op.get("unit", ""), sentbytes=op.get("sentbytes", ""),
                            ecu_tx=ecu_tx, ecu_rx=ecu_rx,
                            comment=op.get("notes_fr", op.get("notes", ""))
                        ))
        except Exception:
            pass

        # ── Source 3: Advanced operations (verified, with proper ECU addresses) ──
        try:
            from obd_core.advanced_operations import _ALL_OPS
            from obd_core.unified_db import make_to_manufacturer
            mfr = make_to_manufacturer(make)
            adv_ops = _ALL_OPS.get(mfr, [])
            for op in adv_ops:
                if op.get("command_type") != "read":
                    continue
                self.all_params.append(SmartParam(
                    name=op["name"].get("fr", op["name"].get("en", "")),
                    key=f"adv_{op.get('id', '')}",
                    source="uds", category=self._categorize(op["name"].get("en", ""), ""),
                    unit="", sentbytes=f"{op.get('service', 0x22):02X}{op.get('did_or_rid', 0):04X}",
                    ecu_tx=op.get("ecu_tx", 0), ecu_rx=op.get("ecu_rx", 0),
                    comment=op.get("desc", {}).get("fr", "")
                ))
        except Exception:
            pass

        # ── Source 4: ECU definitions (use matched files if available, else filter by make) ──
        if self.ecu_database and self.ecu_database.is_loaded:
            from obd_core.ecu_identifier import MAKE_ADDRESSES

            matched_files = getattr(self, '_matched_ecu_files', {})
            allowed_addrs = MAKE_ADDRESSES.get(make.lower(), ["7A"])

            for addr in allowed_addrs:
                addr_info = KNOWN_ADDRESSES.get(addr, {})

                # Prefer matched file (from 2180 identification)
                if addr in matched_files:
                    ecu_defs_to_load = [matched_files[addr]]
                else:
                    # Fallback: first 2 candidates at this address
                    ecus = self.ecu_database.find_ecus_by_address(addr)
                    ecu_defs_to_load = [e.get("filename", "") for e in ecus[:2]]

                for filename in ecu_defs_to_load:
                    ecu_def = self.ecu_database.load_ecu_definition(filename)
                    if not ecu_def:
                        continue
                    for request in ecu_def.get_read_requests()[:5]:
                        for mapping in request.params:
                            param_def = ecu_def.parameters.get(mapping.param_name)
                            if not param_def:
                                continue
                            self.all_params.append(SmartParam(
                                name=mapping.param_name,
                                key=f"ecu_{request.sentbytes}_{mapping.param_name}",
                                source="ecu",
                                category=self._categorize(mapping.param_name, param_def.unit),
                                unit=param_def.unit, sentbytes=request.sentbytes,
                                ecu_tx=addr_info.get("can_tx", 0x7E0),
                                ecu_rx=addr_info.get("can_rx", 0x7E8),
                                ecu_request=request, ecu_def=ecu_def,
                                param_name=mapping.param_name, comment=param_def.comment
                            ))

        # ── Source 5: EV/Battery parameters (cached, filtered by make+model) ──
        try:
            ev_parser = getattr(self.app, 'ev_parser', None)
            if ev_parser and ev_parser.is_loaded:
                ev_params = ev_parser.get_params_for_vehicle(make, model_hint)
                for ep in ev_params[:100]:
                    try:
                        ecu_tx = int(ep.ecu_header, 16) if ep.ecu_header else 0x7E4
                    except ValueError:
                        ecu_tx = 0x7E4
                    # Use proper response ID mapping, not just +8
                    _RX_MAP = {0x7E0: 0x7E8, 0x7E4: 0x7EC, 0x7E1: 0x7E9, 0x7C6: 0x7CE,
                               0x75D: 0x65D, 0x6A8: 0x688}
                    ecu_rx = _RX_MAP.get(ecu_tx, ecu_tx + 8)
                    self.all_params.append(SmartParam(
                        name=ep.name, key=f"ev_{ep.command}_{ep.short_name}",
                        source="ecu", category=self._categorize(ep.name, ep.unit),
                        unit=ep.unit, min_val=ep.min_val, max_val=ep.max_val,
                        sentbytes=ep.command, ecu_tx=ecu_tx, ecu_rx=ecu_rx,
                        comment=f"{ep.equation[:50]}"
                    ))
        except Exception:
            pass

        # ── Source 6: Manufacturer extended DIDs (PSA, Renault, BMW, VAG) ──
        try:
            from obd_core.ecu_database import (PSA_EXTENDED_DIDS, get_extended_dids,
                                                 get_ecus_for_make)
            ext_dids = get_extended_dids(make)
            if ext_dids:
                # Get the ECU addresses for this make
                make_ecus = get_ecus_for_make(make)
                # Default to engine ECU for DID reads
                default_tx = 0x7E0
                default_rx = 0x7E8
                for ecu in make_ecus:
                    if "engine" in ecu.name.lower() or "inject" in ecu.name.lower():
                        default_tx = ecu.request_id
                        default_rx = ecu.response_id
                        break

                seen_dids = {p.sentbytes for p in self.all_params if p.source == "uds"}
                for did, description in ext_dids.items():
                    sentbytes = f"22{did:04X}"
                    if sentbytes in seen_dids:
                        continue  # Already loaded from another source
                    self.all_params.append(SmartParam(
                        name=description, key=f"did_{did:04X}",
                        source="uds", category=self._categorize(description, ""),
                        unit="", sentbytes=sentbytes,
                        ecu_tx=default_tx, ecu_rx=default_rx,
                        comment=f"DID 0x{did:04X}"
                    ))
        except Exception:
            pass

        # ── Source 7: Vehicle profiles (90 vehicles with ECU addresses) ──
        try:
            from obd_core.unified_db import get_unified_db
            udb = get_unified_db()
            if udb._loaded:
                # Search for matching vehicle profile
                vehicle_name = vehicle.get("make", "")
                matches = udb.search_vehicle_profile(vehicle_name) if vehicle_name else []
                if matches:
                    _, ecus_list = matches[0]
                    # Store profile ECU addresses on app for other tabs to use
                    if not hasattr(self.app, '_vehicle_profile_ecus'):
                        self.app._vehicle_profile_ecus = ecus_list
        except Exception:
            pass

        # ── Deduplicate ──
        # Layer 1: Remove exact key duplicates (first occurrence wins)
        # Layer 2: For UDS/ECU, same sentbytes = same command = duplicate
        #          (even if names differ between FR/EN or sources)
        seen_keys = set()
        seen_sentbytes_cmd = set()  # sentbytes alone (for UDS/ECU same command)
        seen_sentbytes_param = set()  # sentbytes+param (for ECU multi-param responses)
        deduped = []
        for p in self.all_params:
            if p.key in seen_keys:
                continue
            if p.source in ("ecu", "uds") and p.sentbytes:
                if p.param_name:
                    # ECU params: same sentbytes + same param_name = dupe
                    dk = f"{p.sentbytes}_{p.param_name}"
                    if dk in seen_sentbytes_param:
                        continue
                    seen_sentbytes_param.add(dk)
                else:
                    # UDS ops: same sentbytes = same command = dupe
                    if p.sentbytes in seen_sentbytes_cmd:
                        continue
                    seen_sentbytes_cmd.add(p.sentbytes)
            seen_keys.add(p.key)
            deduped.append(p)

        before = len(self.all_params)
        self.all_params = deduped

        self.status_label.configure(
            text=f"{len(self.all_params)} params ({make or 'generic'}) [{before - len(self.all_params)} duplicates removed]",
            text_color=COLORS["text_secondary"]
        )

    def _categorize(self, name: str, unit: str) -> str:
        """Categorize parameter by keywords."""
        text = (name + " " + unit).lower()

        if any(x in text for x in ["rpm", "régime", "moteur", "engine", "charge", "couple", "torque"]):
            return "Engine"
        elif any(x in text for x in ["inject", "carburant", "fuel", "richesse", "rampe", "rail", "canister", "purge"]):
            return "Fuel"
        elif any(x in text for x in ["sonde", "o2", "lambda", "papillon", "throttle", "pression", "pressure", "temperature", "temp"]):
            return "Sensors"
        elif any(x in text for x in ["battery", "batterie", "bms", "cell", "soc", "soh", "charge", "kwh", "tension", "voltage", "bobine", "coil", "relais"]):
            return "Electrical"
        elif any(x in text for x in ["vitesse", "speed", "boîte", "gear", "rapport", "clim", "frein", "brake"]):
            return "Vehicle"
        else:
            return "Config"

    def _on_category_filter(self, category: str):
        """Filter parameters by category."""
        self.category_filter = category
        self._rebuild_param_list()

    def _rebuild_param_list(self):
        """Rebuild the parameter list with current filter."""
        # Clear previous list
        for widget in self.params_scroll_frame.winfo_children():
            widget.destroy()

        # Filter by category
        if self.category_filter == "All":
            filtered = self.all_params
        else:
            filtered = [p for p in self.all_params if p.category == self.category_filter]

        if not filtered:
            self.no_params_label = ctk.CTkLabel(
                self.params_scroll_frame, text="No parameters in this category",
                font=FONTS["small"], text_color=COLORS["text_muted"]
            )
            self.no_params_label.pack(pady=32)
            return

        # Build rows
        self.param_rows = {}
        for idx, param in enumerate(filtered):
            bg_color = COLORS["bg_card"] if idx % 2 == 0 else COLORS["bg_secondary"]
            row = self._create_param_row(param, bg_color)
            self.param_rows[param.key] = row
            row.pack(fill="x", pady=2)

    def _create_param_row(self, param: SmartParam, bg_color: str) -> ctk.CTkFrame:
        """Create a single parameter row."""
        row = ctk.CTkFrame(self.params_scroll_frame, fg_color=bg_color, corner_radius=4)

        # Checkbox
        checkbox = ctk.CTkCheckBox(
            row, text="", width=24,
            command=lambda: self._on_param_checkbox(param.key)
        )
        checkbox.pack(side="left", padx=8, pady=6)

        # Source badge
        source_colors = {"obd": COLORS["cyan"], "uds": "#FF9800", "ecu": COLORS["success"]}
        badge_color = source_colors.get(param.source, COLORS["text_secondary"])
        badge = ctk.CTkLabel(
            row, text=f"[{param.source.upper()}]", font=FONTS["small_bold"],
            text_color="white", width=50, anchor="center",
            fg_color=badge_color, corner_radius=4
        )
        badge.pack(side="left", padx=4)

        # Name
        name_label = ctk.CTkLabel(
            row, text=param.name, font=FONTS["small"],
            text_color=COLORS["text_primary"], width=150, anchor="w"
        )
        name_label.pack(side="left", padx=4)

        # Value display
        value_label = ctk.CTkLabel(
            row, text="--", font=FONTS["body_bold"],
            text_color=COLORS["highlight"], width=80, anchor="e"
        )
        value_label.pack(side="left", padx=4)

        # Unit
        unit_label = ctk.CTkLabel(
            row, text=param.unit, font=FONTS["small"],
            text_color=COLORS["text_secondary"], width=60, anchor="w"
        )
        unit_label.pack(side="left", padx=4)

        # Mini graph
        graph = MiniGraph(row, width=150, height=35, color=source_colors.get(param.source, COLORS["success"]))
        graph.pack(side="left", padx=4)
        self.monitored_graphs[param.key] = graph

        # Comment (truncated)
        comment_text = param.comment[:40] if param.comment else ""
        comment_label = ctk.CTkLabel(
            row, text=comment_text, font=FONTS["small"],
            text_color=COLORS["text_muted"], width=150, anchor="w"
        )
        comment_label.pack(side="left", padx=4, fill="x", expand=True)

        # Store references for updates
        row._checkbox = checkbox
        row._value_label = value_label
        row._graph = graph

        return row

    def _on_param_checkbox(self, key: str):
        """Handle parameter checkbox change."""
        if key in self.selected_keys:
            self.selected_keys.remove(key)
        else:
            self.selected_keys.add(key)

    def _on_refresh_rate_change(self, value: str):
        """Handle refresh rate combo change."""
        rate_map = {"250ms": 250, "500ms": 500, "1000ms": 1000, "2000ms": 2000}
        self.refresh_ms = rate_map.get(value, 500)
        self._update_monitor_status()

    def _toggle_monitoring(self):
        """Toggle monitoring on/off."""
        if self.monitoring:
            self.stop_monitoring()
        else:
            self.start_monitoring()

    def start_monitoring(self):
        """Start monitoring selected parameters."""
        if not self.selected_keys:
            self.monitor_status_label.configure(
                text="Select parameters to monitor", text_color=COLORS["warning"]
            )
            return

        self.monitoring = True
        self.start_stop_btn.configure(text="Stop Monitor", fg_color=COLORS["danger"])
        self.update_count = 0
        self._schedule_update()

    def stop_monitoring(self):
        """Stop monitoring."""
        self.monitoring = False
        self.start_stop_btn.configure(text="Start Monitor", fg_color=COLORS["success"])
        if self.update_id:
            self.after_cancel(self.update_id)
            self.update_id = None
        self._update_monitor_status()

    def _schedule_update(self):
        """Schedule the next parameter update."""
        if self.monitoring:
            self._update_parameters()
            self.update_id = self.after(self.refresh_ms, self._schedule_update)

    def _update_parameters(self):
        """Read selected parameters from all sources."""
        if self._update_running or not self.monitoring:
            return

        def task():
            self._update_running = True
            try:
                updates = {}
                selected = [p for p in self.all_params if p.key in self.selected_keys]

                # Phase 1: Standard PIDs (fast, no disconnect)
                obd_params = [p for p in selected if p.source == "obd"]
                for p in obd_params:
                    if self.app.obd_reader:
                        try:
                            val, unit = self.app.obd_reader.read_pid(p.pid)
                            if val is not None:
                                updates[p.key] = val
                        except Exception:
                            pass

                # Phase 2: Manufacturer params (one custom connection session)
                mfr_params = [p for p in selected if p.source in ("uds", "ecu")]
                if mfr_params and self.app.connection and self.app.connection.is_connected():
                    def _read_mfr(conn):
                        conn.send_command("ATE0")
                        results = {}
                        current_tx = None
                        for p in mfr_params:
                            # Set target ECU only when address changes
                            if p.ecu_tx and p.ecu_tx != current_tx:
                                conn.send_command(f"AT SH {p.ecu_tx:03X}")
                                conn.send_command(f"AT CRA {p.ecu_rx:03X}")
                                current_tx = p.ecu_tx

                            try:
                                resp = conn.send_command(p.sentbytes, timeout=3)
                                if resp and "NO DATA" not in resp:
                                    # For ECU definition params, decode with formula
                                    if p.source == "ecu" and p.ecu_def and p.ecu_request:
                                        resp_bytes = self._parse_hex_response(resp)
                                        if resp_bytes:
                                            decoded = p.ecu_def.decode_response(p.ecu_request, resp_bytes)
                                            if p.param_name in decoded:
                                                results[p.key] = decoded[p.param_name]
                                    else:
                                        # UDS ops — store raw hex for now
                                        results[p.key] = resp.strip()[:40]
                            except Exception:
                                pass

                        # Restore defaults
                        try:
                            conn.send_command("AT D")
                            conn.send_command("AT CRA")
                            conn.send_command("AT H0")
                        except Exception:
                            pass
                        return results

                    try:
                        mfr_results = self.app.connection.use_custom_connection(_read_mfr)
                        if mfr_results:
                            updates.update(mfr_results)
                    except Exception:
                        pass

                if updates:
                    self.after(0, self._apply_updates, updates)
            finally:
                self._update_running = False

        threading.Thread(target=task, daemon=True).start()

    def _parse_hex_response(self, response: str) -> Optional[bytes]:
        """Parse ELM327 response to bytes."""
        try:
            clean = response.replace(" ", "").replace("\r", "").replace("\n", "").replace(">", "")
            clean = "".join(c for c in clean if c in "0123456789ABCDEFabcdef")
            if not clean:
                return None
            return bytes.fromhex(clean)
        except (ValueError, TypeError):
            return None

    def _apply_updates(self, updates: Dict):
        """Apply parameter updates to the display."""
        for key, value in updates.items():
            if key not in self.param_rows:
                continue

            row = self.param_rows[key]
            # Update value label
            if hasattr(row, '_value_label'):
                val_text = f"{value:.2f}" if isinstance(value, (int, float)) else str(value)[:20]
                row._value_label.configure(text=val_text)

            # Update graph
            if hasattr(row, '_graph') and isinstance(value, (int, float)):
                row._graph.add_value(value)

        self.update_count += 1
        self._update_monitor_status()

    def _update_monitor_status(self):
        """Update the monitoring status label."""
        if self.monitoring:
            status_text = f"Monitoring {len(self.selected_keys)} parameters | Rate: {self.refresh_ms}ms | Updates: {self.update_count}"
            self.monitor_status_label.configure(text=status_text, text_color=COLORS["text_secondary"])
        else:
            self.monitor_status_label.configure(text="", text_color=COLORS["text_muted"])

    def _on_lang_change(self, lang=None):
        """Update text on language change."""
        if not self.winfo_exists():
            return
        was_monitoring = self.monitoring
        if was_monitoring:
            self.stop_monitoring()
        for widget in self.winfo_children():
            widget.destroy()
        self._build_content()
        if was_monitoring:
            self._load_all_params()
            self._rebuild_param_list()
            self.start_monitoring()
