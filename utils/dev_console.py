"""Developer console — logs all OBD commands, user actions, and errors.

Opens a separate window with a scrollable log. Captures:
- Every OBD command sent to the vehicle and its response
- Every user click (frame navigation, buttons)
- Connection/disconnection events
- DTC read/clear operations
- CSV recording events
- Errors and exceptions
- ECU scan results

Log is also written to data/dev_console.log for sharing.
"""

import logging
import customtkinter as ctk
from datetime import datetime
from pathlib import Path
from collections import deque

from gui.theme import COLORS, FONTS


class DevConsoleHandler(logging.Handler):
    """Custom logging handler that feeds the dev console window."""

    def __init__(self):
        super().__init__()
        self._console = None
        self._buffer = deque(maxlen=5000)
        self._log_file = None
        self._setup_file_log()

    def _setup_file_log(self):
        try:
            log_dir = Path(__file__).parent.parent / "data"
            log_dir.mkdir(parents=True, exist_ok=True)
            self._log_file = open(log_dir / "dev_console.log", "w", encoding="utf-8")
        except Exception:
            pass

    def set_console(self, console):
        self._console = console
        # Flush buffer to console (delayed to ensure window is rendered)
        console.after(200, self._flush_buffer)

    def _flush_buffer(self):
        if self._console:
            for record in self._buffer:
                self._console.append_log(record)

    def emit(self, record):
        try:
            msg = self.format(record)
            self._buffer.append(msg)
            if self._log_file:
                self._log_file.write(msg + "\n")
                self._log_file.flush()
            if self._console:
                self._console.append_log(msg)
        except Exception:
            pass

    def close(self):
        if self._log_file:
            self._log_file.close()
        super().close()


class DevConsoleWindow(ctk.CTkToplevel):
    """Developer console window with scrollable log and copy button."""

    def __init__(self, parent):
        super().__init__(parent)
        self.title("BricarOBD — Developer Console")
        self.configure(fg_color="#0D1117")

        w, h = 900, 600
        x = (self.winfo_screenwidth() - w) // 2
        y = (self.winfo_screenheight() - h) // 2
        self.geometry(f"{w}x{h}+{x}+{y}")

        # Bring to front and keep on top
        self.attributes('-topmost', True)
        self.focus_force()

        self._paused = False
        self._setup_ui()

    def _setup_ui(self):
        # Toolbar
        toolbar = ctk.CTkFrame(self, fg_color="#161B22", height=40)
        toolbar.pack(fill="x", padx=0, pady=0)
        toolbar.pack_propagate(False)

        ctk.CTkLabel(
            toolbar, text="Developer Console", font=("Consolas", 13, "bold"),
            text_color="#58A6FF"
        ).pack(side="left", padx=12)

        # Filter buttons
        self._filter_var = ctk.StringVar(value="ALL")
        for label, value in [("All", "ALL"), ("OBD", "OBD"), ("UI", "UI"), ("ERR", "ERR")]:
            ctk.CTkButton(
                toolbar, text=label, width=50, height=26,
                font=("Consolas", 11),
                fg_color="#21262D" if value != "ALL" else "#1F6FEB",
                hover_color="#30363D",
                command=lambda v=value: self._set_filter(v),
            ).pack(side="left", padx=2, pady=6)

        # Right side buttons
        ctk.CTkButton(
            toolbar, text="Copy All", width=80, height=26,
            font=("Consolas", 11), fg_color="#238636", hover_color="#2EA043",
            command=self._copy_all,
        ).pack(side="right", padx=4, pady=6)

        ctk.CTkButton(
            toolbar, text="Clear", width=60, height=26,
            font=("Consolas", 11), fg_color="#DA3633", hover_color="#F85149",
            command=self._clear,
        ).pack(side="right", padx=4, pady=6)

        self._pause_btn = ctk.CTkButton(
            toolbar, text="Pause", width=60, height=26,
            font=("Consolas", 11), fg_color="#21262D", hover_color="#30363D",
            command=self._toggle_pause,
        )
        self._pause_btn.pack(side="right", padx=4, pady=6)

        # Log count
        self._count_label = ctk.CTkLabel(
            toolbar, text="0 lines", font=("Consolas", 10),
            text_color="#8B949E"
        )
        self._count_label.pack(side="right", padx=8)

        # Log area
        self._textbox = ctk.CTkTextbox(
            self, fg_color="#0D1117", text_color="#C9D1D9",
            font=("Consolas", 11), wrap="none",
            border_width=0,
        )
        self._textbox.pack(fill="both", expand=True, padx=4, pady=(0, 4))
        self._textbox.configure(state="disabled")

        self._line_count = 0

    def append_log(self, msg):
        if self._paused:
            return

        # Color based on content
        self._textbox.configure(state="normal")
        self._textbox.insert("end", msg + "\n")
        self._textbox.see("end")
        self._textbox.configure(state="disabled")

        self._line_count += 1
        self._count_label.configure(text=f"{self._line_count} lines")

    def _copy_all(self):
        content = self._textbox.get("1.0", "end").strip()
        self.clipboard_clear()
        self.clipboard_append(content)
        self._count_label.configure(text="Copied!")
        self.after(2000, lambda: self._count_label.configure(text=f"{self._line_count} lines"))

    def _clear(self):
        self._textbox.configure(state="normal")
        self._textbox.delete("1.0", "end")
        self._textbox.configure(state="disabled")
        self._line_count = 0
        self._count_label.configure(text="0 lines")

    def _toggle_pause(self):
        self._paused = not self._paused
        self._pause_btn.configure(
            text="Resume" if self._paused else "Pause",
            fg_color="#1F6FEB" if self._paused else "#21262D",
        )

    def _set_filter(self, value):
        self._filter_var.set(value)


# Singleton handler
_handler = None


def get_dev_handler() -> DevConsoleHandler:
    global _handler
    if _handler is None:
        _handler = DevConsoleHandler()
        _handler.setFormatter(logging.Formatter(
            "%(asctime)s [%(levelname)s] %(name)s: %(message)s",
            datefmt="%H:%M:%S.%f"
        ))
    return _handler


def setup_dev_logging():
    """Install the dev console handler on the root logger."""
    handler = get_dev_handler()
    root = logging.getLogger()
    if handler not in root.handlers:
        root.addHandler(handler)


def log_obd_command(direction: str, cmd: str, response: str = ""):
    """Log an OBD command sent or received."""
    logger = logging.getLogger("obd.traffic")
    if direction == "TX":
        logger.info(f"TX >>> {cmd}")
    elif direction == "RX":
        logger.info(f"RX <<< {response}")
    else:
        logger.info(f"{direction} {cmd} {response}")


def log_user_action(action: str, detail: str = ""):
    """Log a user interaction."""
    logger = logging.getLogger("ui.action")
    logger.info(f"{action}: {detail}" if detail else action)
