"""Operation audit logging for OBD diagnostic tool."""

import logging
import json
from pathlib import Path
from datetime import datetime

from config import LOG_DIR
LOG_DIR.mkdir(parents=True, exist_ok=True)


class AuditLogger:
    """Manages audit logging for all operations."""

    def __init__(self):
        """Initialize audit logger."""
        self.log_path = LOG_DIR / "audit.log"
        self.file_handler = logging.FileHandler(self.log_path, mode='a')
        self.file_handler.setFormatter(logging.Formatter(
            '%(asctime)s - %(levelname)s - %(message)s'
        ))

    def log_connection(self, port: str, baud_rate: int, result: str) -> None:
        """
        Log connection attempt.

        Args:
            port: Serial port
            baud_rate: Baud rate
            result: Success/failure result
        """
        message = f"Connection | Port: {port}, Baud: {baud_rate}, Result: {result}"
        self.file_handler.emit(logging.LogRecord(
            name="audit", level=logging.INFO, pathname="", lineno=0,
            msg=message, args=(), exc_info=None
        ))

    def log_command(self, command: str, response: str, duration_ms: float) -> None:
        """
        Log OBD command execution.

        Args:
            command: Command sent
            response: Response received
            duration_ms: Duration in milliseconds
        """
        message = f"Command | Cmd: {command}, Response: {response}, Duration: {duration_ms}ms"
        self.file_handler.emit(logging.LogRecord(
            name="audit", level=logging.INFO, pathname="", lineno=0,
            msg=message, args=(), exc_info=None
        ))

    def log_dtc_read(self, dtc_list: list) -> None:
        """
        Log DTC read operation.

        Args:
            dtc_list: List of DTCs read
        """
        message = f"DTC Read | Count: {len(dtc_list)}, DTCs: {','.join(dtc_list)}"
        self.file_handler.emit(logging.LogRecord(
            name="audit", level=logging.INFO, pathname="", lineno=0,
            msg=message, args=(), exc_info=None
        ))

    def log_dtc_clear(self, dtc_list: list, confirmed: bool) -> None:
        """
        Log DTC clear operation.

        Args:
            dtc_list: List of DTCs cleared
            confirmed: Whether operation was user-confirmed
        """
        message = f"DTC Clear | Count: {len(dtc_list)}, DTCs: {','.join(dtc_list)}, Confirmed: {confirmed}"
        self.file_handler.emit(logging.LogRecord(
            name="audit", level=logging.WARNING, pathname="", lineno=0,
            msg=message, args=(), exc_info=None
        ))

    def log_session_export(self, filepath: str, format_type: str) -> None:
        """
        Log session export.

        Args:
            filepath: Export file path
            format_type: Export format (csv, json, html)
        """
        message = f"Session Export | Path: {filepath}, Format: {format_type}"
        self.file_handler.emit(logging.LogRecord(
            name="audit", level=logging.INFO, pathname="", lineno=0,
            msg=message, args=(), exc_info=None
        ))

    def log_session_import(self, filepath: str) -> None:
        """
        Log session import.

        Args:
            filepath: Import file path
        """
        message = f"Session Import | Path: {filepath}"
        self.file_handler.emit(logging.LogRecord(
            name="audit", level=logging.INFO, pathname="", lineno=0,
            msg=message, args=(), exc_info=None
        ))

    def log_error(self, operation: str, error_msg: str) -> None:
        """
        Log error.

        Args:
            operation: Operation that failed
            error_msg: Error message
        """
        message = f"Error | Operation: {operation}, Error: {error_msg}"
        self.file_handler.emit(logging.LogRecord(
            name="audit", level=logging.ERROR, pathname="", lineno=0,
            msg=message, args=(), exc_info=None
        ))

    def get_log_path(self) -> Path:
        """
        Get path to audit log.

        Returns:
            Path to audit.log file
        """
        return self.log_path


def setup_logging():
    """Configure root logger with file and console handlers."""
    root_logger = logging.getLogger()
    root_logger.setLevel(logging.DEBUG)

    console_handler = logging.StreamHandler()
    console_handler.setLevel(logging.INFO)
    console_handler.setFormatter(logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    ))

    file_handler = logging.FileHandler(LOG_DIR / "debug.log", mode='a')
    file_handler.setLevel(logging.DEBUG)
    file_handler.setFormatter(logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    ))

    root_logger.addHandler(console_handler)
    root_logger.addHandler(file_handler)

    return root_logger
