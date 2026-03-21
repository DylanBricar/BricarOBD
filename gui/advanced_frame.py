"""Advanced operations frame — red-themed tab with BricarDB full database."""

from __future__ import annotations

import customtkinter as ctk
import threading
import logging

from gui.theme import COLORS, FONTS, _bind_scroll_recursive
from gui.dialogs import SafetyConfirmDialog
from i18n import t, get_lang, on_lang_change
from obd_core.advanced_operations import (
    get_operations_for_make, get_all_categories, CATEGORIES,
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
        self._create_warning_banner()

        # Tab selector: Verified | BricarDB
        tab_frame = ctk.CTkFrame(self, fg_color="transparent")
        tab_frame.pack(fill="x", padx=16, pady=(4, 0))

        self._tab_var = ctk.StringVar(value="verified")
        self.tab_verified = ctk.CTkButton(
            tab_frame, text=t("adv_tab_verified"), width=160, height=30,
            font=FONTS["body_bold"],
            fg_color=COLORS["danger"], hover_color=COLORS["danger_hover"],
            command=lambda: self._switch_tab("verified"),
        )
        self.tab_verified.pack(side="left", padx=(0, 4))

        self.tab_unified = ctk.CTkButton(
            tab_frame, text="Base complète (0 ops)", width=240, height=30,
            font=FONTS["body_bold"],
            fg_color=COLORS["bg_card"], hover_color=COLORS["bg_card_hover"],
            text_color=COLORS["text_secondary"],
            command=lambda: self._switch_tab("unified"),
        )
        self.tab_unified.pack(side="left", padx=4)

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

    # ── Tab switching ─────────────────────────────────────────
    def _switch_tab(self, tab: str):
        self._tab_var.set(tab)
        for btn in [self.tab_verified, self.tab_unified]:
            btn.configure(fg_color=COLORS["bg_card"], text_color=COLORS["text_secondary"])

        if tab == "verified":
            self.tab_verified.configure(fg_color=COLORS["danger"], text_color=COLORS["text_primary"])
            self.search_frame.pack_forget()
            self._show_verified_ops()
        elif tab == "unified":
            self.tab_unified.configure(fg_color=COLORS["danger"], text_color=COLORS["text_primary"])
            self.search_frame.pack(fill="x", padx=16, pady=(4, 0), before=self.scroll)
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
                "Connectez-vous d'abord pour détecter le véhicule.\n"
                "Connect first to detect the vehicle."
            )
            return

        from obd_core.unified_db import get_unified_db, make_to_manufacturer
        udb = get_unified_db()

        if not udb.is_available():
            self._show_empty(
                "Base unifiée non trouvée (data/unified_database.json).\n"
                "Unified database not found."
            )
            return

        if not udb.load():
            self._show_empty("Failed to load unified database.")
            return

        manufacturer = make_to_manufacturer(make)
        total = udb.count_for_manufacturer(manufacturer)
        self.tab_unified.configure(text=f"Base complète ({total:,} ops)")

        if total == 0:
            self.vehicle_label.configure(text=f"{make}: aucune donnée")
            self._show_empty(
                f"Aucune opération trouvée pour {make} dans la base unifiée.\n"
                f"No operations found for {make} in the unified database."
            )
            return

        self.vehicle_label.configure(
            text=f"{make} ({manufacturer}): {total:,} opérations"
        )

        # Show ECU groups
        groups = udb.get_groups_for_manufacturer(manufacturer)

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
            self.ops_container, text="< Retour / Back", width=120, height=28,
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
            self._show_empty("Unified database not available.")
            return

        manufacturer = make_to_manufacturer(make)

        # Map type filter
        op_type = None
        if "read" in type_filter:
            op_type = "read"
        elif "write" in type_filter:
            op_type = "write"

        self.search_status.configure(text="Searching...")
        self.update_idletasks()

        results = udb.get_operations_for_manufacturer(manufacturer, query=query, op_type=op_type, limit=_MAX_DISPLAY)

        self.search_status.configure(text=f"{len(results)} results")
        self.vehicle_label.configure(text=f"Search ({make}): '{query}' — {len(results)} results")

        if not results:
            self._show_empty(f"No results for '{query}' ({make})")
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
            if service not in ("34", "35", "36", "37", "3D", "28", "11"):
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
        BLOCKED_SERVICES = {"34", "35", "36", "37", "3D", "28", "11"}
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
            mgr = self.app.advanced_manager

            # Set header to target ECU
            send_id = ecu.get("send_id", "7E0")
            send_id_int = int(send_id, 16) if isinstance(send_id, str) else send_id
            mgr._set_header(send_id_int)

            # SAFETY: Only enter extended session for non-read operations
            if not is_read:
                mgr._enter_session(0x03)

            # Log operation BEFORE sending
            self.app.safety.log_operation(
                f"BricarDB:{op.get('name', '?')}", int(service, 16),
                f"TX=0x{send_id} cmd={sentbytes}", "SENDING"
            )

            # Send the command
            response = mgr._send_raw(sentbytes)

            # Log result
            result_text = response if response else "No response"
            self.app.safety.log_operation(
                f"BricarDB:{op.get('name', '?')}", int(service, 16),
                f"TX=0x{send_id} cmd={sentbytes}", f"RESULT: {result_text}"
            )

            # Return to default session if we entered extended
            if not is_read:
                mgr._return_to_default()
            mgr._restore_header()

            status_text[0] = result_text
            logger.info(f"BricarDB exec: [{sentbytes}] -> {result_text}")

            # Update UI with result
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

        # Source link
        source = op.get("source", "")
        if source:
            ctk.CTkLabel(bottom, text=f"Source: {source[:60]}", font=FONTS["small"],
                         text_color=COLORS["text_muted"]).pack(side="left")

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
        self.warning_title.configure(text=t("adv_warning_title"))
        self.warning_text.configure(text=t("adv_warning_text"))
        if self._tab_var.get() == "verified":
            self._show_verified_ops()

    def on_frame_shown(self):
        if self._tab_var.get() == "verified":
            self._show_verified_ops()
