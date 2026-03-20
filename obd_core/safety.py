"""Safety guards for OBD diagnostic operations."""

from __future__ import annotations

import logging
import time
from enum import Enum

from config import BLOCKED_UDS_SERVICES, SAFE_UDS_SERVICES, CONFIRMED_UDS_SERVICES, MAX_FAILED_SECURITY_ATTEMPTS

logger = logging.getLogger(__name__)


class OperationRisk(Enum):
    """Risk level classification for operations."""
    SAFE = "safe"
    CAUTION = "caution"
    DANGEROUS = "dangerous"
    BLOCKED = "blocked"


class SafetyGuard:
    """Manages safety checks and operation logging."""

    def __init__(self):
        """Initialize safety guard."""
        self.operation_log = []
        self.failed_attempts = 0
        self.last_clear_time = 0

    def classify_operation(self, service_id: int) -> OperationRisk:
        """
        Classify operation risk level.

        Args:
            service_id: UDS service ID

        Returns:
            OperationRisk classification
        """
        if service_id in BLOCKED_UDS_SERVICES:
            return OperationRisk.BLOCKED
        elif service_id in SAFE_UDS_SERVICES:
            return OperationRisk.SAFE
        elif service_id in CONFIRMED_UDS_SERVICES:
            return OperationRisk.CAUTION
        else:
            return OperationRisk.DANGEROUS

    def is_operation_allowed(self, service_id: int) -> tuple[bool, str]:
        """
        Check if operation is allowed.

        Args:
            service_id: UDS service ID

        Returns:
            Tuple of (allowed: bool, reason: str)
        """
        risk = self.classify_operation(service_id)

        if risk == OperationRisk.BLOCKED:
            return False, f"Service 0x{service_id:02X} is blocked for safety reasons"
        elif risk == OperationRisk.SAFE:
            return True, ""
        elif risk == OperationRisk.CAUTION:
            return True, "requires_confirmation"
        else:  # DANGEROUS - default deny
            return False, f"Service 0x{service_id:02X} is not in the safety allowlist"

    def validate_dtc_clear(self) -> tuple[bool, str]:
        """
        Validate DTC clear operation.

        Returns:
            Tuple of (allowed: bool, reason: str)
        """
        current_time = time.time()
        time_since_last = current_time - self.last_clear_time

        if time_since_last < 5:
            remaining = 5 - time_since_last
            return False, f"DTC clear cooldown active. Wait {remaining:.1f}s before next clear"

        return True, ""

    def record_dtc_clear(self) -> None:
        """Record that a DTC clear operation was performed."""
        self.last_clear_time = time.time()

    def log_operation(self, operation: str, service_id: int, data: str, result: str) -> None:
        """
        Log an operation.

        Args:
            operation: Operation name
            service_id: UDS service ID
            data: Operation data
            result: Operation result
        """
        log_entry = {
            "timestamp": time.time(),
            "operation": operation,
            "service_id": f"0x{service_id:02X}",
            "data": data,
            "result": result
        }
        self.operation_log.append(log_entry)
        logger.debug(f"Logged: {operation} (0x{service_id:02X})")
        if len(self.operation_log) > 10000:
            self.operation_log = self.operation_log[-5000:]

    def get_operation_log(self) -> list:
        """
        Get operation log.

        Returns:
            List of logged operations
        """
        return self.operation_log

    def get_risk_description(self, service_id: int) -> str:
        """
        Get human-readable risk description.

        Args:
            service_id: UDS service ID

        Returns:
            Risk description
        """
        risk = self.classify_operation(service_id)

        descriptions = {
            OperationRisk.SAFE: "This operation is safe and read-only",
            OperationRisk.CAUTION: "This operation requires confirmation - it modifies vehicle state",
            OperationRisk.DANGEROUS: "This operation is potentially dangerous - use with caution",
            OperationRisk.BLOCKED: "This operation is blocked for safety reasons"
        }

        return descriptions.get(risk, "Unknown risk level")
