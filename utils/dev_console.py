"""Developer console — logs all OBD commands, user actions, and errors."""

import logging
import queue
import customtkinter as ctk
from pathlib import Path
from collections import deque


# Thread-safe queue for passing log messages to the UI
_log_queue = queue.Queue()

# Buffer for logs before console window opens
_log_buffer = deque(maxlen=5000)

# File log
_log_file = None


def _get_log_file():
    global _log_file
    if _log_file is None:
        try:
            log_dir = Path(__file__).parent.parent / "data"
            log_dir.mkdir(parents=True, exist_ok=True)
            _log_file = open(log_dir / "dev_console.log", "w", encoding="utf-8")
        except Exception:
            pass
    return _log_file


class _DevHandler(logging.Handler):
    """Logging handler that pushes to queue + buffer + file."""

    # Noisy loggers to suppress at DEBUG level
    _SUPPRESSED = {"PIL.PngImagePlugin", "PIL.Image"}

    def emit(self, record):
        try:
            # Skip noisy debug loggers
            if record.levelno <= logging.DEBUG and record.name in self._SUPPRESSED:
                return
            msg = self.format(record)
            _log_buffer.append(msg)
            _log_queue.put(msg)
            f = _get_log_file()
            if f:
                f.write(msg + "\n")
                f.flush()
        except Exception:
            pass


class DevConsoleWindow(ctk.CTkToplevel):
    """Developer console window."""

    def __init__(self, parent):
        super().__init__(parent)
        self.title("BricarOBD — Developer Console")
        self.configure(fg_color="#0D1117")
        self.geometry("900x600")
        self.attributes('-topmost', True)

        self._paused = False
        self._line_count = 0
        self._build_ui()

        # Drain the queue (avoid duplicates with buffer)
        while not _log_queue.empty():
            try:
                _log_queue.get_nowait()
            except queue.Empty:
                break

        # Load all buffered logs
        for msg in list(_log_buffer):
            self._insert(msg)

        # Start polling the queue for new messages
        self._poll()

    def _build_ui(self):
        toolbar = ctk.CTkFrame(self, fg_color="#161B22", height=40)
        toolbar.pack(fill="x")
        toolbar.pack_propagate(False)

        ctk.CTkLabel(toolbar, text="Developer Console",
                     font=("Consolas", 13, "bold"),
                     text_color="#58A6FF").pack(side="left", padx=12)

        ctk.CTkButton(toolbar, text="Copy All", width=80, height=26,
                      font=("Consolas", 11), fg_color="#238636",
                      hover_color="#2EA043",
                      command=self._copy_all).pack(side="right", padx=4, pady=6)

        ctk.CTkButton(toolbar, text="Clear", width=60, height=26,
                      font=("Consolas", 11), fg_color="#DA3633",
                      hover_color="#F85149",
                      command=self._clear).pack(side="right", padx=4, pady=6)

        self._pause_btn = ctk.CTkButton(toolbar, text="Pause", width=60, height=26,
                                        font=("Consolas", 11), fg_color="#21262D",
                                        hover_color="#30363D",
                                        command=self._toggle_pause)
        self._pause_btn.pack(side="right", padx=4, pady=6)

        self._count_label = ctk.CTkLabel(toolbar, text="0 lines",
                                         font=("Consolas", 10),
                                         text_color="#8B949E")
        self._count_label.pack(side="right", padx=8)

        self._textbox = ctk.CTkTextbox(self, fg_color="#0D1117",
                                       text_color="#C9D1D9",
                                       font=("Consolas", 11), wrap="none")
        self._textbox.pack(fill="both", expand=True, padx=4, pady=(0, 4))

    def _insert(self, msg):
        self._textbox.insert("end", msg + "\n")
        self._textbox.see("end")
        self._line_count += 1
        self._count_label.configure(text=f"{self._line_count} lines")

    def _poll(self):
        """Poll the queue for new log messages (runs on main thread)."""
        if not self.winfo_exists():
            return
        if not self._paused:
            try:
                while True:
                    msg = _log_queue.get_nowait()
                    self._insert(msg)
            except queue.Empty:
                pass
            except Exception:
                pass  # Widget destroyed mid-insert
        if self.winfo_exists():
            self.after(100, self._poll)

    def _copy_all(self):
        content = self._textbox.get("1.0", "end").strip()
        self.clipboard_clear()
        self.clipboard_append(content)
        self._count_label.configure(text="Copied!")
        self.after(2000, lambda: self._count_label.configure(
            text=f"{self._line_count} lines"))

    def _clear(self):
        self._textbox.delete("1.0", "end")
        self._line_count = 0
        self._count_label.configure(text="0 lines")

    def _toggle_pause(self):
        self._paused = not self._paused
        self._pause_btn.configure(
            text="Resume" if self._paused else "Pause",
            fg_color="#1F6FEB" if self._paused else "#21262D")


# Singleton handler
_handler = None


def setup_dev_logging():
    """Install the dev console handler on the root logger."""
    global _handler
    if _handler is not None:
        return
    _handler = _DevHandler()
    _handler.setFormatter(logging.Formatter(
        "%(asctime)s [%(levelname)s] %(name)s: %(message)s",
        datefmt="%H:%M:%S"))
    _handler.setLevel(logging.DEBUG)
    root = logging.getLogger()
    root.setLevel(logging.DEBUG)
    root.addHandler(_handler)


def log_obd_command(direction, cmd, response=""):
    logger = logging.getLogger("obd.traffic")
    if direction == "TX":
        logger.info(f"TX >>> {cmd}")
    else:
        logger.info(f"RX <<< {response}")


def log_user_action(action, detail=""):
    logger = logging.getLogger("ui.action")
    logger.info(f"{action}: {detail}" if detail else action)
