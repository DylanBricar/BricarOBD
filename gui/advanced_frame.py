"""Advanced operations frame — red-themed tab with BricarDB full database."""

from __future__ import annotations

import customtkinter as ctk
import threading
import logging

from gui.theme import COLORS, FONTS, _bind_scroll_recursive
from gui.dialogs import SafetyConfirmDialog
from i18n import t, get_lang, on_lang_change
from obd_core.advanced_operations import (
    get_operations_for_make, CATEGORIES,
)

logger = logging.getLogger(__name__)

_RED = {
    "banner_bg": "#3B1111",
    "banner_border": "#7F1D1D",
    "card_bg": "#1A1020",
    "card_border": "#4A1A2E",
    "badge_low": "#166534",
    "badge_medium": "#92400E",
    "badge_high": "#991B1B",
    "badge_low_text": "#4ADE80",
    "badge_medium_text": "#FBBF24",
    "badge_high_text": "#FCA5A5",
    "op_card_bg": "#121828",
    "op_card_border": "#1E3050",
    "read_badge": "#1E3A5F",
    "read_text": "#60A5FA",
    "write_badge": "#5F1E1E",
    "write_text": "#FCA5A5",
    "diag_badge": "#3D3D1E",
    "diag_text": "#FBBF24",
}

# Max operations to display at once (performance)
_MAX_DISPLAY = 200


class AdvancedFrame(ctk.CTkFrame):
    """Advanced operations frame with BricarDB database integration."""

    def __init__(self, parent, app):
        super().__init__(parent, fg_color=COLORS["bg_primary"])
        self.app = app
        self.operation_widgets = {}
        self._setup_ui()
        on_lang_change(self._on_lang_change)

    def _setup_ui(self):
        """Build the frame UI."""
        self._loaded = False
        self._create_warning_banner()

        # Single unified view (no tabs)
        title_frame = ctk.CTkFrame(self, fg_color="transparent")
        title_frame.pack(fill="x", padx=16, pady=(4, 0))

        ctk.CTkLabel(
            title_frame, text=t("adv_tab_all_ops"), font=FONTS["body_bold"],
            text_color=COLORS["text_primary"]
        ).pack(side="left")

        # Search bar (for BricarDB tab)
        self.search_frame = ctk.CTkFrame(self, fg_color="transparent")
        self.search_frame.pack(fill="x", padx=16, pady=(4, 0))
        self.search_frame.pack_forget()  # Hidden initially

        self.search_var = ctk.StringVar()
        self.search_entry = ctk.CTkEntry(
            self.search_frame, textvariable=self.search_var,
            placeholder_text="Search: DPF, inject, throttle, EGR, 2E2481...",
            font=FONTS["mono_small"],
            fg_color=COLORS["bg_input"], border_color=COLORS["input_border"],
            width=400,
        )
        self.search_entry.pack(side="left", padx=(0, 8))
        self.search_entry.bind("<Return>", lambda e: self._do_search())

        self.search_btn = ctk.CTkButton(
            self.search_frame, text=t("dtc_search"), width=80, height=28,
            font=FONTS["small_bold"],
            fg_color=COLORS["accent"], hover_color=COLORS["accent_hover"],
            command=self._do_search,
        )
        self.search_btn.pack(side="left", padx=(0, 8))

        # Type filter
        self.type_var = ctk.StringVar(value="all")
        self.type_filter = ctk.CTkComboBox(
            self.search_frame,
            values=["all", "read (0x22)", "write (0x2E)", "routine (0x31)", "actuator (0x2F)"],
            variable=self.type_var, width=150, height=28,
            font=FONTS["small"],
            fg_color=COLORS["bg_input"], border_color=COLORS["input_border"],
            command=lambda _: self._do_search(),
        )
        self.type_filter.pack(side="left", padx=(0, 8))

        self.search_status = ctk.CTkLabel(
            self.search_frame, text="", font=FONTS["small"],
            text_color=COLORS["text_muted"],
        )
        self.search_status.pack(side="left")

        # Scrollable content
        self.scroll = ctk.CTkScrollableFrame(
            self, fg_color=COLORS["bg_primary"],
            scrollbar_button_color=COLORS["bg_card"],
            scrollbar_button_hover_color=COLORS["bg_card_hover"],
        )
        self.scroll.pack(fill="both", expand=True, padx=0, pady=0)
        self.after(500, lambda: _bind_scroll_recursive(self.scroll))

        # Vehicle status
        self.vehicle_label = ctk.CTkLabel(
            self.scroll, text="", font=FONTS["body"],
            text_color=COLORS["text_secondary"],
        )
        self.vehicle_label.pack(anchor="w", padx=16, pady=(8, 4))

        # Operations container
        self.ops_container = ctk.CTkFrame(self.scroll, fg_color="transparent")
        self.ops_container.pack(fill="both", expand=True, padx=8, pady=4)

    def _create_warning_banner(self):
        banner = ctk.CTkFrame(
            self, fg_color=_RED["banner_bg"], corner_radius=8,
            border_width=1, border_color=_RED["banner_border"],
        )
        banner.pack(fill="x", padx=16, pady=(8, 12))

        title_frame = ctk.CTkFrame(banner, fg_color="transparent")
        title_frame.pack(fill="x", padx=16, pady=(12, 4))

        self.warning_icon = ctk.CTkLabel(
            title_frame, text="!!", font=("Helvetica", 18, "bold"),
            text_color=COLORS["danger"],
        )
        self.warning_icon.pack(side="left", padx=(0, 8))

        self.warning_title = ctk.CTkLabel(
            title_frame, text=t("adv_warning_title"),
            font=FONTS["h3"], text_color=COLORS["danger"],
        )
        self.warning_title.pack(side="left")

        self.warning_text = ctk.CTkLabel(
            banner, text=t("adv_warning_text"),
            font=FONTS["small"], text_color=COLORS["danger_light"],
            wraplength=900, justify="left",
        )
        self.warning_text.pack(anchor="w", padx=16, pady=(0, 12))

    # ── Display initialization ────────────────────────────────
    def on_frame_shown(self):
        """Called when user navigates to this tab — reload every time."""
        if not hasattr(self, '_layout_done'):
            self._layout_done = True
            self.search_frame.pack(fill="x", padx=16, pady=(4, 0))
            self.scroll.pack(fill="both", expand=True, padx=0, pady=0)
        self._show_all_ops()

    def _show_all_ops(self):
        """Show all operations from all sources in one unified list."""
        self._clear_ops()
        make = self._get_detected_make()
        if not make:
            self.vehicle_label.configure(text=t("adv_no_vehicle"))
            self._show_empty(t("adv_no_vehicle_detail"))
            return

        # Show unified DB with search
        self._show_unified_ops()

    # ── Verified operations (existing) ────────────────────────
    def _show_verified_ops(self):
        self._clear_ops()
        lang = get_lang()
        make = getattr(self.app, "detected_make", None) or ""

        if not make:
            self.vehicle_label.configure(text=t("adv_no_vehicle"))
            self._show_empty(t("adv_no_vehicle_detail"))
            return

        self.vehicle_label.configure(text=t("adv_vehicle_detected", make=make))
        operations = get_operations_for_make(make)

        if not operations:
            self._show_empty(t("adv_no_operations", make=make))
            return

        by_category = {}
        for op in operations:
            cat = op["category"]
            if cat not in by_category:
                by_category[cat] = []
            by_category[cat].append(op)

        for cat_key, ops in by_category.items():
            self._create_category_section(cat_key, ops, lang)

    # ── ECU DDT2000 (actuations + extended DIDs from ZIP database) ──
    def _show_ecu_ddt_ops(self):
        """Show ECU DDT2000 operations: actuations, configs, extended DIDs."""
        self._clear_ops()

        make = self._get_detected_make()
        if not make:
            self.vehicle_label.configure(text=t("adv_no_vehicle"))
            self._show_empty("Connect to detect vehicle first.")
            return

        ecu_database = getattr(self.app, 'ecu_database', None)
        if not ecu_database or not ecu_database.is_loaded:
            self._show_empty("ECU database not loaded (data/bricarobd_db_part_*)")
            return

        from obd_core.ecu_identifier import KNOWN_ADDRESSES
        from obd_core.ecu_database import get_extended_dids, get_ecus_for_make

        # Determine which addresses to scan for this make
        _MAKE_ADDRS = {
            "peugeot": ["7A", "26", "28", "60"], "citroen": ["7A", "26", "28", "60"],
            "citroën": ["7A", "26", "28", "60"], "opel": ["7A", "26", "28", "60"],
            "renault": ["7A", "26", "10"], "dacia": ["7A", "26", "10"],
            "volkswagen": ["7A", "7B", "7C"], "audi": ["7A", "7B", "7C"],
            "bmw": ["7A", "7B", "7C", "7D"], "mercedes-benz": ["7A", "7B", "7C"],
        }
        allowed = _MAKE_ADDRS.get(make.lower(), ["7A"])

        all_ops = []

        # 1. Load ECU definitions — ALL requests (reads + actuations)
        for addr in allowed:
            addr_info = KNOWN_ADDRESSES.get(addr, {})
            ecus = ecu_database.find_ecus_by_address(addr)
            for ecu_info in ecus[:3]:  # Top 3 per address
                ecu_def = ecu_database.load_ecu_definition(ecu_info.get("filename", ""))
                if not ecu_def:
                    continue
                for req in ecu_def.requests:
                    cmd = req.sentbytes.upper()
                    # Determine type
                    if cmd.startswith(("21", "22", "19")):
                        op_type = "read"
                    elif cmd.startswith(("30", "31", "32", "3B", "2E", "2F")):
                        op_type = "write"
                    elif cmd.startswith(("10", "14", "17")):
                        op_type = "diag"
                    else:
                        op_type = "other"
                    all_ops.append({
                        "name": req.name,
                        "sentbytes": req.sentbytes,
                        "service": cmd[:2] if len(cmd) >= 2 else "",
                        "did": cmd[2:] if len(cmd) > 2 else "",
                        "type": op_type,
                        "ecu_name": ecu_def.ecuname,
                        "ecu_tx": f"{addr_info.get('can_tx', 0x7E0):03X}",
                        "ecu_rx": f"{addr_info.get('can_rx', 0x7E8):03X}",
                        "params_count": len(req.params),
                    })

        # 2. Extended DIDs from ecu_database.py
        ext_dids = get_extended_dids(make)
        make_ecus = get_ecus_for_make(make)
        # Find engine ECU for default addressing
        default_tx, default_rx = "7E0", "7E8"
        for ecu in make_ecus:
            if "engine" in ecu.name.lower() or "inject" in ecu.name.lower():
                default_tx = f"{ecu.request_id:03X}"
                default_rx = f"{ecu.response_id:03X}"
                break
        for did, desc in ext_dids.items():
            all_ops.append({
                "name": f"{desc} (DID 0x{did:04X})",
                "sentbytes": f"22{did:04X}",
                "service": "22",
                "did": f"{did:04X}",
                "type": "read",
                "ecu_name": "Extended DID",
                "ecu_tx": default_tx,
                "ecu_rx": default_rx,
            })

        # Display
        self.vehicle_label.configure(text=f"{make}: {len(all_ops)} ECU operations")

        if not all_ops:
            self._show_empty(f"No ECU data for {make}")
            return

        # Group by ECU name
        by_ecu = {}
        for op in all_ops:
            ecu = op.get("ecu_name", "Unknown")
            if ecu not in by_ecu:
                by_ecu[ecu] = []
            by_ecu[ecu].append(op)

        # Filter by search if active
        query = self.search_var.get().strip().lower()
        type_filter = self.type_var.get()

        for ecu_name, ops in sorted(by_ecu.items()):
            filtered = ops
            if query:
                filtered = [o for o in filtered if query in o["name"].lower() or query in o["sentbytes"].lower()]
            if "read" in type_filter:
                filtered = [o for o in filtered if o["type"] == "read"]
            elif "write" in type_filter:
                filtered = [o for o in filtered if o["type"] == "write"]
            elif "routine" in type_filter:
                filtered = [o for o in filtered if o["sentbytes"][:2] in ("31",)]
            elif "actuator" in type_filter:
                filtered = [o for o in filtered if o["sentbytes"][:2] in ("30", "2F")]

            if not filtered:
                continue

            # ECU section header
            reads = len([o for o in filtered if o["type"] == "read"])
            writes = len(filtered) - reads
            header_text = f"{ecu_name} ({len(filtered)} ops: {reads}R / {writes}W)"
            ctk.CTkLabel(
                self.ops_container, text=header_text,
                font=FONTS["body_bold"], text_color=COLORS["text_secondary"],
            ).pack(anchor="w", padx=16, pady=(12, 4))

            ecu_data = {"send_id": ops[0].get("ecu_tx", "7E0"),
                        "recv_id": ops[0].get("ecu_rx", "7E8"),
                        "ecuname": ecu_name}

            for op in filtered[:_MAX_DISPLAY]:
                self._create_op_row(self.ops_container, ecu_data, op)

    # ── Unified database ─────────────────────────────────────
    def _get_detected_make(self) -> str:
        """Get the detected vehicle make, or empty string."""
        return getattr(self.app, "detected_make", None) or ""

    def _show_unified_ops(self):
        """Show operations from the unified database, filtered by detected vehicle."""
        self._clear_ops()

        make = self._get_detected_make()
        if not make:
            self.vehicle_label.configure(text=t("adv_no_vehicle"))
            self._show_empty(
                "Connect first to detect the vehicle.\n"
                "Connectez-vous d'abord pour détecter le véhicule."
            )
            return

        from obd_core.unified_db import get_unified_db, make_to_manufacturer
        udb = get_unified_db()

        if not udb.is_available():
            self._show_empty(
                "Unified database not found (data/bricarobd_database.zip)"
            )
            return

        if not udb.load():
            self._show_empty("Failed to load unified database")
            return

        manufacturer = make_to_manufacturer(make)

        # Only show WRITE operations (reads are in Live Data / Dashboard)
        all_write_ops = udb.get_operations_for_manufacturer(manufacturer, op_type="write", limit=5000)
        all_diag_ops = udb.get_operations_for_manufacturer(manufacturer, op_type="diag", limit=5000)
        all_other_ops = udb.get_operations_for_manufacturer(manufacturer, op_type="other", limit=5000)

        # Also add ECU DDT2000 write commands
        ecu_write_ops = self._get_ecu_write_ops(make)

        total_writes = len(all_write_ops) + len(all_diag_ops) + len(all_other_ops) + len(ecu_write_ops)

        if total_writes == 0:
            self.vehicle_label.configure(text=f"{make}: no write operations")
            self._show_empty(f"No write/actuation operations for {make}")
            return

        self.vehicle_label.configure(
            text=f"{make}: {total_writes} write/actuation operations"
        )

        # Group by ECU name
        groups = {}
        for op in all_write_ops + all_diag_ops + all_other_ops:
            ecu = op.get("ecu_name", "Unknown")
            if ecu not in groups:
                groups[ecu] = {"name": ecu, "total": 0, "reads": 0, "writes": 0,
                               "ecu_tx": op.get("ecu_tx", ""), "ecu_rx": op.get("ecu_rx", "")}
            groups[ecu]["total"] += 1
            groups[ecu]["writes"] += 1

        for op in ecu_write_ops:
            ecu = op.get("ecu_name", "ECU DDT2000")
            if ecu not in groups:
                groups[ecu] = {"name": ecu, "total": 0, "reads": 0, "writes": 0,
                               "ecu_tx": op.get("ecu_tx", ""), "ecu_rx": op.get("ecu_rx", "")}
            groups[ecu]["total"] += 1
            groups[ecu]["writes"] += 1

        groups = sorted(groups.values(), key=lambda g: -g["total"])

        info = ctk.CTkLabel(
            self.ops_container,
            text=t("adv_ddt_search_hint"),
            font=FONTS["small"], text_color=COLORS["text_muted"],
            wraplength=800,
        )
        info.pack(anchor="w", padx=8, pady=(4, 8))

        for group in groups[:60]:
            self._create_unified_group_header(group, manufacturer)

    def _create_unified_group_header(self, group: dict, manufacturer: str):
        """Create a clickable ECU group header."""
        header = ctk.CTkFrame(self.ops_container, fg_color=_RED["op_card_bg"],
                              corner_radius=6, border_width=1,
                              border_color=_RED["op_card_border"])
        header.pack(fill="x", padx=4, pady=2)

        name = group["name"]
        total = group["total"]
        reads = group["reads"]
        writes = group["writes"]

        btn = ctk.CTkButton(
            header,
            text=f"{name}  ({total} ops: {reads}R / {writes}W)",
            font=FONTS["body_bold"],
            fg_color="transparent",
            hover_color=COLORS["bg_card_hover"],
            text_color=COLORS["text_primary"],
            anchor="w", height=32,
            command=lambda n=name, m=manufacturer: self._expand_unified_group(n, m),
        )
        btn.pack(fill="x", padx=8, pady=4)

        if group.get("ecu_tx"):
            ctk.CTkLabel(
                header, text=f"TX=0x{group['ecu_tx']} RX=0x{group.get('ecu_rx', '?')}",
                font=FONTS["mono_small"], text_color=COLORS["text_muted"],
            ).pack(anchor="w", padx=16, pady=(0, 4))

    def _expand_unified_group(self, ecu_name: str, manufacturer: str):
        """Show all operations for a specific ECU group from unified DB."""
        self._clear_ops()

        from obd_core.unified_db import get_unified_db
        udb = get_unified_db()

        ops = udb.get_operations_for_manufacturer(manufacturer, limit=5000)
        group_ops = [o for o in ops if o.get("ecu_name") == ecu_name]

        self.vehicle_label.configure(text=f"{ecu_name} ({len(group_ops)} ops)")

        back = ctk.CTkButton(
            self.ops_container, text="< Back", width=120, height=28,
            font=FONTS["small_bold"],
            fg_color=COLORS["bg_card"], hover_color=COLORS["bg_card_hover"],
            command=lambda: self._show_unified_ops(),
        )
        back.pack(anchor="w", padx=8, pady=(4, 8))

        if not group_ops:
            self._show_empty("No operations found.")
            return

        ecu_tx = group_ops[0].get("ecu_tx", "")
        ecu_rx = group_ops[0].get("ecu_rx", "")
        ecu_data = {"send_id": ecu_tx, "recv_id": ecu_rx, "ecuname": ecu_name}

        for op in group_ops[:_MAX_DISPLAY]:
            self._create_op_row(self.ops_container, ecu_data, {
                "name": op.get("name", ""),
                "sentbytes": op.get("sentbytes", ""),
                "service": op.get("service", ""),
                "did": op.get("did", ""),
                "type": op.get("type", "read"),
            })

        if len(group_ops) > _MAX_DISPLAY:
            ctk.CTkLabel(
                self.ops_container,
                text=f"Showing {_MAX_DISPLAY}/{len(group_ops)}. Use search to refine.",
                font=FONTS["small"], text_color=COLORS["warning"],
            ).pack(pady=8)

    def _get_ecu_write_ops(self, make):
        """Get write/actuation operations from ECU DDT2000 database."""
        ecu_database = getattr(self.app, 'ecu_database', None)
        if not ecu_database or not ecu_database.is_loaded:
            return []

        from obd_core.ecu_identifier import KNOWN_ADDRESSES, MAKE_ADDRESSES
        allowed = MAKE_ADDRESSES.get(make.lower(), ["7A"])
        ops = []
        seen = set()

        for addr in allowed:
            addr_info = KNOWN_ADDRESSES.get(addr, {})
            ecus = ecu_database.find_ecus_by_address(addr)
            for ecu_info in ecus[:3]:
                ecu_def = ecu_database.load_ecu_definition(ecu_info.get("filename", ""))
                if not ecu_def:
                    continue
                for req in ecu_def.requests:
                    cmd = req.sentbytes.upper()
                    # Only write/actuation services
                    if not cmd.startswith(("30", "31", "32", "3B", "2E", "2F")):
                        continue
                    key = (cmd, ecu_def.ecuname)
                    if key in seen:
                        continue
                    seen.add(key)
                    ops.append({
                        "name": req.name,
                        "sentbytes": req.sentbytes,
                        "service": cmd[:2],
                        "did": cmd[2:] if len(cmd) > 2 else "",
                        "type": "write",
                        "ecu_name": ecu_def.ecuname[:30],
                        "ecu_tx": f"{addr_info.get('can_tx', 0x7E0):03X}",
                        "ecu_rx": f"{addr_info.get('can_rx', 0x7E8):03X}",
                    })
        return ops

    def _do_search(self):
        """Search operations in the unified database."""
        query = self.search_var.get().strip()
        type_filter = self.type_var.get()

        if not query and type_filter == "all":
            self._show_unified_ops()
            return

        self._clear_ops()

        make = self._get_detected_make()
        if not make:
            self._show_empty("Connect to a vehicle first.")
            return

        from obd_core.unified_db import get_unified_db, make_to_manufacturer
        udb = get_unified_db()
        if not udb.load():
            self._show_empty("Unified database not available")
            return

        manufacturer = make_to_manufacturer(make)

        # Map type filter (default to write — reads are in Live Data)
        op_type = "write"  # Default: only writes
        if "read" in type_filter:
            op_type = "read"
        elif "write" in type_filter:
            op_type = "write"
        elif "all" in type_filter:
            op_type = None  # User explicitly wants all

        self.search_status.configure(text="Searching...")
        self.update_idletasks()

        results = udb.get_operations_for_manufacturer(manufacturer, query=query, op_type=op_type, limit=_MAX_DISPLAY)

        self.search_status.configure(text=f"{len(results)} results")
        self.vehicle_label.configure(text=f"Search ({make}): '{query}' — {len(results)} results")

        if not results:
            self._show_empty(f"No results for '{query}'")
            return

        for op in results:
            ecu_data = {"send_id": op.get("ecu_tx", ""), "recv_id": op.get("ecu_rx", ""), "ecuname": op.get("ecu_name", "")}
            self._create_op_row(self.ops_container, ecu_data, {
                "name": op.get("name", ""),
                "sentbytes": op.get("sentbytes", ""),
                "service": op.get("service", ""),
                "did": op.get("did", ""),
                "type": op.get("type", "read"),
            })

    def _create_op_row(self, parent, ecu: dict, op: dict):
        """Create a compact row for a BricarDB operation."""
        row = ctk.CTkFrame(parent, fg_color="transparent", height=26)
        row.pack(fill="x", padx=12, pady=1)

        # Type badge
        type_colors = {
            "read": (_RED["read_badge"], _RED["read_text"], "READ"),
            "write": (_RED["write_badge"], _RED["write_text"], "WRITE"),
            "diag": (_RED["diag_badge"], _RED["diag_text"], "DIAG"),
            "other": (COLORS["bg_card"], COLORS["text_muted"], "OTHER"),
        }
        bg, fg, label = type_colors.get(op["type"], type_colors["other"])
        ctk.CTkLabel(
            row, text=f" {label} ", font=FONTS["small_bold"],
            fg_color=bg, text_color=fg, corner_radius=3, width=50,
        ).pack(side="left", padx=(0, 4))

        # Hex command
        ctk.CTkLabel(
            row, text=op["sentbytes"][:12],
            font=FONTS["mono_small"], text_color=COLORS["accent"],
            width=95,
        ).pack(side="left", padx=(0, 4))

        # Name
        ctk.CTkLabel(
            row, text=op["name"][:70],
            font=FONTS["small"], text_color=COLORS["text_primary"],
        ).pack(side="left", fill="x", expand=True)

        # Execute button (only for valid sentbytes >= 4 chars)
        if len(op.get("sentbytes", "")) >= 4:
            service = op["sentbytes"][:2].upper()
            # Block flash programming services from even having buttons
            if service not in ("34", "35", "36", "37", "3D", "28", "11", "27"):
                is_read = service in ("22", "21", "19")
                if is_read:
                    read_btn = ctk.CTkButton(
                        row, text="Read", width=45, height=22,
                        font=FONTS["small"],
                        fg_color=COLORS["bg_card"],
                        hover_color=COLORS["bg_card_hover"],
                        command=lambda o=op, e=ecu: self._exec_op(o, e),
                    )
                    read_btn.pack(side="right", padx=2)
                else:
                    exec_btn = ctk.CTkButton(
                        row, text="Exec", width=45, height=22,
                        font=FONTS["small"],
                        fg_color=COLORS["danger_dark"],
                        hover_color=COLORS["danger"],
                        command=lambda o=op, e=ecu: self._exec_op(o, e),
                    )
                    exec_btn.pack(side="right", padx=2)

    def _exec_op(self, op: dict, ecu: dict):
        """Execute a BricarDB operation with full safety checks."""
        if not self.app.connection or not self.app.connection.is_connected():
            return

        sentbytes = op.get("sentbytes", "")
        if len(sentbytes) < 4:
            return

        # SAFETY: Block flash programming services unconditionally
        service = sentbytes[:2].upper()
        BLOCKED_SERVICES = {"34", "35", "36", "37", "3D", "28", "11", "27"}
        if service in BLOCKED_SERVICES:
            return

        # SAFETY: Require confirmation for ALL non-read operations
        is_read = service in ("22", "21", "19")
        if not is_read:
            dialog = SafetyConfirmDialog(
                self.winfo_toplevel(),
                t("adv_confirm_title"),
                f"{op.get('name', '?')}\n\n"
                f"Hex: {sentbytes}\n"
                f"Service: 0x{service}\n"
                f"ECU: {ecu.get('ecuname', '?')} (TX=0x{ecu.get('send_id', '?')})\n\n"
                f"Type: {'WRITE' if service in ('2E','2F','30','31','32','3B') else service}",
            )
            if not dialog.wait_for_result():
                return

        # Find or create a status label on the parent
        status_text = [None]

        def run():
            # SAFETY: Check SafetyGuard for write operations
            if not is_read:
                allowed, reason = self.app.safety.is_operation_allowed(int(service, 16))
                if not allowed:
                    self.after(0, lambda: self._show_exec_result(f"BLOCKED: {reason}", False))
                    return

            send_id = ecu.get("send_id", "7E0")
            recv_id = ecu.get("recv_id", "7E8")

            # Use custom connection to ensure serial access works
            def _do_exec(conn):
                conn.send_command("ATE0")
                # Set header
                send_int = int(send_id, 16) if isinstance(send_id, str) else send_id
                recv_int = int(recv_id, 16) if isinstance(recv_id, str) else recv_id
                conn.send_command(f"AT SH {send_int:03X}")
                conn.send_command(f"AT CRA {recv_int:03X}")

                if not is_read:
                    conn.send_command("1003", timeout=3)  # Extended session

                self.app.safety.log_operation(
                    f"BricarDB:{op.get('name', '?')}", int(service, 16),
                    f"TX=0x{send_id} cmd={sentbytes}", "SENDING"
                )

                response = conn.send_command(sentbytes, timeout=5)

                if not is_read:
                    conn.send_command("1001", timeout=2)  # Return to default

                conn.send_command("AT D")
                conn.send_command("AT CRA")
                conn.send_command("AT H0")
                return response

            response = self.app.connection.use_custom_connection(_do_exec)
            result_text = response if response else "No response"
            self.app.safety.log_operation(
                f"BricarDB:{op.get('name', '?')}", int(service, 16),
                f"TX=0x{send_id} cmd={sentbytes}", f"RESULT: {result_text}"
            )
            logger.info(f"BricarDB exec: [{sentbytes}] -> {result_text}")
            self.after(0, lambda: self._show_exec_result(result_text, is_read))

        thread = threading.Thread(target=run, daemon=True)
        thread.start()

    def _show_exec_result(self, result: str, is_read: bool):
        """Show execution result in a small popup."""
        # Remove previous result label if exists
        if hasattr(self, '_last_result_label') and self._last_result_label:
            try:
                self._last_result_label.destroy()
            except Exception:
                pass

        color = COLORS["success"] if not result.startswith("7F") else COLORS["danger"]
        result_label = ctk.CTkLabel(
            self.ops_container,
            text=f"Result: {result[:80]}",
            font=FONTS["mono_small"],
            text_color=color,
        )
        result_label.pack(anchor="w", padx=16, pady=2)
        self._last_result_label = result_label
        self.after(10000, lambda: result_label.destroy() if result_label.winfo_exists() else None)

    # ── Verified operations helpers ───────────────────────────
    def _create_category_section(self, cat_key: str, operations: list, lang: str):
        cat = CATEGORIES.get(cat_key, {})
        cat_name = cat.get("name", {}).get(lang, cat_key)
        cat_risk = cat.get("risk", "medium")

        header = ctk.CTkFrame(self.ops_container, fg_color="transparent")
        header.pack(fill="x", padx=8, pady=(16, 4))
        ctk.CTkLabel(header, text=cat_name, font=FONTS["h3"],
                     text_color=COLORS["text_primary"]).pack(side="left")
        self._create_risk_badge(header, cat_risk)

        for op in operations:
            self._create_operation_card(op, lang)

    def _create_operation_card(self, op: dict, lang: str):
        card = ctk.CTkFrame(self.ops_container, fg_color=_RED["card_bg"],
                            corner_radius=8, border_width=1,
                            border_color=_RED["card_border"])
        card.pack(fill="x", padx=8, pady=4)

        header = ctk.CTkFrame(card, fg_color="transparent")
        header.pack(fill="x", padx=12, pady=(10, 4))

        name = op["name"].get(lang, op["name"].get("en", op["id"]))
        ctk.CTkLabel(header, text=name, font=FONTS["body_bold"],
                     text_color=COLORS["text_primary"]).pack(side="left")

        is_read = op.get("command_type") == "read" and op.get("service") == 0x22
        if not is_read:
            self._create_risk_badge(header, op.get("risk", "medium"))

        ctk.CTkLabel(header, text=f"[{op['ecu_name']}]", font=FONTS["small"],
                     text_color=COLORS["text_muted"]).pack(side="right", padx=(8, 0))

        if op.get("security"):
            ctk.CTkLabel(header, text=t("adv_requires_security"), font=FONTS["small"],
                         text_color=COLORS["warning"]).pack(side="right", padx=(8, 0))

        desc = op["desc"].get(lang, op["desc"].get("en", ""))
        ctk.CTkLabel(card, text=desc, font=FONTS["small"],
                     text_color=COLORS["text_secondary"],
                     wraplength=800, justify="left").pack(anchor="w", padx=12, pady=(0, 4))

        preconditions = op.get("pre_conditions", {}).get(lang, [])
        if preconditions:
            pre_frame = ctk.CTkFrame(card, fg_color="transparent")
            pre_frame.pack(fill="x", padx=12, pady=(0, 4))
            ctk.CTkLabel(pre_frame, text=t("adv_preconditions") + " :",
                         font=FONTS["small_bold"],
                         text_color=COLORS["text_muted"]).pack(anchor="w")
            for cond in preconditions:
                ctk.CTkLabel(pre_frame, text=f"  - {cond}", font=FONTS["small"],
                             text_color=COLORS["text_muted"]).pack(anchor="w")

        param_entries = {}
        if op.get("parameters"):
            params_frame = ctk.CTkFrame(card, fg_color="transparent")
            params_frame.pack(fill="x", padx=12, pady=(4, 4))
            for param in op["parameters"]:
                p_frame = ctk.CTkFrame(params_frame, fg_color="transparent")
                p_frame.pack(fill="x", pady=2)
                p_name = param["name"].get(lang, param["name"].get("en", param["key"]))
                ctk.CTkLabel(p_frame, text=p_name, font=FONTS["small"],
                             text_color=COLORS["text_secondary"], width=180).pack(side="left")
                if param.get("type") == "choice":
                    var = ctk.StringVar(value=param["options"][0])
                    ctk.CTkComboBox(p_frame, values=param["options"], variable=var,
                                    width=200, font=FONTS["small"],
                                    fg_color=COLORS["bg_input"],
                                    border_color=COLORS["input_border"]).pack(side="left", padx=4)
                    param_entries[param["key"]] = var
                else:
                    hint = param.get("hint", {}).get(lang, "")
                    entry = ctk.CTkEntry(p_frame, width=260, font=FONTS["mono_small"],
                                         fg_color=COLORS["bg_input"],
                                         border_color=COLORS["input_border"],
                                         placeholder_text=hint)
                    entry.pack(side="left", padx=4)
                    param_entries[param["key"]] = entry

        notes = op.get("notes", {}).get(lang, "")
        if notes:
            ctk.CTkLabel(card, text=notes, font=FONTS["small"],
                         text_color=COLORS["warning"], wraplength=800,
                         justify="left").pack(anchor="w", padx=12, pady=(0, 4))

        bottom = ctk.CTkFrame(card, fg_color="transparent")
        bottom.pack(fill="x", padx=12, pady=(4, 10))

        status_label = ctk.CTkLabel(bottom, text="", font=FONTS["small"],
                                    text_color=COLORS["text_muted"])
        status_label.pack(side="right", padx=8)

        ctk.CTkButton(
            bottom, text=t("adv_execute"), font=FONTS["body_bold"],
            fg_color=COLORS["danger"], hover_color=COLORS["danger_hover"],
            width=140, height=32,
            command=lambda o=op, p=param_entries, s=status_label: self._on_execute(o, p, s),
        ).pack(side="right")


    def _create_risk_badge(self, parent, risk: str):
        risk_colors = {
            "low": (_RED["badge_low"], _RED["badge_low_text"]),
            "medium": (_RED["badge_medium"], _RED["badge_medium_text"]),
            "high": (_RED["badge_high"], _RED["badge_high_text"]),
        }
        bg, fg = risk_colors.get(risk, risk_colors["medium"])
        ctk.CTkLabel(parent, text=f" {t(f'adv_risk_{risk}')} ",
                     font=FONTS["small_bold"], fg_color=bg, text_color=fg,
                     corner_radius=4).pack(side="left", padx=8)

    def _on_execute(self, op, param_entries, status_label):
        lang = get_lang()
        if not self.app.connection or not self.app.connection.is_connected():
            status_label.configure(text=t("adv_not_connected"), text_color=COLORS["danger"])
            return

        params = {}
        for key, widget in param_entries.items():
            if isinstance(widget, ctk.StringVar):
                params[key] = widget.get()
            elif hasattr(widget, "get"):
                params[key] = widget.get()

        if op.get("parameters"):
            for param in op["parameters"]:
                val = params.get(param["key"], "").strip()
                if not val and param.get("type") != "choice":
                    p_name = param["name"].get(lang, param["key"])
                    status_label.configure(text=f"{p_name} : {t('adv_param_required')}",
                                           text_color=COLORS["warning"])
                    return

        # Skip confirmation for read-only operations
        if op.get("command_type") == "read" and op.get("service") == 0x22:
            # Read operations are safe, execute directly
            pass
        else:
            # Confirmation for write/routine/actuator operations
            op_name = op["name"].get(lang, op["id"])
            risk_text = t(f"adv_risk_{op.get('risk', 'medium')}")
            dialog = SafetyConfirmDialog(
                self.winfo_toplevel(), t("adv_confirm_title"),
                t("adv_confirm_msg", operation=op_name, risk=risk_text))
            if not dialog.wait_for_result():
                return

        status_label.configure(text=t("adv_executing"), text_color=COLORS["warning"])

        def run():
            mgr = self.app.advanced_manager
            result = mgr.execute_operation(op, params)
            def update():
                color = COLORS["success"] if result["success"] else COLORS["danger"]
                status_label.configure(text=result["message"], text_color=color)
            self.after(0, update)

        threading.Thread(target=run, daemon=True).start()

    # ── Helpers ───────────────────────────────────────────────
    def _clear_ops(self):
        for widget in self.ops_container.winfo_children():
            widget.destroy()
        self.operation_widgets.clear()

    def _show_empty(self, text: str):
        ctk.CTkLabel(self.ops_container, text=text, font=FONTS["body"],
                     text_color=COLORS["text_muted"],
                     wraplength=600).pack(pady=40)

    def _on_lang_change(self, lang=None):
        """Rebuild UI on language change."""
        if not self.winfo_exists():
            return
        for widget in self.winfo_children():
            widget.destroy()
        self._setup_ui()

    def on_frame_shown(self):
        if self._tab_var.get() == "verified":
            self._show_verified_ops()
