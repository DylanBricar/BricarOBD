"""ECU auto-identification — detects which ECU definitions match the connected vehicle.

Sends identification commands to known addresses and matches responses
against the database's autoident patterns. Works for ALL manufacturers.

Identification commands by protocol:
- PSA/Renault (KWP2000): 2180 (ReadLocalIdentifier)
- UDS (universal): 22F190 (ReadDataByIdentifier — VIN)
- UDS: 22F18C (ECU Serial), 22F191 (ECU HW version), 22F189 (ECU SW version)
"""

import logging
import re
from typing import Dict, List, Optional, Tuple

from obd_core.database_reader import ECUDatabase, ECUDefinition

logger = logging.getLogger(__name__)

# Diagnostic addresses with CAN IDs
KNOWN_ADDRESSES = {
    "7A": {"name": "Engine ECU", "can_tx": 0x7E0, "can_rx": 0x7E8},
    "7B": {"name": "Transmission", "can_tx": 0x7E1, "can_rx": 0x7E9},
    "7C": {"name": "ABS/ESP", "can_tx": 0x7E2, "can_rx": 0x7EA},
    "7D": {"name": "Airbag", "can_tx": 0x7E3, "can_rx": 0x7EB},
    "26": {"name": "BSI/BCM", "can_tx": 0x75D, "can_rx": 0x65D},
    "10": {"name": "Engine (alt)", "can_tx": 0x6A8, "can_rx": 0x688},
    "08": {"name": "Transmission (alt)", "can_tx": 0x6A9, "can_rx": 0x689},
    "28": {"name": "Climate", "can_tx": 0x76D, "can_rx": 0x66D},
    "60": {"name": "Instrument Cluster", "can_tx": 0x734, "can_rx": 0x714},
    "62": {"name": "Parking Sensors", "can_tx": 0x762, "can_rx": 0x662},
    "24": {"name": "Power Steering", "can_tx": 0x6B2, "can_rx": 0x692},
    "64": {"name": "Airbag (alt)", "can_tx": 0x772, "can_rx": 0x672},
    "00": {"name": "Broadcast/CAN", "can_tx": 0x7DF, "can_rx": 0x7E8},
}

# Which addresses to scan per manufacturer
MAKE_ADDRESSES = {
    "peugeot": ["7A", "26", "28", "60"], "citroen": ["7A", "26", "28", "60"],
    "citroën": ["7A", "26", "28", "60"], "ds": ["7A", "26", "28", "60"],
    "opel": ["7A", "26", "28", "60"], "vauxhall": ["7A", "26", "28", "60"],
    "renault": ["7A", "26", "10", "08"], "dacia": ["7A", "26", "10", "08"],
    "nissan": ["7A", "7B", "7C"], "volkswagen": ["7A", "7B", "7C"],
    "audi": ["7A", "7B", "7C"], "seat": ["7A", "7B", "7C"],
    "skoda": ["7A", "7B", "7C"], "bmw": ["7A", "7B", "7C", "7D"],
    "mini": ["7A", "7B", "7C", "7D"], "mercedes-benz": ["7A", "7B", "7C"],
    "hyundai": ["7A", "7C"], "kia": ["7A", "7C"],
    "ford": ["7A", "7B"], "toyota": ["7A", "7B"],
    "honda": ["7A", "7B"], "volvo": ["7A", "7B", "7C"],
    "fiat": ["7A", "26"], "alfa romeo": ["7A", "26"],
}


class ECUIdentifier:
    """Identifies ECUs by sending diagnostic commands and matching responses."""

    def __init__(self, database: ECUDatabase):
        self.database = database

    def identify_ecus(self, connection, make: str = "") -> List[Dict]:
        """Scan addresses, send identification commands, match against database.

        SAFE: Only sends read commands (2180, 22F190, 22F18C, 22F189).

        Args:
            connection: Active ELM327Connection (inside use_custom_connection callback)
            make: Detected vehicle make (for address filtering)

        Returns:
            List of {address, can_tx, can_rx, ecuname, filename, matched, raw_ident} dicts
        """
        if not self.database.is_loaded:
            return []

        # Pick addresses based on make, or scan common ones
        addresses = MAKE_ADDRESSES.get(make.lower(), ["7A", "7B", "7C"]) if make else ["7A", "7B", "7C"]

        results = []
        connection.send_command("ATE0")

        for addr in addresses:
            addr_info = KNOWN_ADDRESSES.get(addr, {})
            can_tx = addr_info.get("can_tx")
            can_rx = addr_info.get("can_rx")
            if not can_tx:
                continue

            connection.send_command(f"AT SH {can_tx:03X}")
            connection.send_command(f"AT CRA {can_rx:03X}")

            # Try identification: 2180 first (PSA/Renault), then UDS
            ident_response = connection.send_command("2180", timeout=3)
            ident_cmd = "2180"

            if not ident_response or "NO DATA" in ident_response:
                # Try UDS: ReadDataByIdentifier for ECU SW version
                ident_response = connection.send_command("22F189", timeout=3)
                ident_cmd = "22F189"

            if not ident_response or "NO DATA" in ident_response:
                ident_response = connection.send_command("22F190", timeout=3)
                ident_cmd = "22F190"

            if not ident_response or "NO DATA" in ident_response:
                continue

            clean_hex = self._clean_response(ident_response)
            if not clean_hex:
                continue

            logger.info(f"ECU [{addr}] {addr_info.get('name','?')} responded to {ident_cmd}: {clean_hex[:40]}...")

            # Parse identification fields from response
            ident_fields = self._parse_ident_response(clean_hex, ident_cmd)

            # Match against ALL candidates at this address
            candidates = self.database.find_ecus_by_address(addr)
            matched = self._match_ecu_structured(ident_fields, candidates)

            result = {
                "address": addr,
                "can_tx": can_tx,
                "can_rx": can_rx,
                "name": addr_info.get("name", f"ECU 0x{addr}"),
                "raw_response": clean_hex[:100],
                "ident_fields": ident_fields,
                "ident_cmd": ident_cmd,
                "ecuname": matched["ecuname"] if matched else f"Unknown ({len(candidates)} candidates)",
                "filename": matched.get("filename", "") if matched else "",
                "matched": matched is not None,
                "candidates_count": len(candidates),
            }
            results.append(result)
            if matched:
                logger.info(f"  → Matched: {matched['ecuname']} ({matched.get('filename','')})")
            else:
                logger.warning(f"  → No exact match ({len(candidates)} candidates at addr {addr})")

        # Restore
        connection.send_command("AT D")
        connection.send_command("AT CRA")
        connection.send_command("AT H0")

        return results

    def _parse_ident_response(self, hex_str: str, cmd: str) -> Dict[str, str]:
        """Parse identification response into structured fields.

        For 2180 (PSA/Renault KWP2000):
            Response: 61 80 [data...]
            byte[8] = diagnostic_version (1 byte)
            byte[9-11] = supplier_code (3 bytes)
            byte[17-18] = soft_version (2 bytes)
            byte[19-20] = version (2 bytes)

        For 22F189/22F190 (UDS):
            Response: 62 F1 89 [ASCII data...]
        """
        fields = {}
        try:
            raw_bytes = bytes.fromhex(hex_str)
        except ValueError:
            return fields

        if cmd == "2180" and len(raw_bytes) >= 20:
            # PSA/Renault identification frame
            # Skip first 2 bytes (61 80 = positive response)
            # Offsets are from the raw response including the 61 80 prefix
            if raw_bytes[0] == 0x61:
                offset = 2  # Skip 61 80
            else:
                offset = 0

            if len(raw_bytes) >= offset + 19:
                fields["diagnostic_version"] = str(raw_bytes[offset + 6])
                fields["supplier_code"] = raw_bytes[offset + 7:offset + 10].hex().upper().lstrip('0') or '0'
                fields["soft_version"] = raw_bytes[offset + 15:offset + 17].hex().upper()
                fields["version"] = raw_bytes[offset + 17:offset + 19].hex().upper()
                logger.debug(f"  Parsed 2180: diag={fields['diagnostic_version']} "
                           f"supplier={fields['supplier_code']} soft={fields['soft_version']} "
                           f"ver={fields['version']}")

        elif cmd.startswith("22F1") and len(raw_bytes) >= 5:
            # UDS response: 62 F1 xx [data]
            if raw_bytes[0] == 0x62:
                ascii_data = raw_bytes[3:].decode('ascii', errors='ignore').strip('\x00').strip()
                fields["uds_response"] = ascii_data

        return fields

    def _match_ecu_structured(self, ident_fields: Dict[str, str],
                               candidates: List[dict]) -> Optional[dict]:
        """Match parsed identification fields against candidate autoidents.

        Matching strategy:
        1. Exact match on (diagnostic_version + supplier_code + soft_version + version)
        2. Partial match on (supplier_code + soft_version) if no exact match
        3. UDS string match if available
        """
        if not ident_fields:
            return candidates[0] if len(candidates) == 1 else None

        best_match = None
        best_score = 0

        for candidate in candidates:
            autoidents = candidate.get("autoidents", [])
            if not autoidents:
                continue

            for ai in autoidents:
                if not isinstance(ai, dict):
                    continue

                score = 0
                total_fields = 0

                # Compare each field
                for field in ["diagnostic_version", "supplier_code", "soft_version", "version"]:
                    ai_val = str(ai.get(field, "")).upper().lstrip('0') or '0'
                    id_val = str(ident_fields.get(field, "")).upper().lstrip('0') or '0'
                    if ai_val and id_val:
                        total_fields += 1
                        if ai_val == id_val:
                            score += 1

                # Perfect match = all 4 fields match
                if total_fields > 0 and score == total_fields:
                    return candidate  # Exact match — return immediately

                # Track best partial match
                if score > best_score:
                    best_score = score
                    best_match = candidate

            # Also check UDS response
            uds_resp = ident_fields.get("uds_response", "")
            if uds_resp:
                ecuname = candidate.get("ecuname", "")
                if uds_resp in ecuname or ecuname in uds_resp:
                    return candidate

        # Return best partial match if at least 2 fields matched
        if best_score >= 2:
            logger.info(f"  Partial match ({best_score}/4 fields): {best_match.get('ecuname','')}")
            return best_match

        # Fallback: single candidate
        if len(candidates) == 1:
            return candidates[0]

        return None

    def _clean_response(self, response: str) -> str:
        """Clean ELM327 response to hex string."""
        lines = response.replace("\r", "\n").split("\n")
        hex_parts = []
        for line in lines:
            clean = line.strip()
            if not clean or ">" in clean:
                continue
            clean = clean.replace(" ", "")
            if re.match(r'^[0-9A-Fa-f]+$', clean):
                hex_parts.append(clean.upper())
        return "".join(hex_parts)
