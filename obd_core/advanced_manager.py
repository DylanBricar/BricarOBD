"""Advanced operations execution engine.

Executes verified manufacturer-specific UDS commands for advanced diagnostics.
Bypasses the normal safety guard for pre-verified operations only.
All operations are logged for audit trail.
"""

from __future__ import annotations

import logging
import time
from typing import Optional

from i18n import get_lang

logger = logging.getLogger(__name__)


class AdvancedManager:
    """Executes advanced diagnostic operations via UDS.

    Unlike the normal UDS client which blocks services 0x2E/0x2F/0x31/0x27,
    this manager can send those commands — but ONLY for operations that are
    pre-verified in the advanced_operations database.

    The ECU will reject any incorrect command with a Negative Response Code
    (NRC), which is harmless to the vehicle.
    """

    NRC_DESCRIPTIONS = {
        0x10: "General Reject",
        0x11: "Service Not Supported",
        0x12: "Sub-function Not Supported",
        0x13: "Incorrect Message Length",
        0x14: "Response Too Long",
        0x21: "Busy Repeat Request",
        0x22: "Conditions Not Correct",
        0x24: "Request Sequence Error",
        0x25: "No Response From Subnet",
        0x26: "Failure Prevents Execution",
        0x31: "Request Out Of Range",
        0x33: "Security Access Denied",
        0x35: "Invalid Key",
        0x36: "Exceeded Number Of Attempts",
        0x37: "Required Time Delay Not Expired",
        0x72: "General Programming Failure",
        0x78: "Request Correctly Received - Response Pending",
    }

    def __init__(self, connection, safety):
        """Initialize advanced manager.

        Args:
            connection: Vehicle connection (HybridConnection or DemoConnection)
            safety: SafetyGuard instance for logging
        """
        self.connection = connection
        self.safety = safety
        self._original_header = None

    def execute_operation(self, operation: dict, params: dict = None) -> dict:
        """Execute an advanced operation.

        Args:
            operation: Operation dict from advanced_operations database
            params: User-provided parameters (if any)

        Returns:
            Dict with {success: bool, message: str, data: str}
        """
        op_id = operation["id"]
        lang = get_lang()

        logger.warning(f"ADVANCED OPERATION: Executing {op_id}")
        self.safety.log_operation(
            f"Advanced:{op_id}", operation.get("service", 0),
            f"ECU={operation['ecu_name']}", "STARTED"
        )

        try:
            # Step 1: Set CAN header to target ECU
            ecu_tx = operation["ecu_tx"]
            if not self._set_header(ecu_tx):
                return self._result(False, self._t(
                    "Impossible de cibler le calculateur",
                    "Failed to set CAN header to ECU",
                    lang))

            # Step 2: Enter Extended Diagnostic Session (if needed)
            session = operation.get("session", 0x01)
            if session != 0x01:
                if not self._enter_session(session):
                    self._restore_header()
                    return self._result(False, self._t(
                        "Impossible d'entrer en session étendue. Le calculateur ne répond pas.",
                        "Failed to enter extended session. ECU not responding.",
                        lang))

            # Step 3: Security Access (if needed)
            if operation.get("security", False):
                sec_result = self._attempt_security_access()
                if not sec_result["success"]:
                    self._return_to_default()
                    self._restore_header()
                    return self._result(False, self._t(
                        f"Accès sécurisé refusé : {sec_result['message']}. "
                        "Cette opération nécessite un outil constructeur.",
                        f"Security access denied: {sec_result['message']}. "
                        "This operation requires OEM diagnostic tool.",
                        lang))

            # Step 4: Execute the command
            cmd_result = self._execute_command(operation, params)

            # Step 5: Return to default session
            self._return_to_default()

            # Step 6: Restore original header
            self._restore_header()

            self.safety.log_operation(
                f"Advanced:{op_id}", operation.get("service", 0),
                f"ECU={operation['ecu_name']}", "OK" if cmd_result["success"] else "FAILED"
            )

            return cmd_result

        except Exception as e:
            logger.error(f"Advanced operation {op_id} failed: {e}")
            self._return_to_default()
            self._restore_header()
            self.safety.log_operation(
                f"Advanced:{op_id}", operation.get("service", 0),
                f"ECU={operation['ecu_name']}", f"ERROR: {e}"
            )
            return self._result(False, str(e))

    def _set_header(self, ecu_tx: int) -> bool:
        """Set CAN header to target ECU."""
        try:
            response = self.connection.send_command(f"AT SH {ecu_tx:03X}")
            if "OK" in response:
                logger.info(f"Set CAN header to 0x{ecu_tx:03X}")
                return True
            logger.warning(f"ATSH failed: {response}")
        except Exception as e:
            logger.error(f"ATSH error: {e}")
        return False

    def _restore_header(self):
        """Restore default OBD broadcast header."""
        try:
            self.connection.send_command("AT SH 7DF")
        except Exception:
            pass

    def _enter_session(self, session_type: int) -> bool:
        """Enter a diagnostic session."""
        cmd = f"10{session_type:02X}"
        response = self._send_raw(cmd)
        return self._is_positive(response, 0x50)

    def _return_to_default(self):
        """Return to default diagnostic session."""
        try:
            self._send_raw("1001")
        except Exception:
            pass

    def _attempt_security_access(self) -> dict:
        """Attempt UDS SecurityAccess (Service 0x27).

        Tries level 0x01 (standard). If the seed is all zeros, security is
        already unlocked. Otherwise, we cannot compute the key (proprietary).
        """
        # Request seed
        response = self._send_raw("2701")
        if not response:
            return {"success": False, "message": "No response to security seed request"}

        clean = response.replace(" ", "").replace("\n", "")

        # Check for negative response
        if clean.startswith("7F"):
            nrc = int(clean[4:6], 16) if len(clean) >= 6 else 0
            desc = self.NRC_DESCRIPTIONS.get(nrc, f"NRC 0x{nrc:02X}")
            return {"success": False, "message": desc}

        # Check for positive response: 67 01 [seed bytes]
        if clean.startswith("6701"):
            seed = clean[4:]
            # If seed is all zeros, security is already unlocked
            if all(c == '0' for c in seed) or not seed:
                logger.info("Security already unlocked (zero seed)")
                return {"success": True, "message": "Already unlocked"}

            # Non-zero seed: we need the manufacturer-specific key algorithm
            # Try sending key = seed (works on some development ECUs)
            key_cmd = f"2702{seed}"
            key_response = self._send_raw(key_cmd)
            if key_response and self._is_positive(key_response.replace(" ", "").replace("\n", ""), 0x67):
                logger.info("Security unlocked with basic key")
                return {"success": True, "message": "Unlocked"}

            return {"success": False, "message": f"Seed: {seed} — key algorithm required"}

        return {"success": False, "message": "Unexpected response"}

    def _execute_command(self, operation: dict, params: dict = None) -> dict:
        """Execute the main UDS command for an operation."""
        lang = get_lang()
        service = operation["service"]
        did_or_rid = operation["did_or_rid"]
        cmd_type = operation["command_type"]

        if cmd_type == "read":
            return self._execute_read(service, did_or_rid, lang)
        elif cmd_type == "raw":
            return self._execute_raw_fixed(operation, lang)
        elif cmd_type == "routine":
            return self._execute_routine(service, did_or_rid, operation, params, lang)
        elif cmd_type == "write":
            return self._execute_write(service, did_or_rid, operation, params, lang)
        else:
            return self._result(False, f"Unknown command type: {cmd_type}")

    def _execute_read(self, service: int, did: int, lang: str) -> dict:
        """Execute a read command (Service 0x22)."""
        cmd = f"{service:02X}{did:04X}"
        response = self._send_raw(cmd)

        if not response:
            return self._result(False, self._t(
                "Pas de réponse du calculateur.",
                "No response from ECU.", lang))

        clean = response.replace(" ", "").replace("\n", "")

        # Check negative response
        if clean.startswith("7F"):
            nrc = int(clean[4:6], 16) if len(clean) >= 6 else 0
            desc = self.NRC_DESCRIPTIONS.get(nrc, f"NRC 0x{nrc:02X}")
            return self._result(False, self._t(
                f"Réponse négative : {desc}",
                f"Negative response: {desc}", lang))

        # Positive response: 62 [DID] [data]
        expected_prefix = f"62{did:04X}"
        if clean.upper().startswith(expected_prefix.upper()):
            data_hex = clean[len(expected_prefix):]
            # Try to format the data
            formatted = self._format_read_data(data_hex)
            return self._result(True, self._t(
                f"Données lues : {formatted}",
                f"Data read: {formatted}", lang), data_hex)

        return self._result(True, self._t(
            f"Réponse : {clean}",
            f"Response: {clean}", lang), clean)

    def _execute_raw_fixed(self, operation: dict, lang: str) -> dict:
        """Execute a pre-defined raw hex command (e.g., actuator tests)."""
        raw_cmd = operation.get("raw_command", "")
        stop_cmd = operation.get("raw_stop_command", "")

        if not raw_cmd:
            return self._result(False, "No command defined")

        response = self._send_raw(raw_cmd)
        if not response:
            return self._result(False, self._t(
                "Pas de réponse du calculateur.",
                "No response from ECU.", lang))

        clean = response.replace(" ", "").replace("\n", "")

        if clean.startswith("7F"):
            nrc = int(clean[4:6], 16) if len(clean) >= 6 else 0
            desc = self.NRC_DESCRIPTIONS.get(nrc, f"0x{nrc:02X}")
            return self._result(False, self._t(
                f"Réponse négative : {desc}",
                f"Negative response: {desc}", lang))

        # Send stop command after a short delay (for actuator tests)
        if stop_cmd:
            import time
            time.sleep(2)
            self._send_raw(stop_cmd)

        return self._result(True, self._t(
            f"Commande exécutée. Réponse : {clean}",
            f"Command executed. Response: {clean}", lang), clean)

    def _execute_routine(self, service: int, rid: int, op: dict, params: dict, lang: str) -> dict:
        """Execute a routine command (Service 0x31 — StartRoutine)."""
        # 31 01 [RID_high] [RID_low] [optional_data]
        cmd = f"3101{rid:04X}"

        # Append data if routine needs parameters
        if params and op.get("data_template") == "battery_params":
            capacity = int(params.get("capacity", 80))
            cmd += f"{capacity:02X}"

        response = self._send_raw(cmd)

        if not response:
            return self._result(False, self._t(
                "Pas de réponse du calculateur.",
                "No response from ECU.", lang))

        clean = response.replace(" ", "").replace("\n", "")

        # Check negative response
        if clean.startswith("7F"):
            nrc = int(clean[4:6], 16) if len(clean) >= 6 else 0
            desc = self.NRC_DESCRIPTIONS.get(nrc, f"NRC 0x{nrc:02X}")

            if nrc == 0x31:
                return self._result(False, self._t(
                    "Routine non supportée par ce calculateur. "
                    "Vérifiez la compatibilité avec votre véhicule.",
                    "Routine not supported by this ECU. "
                    "Check compatibility with your vehicle.", lang))
            elif nrc == 0x22:
                return self._result(False, self._t(
                    "Conditions non remplies. Vérifiez les pré-requis.",
                    "Conditions not correct. Check pre-conditions.", lang))
            elif nrc == 0x33:
                return self._result(False, self._t(
                    "Accès sécurisé requis. Utilisez un outil constructeur.",
                    "Security access required. Use OEM diagnostic tool.", lang))

            return self._result(False, self._t(
                f"Réponse négative : {desc}",
                f"Negative response: {desc}", lang))

        # Positive response: 71 01 [RID] [status]
        if clean.startswith("7101"):
            return self._result(True, self._t(
                "Routine démarrée avec succès.",
                "Routine started successfully.", lang), clean)

        return self._result(True, self._t(
            f"Réponse : {clean}",
            f"Response: {clean}", lang), clean)

    def _execute_write(self, service: int, did: int, op: dict, params: dict, lang: str) -> dict:
        """Execute a write command (Service 0x2E)."""
        if not params:
            return self._result(False, self._t(
                "Paramètres manquants.",
                "Missing parameters.", lang))

        template = op.get("data_template", "")

        if template == "raw_hex":
            return self._write_raw_hex(did, op, params, lang)
        elif template == "per_cylinder":
            return self._write_per_cylinder(did, op, params, lang)
        elif template == "battery_params":
            return self._write_battery_params(did, op, params, lang)
        else:
            return self._result(False, self._t(
                "Format de données non supporté.",
                "Data format not supported.", lang))

    def _write_raw_hex(self, did: int, op: dict, params: dict, lang: str) -> dict:
        """Write raw hex data to a DID (e.g., PSA BSI zone)."""
        raw_data = params.get("raw_data", "").strip().replace(" ", "").upper()
        if not raw_data:
            return self._result(False, self._t(
                "Données hex manquantes.",
                "Missing hex data.", lang))

        if not all(c in "0123456789ABCDEF" for c in raw_data):
            return self._result(False, self._t(
                "Données hex invalides.",
                "Invalid hex data.", lang))

        cmd = f"2E{did:04X}{raw_data}"
        response = self._send_raw(cmd)

        if response:
            clean = response.replace(" ", "").replace("\n", "")
            if clean.upper().startswith("6E"):
                return self._result(True, self._t(
                    f"Zone 0x{did:04X} écrite avec succès.",
                    f"Zone 0x{did:04X} written successfully.", lang))
            elif clean.startswith("7F"):
                nrc = int(clean[4:6], 16) if len(clean) >= 6 else 0
                desc = self.NRC_DESCRIPTIONS.get(nrc, f"0x{nrc:02X}")
                return self._result(False, self._t(
                    f"Échec écriture : {desc}",
                    f"Write failed: {desc}", lang))

        return self._result(False, self._t(
            "Pas de réponse du calculateur.",
            "No response from ECU.", lang))

    def _write_per_cylinder(self, base_did: int, op: dict, params: dict, lang: str) -> dict:
        """Write data per cylinder (e.g., injector coding)."""
        results = []
        success_count = 0

        for i in range(4):
            key = f"ima_cyl{i+1}"
            value = params.get(key, "").strip()
            if not value:
                continue

            # Clean hex value
            clean_value = value.replace(" ", "").upper()
            if not all(c in "0123456789ABCDEF" for c in clean_value):
                results.append(f"Cyl {i+1}: invalid hex")
                continue

            did = base_did + i
            cmd = f"2E{did:04X}{clean_value}"
            response = self._send_raw(cmd)

            if response:
                clean_resp = response.replace(" ", "").replace("\n", "")
                if clean_resp.startswith("6E"):
                    results.append(f"Cyl {i+1}: OK")
                    success_count += 1
                elif clean_resp.startswith("7F"):
                    nrc = int(clean_resp[4:6], 16) if len(clean_resp) >= 6 else 0
                    desc = self.NRC_DESCRIPTIONS.get(nrc, f"0x{nrc:02X}")
                    results.append(f"Cyl {i+1}: {desc}")
                else:
                    results.append(f"Cyl {i+1}: {clean_resp}")
            else:
                results.append(f"Cyl {i+1}: no response")

        detail = " | ".join(results)
        if success_count > 0:
            return self._result(True, self._t(
                f"Codage : {detail}",
                f"Coding: {detail}", lang))
        return self._result(False, self._t(
            f"Échec codage : {detail}",
            f"Coding failed: {detail}", lang))

    def _write_battery_params(self, did: int, op: dict, params: dict, lang: str) -> dict:
        """Write battery parameters."""
        capacity = int(params.get("capacity", 80))
        tech = params.get("technology", "Wet (Standard)")

        # Technology byte mapping
        tech_map = {"Wet (Standard)": 0x01, "AGM": 0x02, "EFB": 0x03, "GEL": 0x04}
        tech_byte = tech_map.get(tech, 0x01)

        cmd = f"2E{did:04X}{capacity:02X}{tech_byte:02X}"
        response = self._send_raw(cmd)

        if response:
            clean = response.replace(" ", "").replace("\n", "")
            if clean.startswith("6E"):
                return self._result(True, self._t(
                    f"Batterie enregistrée : {capacity}Ah, {tech}",
                    f"Battery registered: {capacity}Ah, {tech}", lang))
            elif clean.startswith("7F"):
                nrc = int(clean[4:6], 16) if len(clean) >= 6 else 0
                desc = self.NRC_DESCRIPTIONS.get(nrc, f"0x{nrc:02X}")
                return self._result(False, self._t(
                    f"Échec : {desc}",
                    f"Failed: {desc}", lang))

        return self._result(False, self._t(
            "Pas de réponse du calculateur.",
            "No response from ECU.", lang))

    def _send_raw(self, hex_cmd: str) -> str:
        """Send raw hex command to the vehicle.

        This bypasses the normal safety check in UDS client.
        The safety guard still logs the operation.
        """
        try:
            response = self.connection.send_raw(hex_cmd)

            # Handle Response Pending (NRC 0x78)
            for _ in range(10):
                clean = response.replace(" ", "").replace("\n", "")
                if clean.startswith("7F") and len(clean) >= 6:
                    nrc = int(clean[4:6], 16)
                    if nrc == 0x78:
                        logger.debug("Response pending, waiting...")
                        time.sleep(2)
                        try:
                            response = self.connection._read_until_prompt(5)
                        except Exception:
                            break
                        continue
                break

            return response
        except Exception as e:
            logger.error(f"send_raw failed: {e}")
            return ""

    def _is_positive(self, response: str, expected_sid_response: int) -> bool:
        """Check if a UDS response is positive."""
        if not response:
            return False
        clean = response.replace(" ", "").replace("\n", "")
        return clean.startswith(f"{expected_sid_response:02X}")

    def _format_read_data(self, hex_data: str) -> str:
        """Format raw hex data for display."""
        if not hex_data:
            return "(empty)"

        # Try ASCII interpretation
        try:
            ascii_text = bytes.fromhex(hex_data).decode("ascii", errors="ignore").strip("\x00").strip()
            if ascii_text and ascii_text.isprintable() and len(ascii_text) > 2:
                return f"{ascii_text} ({hex_data})"
        except Exception:
            pass

        # Format as spaced hex
        spaced = " ".join(hex_data[i:i+2] for i in range(0, len(hex_data), 2))
        return spaced

    @staticmethod
    def _t(fr: str, en: str, lang: str) -> str:
        """Quick bilingual text selection."""
        return fr if lang == "fr" else en

    @staticmethod
    def _result(success: bool, message: str, data: str = "") -> dict:
        """Create a standardized result dict."""
        return {"success": success, "message": message, "data": data}
